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

        #[cfg(feature = "sf3")]
        let ver = 3;
        #[cfg(not(feature = "sf3"))]
        let ver = 2;

        if data.info.version.major > ver {
            log::error!("Unsupported version: {:?}", data.info.version);
            return Err(());
        }

        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let sample_pos = smpl.offset() + 8;
        let sample_size = smpl.len() as usize;

        let sample_data = Rc::new(Self::load_sampledata(file, sample_pos, sample_size)?);

        let mut samples = Vec::new();

        for sfsample in sf2.sample_headers.iter() {
            let mut sample = Sample::import_sfont(sfsample, sample_data.clone())?;

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

        use byteorder::{LittleEndian, ReadBytesExt};

        let mut sample_data = vec![0i16; sample_size / 2];
        if file
            .read_i16_into::<LittleEndian>(&mut sample_data)
            .is_err()
        {
            log::error!("Failed to read sample data");
            return Err(());
        }

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
        let name = if sfpreset.header.name.len() != 0 {
            sfpreset.header.name.clone()
        } else {
            format!(
                "Bank:{},Preset{}",
                sfpreset.header.bank, sfpreset.header.preset
            )
        };

        let mut global_zone = None;
        let mut zones = Vec::new();

        for (id, sfzone) in sfpreset.zones.iter().enumerate() {
            let name = format!("{}/{}", sfpreset.header.name, id);
            let zone = PresetZone::import(name, sf2, sfzone, samples)?;

            if id == 0 && zone.inst.is_none() {
                global_zone = Some(zone);
            } else {
                zones.push(zone);
            }
        }

        Ok(Self {
            name,
            bank: sfpreset.header.bank as u32,
            num: sfpreset.header.preset as u32,
            global_zone,
            zones,
        })
    }
}

pub(crate) struct PresetZone {
    #[allow(dead_code)]
    pub name: String,
    pub inst: Option<Instrument>,
    pub keylo: u8,
    pub keyhi: u8,
    pub vello: u8,
    pub velhi: u8,
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
        let mut keylo = 0;
        let mut keyhi = 128;
        let mut vello = 0;
        let mut velhi = 128;

        let mut gen = generator::get_default_values();

        for sfgen in sfzone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::Instrument)
        {
            match sfgen.ty {
                sf2::data::GeneratorType::KeyRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    keylo = amount.low;
                    keyhi = amount.high;
                }
                sf2::data::GeneratorType::VelRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    vello = amount.low;
                    velhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[sfgen.ty as usize].val = *sfgen.amount.as_i16().unwrap() as f64;
                    gen[sfgen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }

        let inst = if let Some(id) = sfzone.instrument() {
            let i = Instrument::import(sf2, &sf2.instruments[*id as usize], samples)?;
            Some(i)
        } else {
            None
        };

        // Import the modulators (only SF2.1 and higher)
        let mods: Vec<_> = sfzone
            .mod_list
            .iter()
            .map(|mod_src| {
                /* Store the new modulator in the zone The order of modulators
                 * will make a difference, at least in an instrument context: The
                 * second modulator overwrites the first one, if they only differ
                 * in amount. */
                Mod::from(mod_src)
            })
            .collect();

        Ok(Self {
            name,
            inst,
            keylo,
            keyhi,
            vello,
            velhi,
            gen,
            mods,
        })
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
        let name = if new_inst.header.name.len() > 0 {
            new_inst.header.name.clone()
        } else {
            "<untitled>".into()
        };

        let mut global_zone = None;
        let mut zones = Vec::new();

        for (id, new_zone) in new_inst.zones.iter().enumerate() {
            let name = format!("{}/{}", new_inst.header.name, id);
            let zone = InstrumentZone::import(name, sf2, new_zone, samples)?;
            if id == 0 && zone.sample.is_none() {
                global_zone = Some(zone);
            } else {
                zones.push(zone);
            }
        }

        Ok(Self {
            name,
            global_zone,
            zones,
        })
    }
}

#[derive(Clone)]
#[repr(C)]
pub(crate) struct InstrumentZone {
    pub name: String,
    pub sample: Option<Rc<Sample>>,
    pub keylo: u8,
    pub keyhi: u8,
    pub vello: u8,
    pub velhi: u8,
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
        let mut vello = 0;
        let mut velhi = 128;

        let mut gen = generator::get_default_values();

        for new_gen in new_zone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::SampleID)
        {
            match new_gen.ty {
                sf2::data::GeneratorType::KeyRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    keylo = amount.low;
                    keyhi = amount.high;
                }
                sf2::data::GeneratorType::VelRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    vello = amount.low;
                    velhi = amount.high;
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

            // if let Some(sample) = &sample {
            //     println!("{:?}", sample.start);
            // }

            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }

            sample
        } else {
            None
        };

        let mods = new_zone
            .mod_list
            .iter()
            .map(|new_mod| {
                /* Store the new modulator in the zone
                 * The order of modulators will make a difference, at least in an instrument context:
                 * The second modulator overwrites the first one, if they only differ in amount. */
                Mod::from(new_mod)
            })
            .collect();

        Ok(Self {
            name,
            sample,
            keylo,
            keyhi,
            vello,
            velhi,
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
            sampletype: sfsample.sample_type,
            valid: true,
            data,

            amplitude_that_reaches_noise_floor_is_valid: 0,
            amplitude_that_reaches_noise_floor: 0.0,
        };

        #[cfg(feature = "sf3")]
        {
            use byte_slice_cast::AsByteSlice;

            if sample.sampletype.is_vorbis() {
                let start = sample.start as usize;
                let end = sample.end as usize;

                let sample_data = sample.data.as_byte_slice();

                use lewton::inside_ogg::OggStreamReader;
                use std::io::Cursor;

                let buf = Cursor::new(&sample_data[start..end]);

                let mut reader = OggStreamReader::new(buf).unwrap();

                let mut new = Vec::new();

                while let Some(mut pck) = reader.read_dec_packet().unwrap() {
                    new.append(&mut pck[0]);
                }

                sample.start = 0;
                sample.end = (new.len() - 1) as u32;
                sample.data = Rc::new(new);

                // loop is fowled?? (cluck cluck :)
                if sample.loopend > sample.end
                    || sample.loopstart >= sample.loopend
                    || sample.loopstart <= sample.start
                {
                    // can pad loop by 8 samples and ensure at least 4 for loop (2*8+4)
                    if (sample.end - sample.start) >= 20 {
                        sample.loopstart = sample.start + 8;
                        sample.loopend = sample.end - 8;
                    } else {
                        // loop is fowled, sample is tiny (can't pad 8 samples)
                        sample.loopstart = sample.start + 1;
                        sample.loopend = sample.end - 1;
                    }
                }

                // Mark it as no longer compresed sample
                use sf2::data::sample::SampleLink;
                sample.sampletype = match sample.sampletype {
                    SampleLink::VorbisMonoSample => SampleLink::MonoSample,
                    SampleLink::VorbisRightSample => SampleLink::RightSample,
                    SampleLink::VorbisLeftSample => SampleLink::LeftSample,
                    SampleLink::VorbisLinkedSample => SampleLink::LinkedSample,
                    _ => unreachable!("Not Vorbis"),
                };
            }
        }

        if sample.end - sample.start < 8 {
            sample.valid = false;
            log::warn!(
                "Ignoring sample {:?}: too few sample data points",
                sample.name
            );
            Ok(sample)
        } else {
            if sample.sampletype.is_rom() {
                sample.valid = false;
                log::warn!("Ignoring sample: can't use ROM samples");
                // TODO: It's not realy "Ok"
                Ok(sample)
            } else {
                Ok(sample)
            }
        }
    }

    /// - Scan the loop
    /// - determine the peak level
    /// - Calculate, what factor will make the loop inaudible
    /// - Store in sample
    fn optimize_sample(&mut self) {
        if self.valid == false || self.sampletype.is_vorbis() {
            return;
        }
        if self.amplitude_that_reaches_noise_floor_is_valid == 0 {
            let mut peak_max = 0;
            let mut peak_min = 0;

            /* Scan the loop */
            for i in self.loopstart..self.loopend {
                let val = self.data[i as usize] as i32;
                if val > peak_max {
                    peak_max = val
                } else if val < peak_min {
                    peak_min = val
                }
            }

            /* Determine the peak level */
            let peak = if peak_max > -peak_min {
                peak_max
            } else {
                -peak_min
            };

            /* Avoid division by zero */
            let peak = if peak == 0 { 1 } else { peak };

            /* Calculate what factor will make the loop inaudible
             * For example: Take a peak of 3277 (10 % of 32768).  The
             * normalized amplitude is 0.1 (10 % of 32768).  An amplitude
             * factor of 0.0001 (as opposed to the default 0.00001) will
             * drop this sample to the noise floor.
             */

            /* 16 bits => 96+4=100 dB dynamic range => 0.00001 */
            let normalized_amplitude_during_loop = peak as f32 / 32768.0;
            let result = 0.00003 / normalized_amplitude_during_loop as f64;

            /* Store in sample */
            self.amplitude_that_reaches_noise_floor = result;
            self.amplitude_that_reaches_noise_floor_is_valid = 1 as i32
        }
    }
}
