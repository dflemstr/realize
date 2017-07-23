use std::any;
use std::fmt;
use std::fs;
use std::io;
use std::os;
use std::path;

use error;
use resource;
use util;

#[derive(Debug, Eq, PartialEq)]
pub struct File {
    path: path::PathBuf,
    file_type: FileType,
}

#[derive(Debug, Eq, PartialEq)]
enum FileType {
    Absent,
    File { contents: Option<Vec<u8>> },
    Dir,
    Symlink { target: path::PathBuf },
}

impl File {
    pub fn at<P>(path: P) -> File
    where
        P: Into<path::PathBuf>,
    {
        File {
            path: path.into(),
            file_type: FileType::File { contents: None },
        }
    }

    pub fn contains<B>(mut self, contents: B) -> File
    where
        B: Into<Vec<u8>>,
    {
        self.file_type = FileType::File { contents: Some(contents.into()) };
        self
    }

    pub fn contains_str<S>(self, contents: S) -> File
    where
        S: Into<String>,
    {
        self.contains(contents.into().into_bytes())
    }

    pub fn is_file(mut self) -> File {
        self.file_type = FileType::File { contents: None };
        self
    }

    pub fn is_dir(mut self) -> File {
        self.file_type = FileType::Dir;
        self
    }

    pub fn points_to<P>(mut self, path: P) -> File
    where
        P: Into<path::PathBuf>,
    {
        self.file_type = FileType::Symlink { target: path.into() };
        self
    }

    pub fn is_absent(mut self) -> File {
        self.file_type = FileType::Absent;
        self
    }
}

impl resource::Resource for File {
    fn key(&self) -> resource::Key {
        resource::Key::Path(self.path.clone())
    }

    fn realize(&self, &resource::Context { log, .. }: &resource::Context) -> error::Result<()> {
        use error::ResultExt;
        let path = self.path.to_string_lossy().into_owned();
        let log = log.new(o!("path" => path));

        match self.file_type {
            FileType::Absent => {
                if self.path.is_dir() {
                    trace!(log, "Deleting directory");
                    fs::remove_dir(&self.path).chain_err(|| {
                        format!("Failed to delete directory {:?}", self.path)
                    })?;
                }
                if self.path.is_file() {
                    trace!(log, "Deleting file");
                    fs::remove_file(&self.path).chain_err(|| {
                        format!("Failed to delete file {:?}", self.path)
                    })?;
                }
            }
            FileType::File { ref contents } => {
                if let Some(ref contents) = *contents {
                    use std::io::Write;

                    trace!(log, "Updating file contents");
                    let mut f = fs::File::create(&self.path).chain_err(|| {
                        format!("Failed to create file {:?}", self.path)
                    })?;
                    f.write_all(contents).chain_err(|| {
                        format!("Failed to write to file {:?}", self.path)
                    })?;
                }
            }
            FileType::Dir => {
                fs::create_dir_all(&self.path).chain_err(|| {
                    format!("Failed to create directory {:?}", self.path)
                })?;
            }
            FileType::Symlink { ref target } => {
                // TODO: add support for other OSes
                os::unix::fs::symlink(target, &self.path).chain_err(|| {
                    format!("Failed to create symlink {:?}", self.path)
                })?;
            }
        }
        Ok(())
    }

    fn verify(&self, &resource::Context { log, .. }: &resource::Context) -> error::Result<bool> {
        use error::ResultExt;

        let log = log.new(o!("path" => self.path.to_string_lossy().into_owned()));

        if !self.path.exists() {
            debug!(log, "Path does not exist");
            return Ok(false);
        }

        let metadata = fs::metadata(&self.path).chain_err(|| {
            format!("Failed to gather metadata about path {:?}", self.path)
        })?;
        match self.file_type {
            FileType::File { ref contents } => {
                if !metadata.file_type().is_file() {
                    debug!(log, "Path doesn't point to a regular file");
                    return Ok(false);
                }

                if let Some(ref contents) = *contents {
                    let file = fs::File::open(&self.path).chain_err(|| {
                        format!("Failed to open file {:?} for hashing", self.path)
                    })?;
                    let old_sha1 = util::sha1(file)
                        .chain_err(|| {
                            format!("Failed to compute SHA-1 digest of file {:?}", self.path)
                        })?
                        .to_string();
                    let new_sha1 = util::sha1(io::Cursor::new(contents))
                        .chain_err(|| "Failed to compute SHA-1 digest")?
                        .to_string();
                    if old_sha1 != new_sha1 {
                        debug!(log, "File has wrong contents";
                               "old_sha1" => old_sha1, "new_sha1" => new_sha1);
                        return Ok(false);
                    }
                }
            }
            FileType::Dir => {
                if !metadata.file_type().is_dir() {
                    debug!(log, "Path doesn't point to a directory");
                    return Ok(false);
                }
            }
            FileType::Symlink { target: ref new_target } => {
                if !metadata.file_type().is_symlink() {
                    debug!(log, "Path doesn't point to a symlink");
                    return Ok(false);
                }

                let old_target = fs::read_link(&self.path).chain_err(|| {
                    format!("Failed to read link target of {:?}", self.path)
                })?;
                if old_target != *new_target {
                    let old_target = old_target.to_string_lossy().into_owned();
                    let new_target = new_target.to_string_lossy().into_owned();
                    debug!(log, "Symlink target is wrong";
                           "old_target" => old_target, "new_target" => new_target);
                    return Ok(false);
                }
            }
            FileType::Absent => {}
        }

        trace!(log, "File is up to date");
        Ok(true)
    }

    fn as_any(&self) -> &any::Any {
        self
    }
}

impl resource::UnresolvedResource for File {
    fn implicit_ensure<E>(&self, ensurer: &mut E)
    where
        E: resource::Ensurer,
    {
        if let Some(parent) = self.path.parent() {
            ensurer.ensure(File::at(parent).is_dir());
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.file_type {
            FileType::Absent { .. } => write!(f, "absent")?,
            FileType::File { .. } => write!(f, "file")?,
            FileType::Dir { .. } => write!(f, "directory")?,
            FileType::Symlink { .. } => write!(f, "symlink")?,
        }

        write!(f, " {:?}", self.path)?;

        match self.file_type {
            FileType::File { contents: Some(ref contents) } => {
                let sha1 = util::sha1(io::Cursor::new(contents)).unwrap();
                write!(f, " with sha1 {}", &format!("{}", sha1)[..8])?
            }
            FileType::Symlink { ref target } => write!(f, " with target {:?}", target)?,
            _ => (),
        }
        Ok(())
    }
}
