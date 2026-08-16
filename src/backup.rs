use std::{
    ffi::OsString,
    fs::{File, FileTimes, Permissions},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};
use tempfile::{Builder, NamedTempFile};

#[derive(Debug)]
pub struct Backup {
    path: PathBuf,
    unaltered_permissions: Permissions,
    tempfile: Option<NamedTempFile>,
}

impl Backup {
    /// Creates a new [`Backup`] of the file at `path`. When dropped, the `Backup` attempts to
    /// restore the file to its original contents. Alternatively, one can call
    /// [`Backup::disable`] so that changes to the file at `path` are preserved.
    ///
    /// # Errors
    ///
    /// [`Backup::new`] requires that the file at `path` not be marked read-only, and returns
    /// [`ErrorKind::PermissionDenied`] when it is. Read-only files are rejected because restoring
    /// their contents may fail and, on Windows, the copied backup file may not be deletable.
    ///
    /// Note that this check is intended to prevent common failures but cannot prevent all of them.
    /// For example, the original file's permissions could change after they are checked.
    ///
    /// `Backup::new` can also return other I/O errors, e.g., if the file's metadata cannot be
    /// obtained.
    ///
    /// # Panics
    ///
    /// Panics when debug assertions are enabled and copying the original file results in a backup
    /// that is marked read-only. Note that, because of the check described above, such a panic
    /// should occur only when a permissions change races with `Backup::new`.
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let original_permissions = get_permissions_from_path(path)?;
        if original_permissions.readonly() {
            return Err(Error::from(ErrorKind::PermissionDenied));
        }
        let tempfile = sibling_tempfile(path)?;
        std::fs::copy(path, &tempfile)?;
        let unaltered_permissions = get_permissions_from_file(tempfile.as_file())?;
        debug_assert!(!unaltered_permissions.readonly());
        let readonly_permissions = readonly_permissions(&unaltered_permissions);
        tempfile.as_file().set_permissions(readonly_permissions)?;
        Ok(Self {
            path: path.to_path_buf(),
            unaltered_permissions,
            tempfile: Some(tempfile),
        })
    }

    pub fn disable(&mut self) -> Result<()> {
        let Some(tempfile) = self.tempfile.take() else {
            return Ok(());
        };

        let mut result = Ok(());

        // smoelius: On Windows, a read-only file cannot be deleted. However, Linux and macOS do not
        // have this restriction.
        #[cfg(windows)]
        {
            result = result.and(
                tempfile
                    .as_file()
                    .set_permissions(self.unaltered_permissions.clone()),
            );
        }

        result = result.and(tempfile.close());

        result
    }
}

impl Drop for Backup {
    fn drop(&mut self) {
        let Some(tempfile) = self.tempfile.take() else {
            return;
        };

        // smoelius: On Linux and macOS, `std::fs::copy` copies the tempfile's permission bits to
        // the original file. And, on Windows, a read-only file cannot be deleted. Hence, on all
        // three platforms, the tempfile's unaltered permissions must be restored. If the attempt
        // fails, continue, so that the original file is still restored. However, this may cause the
        // original file to become read-only.
        let _: Result<()> = tempfile
            .as_file()
            .set_permissions(self.unaltered_permissions.clone());

        // smoelius: Try to get the file's mtime before the copy, so that we can check whether it
        // was updated after the copy. A useful relevant article: https://apenwarr.ca/log/20181113
        let before = get_mtime(&self.path).ok();

        // smoelius: Copy the backup over the original file. If the copy fails, return.
        if std::fs::copy(&tempfile, &self.path).is_err() {
            return;
        }

        // smoelius: Did we get the file's mtime before the copy? If not, return, because we have
        // nothing to compare to.
        let Some(before) = before else {
            return;
        };

        // smoelius: Can we get the file's mtime after the copy, and is it later than before? If
        // "yes" to both, consider that success and return.
        if get_mtime(&self.path).is_ok_and(|after| before < after) {
            return;
        }

        // smoelius: If before is in the future, return, because it's hard to know what to do in
        // that situation.
        let now = SystemTime::now();
        if now < before {
            return;
        }

        // smoelius: Try to set the file's mtime to now. If we can read back something that is later
        // than before, consider that success and return.
        if set_mtime(&self.path, now).is_ok()
            && get_mtime(&self.path).is_ok_and(|nowish| before < nowish)
        {
            return;
        }

        // smoelius: Since nothing else has worked, pick a time in the future, sleep until then, and
        // then set the file's mtime to that time.
        let _: Result<SystemTime> = sleep_and_set_mtime(&self.path, before, now);
    }
}

#[allow(clippy::disallowed_methods)]
fn get_mtime(path: &Path) -> Result<SystemTime> {
    path.metadata().and_then(|metadata| metadata.modified())
}

fn sleep_and_set_mtime(path: &Path, before: SystemTime, now: SystemTime) -> Result<SystemTime> {
    // smoelius: For FAT file systems, "write time has a resolution of 2 seconds" according to the
    // following link: https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times
    const MIN_DURATION: Duration = Duration::from_secs(2);

    let Some(deadline) = before.checked_add(MIN_DURATION) else {
        return Err(Error::other("overflow"));
    };

    if let Ok(duration) = deadline.duration_since(now) {
        thread::sleep(duration);
    }

    set_mtime(path, deadline).map(|()| deadline)
}

fn set_mtime(path: &Path, modified: SystemTime) -> Result<()> {
    let file = File::options().write(true).open(path)?;
    let times = FileTimes::new().set_modified(modified);
    file.set_times(times)
}

fn sibling_tempfile(path: &Path) -> Result<NamedTempFile> {
    let canonical_path = path.canonicalize()?;
    let parent = canonical_path
        .parent()
        .expect("should not fail for a canonical path");
    let prefix = path
        .file_stem()
        .map_or(OsString::from(".tmp"), |file_stem| {
            let mut prefix = OsString::from(".");
            prefix.push(file_stem);
            prefix.push("-");
            prefix
        });
    let suffix = path.extension().map_or(OsString::new(), |extension| {
        let mut suffix = OsString::from(".");
        suffix.push(extension);
        suffix
    });
    Builder::new()
        .prefix(&prefix)
        .suffix(&suffix)
        .tempfile_in(parent)
}

#[allow(clippy::disallowed_methods)]
fn get_permissions_from_path(path: &Path) -> Result<Permissions> {
    path.metadata().map(|metadata| metadata.permissions())
}

#[allow(clippy::disallowed_methods)]
fn get_permissions_from_file(file: &File) -> Result<Permissions> {
    file.metadata().map(|metadata| metadata.permissions())
}

fn readonly_permissions(permissions: &Permissions) -> Permissions {
    let mut permissions = permissions.clone();
    permissions.set_readonly(true);
    permissions
}

#[cfg(test)]
mod tests {
    #![cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]

    use super::*;
    use std::fs::{read_dir, read_to_string, write};
    use tempfile::tempdir;

    #[test]
    fn mtime_is_updated() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        let before = get_mtime(tempfile.path()).unwrap();

        drop(backup);

        let after = get_mtime(tempfile.path()).unwrap();

        assert!(before < after, "{before:?} not less than {after:?}");
    }

    #[test]
    fn prefix_and_suffix() {
        let tempdir = tempdir().unwrap();

        let lib_rs_path = tempdir.path().join("lib.rs");

        write(&lib_rs_path, "").unwrap();

        let _backup = Backup::new(&lib_rs_path).unwrap();

        for result in read_dir(&tempdir).unwrap() {
            let entry = result.unwrap();
            let path = entry.path();
            if path == lib_rs_path {
                continue;
            }
            let file_name = entry.file_name();
            let s = file_name.to_str().unwrap();
            assert!(s.starts_with(".lib-"));
            assert!(
                Path::new(s)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            );
        }
    }

    #[test]
    fn sanity() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        write(&tempfile, "x").unwrap();

        assert_eq!("x", read_to_string(&tempfile).unwrap());

        drop(backup);

        assert_eq!("", read_to_string(&tempfile).unwrap());

        let permissions = get_permissions_from_file(tempfile.as_file()).unwrap();
        assert!(!permissions.readonly());
    }

    #[test]
    fn backup_rejects_read_only_file() {
        let tempfile = NamedTempFile::new().unwrap();
        let unaltered_permissions = get_permissions_from_file(tempfile.as_file()).unwrap();
        let readonly_permissions = readonly_permissions(&unaltered_permissions);
        tempfile
            .as_file()
            .set_permissions(readonly_permissions)
            .unwrap();

        let result = Backup::new(&tempfile);

        // smoelius: Restore the tempfile's unaltered permission so that it can be deleted on
        // Windows.
        tempfile
            .as_file()
            .set_permissions(unaltered_permissions)
            .unwrap();

        let error = result.unwrap_err();
        assert_eq!(ErrorKind::PermissionDenied, error.kind());
    }

    #[test]
    fn backup_is_readonly() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        let backup_tempfile = backup.tempfile.as_ref().unwrap();
        let backup_permissions = get_permissions_from_file(backup_tempfile.as_file()).unwrap();
        assert!(backup_permissions.readonly());
    }

    #[cfg(unix)]
    #[test]
    fn backup_preserves_read_and_execute_permission_bits() {
        use std::os::unix::fs::PermissionsExt;

        const ORIGINAL_MODE: u32 = 0o751;
        const READONLY_MODE: u32 = 0o551;
        const PERMISSION_MASK: u32 = 0o777;

        let original = NamedTempFile::new().unwrap();
        let original_permissions = std::fs::Permissions::from_mode(ORIGINAL_MODE);
        original
            .as_file()
            .set_permissions(original_permissions)
            .unwrap();

        let backup = Backup::new(&original).unwrap();

        let backup_tempfile = backup.tempfile.as_ref().unwrap();
        let backup_permissions = get_permissions_from_file(backup_tempfile.as_file()).unwrap();
        assert_eq!(READONLY_MODE, backup_permissions.mode() & PERMISSION_MASK);

        drop(backup);

        let original_permissions = get_permissions_from_file(original.as_file()).unwrap();
        assert_eq!(ORIGINAL_MODE, original_permissions.mode() & PERMISSION_MASK);
    }

    #[test]
    fn disable_preserves_changes_and_removes_backup() {
        let tempfile = NamedTempFile::new().unwrap();

        let mut backup = Backup::new(&tempfile).unwrap();
        let backup_path = backup.tempfile.as_ref().unwrap().path().to_path_buf();

        write(&tempfile, "x").unwrap();

        backup.disable().unwrap();

        assert!(!backup_path.exists());

        drop(backup);

        assert_eq!("x", read_to_string(&tempfile).unwrap());
    }
}
