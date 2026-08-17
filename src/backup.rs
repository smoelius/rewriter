use std::{
    ffi::OsString,
    fs::{File, FileTimes},
    io::{Error, Result},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};
use tempfile::{Builder, NamedTempFile};

pub struct Backup {
    path: PathBuf,
    tempfile: Option<NamedTempFile>,
}

impl Backup {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let tempfile = sibling_tempfile(path.as_ref())?;
        std::fs::copy(&path, &tempfile)?;
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            tempfile: Some(tempfile),
        })
    }

    pub fn disable(&mut self) -> Result<()> {
        self.tempfile.take().map_or(Ok(()), NamedTempFile::close)
    }
}

impl Drop for Backup {
    fn drop(&mut self) {
        let Some(tempfile) = self.tempfile.take() else {
            return;
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{read_dir, read_to_string, write};
    use tempfile::tempdir;

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    #[test]
    fn mtime_is_updated() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        let before = get_mtime(tempfile.path()).unwrap();

        drop(backup);

        let after = get_mtime(tempfile.path()).unwrap();

        assert!(before < after, "{before:?} not less than {after:?}");
    }

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
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

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    #[test]
    fn sanity() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        write(&tempfile, "x").unwrap();

        assert_eq!("x", read_to_string(&tempfile).unwrap());

        drop(backup);

        assert!(read_to_string(&tempfile).unwrap().is_empty());
    }
}
