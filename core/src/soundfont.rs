pub(crate) mod loader;

use ::soundfont::data::hydra::sample::SampleLink;
use loader::{PresetData, SoundFontData};

use std::io::{Read, Seek};
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct Preset {
    pub(crate) data: Rc<PresetData>,
    pub sfont_id: SoundFontId,
}

impl Preset {
    pub fn get_name(&self) -> &str {
        &self.data.name
    }

    pub fn get_banknum(&self) -> u32 {
        self.data.bank
    }

    pub fn get_num(&self) -> u32 {
        self.data.num
    }
}

// TODO: this probbably should be `generational-arena` based
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundFontId(pub(crate) usize);

impl SoundFontId {
    pub fn inner(&self) -> usize {
        self.0
    }
}

pub struct SoundFont {
    data: SoundFontData,
    pub(crate) id: SoundFontId,
}

impl SoundFont {
    pub(crate) fn load<F: Read + Seek>(file: &mut F, id: usize) -> Result<Self, ()> {
        SoundFontData::load(file).map(|defsfont| Self {
            data: defsfont,
            id: SoundFontId(id),
        })
    }

    pub fn get_id(&self) -> SoundFontId {
        self.id
    }

    pub fn get_name(&self) -> &Path {
        &self.data.filename
    }

    pub fn get_preset(&self, bank: u32, prenum: u8) -> Option<Preset> {
        let defpreset = self
            .data
            .presets
            .iter()
            .find(|p| p.bank == bank && p.num == prenum as u32);

        if let Some(defpreset) = defpreset {
            let preset = Preset {
                sfont_id: self.id,
                data: defpreset.clone(),
            };

            Some(preset)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub(crate) struct Sample {
    // pub name: [u8; 21],
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub loopstart: u32,
    pub loopend: u32,
    pub samplerate: u32,
    pub origpitch: i32,
    pub pitchadj: i32,
    pub sampletype: SampleLink,
    pub valid: bool,
    pub data: Rc<Vec<i16>>,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
}
