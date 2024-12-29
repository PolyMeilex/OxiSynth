use std::sync::Arc;

use crate::{core::soundfont::SoundFont, GeneratorType, OxiError, Preset, SoundFontId, Synth};

/// SoundFont management
impl Synth {
    /// Loads a SoundFont. The newly
    /// loaded SoundFont will be put on top of the SoundFont
    /// stack. Presets are searched starting from the SoundFont on the
    /// top of the stack, working the way down the stack until a preset
    /// is found.
    pub fn add_font(&mut self, font: SoundFont, reset_presets: bool) -> SoundFontId {
        let id = self.core.font_bank.add_font(font);

        if reset_presets {
            self.reset_program();
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
    pub fn remove_font(&mut self, id: SoundFontId, reset_presets: bool) -> Option<SoundFont> {
        let sfont = self.core.font_bank.remove_font(id);

        if let Some(font) = sfont {
            if reset_presets {
                self.reset_program();
            } else {
                self.update_presets();
            }

            Some(font)
        } else {
            log::error!("No SoundFont with id = {:?}", id);
            None
        }
    }

    /// Select a sfont.
    pub fn select_sound_font(
        &mut self,
        channel: u8,
        sfont_id: SoundFontId,
    ) -> Result<(), OxiError> {
        self.core
            .channels
            .get_mut(channel as usize)?
            .set_sfontnum(Some(sfont_id));
        Ok(())
    }

    /// Count the number of loaded SoundFonts.
    pub fn sound_font_count(&self) -> usize {
        self.core.font_bank.count()
    }

    /// Get a SoundFont. The SoundFont is specified by its index on the
    /// stack. The top of the stack has index zero.
    ///
    /// - `num` The number of the SoundFont (0 <= num < sfcount)
    pub fn nth_sound_font(&self, num: usize) -> Option<&SoundFont> {
        self.core.font_bank.nth_font(num)
    }

    /// Get a SoundFont. The SoundFont is specified by its ID.
    pub fn sound_font(&self, id: SoundFontId) -> Option<&SoundFont> {
        self.core.font_bank.font(id)
    }

    /// Offset the bank numbers in a SoundFont.
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

    pub fn channel_preset(&self, channel: u8) -> Option<Arc<Preset>> {
        if let Ok(channel) = self.core.channels.get(channel as usize) {
            channel.preset().cloned()
        } else {
            log::warn!("Channel out of range");
            None
        }
    }
}

/// SoundFont generator interface
impl Synth {
    /// Change the value of a generator. This function allows to control
    /// all synthesis parameters in real-time. The changes are additive,
    /// i.e. they add up to the existing parameter value. This function is
    /// similar to sending an NRPN message to the synthesizer. The
    /// function accepts a float as the value of the parameter. The
    /// parameter numbers and ranges are described in the SoundFont 2.01
    /// specification, paragraph 8.1.3, page 48.
    pub fn set_gen(
        &mut self,
        chan: usize,
        param: GeneratorType,
        value: f32,
    ) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(chan)?;

        crate::core::midi::set_gen(channel, &mut self.core.voices, param, value);

        Ok(())
    }

    /// Retrieve the value of a generator. This function returns the value
    /// set by a previous call 'set_gen()' or by an NRPN message.
    ///
    /// Returns the value of the generator.
    pub fn gen(&self, chan: u8, param: GeneratorType) -> Result<f32, OxiError> {
        let channel = self.core.channels.get(chan as usize)?;
        Ok(channel.gen(param))
    }
}

#[cfg(test)]
mod test {
    use crate::{SoundFont, Synth, SynthDescriptor};

    #[test]
    fn font_and_preset() {
        let mut synth = Synth::new(SynthDescriptor::default()).unwrap();
        assert_eq!(synth.sound_font_count(), 0);

        // Load first font
        let sin = {
            let mut file = std::fs::File::open("../testdata/sin.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.sound_font_count(), 1);

            let font = synth.sound_font(id).unwrap();

            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Sine Wave");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
            id
        };

        // Load next font
        let boom = {
            let mut file = std::fs::File::open("../testdata/Boomwhacker.sf2").unwrap();
            let font = SoundFont::load(&mut file).unwrap();

            let id = synth.add_font(font, true);

            assert_eq!(synth.sound_font_count(), 2);

            let font = synth.sound_font(id).unwrap();
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
            let font = synth.sound_font(sin).unwrap();
            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Sine Wave");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
        }
        // Check Boomwhacker ID
        {
            let font = synth.sound_font(boom).unwrap();
            let preset = font.preset(0, 0).unwrap();

            assert_eq!(preset.name(), "Boomwhacker");
            assert_eq!(preset.banknum(), 0);
            assert_eq!(preset.num(), 0);
        }
    }
}
