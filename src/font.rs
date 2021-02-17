use crate::{engine, Bank, FontId, PresetId};
use std::marker::PhantomData;

/**
The SoundFont interface
 */
pub trait IsFont {
    fn get_id(&self) -> FontId;
    fn get_name(&self) -> Option<String>;
    fn get_preset(&self, bank: Bank, num: PresetId) -> Option<PresetRef<'_>>;
}

/**
The SoundFont preset interface
 */
pub trait IsPreset {
    fn get_name(&self) -> Option<String>;
    fn get_banknum(&self) -> Option<Bank>;
    fn get_num(&self) -> Option<PresetId>;
}

/**
Reference to SoundFont object
 */
#[repr(transparent)]
pub struct FontRef<'a> {
    handle: *mut engine::soundfont::SoundFont,
    phantom: PhantomData<&'a ()>,
}

impl<'a> FontRef<'a> {
    pub(crate) fn from_ptr(handle: *mut engine::soundfont::SoundFont) -> Self {
        Self {
            handle,
            phantom: PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut engine::soundfont::SoundFont {
        self.handle
    }
}

/**
Reference to Preset object
 */
#[repr(transparent)]
pub struct PresetRef<'a> {
    handle: *mut engine::soundfont::Preset,
    phantom: PhantomData<&'a ()>,
}

impl<'a> PresetRef<'a> {
    pub(crate) fn from_ptr(handle: *mut engine::soundfont::Preset) -> Self {
        Self {
            handle,
            phantom: PhantomData,
        }
    }
}

mod private {
    use crate::{
        engine, option_from_ptr, private::HasHandle, Bank, FontId, FontRef, IsFont, IsPreset,
        PresetId, PresetRef,
    };

    impl<X> IsFont for X
    where
        X: HasHandle<Handle = engine::soundfont::SoundFont>,
    {
        fn get_id(&self) -> FontId {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            font_c.id
        }

        fn get_name(&self) -> Option<String> {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            let get_name = font_c.get_name?;
            let name = unsafe { (get_name)(handle) };
            name.and_then(|x| String::from_utf8(x).ok())
        }

        fn get_preset(&self, bank: Bank, num: PresetId) -> Option<PresetRef<'_>> {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            let get_preset = font_c.get_preset?;
            option_from_ptr(unsafe { (get_preset)(handle, bank, num) }).map(PresetRef::from_ptr)
        }
    }

    impl<'a> HasHandle for FontRef<'a> {
        type Handle = engine::soundfont::SoundFont;

        fn get_handle(&self) -> *const Self::Handle {
            self.handle
        }

        fn get_mut_handle(&mut self) -> *mut Self::Handle {
            self.handle
        }
    }

    impl<X> IsPreset for X
    where
        X: HasHandle<Handle = engine::soundfont::Preset>,
    {
        fn get_name(&self) -> Option<String> {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            let get_name = font_c.get_name?;
            let name = unsafe { (get_name)(handle) };
            String::from_utf8(name).ok()
        }

        fn get_banknum(&self) -> Option<Bank> {
            let handle = self.get_handle();
            let preset_c = unsafe { &*handle };
            let get_banknum = preset_c.get_banknum?;
            let num = unsafe { (get_banknum)(handle) };
            if num < 0 {
                None
            } else {
                Some(num as _)
            }
        }

        fn get_num(&self) -> Option<PresetId> {
            let handle = self.get_handle();
            let preset_c = unsafe { &*handle };
            let get_num = preset_c.get_num?;
            let num = unsafe { (get_num)(handle) };
            if num < 0 {
                None
            } else {
                Some(num as _)
            }
        }
    }

    impl<'a> HasHandle for PresetRef<'a> {
        type Handle = engine::soundfont::Preset;

        fn get_handle(&self) -> *const Self::Handle {
            self.handle
        }

        fn get_mut_handle(&mut self) -> *mut Self::Handle {
            self.handle
        }
    }
}
