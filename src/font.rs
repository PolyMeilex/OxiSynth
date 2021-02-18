use crate::{engine, Bank, FontId, PresetId};
use std::marker::PhantomData;

/**
The SoundFont interface
 */
pub trait IsFont {
    fn get_id(&self) -> FontId;
    fn get_name(&self) -> String;
    fn get_preset(&self, bank: Bank, num: PresetId) -> Option<engine::soundfont::Preset>;
}

/**
The SoundFont preset interface
 */
pub trait IsPreset {
    fn get_name(&self) -> String;
    fn get_banknum(&self) -> Bank;
    fn get_num(&self) -> PresetId;
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

// impl<'a> PresetRef<'a> {
//     pub(crate) fn from_ptr(handle: *mut engine::soundfont::Preset) -> Self {
//         Self {
//             handle,
//             phantom: PhantomData,
//         }
//     }
// }

mod private {
    use crate::{
        engine, private::HasHandle, Bank, FontId, FontRef, IsFont, IsPreset, PresetId, PresetRef,
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

        fn get_name(&self) -> String {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            let name = font_c.get_name();
            name
        }

        fn get_preset(&self, bank: Bank, num: PresetId) -> Option<engine::soundfont::Preset> {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            font_c.get_preset(bank, num)
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
        fn get_name(&self) -> String {
            let handle = self.get_handle();
            let font_c = unsafe { &*handle };
            let name = font_c.get_name();
            name
        }

        fn get_banknum(&self) -> Bank {
            let handle = self.get_handle();
            let preset_c = unsafe { &*handle };
            let num = preset_c.get_banknum();

            num as _
        }

        fn get_num(&self) -> PresetId {
            let handle = self.get_handle();
            let preset_c = unsafe { &*handle };
            let num = preset_c.get_num();
            num as _
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
