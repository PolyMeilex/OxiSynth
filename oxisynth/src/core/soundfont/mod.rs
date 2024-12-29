pub mod generator;
mod instrument;
pub(crate) mod modulator;
mod preset;
mod sample;
mod sample_data;

use std::{
    io::{Read, Seek},
    sync::Arc,
};

use crate::error::LoadError;

pub(crate) use {
    instrument::InstrumentZone, preset::PresetZone, sample::Sample, sample_data::SampleData,
};

pub use preset::Preset;

pub struct SoundFont {
    presets: Vec<Arc<Preset>>,
}

impl SoundFont {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, LoadError> {
        let sf2 = soundfont::SoundFont2::load(file)?;

        #[cfg(feature = "sf3")]
        let max_ver = 3;
        #[cfg(not(feature = "sf3"))]
        let max_ver = 2;

        if sf2.info.version.major > max_ver {
            return Err(LoadError::Version {
                version: sf2.info.version,
                max: max_ver,
            });
        }

        let sf2 = sf2.sort_presets();

        let sample_data = SampleData::load(file, sf2.sample_data.smpl.as_ref().unwrap())?;

        let mut samples = Vec::new();

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import(sfsample, sample_data.clone())?;
            samples.push(sample);
        }

        let mut presets = Vec::new();
        for sfpreset in sf2.presets.iter() {
            let preset = Preset::import(&sf2, sfpreset, &samples)?;
            presets.push(Arc::new(preset));
        }

        Ok(Self { presets })
    }

    pub(crate) fn preset(&self, bank: u32, prenum: u8) -> Option<Arc<Preset>> {
        self.presets
            .iter()
            .find(|p| p.banknum() == bank && p.num() == prenum as u32)
            .cloned()
    }
}
