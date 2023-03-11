use std::sync::Arc;

use soundfont::data::hydra::sample::SampleLink;

use super::SampleData;

#[derive(Clone, Debug)]
pub struct Sample {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub loop_start: u32,
    pub loop_end: u32,
    pub sample_rate: u32,
    pub origpitch: u8,
    pub pitchadj: i8,
    pub sample_type: SampleLink,
    pub valid: bool,
    pub data: Arc<SampleData>,
    pub amplitude_that_reaches_noise_floor_is_valid: i32,
    pub amplitude_that_reaches_noise_floor: f64,
}

impl Sample {
    pub fn import(
        sample: &soundfont::data::SampleHeader,
        data: Arc<SampleData>,
    ) -> Result<Sample, ()> {
        let mut sample = Sample {
            name: sample.name.clone(),
            start: sample.start,
            end: sample.end,
            loop_start: sample.loop_start,
            loop_end: sample.loop_end,
            sample_rate: sample.sample_rate,
            origpitch: sample.origpitch,
            pitchadj: sample.pitchadj,
            sample_type: sample.sample_type,
            valid: true,
            data,

            amplitude_that_reaches_noise_floor_is_valid: 0,
            amplitude_that_reaches_noise_floor: 0.0,
        };

        #[cfg(feature = "sf3")]
        {
            use byte_slice_cast::AsByteSlice;

            if sample.sample_type.is_vorbis() {
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
                sample.data = Rc::new(SampleData::new(new));

                // loop is fowled?? (cluck cluck :)
                if sample.loop_end > sample.end
                    || sample.loop_start >= sample.loop_end
                    || sample.loop_start <= sample.start
                {
                    // can pad loop by 8 samples and ensure at least 4 for loop (2*8+4)
                    if (sample.end - sample.start) >= 20 {
                        sample.loop_start = sample.start + 8;
                        sample.loop_end = sample.end - 8;
                    } else {
                        // loop is fowled, sample is tiny (can't pad 8 samples)
                        sample.loop_start = sample.start + 1;
                        sample.loop_end = sample.end - 1;
                    }
                }

                // Mark it as no longer compresed sample
                sample.sample_type = match sample.sample_type {
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
        } else if sample.sample_type.is_rom() {
            sample.valid = false;
            log::warn!("Ignoring sample: can't use ROM samples");
            // TODO: It's not realy "Ok"
            Ok(sample)
        } else {
            Ok(sample)
        }
    }

    /// - Scan the loop
    /// - determine the peak level
    /// - Calculate, what factor will make the loop inaudible
    /// - Store in sample
    pub fn optimize_sample(mut self) -> Self {
        if !self.valid || self.sample_type.is_vorbis() {
            return self;
        }
        if self.amplitude_that_reaches_noise_floor_is_valid == 0 {
            let mut peak_max = 0;
            let mut peak_min = 0;

            /* Scan the loop */
            for i in self.loop_start..self.loop_end {
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
            self.amplitude_that_reaches_noise_floor_is_valid = 1;
        }

        self
    }
}
