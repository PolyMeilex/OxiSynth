use crate::oxi::{SoundFont, SoundFontId};
use crate::Synth;

/**
SoundFont management
 */
impl Synth {
    /**
    Loads a SoundFont. The newly
    loaded SoundFont will be put on top of the SoundFont
    stack. Presets are searched starting from the SoundFont on the
    top of the stack, working the way down the stack until a preset
    is found.
     */
    pub fn add_font(&mut self, font: SoundFont, reset_presets: bool) -> SoundFontId {
        self.handle.add_font(font, reset_presets)
    }

    /**
    Removes a SoundFont from the stack and deallocates it.
     */
    pub fn sfunload(&mut self, id: SoundFontId, reset_presets: bool) -> Result<(), ()> {
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
    pub fn get_nth_sfont(&self, num: usize) -> Option<&SoundFont> {
        self.handle.get_nth_sfont(num)
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn get_sfont(&mut self, id: SoundFontId) -> Option<&SoundFont> {
        self.handle.get_sfont(id)
    }

    /**
    Remove a SoundFont that was previously added using
    fluid_synth_add_sfont(). The synthesizer does not delete the
    SoundFont; this is responsability of the caller.
     */
    pub fn remove_sfont(&mut self, id: SoundFontId) {
        self.handle.remove_sfont(id);
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occured (out of memory or negative offset)
     */
    pub fn set_bank_offset(&mut self, sfont_id: SoundFontId, offset: u32) {
        self.handle.set_bank_offset(sfont_id, offset)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get_bank_offset(&self, sfont_id: SoundFontId) -> Option<u32> {
        self.handle.get_bank_offset(sfont_id).map(|o| o.offset)
    }
}

#[cfg(test)]
mod test {
    use crate::{SoundFont, Synth, SynthDescriptor};

    #[test]
    fn font_and_preset() {
        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();
        assert_eq!(synth.sfcount(), 0);

        // Load first font
        let sin = {
            let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.sfcount(), 1);

            let font = synth.get_sfont(id).unwrap();

            let preset = font.get_preset(0, 0).unwrap();

            assert_eq!(preset.get_name(), "Sine Wave");
            assert_eq!(preset.get_banknum(), 0);
            assert_eq!(preset.get_num(), 0);
            id
        };

        // Load next font
        let boom = {
            let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.sfcount(), 2);

            let font = synth.get_sfont(id).unwrap();
            let preset = font.get_preset(0, 0).unwrap();

            assert_eq!(preset.get_name(), "Boomwhacker");
            assert_eq!(preset.get_banknum(), 0);
            assert_eq!(preset.get_num(), 0);
            id
        };

        // Check If Sin Font Is Second
        {
            // let font = synth.get_nth_sfont(1).unwrap();
        }

        // Check Sin ID
        {
            let font = synth.get_sfont(sin).unwrap();
            let preset = font.get_preset(0, 0).unwrap();

            assert_eq!(preset.get_name(), "Sine Wave");
            assert_eq!(preset.get_banknum(), 0);
            assert_eq!(preset.get_num(), 0);
        }
        // Check Boomwhacker ID
        {
            let font = synth.get_sfont(boom).unwrap();
            let preset = font.get_preset(0, 0).unwrap();

            assert_eq!(preset.get_name(), "Boomwhacker");
            assert_eq!(preset.get_banknum(), 0);
            assert_eq!(preset.get_num(), 0);
        }
    }
}
