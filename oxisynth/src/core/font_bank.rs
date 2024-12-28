use std::sync::Arc;

use crate::arena::{Arena, Index};
use crate::core::{soundfont::Preset, SoundFont};

#[derive(Default)]
pub struct FontBank {
    fonts: Arena<SoundFont>,
    stack: Vec<Index<SoundFont>>,

    pub bank_offsets: BankOffsets,
}

impl FontBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_font(&mut self, font: SoundFont) -> Index<SoundFont> {
        let id = self.fonts.insert(font);

        // Put SoundFont on top of the stack
        self.stack.insert(0, id);

        id
    }

    pub fn remove_font(&mut self, id: Index<SoundFont>) -> Option<SoundFont> {
        let sfont = self.fonts.remove(id);
        self.stack.retain(|i| i == &id);
        sfont
    }

    /**
    Count the number of loaded SoundFonts.
     */
    pub fn count(&self) -> usize {
        self.fonts.len()
    }

    /**
    Get a SoundFont. The SoundFont is specified by its ID.
     */
    pub fn get_font(&self, id: Index<SoundFont>) -> Option<&SoundFont> {
        self.fonts.get(id)
    }

    /**
    Get a SoundFont. The SoundFont is specified by its index on the
    stack. The top of the stack has index zero.

    - `num` The number of the SoundFont (0 <= num < sfcount)
     */
    pub fn get_nth_font(&self, num: usize) -> Option<&SoundFont> {
        let id = self.stack.get(num);
        if let Some(id) = id {
            self.fonts.get(*id)
        } else {
            None
        }
    }

    pub fn iter_stack(&self) -> impl Iterator<Item = &SoundFont> {
        self.stack.iter().filter_map(move |f| self.fonts.get(*f))
    }

    pub fn preset(
        &self,
        sfont_id: Index<SoundFont>,
        banknum: u32,
        prognum: u8,
    ) -> Option<Arc<Preset>> {
        let sfont = self.get_font(sfont_id);
        if let Some(sfont) = sfont {
            let offset = self
                .bank_offsets
                .get(sfont_id)
                .map(|o| o.offset)
                .unwrap_or_default();
            sfont.preset(banknum.wrapping_sub(offset), prognum)
        } else {
            None
        }
    }

    pub fn find_preset(
        &self,
        banknum: u32,
        prognum: u8,
    ) -> Option<(Index<SoundFont>, Arc<Preset>)> {
        for id in self.stack.iter() {
            let sfont = self.get_font(*id);
            if let Some(sfont) = sfont {
                let offset = self
                    .bank_offsets
                    .get(*id)
                    .map(|o| o.offset)
                    .unwrap_or_default();

                let preset = sfont.preset(banknum.wrapping_sub(offset), prognum);
                if let Some(preset) = preset {
                    return Some((*id, preset));
                }
            }
        }
        None
    }
}

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: Index<SoundFont>,
    pub offset: u32,
}

#[derive(Default)]
pub struct BankOffsets(Vec<BankOffset>);

impl BankOffsets {
    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get(&self, sfont_id: Index<SoundFont>) -> Option<&BankOffset> {
        self.0.iter().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    fn get_mut(&mut self, sfont_id: Index<SoundFont>) -> Option<&mut BankOffset> {
        self.0.iter_mut().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occurred (out of memory or negative offset)
     */
    pub fn set(&mut self, sfont_id: Index<SoundFont>, offset: u32) {
        let bank_offset = self.get_mut(sfont_id);

        if let Some(bank_offset) = bank_offset {
            bank_offset.offset = offset
        } else {
            let bank_offset = BankOffset { sfont_id, offset };
            self.0.insert(0, bank_offset);
        }
    }

    pub fn remove(&mut self, sfont_id: Index<SoundFont>) {
        self.0.retain(|x| x.sfont_id != sfont_id);
    }
}
