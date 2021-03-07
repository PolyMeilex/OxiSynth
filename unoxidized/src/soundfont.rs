mod sfloader;

use sfloader::DefaultPreset;
use sfloader::DefaultSoundFont;
use soundfont_rs as sf2;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct Preset {
    data: Rc<DefaultPreset>,
    pub sfont_id: u32,
}

impl Preset {
    pub fn get_name(&self) -> String {
        self.data.name.clone()
    }

    pub fn get_banknum(&self) -> u32 {
        self.data.bank
    }

    pub fn get_num(&self) -> u32 {
        self.data.num
    }
}

pub struct SoundFont {
    data: DefaultSoundFont,
    pub id: u32,
}

impl SoundFont {
    pub(crate) fn load(filename: &Path) -> Result<Self, ()> {
        DefaultSoundFont::load(filename).map(|defsfont| Self {
            data: defsfont,
            id: 0,
        })
    }

    pub fn get_name(&self) -> &Path {
        &self.data.filename
    }

    pub fn get_preset(&self, bank: u32, prenum: u32) -> Option<Preset> {
        let defpreset = self
            .data
            .presets
            .iter()
            .find(|p| p.bank == bank && p.num == prenum);

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
pub struct Sample {
    // pub name: [u8; 21],
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub loopstart: u32,
    pub loopend: u32,
    pub samplerate: u32,
    pub origpitch: i32,
    pub pitchadj: i32,
    pub sampletype: i32,
    pub valid: i32,
    pub data: Rc<Vec<i16>>,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
}

impl Sample {
    fn import_sfont(
        sfsample: &sf2::data::SFSampleHeader,
        data: Rc<Vec<i16>>,
    ) -> Result<Sample, ()> {
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
