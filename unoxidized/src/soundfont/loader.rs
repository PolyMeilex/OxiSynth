use soundfont as sf2;
use std::rc::Rc;

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use super::Sample;
use crate::generator::{self, Gen};
use crate::synth::modulator::Mod;

const GEN_SET: u32 = 1;

pub(crate) struct SoundFontData {
    pub filename: PathBuf,
    pub presets: Vec<Rc<PresetData>>,
}

impl SoundFontData {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ()> {
        let data = sf2::data::SFData::load(file);

        let data = match data {
            Ok(data) => data,
            Err(err) => {
                log::error!("{:#?}", err);
                return Err(());
            }
        };

        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let sample_pos = smpl.offset() + 8;
        let sample_size = smpl.len() as usize;

        let sample_data = Rc::new(Self::load_sampledata(file, sample_pos, sample_size)?);

        let mut samples = Vec::new();

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import_sfont(sfsample, sample_data.clone())?;
            let mut sample = sample;

            sample.optimize_sample();

            samples.push(Rc::new(sample));
        }

        let mut presets = Vec::new();
        for sfpreset in sf2.presets.iter() {
            let preset = PresetData::import(&sf2, sfpreset, &samples)?;
            presets.push(Rc::new(preset));
        }

        Ok(Self {
            filename: PathBuf::from(""),
            presets,
        })
    }

    fn load_sampledata<F: Read + Seek>(
        file: &mut F,
        sample_pos: u64,
        sample_size: usize,
    ) -> Result<Vec<i16>, ()> {
        if file.seek(SeekFrom::Start(sample_pos)).is_err() {
            log::error!("Failed to seek position in data file",);
            return Err(());
        }

        let mut sample_data = vec![0u8; sample_size];
        if file.read_exact(&mut sample_data).is_err() {
            log::error!("Failed to read sample data");
            return Err(());
        }

        let sample_data: Vec<i16> = sample_data
            .chunks(2)
            .map(|num| {
                if num.len() == 2 {
                    i16::from_le_bytes([num[0], num[1]])
                } else {
                    log::error!("Wrong sample data");
                    0
                }
            })
            .collect();

        Ok(sample_data)
    }
}

pub(crate) struct PresetData {
    pub name: String,
    pub bank: u32,
    pub num: u32,
    pub global_zone: Option<PresetZone>,
    pub zones: Vec<PresetZone>,
}

impl PresetData {
    fn import(
        sf2: &sf2::SoundFont2,
        sfpreset: &sf2::Preset,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut preset = Self {
            name: String::new(),
            bank: 0 as i32 as u32,
            num: 0 as i32 as u32,
            global_zone: None,
            zones: Vec::new(),
        };

        if sfpreset.header.name.len() != 0 {
            preset.name = sfpreset.header.name.clone();
        } else {
            preset.name = format!(
                "Bank:{},Preset{}",
                sfpreset.header.bank, sfpreset.header.preset
            );
        }

        preset.bank = sfpreset.header.bank as u32;
        preset.num = sfpreset.header.preset as u32;

        for (id, sfzone) in sfpreset.zones.iter().enumerate() {
            let name = format!("{}/{}", sfpreset.header.name, id);
            let zone = PresetZone::import(name, sf2, sfzone, samples)?;

            if id == 0 && zone.inst.is_none() {
                preset.global_zone = Some(zone);
            } else {
                preset.zones.push(zone);
            }
        }

        Ok(preset)
    }
}

pub(crate) struct PresetZone {
    #[allow(dead_code)]
    pub name: String,
    pub inst: Option<Instrument>,
    pub keylo: u8,
    pub keyhi: u8,
    pub vello: i32,
    pub velhi: i32,
    pub gen: [Gen; 60],
    pub mods: Vec<Mod>,
}

impl PresetZone {
    fn import(
        name: String,
        sf2: &sf2::SoundFont2,
        sfzone: &sf2::Zone,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut zone = Self {
            name,
            inst: None,
            keylo: 0,
            keyhi: 128,
            vello: 0 as i32,
            velhi: 128 as i32,
            gen: generator::get_default_values(),
            mods: Vec::new(),
        };

        for sfgen in sfzone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::Instrument)
        {
            match sfgen.ty {
                sf2::data::GeneratorType::KeyRange | sf2::data::GeneratorType::VelRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    zone.keylo = amount.low;
                    zone.keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    zone.gen[sfgen.ty as usize].val = *sfgen.amount.as_i16().unwrap() as f64;
                    zone.gen[sfgen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }
        if let Some(id) = sfzone.instrument() {
            let inst = Instrument::import(sf2, &sf2.instruments[*id as usize], samples)?;

            zone.inst = Some(inst);
        }
        // Import the modulators (only SF2.1 and higher)
        for mod_src in sfzone.mod_list.iter() {
            let mod_dest = Mod::from(mod_src);

            /* Store the new modulator in the zone The order of modulators
             * will make a difference, at least in an instrument context: The
             * second modulator overwrites the first one, if they only differ
             * in amount. */
            zone.mods.push(mod_dest);
        }

        Ok(zone)
    }
}

#[derive(Clone)]
pub(crate) struct Instrument {
    pub name: String,
    pub global_zone: Option<InstrumentZone>,
    pub zones: Vec<InstrumentZone>,
}

impl Instrument {
    fn import(
        sf2: &sf2::SoundFont2,
        new_inst: &sf2::Instrument,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut inst = Self {
            name: String::new(),
            global_zone: None,
            zones: Vec::new(),
        };

        if new_inst.header.name.len() > 0 {
            inst.name = new_inst.header.name.clone();
        } else {
            inst.name = "<untitled>".into();
        }
        for (id, new_zone) in new_inst.zones.iter().enumerate() {
            let name = format!("{}/{}", new_inst.header.name, id);
            let zone = InstrumentZone::import(name, sf2, new_zone, samples)?;
            if id == 0 && zone.sample.is_none() {
                inst.global_zone = Some(zone);
            } else {
                inst.zones.push(zone);
            }
        }

        Ok(inst)
    }
}

#[derive(Clone)]
#[repr(C)]
pub(crate) struct InstrumentZone {
    pub name: String,
    pub sample: Option<Rc<Sample>>,
    pub keylo: u8,
    pub keyhi: u8,
    pub vello: i32,
    pub velhi: i32,
    pub gen: [Gen; 60],
    pub mods: Vec<Mod>,
}

impl InstrumentZone {
    fn import(
        name: String,
        sf2: &sf2::SoundFont2,
        new_zone: &sf2::Zone,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<InstrumentZone, ()> {
        let mut keylo = 0;
        let mut keyhi = 128;
        let mut gen = generator::get_default_values();

        for new_gen in new_zone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::SampleID)
        {
            match new_gen.ty {
                sf2::data::GeneratorType::KeyRange | sf2::data::GeneratorType::VelRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    keylo = amount.low;
                    keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[new_gen.ty as usize].val = *new_gen.amount.as_i16().unwrap() as f64;
                    gen[new_gen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }

        let sample = if let Some(sample_id) = new_zone.sample() {
            let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();
            let name = &sample.name;

            // Find Sample by name:
            let sample = samples
                .iter()
                .find(|sample| &sample.name == name)
                .map(|s| s.clone());

            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }
            sample
        } else {
            None
        };

        let mut mods = Vec::new();

        for new_mod in new_zone.mod_list.iter() {
            let mod_dest = Mod::from(new_mod);
            /* Store the new modulator in the zone
             * The order of modulators will make a difference, at least in an instrument context:
             * The second modulator overwrites the first one, if they only differ in amount. */
            mods.push(mod_dest);
        }

        Ok(InstrumentZone {
            name: name,
            sample,
            keylo,
            keyhi,
            vello: 0,
            velhi: 128,
            gen,
            mods,
        })
    }
}

impl Sample {
    fn import_sfont(sfsample: &sf2::data::SampleHeader, data: Rc<Vec<i16>>) -> Result<Sample, ()> {
        let mut sample = Sample {
            name: sfsample.name.clone(),
            start: sfsample.start,
            end: sfsample.end,
            loopstart: sfsample.loop_start,
            loopend: sfsample.loop_end,
            samplerate: sfsample.sample_rate,
            origpitch: sfsample.origpitch as i32,
            pitchadj: sfsample.pitchadj as i32,
            sampletype: sfsample.sample_type as i32,
            valid: 1,
            data,

            amplitude_that_reaches_noise_floor_is_valid: 0,
            amplitude_that_reaches_noise_floor: 0.0,
        };

        if (sample.sampletype & 0x10 as i32) != 0 {
            // vorbis?
            return Ok(sample);
        }
        if sample.sampletype & 0x8000 as i32 != 0 {
            sample.valid = 0 as i32;
            log::warn!("Ignoring sample: can\'t use ROM samples",);
        }
        if sample.end.wrapping_sub(sample.start) < 8 as i32 as u32 {
            sample.valid = 0 as i32;
            log::warn!("Ignoring sample: too few sample data points",);
        }

        return Ok(sample);
    }

    fn optimize_sample(&mut self) {
        if self.valid == 0 || self.sampletype & 0x10 != 0 {
            return;
        }
        if self.amplitude_that_reaches_noise_floor_is_valid == 0 {
            let mut peak_max = 0;
            let mut peak_min = 0;

            for i in self.loopstart..self.loopend {
                let val = self.data[i as usize] as i32;
                if val > peak_max {
                    peak_max = val
                } else if val < peak_min {
                    peak_min = val
                }
            }

            let peak = if peak_max > -peak_min {
                peak_max
            } else {
                -peak_min
            };

            let peak = if peak == 0 { 1 } else { peak };

            let normalized_amplitude_during_loop = peak as f32 / 32768.0;
            let result = 0.00003 / normalized_amplitude_during_loop as f64;
            self.amplitude_that_reaches_noise_floor = result;
            self.amplitude_that_reaches_noise_floor_is_valid = 1 as i32
        }
    }
}
