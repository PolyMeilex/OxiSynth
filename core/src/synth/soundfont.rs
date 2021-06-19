pub mod generator;
mod instrument;
pub(crate) mod modulator;
mod preset;
mod sample;
mod sample_data;

use std::{
    io::{Read, Seek},
    rc::Rc,
};

pub(crate) use {
    instrument::InstrumentZone, preset::PresetZone, sample::Sample, sample_data::SampleData,
};

pub use preset::Preset;

pub struct SoundFont {
    presets: Vec<Rc<Preset>>,
}

impl SoundFont {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ()> {
        let sf2 = soundfont::SoundFont2::load(file);

        let sf2 = match sf2 {
            Ok(data) => data,
            Err(err) => {
                log::error!("{:#?}", err);
                return Err(());
            }
        };

        #[cfg(feature = "sf3")]
        let ver = 3;
        #[cfg(not(feature = "sf3"))]
        let ver = 2;

        if sf2.info.version.major > ver {
            log::error!("Unsupported version: {:?}", sf2.info.version);
            return Err(());
        }

        let sf2 = sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let sample_pos = smpl.offset() + 8;
        let sample_size = smpl.len() as usize;

        let sample_data = Rc::new(SampleData::load(file, sample_pos, sample_size)?);

        let mut samples = Vec::new();

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import(sfsample, sample_data.clone())?.optimize_sample();
            samples.push(Rc::new(sample));
        }

        let mut presets = Vec::new();
        for sfpreset in sf2.presets.iter() {
            let preset = Preset::import(&sf2, sfpreset, &samples)?;
            presets.push(Rc::new(preset));
        }

        Ok(Self { presets })
    }

    pub fn preset(&self, bank: u32, prenum: u8) -> Option<Rc<Preset>> {
        self.presets
            .iter()
            .find(|p| p.banknum() == bank && p.num() == prenum as u32)
            .cloned()
    }
}
