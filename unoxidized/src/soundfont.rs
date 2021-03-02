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
            .preset
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
    // TODO: Rc<Vec<i16>>
    pub data: Vec<i16>,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
}

impl Sample {
    fn import_sfont(
        sfsample: &sf2::data::SFSampleHeader,
        sfont: &DefaultSoundFont,
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
            // TODO: Rc<Vec<i16>>
            data: sfont.sampledata.clone(),

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

    unsafe fn optimize_sample(&mut self) {
        let mut peak_max: i16 = 0 as i32 as i16;
        let mut peak_min: i16 = 0 as i32 as i16;
        let mut peak;
        let normalized_amplitude_during_loop;
        let result;
        let mut i;
        if self.valid == 0 || self.sampletype & 0x10 as i32 != 0 {
            return;
        }
        if self.amplitude_that_reaches_noise_floor_is_valid == 0 {
            i = self.loopstart as i32;
            while i < self.loopend as i32 {
                let val: i16 = *self.data.as_ptr().offset(i as isize);
                if val as i32 > peak_max as i32 {
                    peak_max = val
                } else if (val as i32) < peak_min as i32 {
                    peak_min = val
                }
                i += 1
            }
            if peak_max as i32 > -(peak_min as i32) {
                peak = peak_max
            } else {
                peak = -(peak_min as i32) as i16
            }
            if peak as i32 == 0 as i32 {
                peak = 1 as i32 as i16
            }
            normalized_amplitude_during_loop = (peak as f32 as f64 / 32768.0f64) as f32;
            result = 0.00003f64 / normalized_amplitude_during_loop as f64;
            self.amplitude_that_reaches_noise_floor = result;
            self.amplitude_that_reaches_noise_floor_is_valid = 1 as i32
        }
    }
}
