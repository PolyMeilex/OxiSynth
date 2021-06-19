use crate::soundfont::SoundFont;
use crate::synth::Synth;
use crate::utils::TypedIndex;

impl Synth {
    fn update_presets(&mut self) {
        for id in 0..self.channels.len() {
            let sfontnum = self.channels[id].sfontnum();
            if let Some(sfontnum) = sfontnum {
                let banknum = self.channels[id].banknum();
                let prognum = self.channels[id].prognum();

                let preset = self.font_bank.preset(sfontnum, banknum, prognum);
                self.channels[id].set_preset(preset);
            }
        }
    }
}

impl Synth {
    /**
    Loads a SoundFont. The newly
    loaded SoundFont will be put on top of the SoundFont
    stack. Presets are searched starting from the SoundFont on the
    top of the stack, working the way down the stack until a preset
    is found.
     */
    pub fn add_font(&mut self, font: SoundFont, reset_presets: bool) -> TypedIndex<SoundFont> {
        let id = self.font_bank.add_font(font);

        if reset_presets {
            self.program_reset();
        }

        id
    }

    /**
    Removes a SoundFont from the stack and deallocates it.
     */
    pub fn remove_font(
        &mut self,
        id: TypedIndex<SoundFont>,
        reset_presets: bool,
    ) -> Result<(), ()> {
        let sfont = self.font_bank.remove_font(id);

        if sfont.is_some() {
            if reset_presets {
                self.program_reset();
            } else {
                self.update_presets();
            }

            Ok(())
        } else {
            log::error!("No SoundFont with id = {:?}", id);

            Err(())
        }
    }

    /**
    Count the number of loaded SoundFonts.
     */
    pub fn count_fonts(&self) -> usize {
        self.font_bank.count()
    }

    /**
    Get a SoundFont. The SoundFont is specified by its index on the
    stack. The top of the stack has index zero.

    - `num` The number of the SoundFont (0 <= num < sfcount)
     */
    pub fn nth_font(&self, num: usize) -> Option<&SoundFont> {
        self.font_bank.get_nth_font(num)
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn sfont(&self, id: TypedIndex<SoundFont>) -> Option<&SoundFont> {
        self.font_bank.get_font(id)
    }
}
