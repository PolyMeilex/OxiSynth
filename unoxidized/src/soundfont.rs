use crate::sfloader::DefaultPreset;
use crate::sfloader::DefaultSoundFont;

pub struct SoundFontLoader {}

#[derive(Copy, Clone)]
pub struct Preset {
    pub data: *mut DefaultPreset,
    pub sfont: *const SoundFont,
}

pub struct SoundFont {
    pub data: DefaultSoundFont,
    pub id: u32,
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
    pub data: *mut i16,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
    pub refcount: u32,
}

impl Sample {
    pub fn import_sfont(
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
            data: sfont.sampledata,

            amplitude_that_reaches_noise_floor_is_valid: 0,
            amplitude_that_reaches_noise_floor: 0.0,
            refcount: 0,
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
}
