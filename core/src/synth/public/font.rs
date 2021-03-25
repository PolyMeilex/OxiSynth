use crate::soundfont::SoundFontId;
use crate::synth::SoundFont;
use crate::synth::Synth;

use std::io::{Read, Seek};

impl Synth {
    /**
    Loads a SoundFont file and creates a new SoundFont. The newly
    loaded SoundFont will be put on top of the SoundFont
    stack. Presets are searched starting from the SoundFont on the
    top of the stack, working the way down the stack until a preset
    is found.
     */
    pub fn sfload<F: Read + Seek>(
        &mut self,
        file: &mut F,
        reset_presets: bool,
    ) -> Result<SoundFontId, ()> {
        let sfont = SoundFont::load(file);

        match sfont {
            Ok(sfont) => {
                let id = self.sfont.insert(sfont);

                if reset_presets {
                    self.program_reset();
                }

                Ok(SoundFontId(id))
            }
            Err(err) => {
                log::error!("Failed to load SoundFont");
                Err(err)
            }
        }
    }

    /**
    Removes a SoundFont from the stack and deallocates it.
     */
    pub fn sfunload(&mut self, id: SoundFontId, reset_presets: bool) -> Result<(), ()> {
        let sfont = self.sfont.remove(id.0);
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
    Reload a SoundFont. The reloaded SoundFont retains its ID and
    index on the stack.
     */
    pub fn sfreload(&mut self, _id: u32) -> Result<u32, ()> {
        unimplemented!("sfreload");
        // let index = self
        //     .sfont
        //     .iter()
        //     .position(|x| x.id == id)
        //     .expect("SoundFont with ID");

        // if let Ok(_) = self.sfunload(id, false) {
        //     let sfont = &self.sfont[index];
        //     let filename = sfont.get_name();
        //     match SoundFont::load(filename) {
        //         Ok(mut sfont) => {
        //             sfont.id = id;
        //             self.sfont.insert(index, sfont);
        //             self.update_presets();
        //             return Ok(id);
        //         }
        //         Err(_) => {
        //             log::error!("Failed to load SoundFont '{:?}'", sfont.get_name());
        //             Err(())
        //         }
        //     }
        // } else {
        //     Err(())
        // }
    }

    /**
    Remove a SoundFont that was previously added using
    fluid_synth_add_sfont(). The synthesizer does not delete the
    SoundFont; this is responsability of the caller.
     */
    pub fn remove_sfont(&mut self, id: SoundFontId) {
        self.sfont.remove(id.0);
        // self.sfont.retain(|s| s.id != id);
        self.remove_bank_offset(id);
        self.program_reset();
    }

    /**
    Count the number of loaded SoundFonts.
     */
    pub fn sfcount(&self) -> usize {
        self.sfont.len()
    }

    /**
    Get a SoundFont. The SoundFont is specified by its index on the
    stack. The top of the stack has index zero.

    - `num` The number of the SoundFont (0 <= num < sfcount)
     */
    pub fn get_nth_sfont(&self, num: usize) -> Option<&SoundFont> {
        // self.sfont.get(num)
        unimplemented!("get_sfont");
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn get_sfont(&self, id: SoundFontId) -> Option<&SoundFont> {
        self.sfont.get(id.0)
    }
}
