pub(crate) mod loader;

use ::soundfont::data::hydra::sample::SampleLink;
use loader::{PresetData, SoundFontData};

use std::io::{Read, Seek};
use std::path::Path;
use std::rc::Rc;

use generational_arena::Index;

#[derive(Clone)]
pub struct Preset {
    pub(crate) data: Rc<PresetData>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundFontId(pub(crate) Index);

impl SoundFontId {
    pub fn inner(&self) -> Index {
        self.0
    }
}

pub struct SoundFont {
    data: SoundFontData,
}

impl SoundFont {
    pub(crate) fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ()> {
        SoundFontData::load(file).map(|defsfont| Self { data: defsfont })
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
