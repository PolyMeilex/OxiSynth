use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

pub struct DefaultFileSystem;

pub struct DefaultFile {
    file: std::fs::File,
}

impl DefaultFile {
    pub fn read(&mut self, buf: &mut [u8]) -> bool {
        self.file.read(buf).is_ok()
    }

    pub fn seek(&mut self, pos: SeekFrom) -> bool {
        self.file.seek(pos).is_ok()
    }

    pub fn tell(&mut self) -> Option<u64> {
        self.file.seek(SeekFrom::Current(0)).ok()
    }
}

impl DefaultFileSystem {
    pub fn open(&mut self, filename: &Path) -> Option<DefaultFile> {
        std::fs::File::open(filename)
            .ok()
            .map(|file| DefaultFile { file })
    }
}
