use super::Voice;
pub type Phase = u64;
pub type GenType = u32;
pub const GEN_SAMPLEMODE: GenType = 54;
pub type VoiceEnvelopeIndex = u32;
pub const FLUID_VOICE_ENVRELEASE: VoiceEnvelopeIndex = 5;
pub const FLUID_LOOP_UNTIL_RELEASE: LoopMode = 3;
pub const FLUID_LOOP_DURING_RELEASE: LoopMode = 1;
pub type LoopMode = u32;

pub struct DspFloatGlobal {
    interp_coeff_linear: [[f32; 2]; 256],
    interp_coeff: [[f32; 4]; 256],
    sinc_table7: [[f32; 7]; 256],
}
impl DspFloatGlobal {
    fn new() -> Self {
        let mut global = DspFloatGlobal {
            interp_coeff_linear: [[0.; 2]; 256],
            interp_coeff: [[0.; 4]; 256],
            sinc_table7: [[0.; 7]; 256],
        };

        let mut i: usize;
        let mut i2: i32;
        let mut x: f64;
        let mut v: f64;
        let mut i_shifted: f64;
        i = 0 as usize;
        while i < 256 {
            x = i as f64 / 256 as i32 as f64;
            global.interp_coeff[i][0] = (x * (-0.5f64 + x * (1 as i32 as f64 - 0.5f64 * x))) as f32;
            global.interp_coeff[i][1] = (1.0f64 + x * x * (1.5f64 * x - 2.5f64)) as f32;
            global.interp_coeff[i][2] = (x * (0.5f64 + x * (2.0f64 - 1.5f64 * x))) as f32;
            global.interp_coeff[i][3] = (0.5f64 * x * x * (x - 1.0f64)) as f32;
            global.interp_coeff_linear[i][0] = (1.0f64 - x) as f32;
            global.interp_coeff_linear[i][1] = x as f32;
            i += 1
        }
        i = 0;
        while i < 7 {
            i2 = 0 as i32;
            while i2 < 256 as i32 {
                i_shifted = i as f64 - 7 as i32 as f64 / 2.0f64 + i2 as f64 / 256 as i32 as f64;
                if f64::abs(i_shifted) > 0.000001f64 {
                    v = f64::sin(i_shifted * std::f64::consts::PI) as f32 as f64
                        / (std::f64::consts::PI * i_shifted);
                    v *= 0.5f64
                        * (1.0f64
                            + f64::cos(
                                2.0f64 * std::f64::consts::PI * i_shifted / 7 as i32 as f32 as f64,
                            ))
                } else {
                    v = 1.0f64
                }
                global.sinc_table7[(256 as i32 - i2 - 1 as i32) as usize][i as usize] = v as f32;
                i2 += 1
            }
            i += 1
        }

        global
    }
}

lazy_static! {
    static ref DSP_FLOAT_GLOBAL: DspFloatGlobal = DspFloatGlobal::new();
}

impl Voice {
    pub unsafe fn dsp_float_interpolate_none(&mut self, dsp_buf: &mut [f32; 64]) -> i32 {
        let dsp_buf = dsp_buf.as_mut_ptr();

        let mut dsp_phase: Phase = self.phase;
        let dsp_phase_incr: Phase;
        let dsp_data: &[i16] = &self.sample.as_ref().unwrap().data;
        let mut dsp_amp: f32 = self.amp;
        let dsp_amp_incr: f32 = self.amp_incr;
        let mut dsp_i: u32 = 0 as i32 as u32;
        let mut dsp_phase_index: u32;
        let end_index: u32;
        let looping: i32;
        dsp_phase_incr = (self.phase_incr as u64) << 32 as i32
            | ((self.phase_incr as f64 - self.phase_incr as f64) * 4294967296.0) as u64;
        looping = (self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < FLUID_VOICE_ENVRELEASE as i32) as i32;
        end_index = if looping != 0 {
            self.loopend - 1 as i32
        } else {
            self.end
        } as u32;
        loop {
            dsp_phase_index =
                (dsp_phase.wrapping_add(0x80000000 as u32 as u64) >> 32 as i32) as u32;
            while dsp_i < 64 as i32 as u32 && dsp_phase_index <= end_index {
                *dsp_buf.offset(dsp_i as isize) =
                    dsp_amp * dsp_data[dsp_phase_index as usize] as f32;
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index =
                    (dsp_phase.wrapping_add(0x80000000 as u32 as u64) >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if looping == 0 {
                break;
            }
            if dsp_phase_index > end_index {
                dsp_phase = (dsp_phase as u64)
                    .wrapping_sub(((self.loopend - self.loopstart) as u64) << 32 as i32)
                    as Phase as Phase;
                self.has_looped = 1 as i32
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
        }
        self.phase = dsp_phase;
        self.amp = dsp_amp;
        return dsp_i as i32;
    }

    pub unsafe fn dsp_float_interpolate_linear(&mut self, dsp_buf: &mut [f32; 64]) -> i32 {
        let dsp_buf = dsp_buf.as_mut_ptr();

        let mut dsp_phase: Phase = self.phase;
        let dsp_phase_incr: Phase;
        let dsp_data: &[i16] = &self.sample.as_ref().unwrap().data;
        let mut dsp_amp: f32 = self.amp;
        let dsp_amp_incr: f32 = self.amp_incr;
        let mut dsp_i: u32 = 0 as i32 as u32;
        let mut dsp_phase_index: u32;
        let mut end_index: u32;
        let looping: i32;
        dsp_phase_incr = (self.phase_incr as u64) << 32 as i32
            | ((self.phase_incr as f64 - self.phase_incr as i32 as f64) * 4294967296.0f64) as u32
                as u64;
        looping = (self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < FLUID_VOICE_ENVRELEASE as i32) as i32;
        end_index = ((if looping != 0 {
            (self.loopend) - 1 as i32
        } else {
            self.end
        }) - 1 as i32) as u32;

        let point = if looping != 0 {
            dsp_data[self.loopstart as usize]
        } else {
            dsp_data[self.end as usize]
        };

        loop {
            dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
            while dsp_i < 64 as i32 as u32 && dsp_phase_index <= end_index {
                let coeffs = DSP_FLOAT_GLOBAL.interp_coeff_linear[(((dsp_phase
                    & 0xffffffff as u32 as u64)
                    as u32
                    & 0xff000000 as u32)
                    >> 24 as i32)
                    as usize]
                    .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index as usize] as i32 as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1 as i32 as u32) as usize]
                                as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 {
                let coeffs = DSP_FLOAT_GLOBAL.interp_coeff_linear[(((dsp_phase
                    & 0xffffffff as u32 as u64)
                    as u32
                    & 0xff000000 as u32)
                    >> 24 as i32)
                    as usize]
                    .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(1 as i32 as isize) * point as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if looping == 0 {
                break;
            }
            if dsp_phase_index > end_index {
                dsp_phase = (dsp_phase as u64)
                    .wrapping_sub(((self.loopend - self.loopstart) as u64) << 32 as i32)
                    as Phase as Phase;
                self.has_looped = 1 as i32
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_sub(1)
        }
        self.phase = dsp_phase;
        self.amp = dsp_amp;
        return dsp_i as i32;
    }

    pub unsafe fn dsp_float_interpolate_4th_order(&mut self, dsp_buf: &mut [f32; 64]) -> i32 {
        let dsp_buf = dsp_buf.as_mut_ptr();

        let mut dsp_phase: Phase = (self).phase;
        let dsp_phase_incr: Phase;
        let dsp_data: &[i16] = &self.sample.as_ref().unwrap().data;
        let mut dsp_amp: f32 = (self).amp;
        let dsp_amp_incr: f32 = (self).amp_incr;
        let mut dsp_i: u32 = 0 as i32 as u32;
        let mut dsp_phase_index: u32;
        let mut start_index: u32;
        let mut end_index: u32;
        let mut start_point: i16;
        let end_point1: i16;
        let end_point2: i16;
        let looping: i32;
        dsp_phase_incr = ((self).phase_incr as u64) << 32 as i32
            | (((self).phase_incr as f64 - (self).phase_incr as i32 as f64) * 4294967296.0f64)
                as u32 as u64;
        looping = ((self).gen[GEN_SAMPLEMODE as i32 as usize].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || (self).gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_UNTIL_RELEASE as i32
                && (self).volenv_section < FLUID_VOICE_ENVRELEASE as i32) as i32;
        end_index = ((if looping != 0 {
            ((self).loopend) - 1 as i32
        } else {
            (self).end
        }) - 2 as i32) as u32;
        if (self).has_looped != 0 {
            start_index = (self).loopstart as u32;
            start_point = dsp_data[((self).loopend - 1 as i32) as usize];
        } else {
            start_index = (self).start as u32;
            start_point = dsp_data[(self).start as usize]
        }
        if looping != 0 {
            end_point1 = dsp_data[(self).loopstart as usize];
            end_point2 = dsp_data[((self).loopstart + 1 as i32) as usize];
        } else {
            end_point1 = dsp_data[(self).end as usize];
            end_point2 = end_point1
        }
        loop {
            dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
            while dsp_phase_index == start_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.interp_coeff[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize) * start_point as i32 as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as i32 as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1 as i32 as u32) as usize]
                                as i32 as f32
                        + *coeffs.offset(3 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(2 as i32 as u32) as usize]
                                as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            while dsp_i < 64 as i32 as u32 && dsp_phase_index <= end_index {
                let coeffs =
                    DSP_FLOAT_GLOBAL.interp_coeff[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(1 as i32 as u32) as usize] as i32
                            as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as i32 as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1 as i32 as u32) as usize]
                                as i32 as f32
                        + *coeffs.offset(3 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(2 as i32 as u32) as usize]
                                as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.interp_coeff[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(1 as i32 as u32) as usize] as i32
                            as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as i32 as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1 as i32 as u32) as usize]
                                as i32 as f32
                        + *coeffs.offset(3 as i32 as isize) * end_point1 as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.interp_coeff[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(1 as i32 as u32) as usize] as i32
                            as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as i32 as f32
                        + *coeffs.offset(2 as i32 as isize) * end_point1 as i32 as f32
                        + *coeffs.offset(3 as i32 as isize) * end_point2 as i32 as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if looping == 0 {
                break;
            }
            if dsp_phase_index > end_index {
                dsp_phase = (dsp_phase as u64)
                    .wrapping_sub((((self).loopend - (self).loopstart) as u64) << 32 as i32)
                    as Phase as Phase;
                if (self).has_looped == 0 {
                    (self).has_looped = 1 as i32;
                    start_index = (self).loopstart as u32;
                    start_point = dsp_data[(self.loopend - 1) as usize]
                }
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_sub(2 as i32 as u32)
        }
        (self).phase = dsp_phase;
        (self).amp = dsp_amp;
        return dsp_i as i32;
    }

    pub unsafe fn dsp_float_interpolate_7th_order(&mut self, dsp_buf: &mut [f32; 64]) -> i32 {
        let dsp_buf = dsp_buf.as_mut_ptr();

        let mut dsp_phase: Phase = (self).phase;
        let dsp_phase_incr: Phase;
        let dsp_data: &[i16] = &self.sample.as_ref().unwrap().data;
        let mut dsp_amp: f32 = (self).amp;
        let dsp_amp_incr: f32 = (self).amp_incr;
        let mut dsp_i: u32 = 0 as i32 as u32;
        let mut dsp_phase_index: u32;
        let mut start_index: u32;
        let mut end_index: u32;
        let mut start_points: [i16; 3] = [0; 3];
        let mut end_points: [i16; 3] = [0; 3];
        let looping: i32;
        dsp_phase_incr = (self.phase_incr as u64) << 32 as i32
            | ((self.phase_incr as f64 - (self).phase_incr as i32 as f64) * 4294967296.0f64) as u32
                as u64;
        dsp_phase = (dsp_phase as u64).wrapping_add(0x80000000 as u32 as Phase) as Phase as Phase;
        looping = (self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
            == FLUID_LOOP_DURING_RELEASE as i32
            || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_UNTIL_RELEASE as i32
                && self.volenv_section < FLUID_VOICE_ENVRELEASE as i32) as i32;
        end_index = ((if looping != 0 {
            self.loopend - 1
        } else {
            self.end
        }) - 3 as i32) as u32;
        if self.has_looped != 0 {
            start_index = self.loopstart as u32;
            start_points[0 as usize] = dsp_data[(self.loopend - 1) as usize];
            start_points[1 as usize] = dsp_data[(self.loopend - 2) as usize];
            start_points[2 as usize] = dsp_data[(self.loopend - 3) as usize];
        } else {
            start_index = self.start as u32;
            start_points[0 as usize] = dsp_data[self.start as usize];
            start_points[1 as usize] = start_points[0];
            start_points[2 as usize] = start_points[0]
        }
        if looping != 0 {
            end_points[0 as usize] = dsp_data[self.loopstart as usize];
            end_points[1 as usize] = dsp_data[(self.loopstart + 1) as usize];
            end_points[2 as usize] = dsp_data[(self.loopstart + 2) as usize];
        } else {
            end_points[0 as usize] = dsp_data[self.end as usize];
            end_points[1 as usize] = end_points[0];
            end_points[2 as usize] = end_points[0]
        }
        loop {
            dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
            while dsp_phase_index == start_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize) * start_points[2 as i32 as usize] as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * start_points[1 as i32 as usize] as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * start_points[0 as i32 as usize] as f32
                        + *coeffs.offset(3 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(2) as usize] as f32
                        + *coeffs.offset(6 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(3) as usize] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            start_index = start_index.wrapping_add(1);
            while dsp_phase_index == start_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0) * start_points[1] as f32
                        + *coeffs.offset(1) * start_points[0] as f32
                        + *coeffs.offset(2)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3) * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5)
                            * dsp_data[dsp_phase_index.wrapping_add(2) as usize] as f32
                        + *coeffs.offset(6)
                            * dsp_data[dsp_phase_index.wrapping_add(3) as usize] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            start_index = start_index.wrapping_add(1);
            while dsp_phase_index == start_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize) * start_points[0] as f32
                        + *coeffs.offset(1)
                            * dsp_data[dsp_phase_index.wrapping_sub(2) as usize] as f32
                        + *coeffs.offset(2)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3) * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5)
                            * dsp_data[dsp_phase_index.wrapping_add(2) as usize] as f32
                        + *coeffs.offset(6)
                            * dsp_data[dsp_phase_index.wrapping_add(3) as usize] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            start_index = start_index.wrapping_sub(2 as i32 as u32);
            while dsp_i < 64 as i32 as u32 && dsp_phase_index <= end_index {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(3) as usize] as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_sub(2) as usize] as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(2) as usize] as f32
                        + *coeffs.offset(6 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(3) as usize] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(3) as usize] as f32
                        + *coeffs.offset(1 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_sub(2) as usize] as f32
                        + *coeffs.offset(2 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3 as i32 as isize)
                            * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5 as i32 as isize)
                            * dsp_data[dsp_phase_index.wrapping_add(2) as usize] as f32
                        + *coeffs.offset(6 as i32 as isize) * end_points[0 as i32 as usize] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0)
                        * dsp_data[dsp_phase_index.wrapping_sub(3) as usize] as f32
                        + *coeffs.offset(1)
                            * dsp_data[dsp_phase_index.wrapping_sub(2) as usize] as f32
                        + *coeffs.offset(2)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3) * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4)
                            * dsp_data[dsp_phase_index.wrapping_add(1) as usize] as f32
                        + *coeffs.offset(5) * end_points[0] as f32
                        + *coeffs.offset(6) * end_points[1] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            end_index = end_index.wrapping_add(1);
            while dsp_phase_index <= end_index && dsp_i < 64 as i32 as u32 {
                let coeffs =
                    DSP_FLOAT_GLOBAL.sinc_table7[(((dsp_phase & 0xffffffff as u32 as u64) as u32
                        & 0xff000000 as u32)
                        >> 24 as i32) as usize]
                        .as_ptr();
                *dsp_buf.offset(dsp_i as isize) = dsp_amp
                    * (*coeffs.offset(0 as i32 as isize)
                        * dsp_data[dsp_phase_index.wrapping_sub(3) as usize] as f32
                        + *coeffs.offset(1)
                            * dsp_data[dsp_phase_index.wrapping_sub(2) as usize] as f32
                        + *coeffs.offset(2)
                            * dsp_data[dsp_phase_index.wrapping_sub(1) as usize] as f32
                        + *coeffs.offset(3) * dsp_data[dsp_phase_index as usize] as f32
                        + *coeffs.offset(4) * end_points[0] as f32
                        + *coeffs.offset(5) * end_points[1] as f32
                        + *coeffs.offset(6) * end_points[2] as f32);
                dsp_phase = (dsp_phase as u64).wrapping_add(dsp_phase_incr) as Phase as Phase;
                dsp_phase_index = (dsp_phase >> 32 as i32) as u32;
                dsp_amp += dsp_amp_incr;
                dsp_i = dsp_i.wrapping_add(1)
            }
            if looping == 0 {
                break;
            }
            if dsp_phase_index > end_index {
                dsp_phase = (dsp_phase as u64)
                    .wrapping_sub(((self.loopend - self.loopstart) as u64) << 32 as i32)
                    as Phase as Phase;
                if self.has_looped == 0 {
                    self.has_looped = 1 as i32;
                    start_index = self.loopstart as u32;
                    start_points[0] = dsp_data[(self.loopend - 1) as usize];
                    start_points[1] = dsp_data[(self.loopend - 2) as usize];
                    start_points[2] = dsp_data[(self.loopend - 3) as usize];
                }
            }
            if dsp_i >= 64 as i32 as u32 {
                break;
            }
            end_index = end_index.wrapping_sub(3 as i32 as u32)
        }
        dsp_phase = (dsp_phase as u64).wrapping_sub(0x80000000 as u32 as Phase) as Phase as Phase;
        (self).phase = dsp_phase;
        (self).amp = dsp_amp;
        return dsp_i as i32;
    }
}
