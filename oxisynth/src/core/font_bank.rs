use std::sync::Arc;

use crate::arena::{Arena, Index};
use crate::core::{soundfont::Preset, SoundFont};
use crate::SoundFontId;

#[derive(Default)]
pub(crate) struct FontBank {
    fonts: Arena<SoundFont>,
    // TODO: VecDeq
    stack: Vec<Index<SoundFont>>,

    pub(crate) bank_offsets: BankOffsets,
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

    pub fn remove_font(&mut self, id: SoundFontId) -> Option<SoundFont> {
        let sfont = self.fonts.remove(id);

        if let Some(pos) = self.stack.iter().position(|i| *i == id) {
            self.stack.remove(pos);
        }

        self.bank_offsets.remove(id);

        sfont
    }

    /// Count the number of loaded SoundFonts.
    pub fn count(&self) -> usize {
        self.fonts.len()
    }

    /// Get a SoundFont. The SoundFont is specified by its ID.
    pub fn font(&self, id: SoundFontId) -> Option<&SoundFont> {
        self.fonts.get(id)
    }

    /// Get a SoundFont. The SoundFont is specified by its index on the
    /// stack. The top of the stack has index zero.
    ///
    /// - `num` The number of the SoundFont (0 <= num < sfcount)
    pub fn nth_font(&self, num: usize) -> Option<&SoundFont> {
        let id = self.stack.get(num);
        if let Some(id) = id {
            self.fonts.get(*id)
        } else {
            None
        }
    }

    pub fn preset(&self, sfont_id: SoundFontId, banknum: u32, prognum: u8) -> Option<Arc<Preset>> {
        let sfont = self.font(sfont_id)?;
        let offset = self
            .bank_offsets
            .get(sfont_id)
            .map(|o| o.offset)
            .unwrap_or_default();

        let banknum = banknum.saturating_sub(offset);

        sfont.preset(banknum, prognum)
    }

    pub fn find_preset(&self, banknum: u32, prognum: u8) -> Option<(SoundFontId, Arc<Preset>)> {
        for id in self.stack.iter() {
            let sfont = self.font(*id);
            if let Some(sfont) = sfont {
                let offset = self
                    .bank_offsets
                    .get(*id)
                    .map(|o| o.offset)
                    .unwrap_or_default();

                let banknum = banknum.saturating_sub(offset);

                let preset = sfont.preset(banknum, prognum);
                if let Some(preset) = preset {
                    return Some((*id, preset));
                }
            }
        }
        None
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BankOffset {
    pub sfont_id: SoundFontId,
    pub offset: u32,
}

#[derive(Default)]
pub(crate) struct BankOffsets(Vec<BankOffset>);

impl BankOffsets {
    /// Get the offset of the bank numbers in a SoundFont.
    pub fn get(&self, sfont_id: SoundFontId) -> Option<&BankOffset> {
        self.0.iter().find(|x| x.sfont_id == sfont_id)
    }

    /// Get the offset of the bank numbers in a SoundFont.
    fn get_mut(&mut self, sfont_id: SoundFontId) -> Option<&mut BankOffset> {
        self.0.iter_mut().find(|x| x.sfont_id == sfont_id)
    }

    /// Offset the bank numbers in a SoundFont.
    pub fn set(&mut self, sfont_id: SoundFontId, offset: u32) {
        let bank_offset = self.get_mut(sfont_id);

        if let Some(bank_offset) = bank_offset {
            bank_offset.offset = offset
        } else {
            let bank_offset = BankOffset { sfont_id, offset };
            self.0.insert(0, bank_offset);
        }
    }

    fn remove(&mut self, sfont_id: SoundFontId) {
        if let Some(pos) = self.0.iter().position(|x| x.sfont_id == sfont_id) {
            self.0.remove(pos);
        }
    }
}
