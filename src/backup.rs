use std::{
    ffi::OsString,
    io::Result,
    path::{Path, PathBuf},
    time::SystemTime,
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
        if let Some(tempfile) = self.tempfile.take() {
            // smoelius: Ensure the file's mtime is updated, e.g., for build systems that rely on
            // this information. A useful relevant article: https://apenwarr.ca/log/20181113
            let before = mtime(&self.path).ok();

            loop {
                #[cfg(target_os = "linux")]
                let result = std::fs::copy(&tempfile, &self.path);

                #[cfg(not(target_os = "linux"))]
                let result = manual_copy(tempfile.path(), &self.path);

                if result.is_err() {
                    break;
                }

                let after = mtime(&self.path).ok();

                if before
                    .zip(after)
                    .is_none_or(|(before, after)| before < after)
                {
                    break;
                }
            }
        }
    }
}

fn mtime(path: &Path) -> Result<SystemTime> {
    path.metadata().and_then(|metadata| metadata.modified())
}

#[cfg(not(target_os = "linux"))]
fn manual_copy(from: &Path, to: &Path) -> Result<()> {
    let contents = std::fs::read(from)?;
    std::fs::write(to, contents)
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
mod test {
    use super::*;
    use std::{
        ffi::OsStr,
        fs::{read_dir, read_to_string, write},
    };
    use tempfile::tempdir;

    #[cfg_attr(dylint_lib = "general", allow(non_thread_safe_call_in_test))]
    #[test]
    fn mtime_is_updated() {
        let tempfile = NamedTempFile::new().unwrap();

        let backup = Backup::new(&tempfile).unwrap();

        let before = mtime(tempfile.path()).unwrap();

        drop(backup);

        let after = mtime(tempfile.path()).unwrap();

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
            let s = path.file_name().and_then(OsStr::to_str).unwrap();
            assert!(s.starts_with(".lib-"));
            assert!(s.ends_with(".rs"));
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
