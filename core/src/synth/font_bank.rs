use std::rc::Rc;

use crate::{bank::BankOffsets, soundfont::Preset, utils::TypedArena, SoundFont, TypedIndex};

pub struct FontBank {
    fonts: TypedArena<SoundFont>,
    stack: Vec<TypedIndex<SoundFont>>,

    pub bank_offsets: BankOffsets,
}

impl FontBank {
    pub fn new() -> Self {
        Self {
            fonts: TypedArena::new(),
            stack: Vec::new(),
            bank_offsets: BankOffsets::default(),
        }
    }

    pub fn add_font(&mut self, font: SoundFont) -> TypedIndex<SoundFont> {
        let id = self.fonts.insert(font);

        // Put SoundFont on top of the stack
        self.stack.insert(0, id);

        id
    }

    pub fn remove_font(&mut self, id: TypedIndex<SoundFont>) -> Option<SoundFont> {
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
    pub fn get_font(&self, id: TypedIndex<SoundFont>) -> Option<&SoundFont> {
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

    pub fn get_preset(
        &self,
        sfont_id: TypedIndex<SoundFont>,
        banknum: u32,
        prognum: u8,
    ) -> Option<Rc<Preset>> {
        let sfont = self.get_font(sfont_id);
        if let Some(sfont) = sfont {
            let offset = self
                .bank_offsets
                .get(sfont_id)
                .map(|o| o.offset)
                .unwrap_or_default();
            sfont.get_preset(banknum.wrapping_sub(offset as u32), prognum)
        } else {
            None
        }
    }

    pub fn find_preset(
        &self,
        banknum: u32,
        prognum: u8,
    ) -> Option<(TypedIndex<SoundFont>, Rc<Preset>)> {
        for id in self.stack.iter() {
            let sfont = self.get_font(*id);
            if let Some(sfont) = sfont {
                let offset = self
                    .bank_offsets
                    .get(*id)
                    .map(|o| o.offset)
                    .unwrap_or_default();

                let preset = sfont.get_preset(banknum.wrapping_sub(offset), prognum);
                if let Some(preset) = preset {
                    return Some((*id, preset));
                }
            }
        }
        None
    }
}
