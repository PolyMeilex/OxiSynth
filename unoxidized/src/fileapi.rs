use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

/** The file API. */
pub trait File {
    /// Read binary data from file descriptor
    fn read(&mut self, buf: &mut [u8]) -> bool;

    /// Seek current reading position
    fn seek(&mut self, pos: SeekFrom) -> bool;

    /// Get current reading position from beginning of file
    fn tell(&mut self) -> Option<u64>;
}

/** The file system API. */
pub trait FileSystem {
    /// Open file with specified name
    fn open(&mut self, filename: &Path) -> Option<Box<dyn File>>;
}

struct DefaultFileSystem;

struct DefaultFile {
    file: std::fs::File,
}

impl File for DefaultFile {
    fn read(&mut self, buf: &mut [u8]) -> bool {
        self.file.read(buf).is_ok()
    }

    fn seek(&mut self, pos: SeekFrom) -> bool {
        self.file.seek(pos).is_ok()
    }

    fn tell(&mut self) -> Option<u64> {
        self.file.seek(SeekFrom::Current(0)).ok()
    }
}

impl FileSystem for DefaultFileSystem {
    fn open(&mut self, filename: &Path) -> Option<Box<dyn File>> {
        std::fs::File::open(filename)
            .ok()
            .map(|file| -> Box<dyn File> { Box::new(DefaultFile { file }) })
    }
}

pub(crate) fn make_default_fs() -> Box<dyn FileSystem> {
    return Box::new(DefaultFileSystem {});
}
