use std::{f64::consts::PI, sync::LazyLock};

use crate::GeneratorType;

use super::{EnvelopeStep, Voice};
pub const FLUID_LOOP_UNTIL_RELEASE: LoopMode = 3;
pub const FLUID_LOOP_DURING_RELEASE: LoopMode = 1;
const INTERP_MAX: usize = 256;
const SINC_INTERP_ORDER: usize = 7; /* 7th order constant */

pub type LoopMode = u32;

pub struct DspFloatGlobal {
    interp_coeff_linear: [[f32; 2]; 256],
    interp_coeff: [[f32; 4]; 256],
    sinc_table7: [[f32; 7]; 256],
}
impl DspFloatGlobal {
    /// Initializes interpolation tables
    fn new() -> Self {
        // Initialize the coefficients for the interpolation. The math comes
        // from a mail, posted by Olli Niemitalo to the music-dsp mailing
        // list (http://www.smartelectronix.com/musicdsp).

        let mut interp_coeff = [[0.0; 4]; INTERP_MAX];
        let mut interp_coeff_linear = [[0.0; 2]; INTERP_MAX];

        for (i, (coeff, coeff_linear)) in interp_coeff
            .iter_mut()
            .zip(interp_coeff_linear.iter_mut())
            .enumerate()
        {
            let x = i as f64 / INTERP_MAX as f64;
            coeff[0] = (x * (-0.5 + x * (1.0 - 0.5 * x))) as f32;
            coeff[1] = (1.0 + x * x * (1.5 * x - 2.5)) as f32;
            coeff[2] = (x * (0.5 + x * (2.0 - 1.5 * x))) as f32;
            coeff[3] = (0.5 * x * x * (x - 1.0)) as f32;

            coeff_linear[0] = (1.0 - x) as f32;
            coeff_linear[1] = x as f32;
        }

        let mut sinc_table7 = [[0.0; 7]; INTERP_MAX];

        // i: Offset in terms of whole samples
        for i in 0..SINC_INTERP_ORDER {
            // i2: Offset in terms of fractional samples ('subsamples')
            let mut i2 = 0;
            while i2 < INTERP_MAX as i32 {
                // center on middle of table
                let i_shifted = i as f64 - 7.0 / 2.0 + i2 as f64 / INTERP_MAX as f64;

                // sinc(0) cannot be calculated straightforward (limit needed for 0/0)
                let v = if i_shifted.abs() > 0.000001 {
                    let mut v = f64::sin(i_shifted * PI) as f32 as f64 / (PI * i_shifted);
                    // Hamming window
                    v *= 0.5 * (1.0 + f64::cos(2.0 * PI * i_shifted / 7.0));
                    v
                } else {
                    1.0
                };

                sinc_table7[(INTERP_MAX as i32 - i2 - 1) as usize][i] = v as f32;
                i2 += 1
            }
        }

        Self {
            interp_coeff_linear,
            interp_coeff,
            sinc_table7,
        }
    }
}

static DSP_FLOAT_GLOBAL: LazyLock<DspFloatGlobal> = LazyLock::new(DspFloatGlobal::new);

/// Return the index and the fractional part, respectively.
#[inline(always)]
fn phase_fract(dsp_phase: usize) -> usize {
    dsp_phase & 0xffffffff
}

/* Purpose:
 * Takes the fractional part of the argument phase and
 * calculates the corresponding position in the interpolation table.
 * The fractional position of the playing pointer is calculated with a quite high
 * resolution (32 bits). It would be unpractical to keep a set of interpolation
 * coefficients for each possible fractional part...
 */
#[inline(always)]
fn phase_fract_to_tablerow(dsp_phase: usize) -> usize {
    const INTERP_BITS_MASK: usize = 0xff000000;
    const INTERP_BITS_SHIFT: usize = 24;
    (phase_fract(dsp_phase) & INTERP_BITS_MASK) >> INTERP_BITS_SHIFT
}

/// Purpose:
///
/// Sets the phase a to a phase increment given in b.
/// For example, assume b is 0.9. After setting a to it, adding a to
/// the playing pointer will advance it by 0.9 samples.
#[inline(always)]
fn phase_set_float(b: f32) -> u64 {
    const FRACT_MAX: f64 = 4294967296.0;

    let float = b as f64;
    let double = b as f64;
    let int = b as i32;

    let left = (float as u64) << 32i32;
    let right = ((double - (int as f64)) * FRACT_MAX) as u64;
    left | right
}

impl Voice {
    /// No interpolation. Just take the sample, which is closest to
    /// the playback pointer.  Questionable quality, but very
    /// efficient.
    pub fn dsp_float_interpolate_none(
        &mut self,
        dsp_buf: &mut [f32; 64],
        dsp_amp_incr: f32,
        phase_incr: f32,
    ) -> usize {
        let mut dsp_phase = self.phase;
        let dsp_data = &self.sample.data;
        let mut dsp_amp = self.amp;

        /* Convert playback "speed" floating point value to phase index/fract */
        let dsp_phase_incr = phase_set_float(phase_incr);

        /* voice is currently looping? */
        let looping = (self.gen[GeneratorType::SampleMode].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GeneratorType::SampleMode].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < EnvelopeStep::Release) as i32;

        let end_index = if looping != 0 {
            self.loopend - 1
        } else {
            self.end
        } as usize;

        let mut dsp_i: usize = 0;
        loop {
            /* round to nearest point */
            let mut dsp_phase_index = (dsp_phase.wrapping_add(0x80000000) >> 32) as usize;

            /* interpolate sequence of sample points */
            while dsp_i < 64 && dsp_phase_index <= end_index {
                dsp_buf[dsp_i] = dsp_amp * dsp_data[dsp_phase_index] as f32;

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                /* round to nearest point */
                dsp_phase_index = (dsp_phase.wrapping_add(0x80000000) >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            /* break out if not looping (buffer may not be full) */
            if looping == 0 {
                break;
            }
            /* go back to loop start */
            if dsp_phase_index > end_index {
                dsp_phase = dsp_phase.wrapping_sub(((self.loopend - self.loopstart) as u64) << 32);
                self.has_looped = true;
            }

            /* break out if filled buffer */
            if dsp_i >= 64 {
                break;
            }
        }

        self.phase = dsp_phase;
        self.amp = dsp_amp;

        dsp_i
    }

    /// Straight line interpolation.
    /// Returns number of samples processed (usually FLUID_BUFSIZE but could be
    /// smaller if end of sample occurs).
    pub fn dsp_float_interpolate_linear(
        &mut self,
        dsp_buf: &mut [f32; 64],
        dsp_amp_incr: f32,
        phase_incr: f32,
    ) -> usize {
        let mut dsp_phase = self.phase;
        let dsp_data: &[i16] = &self.sample.data;
        let mut dsp_amp: f32 = self.amp;

        /* Convert playback "speed" floating point value to phase index/fract */
        let dsp_phase_incr = phase_set_float(phase_incr);

        /* voice is currently looping? */
        let looping = (self.gen[GeneratorType::SampleMode].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GeneratorType::SampleMode].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < EnvelopeStep::Release) as i32;

        /* last index before 2nd interpolation point must be specially handled */
        let mut end_index = if looping != 0 {
            self.loopend - 1 - 1
        } else {
            self.end - 1
        } as usize;

        /* 2nd interpolation point to use at end of loop or sample */
        let point = if looping != 0 {
            /* loop start */
            dsp_data[self.loopstart as usize]
        } else {
            /* duplicate end for samples no longer looping */
            dsp_data[self.end as usize]
        };

        let mut dsp_i: usize = 0;
        loop {
            let mut dsp_phase_index = (dsp_phase >> 32) as usize;

            /* interpolate the sequence of sample points */
            while dsp_i < 64 && dsp_phase_index <= end_index {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff_linear[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32);
                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* break out if buffer filled */
            if dsp_i >= 64 {
                break;
            }
            /* we're now interpolating the last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff_linear[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index] as f32 + coeffs[1] * point as f32);
                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                /* increment amplitude */
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            /* break out if not looping (end of sample) */
            if looping == 0 {
                break;
            }

            /* go back to loop start (if past */
            if dsp_phase_index > end_index {
                dsp_phase = dsp_phase.wrapping_sub(((self.loopend - self.loopstart) as u64) << 32);
                self.has_looped = true;
            }

            /* break out if filled buffer */
            if dsp_i >= 64 {
                break;
            }

            /* set end back to second to last sample point */
            end_index = end_index.wrapping_sub(1)
        }
        self.phase = dsp_phase;
        self.amp = dsp_amp;

        dsp_i
    }

    /// 4th order (cubic) interpolation.
    /// Returns number of samples processed (usually FLUID_BUFSIZE but could be
    /// smaller if end of sample occurs).
    pub fn dsp_float_interpolate_4th_order(
        &mut self,
        dsp_buf: &mut [f32; 64],
        dsp_amp_incr: f32,
        phase_incr: f32,
    ) -> usize {
        let mut dsp_phase = self.phase;
        let dsp_data: &[i16] = &self.sample.data;
        let mut dsp_amp: f32 = self.amp;
        let end_point1: i16;
        let end_point2: i16;

        /* Convert playback "speed" floating point value to phase index/fract */
        let dsp_phase_incr = phase_set_float(phase_incr);

        /* voice is currently looping? */
        let looping = (self.gen[GeneratorType::SampleMode].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GeneratorType::SampleMode].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < EnvelopeStep::Release) as i32;

        /* last index before 4th interpolation point must be specially handled */
        let mut end_index = if looping != 0 {
            self.loopend - 1 - 2
        } else {
            self.end - 2
        } as usize;

        let mut start_index: usize;
        let mut start_point: i16;

        if self.has_looped {
            /* set start_index and start point if looped or not */
            start_index = self.loopstart as usize;
            /* last point in loop (wrap around) */
            start_point = dsp_data[(self.loopend - 1) as usize];
        } else {
            start_index = self.start as usize;
            /* just duplicate the point */
            start_point = dsp_data[self.start as usize];
        }

        /* get points off the end (loop start if looping, duplicate point if end) */
        if looping != 0 {
            end_point1 = dsp_data[self.loopstart as usize];
            end_point2 = dsp_data[self.loopstart as usize + 1];
        } else {
            end_point1 = dsp_data[self.end as usize];
            end_point2 = end_point1
        }

        let mut dsp_i: usize = 0;
        loop {
            let mut dsp_phase_index = (dsp_phase >> 32) as usize;
            /* interpolate first sample point (start or loop start) if needed */
            while dsp_phase_index == start_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * start_point as f32
                        + coeffs[1] * dsp_data[dsp_phase_index] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* interpolate the sequence of sample points */
            while dsp_i < 64 && dsp_phase_index <= end_index {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* break out if buffer filled */
            if dsp_i >= 64 {
                break;
            }

            /* we're now interpolating the 2nd to last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within 2nd to last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[3] * end_point1 as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            /* we're now interpolating the last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within the last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.interp_coeff[id];

                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index] as f32
                        + coeffs[2] * end_point1 as f32
                        + coeffs[3] * end_point2 as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* break out if not looping (end of sample) */
            if looping == 0 {
                break;
            }

            /* go back to loop start */
            if dsp_phase_index > end_index {
                dsp_phase = dsp_phase.wrapping_sub(((self.loopend - self.loopstart) as u64) << 32);
                if !self.has_looped {
                    self.has_looped = true;
                    start_index = self.loopstart as usize;
                    start_point = dsp_data[(self.loopend - 1) as usize];
                }
            }

            /* break out if filled buffer */
            if dsp_i >= 64 {
                break;
            }

            /* set end back to third to last sample point */
            end_index = end_index.wrapping_sub(2)
        }
        self.phase = dsp_phase;
        self.amp = dsp_amp;

        dsp_i
    }

    pub fn dsp_float_interpolate_7th_order(
        &mut self,
        dsp_buf: &mut [f32; 64],
        dsp_amp_incr: f32,
        phase_incr: f32,
    ) -> usize {
        let dsp_data: &[i16] = &self.sample.data;
        let mut dsp_amp: f32 = self.amp;

        /* Convert playback "speed" floating point value to phase index/fract */
        let dsp_phase_incr = phase_set_float(phase_incr);

        let dsp_phase = self.phase;
        /* add 1/2 sample to dsp_phase since 7th order interpolation is centered on
         * the 4th sample point */
        let mut dsp_phase = dsp_phase.wrapping_add(0x80000000);

        /* voice is currently looping? */
        let looping = (self.gen[GeneratorType::SampleMode].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GeneratorType::SampleMode].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < EnvelopeStep::Release) as i32;

        /* last index before 7th interpolation point must be specially handled */
        let mut end_index = ((if looping != 0 {
            self.loopend - 1
        } else {
            self.end
        }) - 3) as usize;

        let mut start_index: usize;
        let mut start_points: [i16; 3] = [0; 3];
        let mut end_points: [i16; 3] = [0; 3];

        if self.has_looped {
            /* set start_index and start point if looped or not */

            start_index = self.loopstart as usize;
            start_points[0] = dsp_data[(self.loopend - 1) as usize];
            start_points[1] = dsp_data[(self.loopend - 2) as usize];
            start_points[2] = dsp_data[(self.loopend - 3) as usize];
        } else {
            start_index = self.start as usize;
            /* just duplicate the start point */
            start_points[0] = dsp_data[self.start as usize];
            start_points[1] = start_points[0];
            start_points[2] = start_points[0]
        }

        /* get the 3 points off the end (loop start if looping, duplicate point if end) */
        if looping != 0 {
            end_points[0] = dsp_data[self.loopstart as usize];
            end_points[1] = dsp_data[(self.loopstart + 1) as usize];
            end_points[2] = dsp_data[(self.loopstart + 2) as usize];
        } else {
            end_points[0] = dsp_data[self.end as usize];
            end_points[1] = end_points[0];
            end_points[2] = end_points[0]
        }

        let mut dsp_i: usize = 0;
        let mut dsp_phase_index: usize;
        loop {
            dsp_phase_index = (dsp_phase >> 32) as usize;

            /* interpolate first sample point (start or loop start) if needed */
            while dsp_phase_index == start_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * start_points[2] as f32
                        + coeffs[1] * start_points[1] as f32
                        + coeffs[2] * start_points[0] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32
                        + coeffs[6] * dsp_data[dsp_phase_index.wrapping_add(3)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            start_index = start_index.wrapping_add(1);

            /* interpolate 2nd to first sample point (start or loop start) if needed */
            while dsp_phase_index == start_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * start_points[1] as f32
                        + coeffs[1] * start_points[0] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32
                        + coeffs[6] * dsp_data[dsp_phase_index.wrapping_add(3)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            start_index = start_index.wrapping_add(1);

            /* interpolate 3rd to first sample point (start or loop start) if needed */
            while dsp_phase_index == start_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * start_points[0] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_sub(2)] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32
                        + coeffs[6] * dsp_data[dsp_phase_index.wrapping_add(3)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* set back to original start index */
            start_index = start_index.wrapping_sub(2);

            /* interpolate the sequence of sample points */
            while dsp_i < 64 && dsp_phase_index <= end_index {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(3)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_sub(2)] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32
                        + coeffs[6] * dsp_data[dsp_phase_index.wrapping_add(3)] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* break out if buffer filled */
            if dsp_i >= 64 {
                break;
            }

            /* we're now interpolating the 3rd to last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within 3rd to last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(3)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_sub(2)] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * dsp_data[dsp_phase_index.wrapping_add(2)] as f32
                        + coeffs[6] * end_points[0] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* we're now interpolating the 2nd to last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within 2nd to last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(3)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_sub(2)] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * dsp_data[dsp_phase_index.wrapping_add(1)] as f32
                        + coeffs[5] * end_points[0] as f32
                        + coeffs[6] * end_points[1] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* we're now interpolating the last point */
            end_index = end_index.wrapping_add(1);

            /* interpolate within last point */
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let id = phase_fract_to_tablerow(dsp_phase as usize);
                let coeffs = &DSP_FLOAT_GLOBAL.sinc_table7[id];
                dsp_buf[dsp_i] = dsp_amp
                    * (coeffs[0] * dsp_data[dsp_phase_index.wrapping_sub(3)] as f32
                        + coeffs[1] * dsp_data[dsp_phase_index.wrapping_sub(2)] as f32
                        + coeffs[2] * dsp_data[dsp_phase_index.wrapping_sub(1)] as f32
                        + coeffs[3] * dsp_data[dsp_phase_index] as f32
                        + coeffs[4] * end_points[0] as f32
                        + coeffs[5] * end_points[1] as f32
                        + coeffs[6] * end_points[2] as f32);

                /* increment phase and amplitude */
                dsp_phase = dsp_phase.wrapping_add(dsp_phase_incr);
                dsp_phase_index = (dsp_phase >> 32) as usize;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }

            /* break out if not looping (end of sample) */
            if looping == 0 {
                break;
            }

            /* go back to loop start */
            if dsp_phase_index > end_index {
                dsp_phase = dsp_phase.wrapping_sub(((self.loopend - self.loopstart) as u64) << 32);

                if !self.has_looped {
                    self.has_looped = true;
                    start_index = self.loopstart as usize;
                    start_points[0] = dsp_data[(self.loopend - 1) as usize];
                    start_points[1] = dsp_data[(self.loopend - 2) as usize];
                    start_points[2] = dsp_data[(self.loopend - 3) as usize];
                }
            }

            /* break out if filled buffer */
            if dsp_i >= 64 {
                break;
            }

            /* set end back to 4th to last sample point */
            end_index = end_index.wrapping_sub(3)
        }

        /* sub 1/2 sample from dsp_phase since 7th order interpolation is centered on
         * the 4th sample point (correct back to real value) */
        let dsp_phase = dsp_phase.wrapping_sub(0x80000000);

        self.phase = dsp_phase;
        self.amp = dsp_amp;

        dsp_i
    }
}
