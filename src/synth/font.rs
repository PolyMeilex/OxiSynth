use crate::oxi::soundfont::SoundFont;
use crate::Synth;
use std::io::{Read, Seek};

/**
SoundFont management
 */
impl Synth {
    /**
    Loads a SoundFont file and creates a new SoundFont. The newly
    loaded SoundFont will be put on top of the SoundFont
    stack. Presets are searched starting from the SoundFont on the
    top of the stack, working the way down the stack until a preset
    is found.
     */
    pub fn sfload<P: Read + Seek>(&mut self, file: &mut P, reset_presets: bool) -> Result<u32, ()> {
        self.handle.sfload(file, reset_presets)
    }

    /**
    Reload a SoundFont. The reloaded SoundFont retains its ID and
    index on the stack.
     */
    pub fn sfreload(&mut self, id: u32) -> Result<u32, ()> {
        self.handle.sfreload(id)
    }

    /**
    Removes a SoundFont from the stack and deallocates it.
     */
    pub fn sfunload(&mut self, id: u32, reset_presets: bool) -> Result<(), ()> {
        self.handle.sfunload(id, reset_presets)
    }

    /**
    Count the number of loaded SoundFonts.
     */
    pub fn sfcount(&self) -> usize {
        self.handle.sfcount()
    }

    /**
    Get a SoundFont. The SoundFont is specified by its index on the
    stack. The top of the stack has index zero.

    - `num` The number of the SoundFont (0 <= num < sfcount)
     */
    pub fn get_sfont(&self, num: u32) -> Option<&SoundFont> {
        self.handle.get_sfont(num)
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn get_sfont_by_id(&mut self, id: u32) -> Option<&SoundFont> {
        self.handle.get_sfont_by_id(id)
    }

    /**
    Remove a SoundFont that was previously added using
    fluid_synth_add_sfont(). The synthesizer does not delete the
    SoundFont; this is responsability of the caller.
     */
    pub fn remove_sfont(&mut self, id: u32) {
        self.handle.remove_sfont(id);
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occured (out of memory or negative offset)
     */
    pub fn set_bank_offset(&mut self, sfont_id: u32, offset: u32) {
        self.handle.set_bank_offset(sfont_id, offset)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get_bank_offset(&self, sfont_id: u32) -> Option<u32> {
        self.handle.get_bank_offset(sfont_id).map(|o| o.offset)
    }
}

#[cfg(test)]
mod test {
    use crate::{Synth, SynthDescriptor};

    #[test]
    fn font_and_preset() {
        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();

        assert_eq!(synth.sfcount(), 0);

        let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();
        synth.sfload(&mut file, true).unwrap();

        assert_eq!(synth.sfcount(), 1);

        let font = synth.get_sfont(0).unwrap();

        assert_eq!(font.get_id(), 1);

        let preset = font.get_preset(0, 0).unwrap();

        assert_eq!(preset.get_name(), "Sine Wave");
        assert_eq!(preset.get_banknum(), 0);
        assert_eq!(preset.get_num(), 0);
    }
}
