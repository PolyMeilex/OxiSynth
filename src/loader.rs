use crate::{engine, fileapi::FileSystem, result_from_ptr, Result};
use std::mem::transmute;

/**
The SoundFont loader object
 */
#[repr(transparent)]
pub struct Loader {
    handle: *mut engine::soundfont::SoundFontLoader,
}

unsafe impl Send for Loader {}

impl Loader {
    /**
    Create default SoundFont loader
     */
    pub fn new_default() -> Result<Self> {
        result_from_ptr(engine::sfloader::new_fluid_defsfloader()).map(|handle| Self { handle })
    }

    pub(crate) fn into_ptr(self) -> *mut engine::soundfont::SoundFontLoader {
        unsafe { transmute(self) }
    }

    /**
    Set the file reading API which will be used by loader
     */
    pub fn set_file_api(&self, filesystem: Box<dyn FileSystem>) {
        let handle = unsafe { &mut *self.handle };
        handle.filesystem = filesystem;
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        unsafe {
            engine::sfloader::delete_fluid_defsfloader(self.handle);
        }
    }
}
