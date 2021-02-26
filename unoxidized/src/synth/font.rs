use crate::synth::SoundFont;
use crate::synth::Synth;

use std::path::Path;

impl Synth {
    pub fn sfload(&mut self, filename: &Path, reset_presets: bool) -> Result<u32, ()> {
        let sfont = self.loader.load(filename);

        match sfont {
            Ok(mut sfont) => {
                self.sfont_id = self.sfont_id.wrapping_add(1);
                sfont.id = self.sfont_id;
                self.sfont.insert(0, sfont);
                if reset_presets {
                    self.program_reset();
                }
                Ok(self.sfont_id)
            }
            Err(err) => {
                log::error!("Failed to load SoundFont '{:?}'", filename);
                Err(err)
            }
        }
    }

    pub fn sfunload(&mut self, id: u32, reset_presets: bool) -> Result<(), ()> {
        let sfont = self.get_sfont_by_id(id);
        if let Some(id) = sfont.map(|sfont| sfont.id) {
            self.sfont.retain(|s| s.id != id);
            if reset_presets {
                self.program_reset();
            } else {
                self.update_presets();
            }

            Ok(())
        } else {
            log::error!("No SoundFont with id = {}", id);

            Err(())
        }
    }

    pub fn sfreload(&mut self, id: u32) -> Result<u32, ()> {
        let index = self
            .sfont
            .iter()
            .position(|x| x.id == id)
            .expect("SoundFont with ID");

        if let Ok(_) = self.sfunload(id, false) {
            let sfont = &self.sfont[index];
            let filename = sfont.get_name();
            match self.loader.load(filename) {
                Ok(mut sfont) => {
                    sfont.id = id;
                    self.sfont.insert(index, sfont);
                    self.update_presets();
                    return Ok(id);
                }
                Err(_) => {
                    log::error!("Failed to load SoundFont '{:?}'", sfont.get_name());
                    Err(())
                }
            }
        } else {
            Err(())
        }
    }

    pub fn remove_sfont(&mut self, id: u32) {
        self.sfont.retain(|s| s.id != id);
        self.remove_bank_offset(id);
        self.program_reset();
    }

    pub fn sfcount(&self) -> usize {
        self.sfont.len()
    }

    pub fn get_sfont(&self, num: u32) -> Option<&SoundFont> {
        self.sfont.get(num as usize)
    }

    pub fn get_sfont_by_id(&self, id: u32) -> Option<&SoundFont> {
        self.sfont.iter().find(|x| x.id == id)
    }

    pub fn get_sfont_by_id_mut(&mut self, id: u32) -> Option<&mut SoundFont> {
        self.sfont.iter_mut().find(|x| x.id == id)
    }
}
