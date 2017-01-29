use std::fs;
use std::io;
use std::os;
use std::path;

use error;
use resource;
use util;

#[derive(Debug)]
pub struct File {
    path: path::PathBuf,
    file_type: FileType,
}

#[derive(Debug)]
enum FileType {
    Absent,
    File { contents: Option<Vec<u8>> },
    Dir,
    Symlink { target: path::PathBuf },
}

impl File {
    pub fn at<P>(path: P) -> File
        where P: Into<path::PathBuf>
    {
        File {
            path: path.into(),
            file_type: FileType::File { contents: None },
        }
    }

    pub fn contains<B>(mut self, contents: B) -> File where B: Into<Vec<u8>> {
        self.file_type = FileType::File { contents: Some(contents.into()) };
        self
    }

    pub fn contains_str<S>(self, contents: S) -> File where S: Into<String> {
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
        where P: Into<path::PathBuf>
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
    fn realize(&self, &resource::Context { log, .. }: &resource::Context) -> error::Result<()> {
        let path = self.path.to_string_lossy().into_owned();
        let log = log.new(o!("path" => path));

        match self.file_type {
            FileType::Absent => {
                if self.path.is_dir() {
                    trace!(log, "Deleting directory");
                    fs::remove_dir(&self.path)?;
                }
                if self.path.is_file() {
                    trace!(log, "Deleting file");
                    fs::remove_file(&self.path)?;
                }
            }
            FileType::File { ref contents } => {
                if let Some(ref contents) = *contents {
                    use std::io::Write;

                    trace!(log, "Updating file contents");
                    let mut f = try!(fs::File::create(&self.path));
                    try!(f.write_all(contents));
                }
            }
            FileType::Dir => {
                fs::create_dir_all(&self.path)?;
            }
            FileType::Symlink { ref target } => {
                // TODO: add support for other OSes
                os::unix::fs::symlink(target, &self.path)?;
            }
        }
        Ok(())
    }

    fn verify(&self, &resource::Context { log, .. }: &resource::Context) -> error::Result<bool> {
        let path = self.path.to_string_lossy().into_owned();
        let log = log.new(o!("path" => path));

        if !self.path.exists() {
            debug!(log, "Path does not exist");
            return Ok(false);
        }

        let metadata = try!(fs::metadata(&self.path));
        match self.file_type {
            FileType::File { ref contents } => {
                if !metadata.file_type().is_file() {
                    debug!(log, "Path doesn't point to a regular file");
                    return Ok(false);
                }

                if let Some(ref contents) = *contents {
                    let old_sha1 = util::sha1(fs::File::open(&self.path)?)?.to_string();
                    let new_sha1 = util::sha1(io::Cursor::new(contents))?.to_string();
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

                let old_target = fs::read_link(&self.path)?;
                if old_target != *new_target {
                    let old_target = old_target.to_string_lossy().into_owned();
                    let new_target = new_target.to_string_lossy().into_owned();
                    debug!(log, "Symlink target is wrong";
                           "old_target" => old_target, "new_target" => new_target);
                    return Ok(false);
                }
            }
            FileType::Absent => {},
        }

        trace!(log, "File is up to date");
        Ok(true)
    }
}

impl resource::UnresolvedResource for File {
    fn implicit_ensure<E>(&self, ensurer: &mut E)
        where E: resource::Ensurer
    {
        if let Some(parent) = self.path.parent() {
            ensurer.ensure(File::at(parent).is_dir());
        }
    }
}
