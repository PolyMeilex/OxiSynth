use crate::{soundfont::SoundFont, TypedIndex};

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: TypedIndex<SoundFont>,
    pub offset: u32,
}

#[derive(Default)]
pub struct BankOffsets(Vec<BankOffset>);

impl BankOffsets {
    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get(&self, sfont_id: TypedIndex<SoundFont>) -> Option<&BankOffset> {
        self.0.iter().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    fn get_mut(&mut self, sfont_id: TypedIndex<SoundFont>) -> Option<&mut BankOffset> {
        self.0.iter_mut().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occured (out of memory or negative offset)
     */
    pub fn set(&mut self, sfont_id: TypedIndex<SoundFont>, offset: u32) {
        let bank_offset = self.get_mut(sfont_id);

        if let Some(mut bank_offset) = bank_offset {
            bank_offset.offset = offset
        } else {
            let bank_offset = BankOffset { sfont_id, offset };
            self.0.insert(0, bank_offset);
        }
    }

    pub fn remove(&mut self, sfont_id: TypedIndex<SoundFont>) {
        self.0.retain(|x| x.sfont_id != sfont_id);
    }
}
