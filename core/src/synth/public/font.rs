use crate::soundfont::SoundFont;
use crate::synth::Synth;
use crate::utils::TypedIndex;

impl Synth {
    /**
    Loads a SoundFont. The newly
    loaded SoundFont will be put on top of the SoundFont
    stack. Presets are searched starting from the SoundFont on the
    top of the stack, working the way down the stack until a preset
    is found.
     */
    pub fn add_font(&mut self, font: SoundFont, reset_presets: bool) -> TypedIndex<SoundFont> {
        let id = self.fonts.insert(font);

        // Put SoundFont on top of the stack
        self.fonts_stack.insert(0, id);

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
        let sfont = self.fonts.remove(id);
        self.fonts_stack.retain(|i| i == &id);

        if let Some(_) = sfont {
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
        self.fonts.len()
    }

    /**
    Get a SoundFont. The SoundFont is specified by its index on the
    stack. The top of the stack has index zero.

    - `num` The number of the SoundFont (0 <= num < sfcount)
     */
    pub fn get_nth_font(&self, num: usize) -> Option<&SoundFont> {
        let id = self.fonts_stack.get(num);
        if let Some(id) = id {
            self.fonts.get(*id)
        } else {
            None
        }
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn get_sfont(&self, id: TypedIndex<SoundFont>) -> Option<&SoundFont> {
        self.fonts.get(id)
    }
}
