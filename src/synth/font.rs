use crate::core::SoundFont;
use crate::SoundFontId;
use crate::Synth;

/**
SoundFont management
 */
impl Synth {
    /// Loads a SoundFont. The newly
    /// loaded SoundFont will be put on top of the SoundFont
    /// stack. Presets are searched starting from the SoundFont on the
    /// top of the stack, working the way down the stack until a preset
    /// is found.
    pub fn add_font(&mut self, font: SoundFont, reset_presets: bool) -> SoundFontId {
        let id = self.core.font_bank.add_font(font);

        if reset_presets {
            self.core.program_reset();
        }

        id
    }

    fn update_presets(&mut self) {
        for id in 0..self.core.channels.len() {
            let sfontnum = self.core.channels[id].sfontnum();
            if let Some(sfontnum) = sfontnum {
                let banknum = self.core.channels[id].banknum();
                let prognum = self.core.channels[id].prognum();

                let preset = self.core.font_bank.preset(sfontnum, banknum, prognum);
                self.core.channels[id].set_preset(preset);
            }
        }
    }

    /// Removes a SoundFont from the stack and deallocates it.
    pub fn remove_font(&mut self, id: SoundFontId, reset_presets: bool) -> Result<(), ()> {
        let sfont = self.core.font_bank.remove_font(id);

        if sfont.is_some() {
            if reset_presets {
                self.core.program_reset();
            } else {
                self.update_presets();
            }

            Ok(())
        } else {
            log::error!("No SoundFont with id = {:?}", id);

            Err(())
        }
    }

    /// Count the number of loaded SoundFonts.
    pub fn count_fonts(&self) -> usize {
        self.core.font_bank.count()
    }

    /// Get a SoundFont. The SoundFont is specified by its index on the
    /// stack. The top of the stack has index zero.
    ///
    /// - `num` The number of the SoundFont (0 <= num < sfcount)
    pub fn nth_sfont(&self, num: usize) -> Option<&SoundFont> {
        self.core.font_bank.get_nth_font(num)
    }

    /// Get a SoundFont. The SoundFont is specified by its ID.
    pub fn sfont(&self, id: SoundFontId) -> Option<&SoundFont> {
        self.core.font_bank.get_font(id)
    }

    /// Offset the bank numbers in a SoundFont.
    /// Returns -1 if an error occured (out of memory or negative offset)
    pub fn set_bank_offset(&mut self, sfont_id: SoundFontId, offset: u32) {
        self.core.font_bank.bank_offsets.set(sfont_id, offset)
    }

    /// Get the offset of the bank numbers in a SoundFont.
    pub fn bank_offset(&self, sfont_id: SoundFontId) -> Option<u32> {
        self.core
            .font_bank
            .bank_offsets
            .get(sfont_id)
            .map(|o| o.offset)
    }
}

#[cfg(test)]
mod test {
    use crate::{SoundFont, Synth, SynthDescriptor};

    #[test]
    fn font_and_preset() {
        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();
        assert_eq!(synth.count_fonts(), 0);

        // Load first font
        let sin = {
            let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.count_fonts(), 1);

            let font = synth.sfont(id).unwrap();

            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Sine Wave");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
            id
        };

        // Load next font
        let boom = {
            let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.count_fonts(), 2);

            let font = synth.sfont(id).unwrap();
            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Boomwhacker");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
            id
        };

        // Check If Sin Font Is Second
        {
            // let font = synth.get_nth_sfont(1).unwrap();
        }

        // Check Sin ID
        {
            let font = synth.sfont(sin).unwrap();
            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Sine Wave");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
        }
        // Check Boomwhacker ID
        {
            let font = synth.sfont(boom).unwrap();
            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Boomwhacker");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
        }
    }
}
