use crate::synth::Synth;
use crate::synth::FLUID_SYNTH_PLAYING;

lazy_static! {
    static ref RAND_TABLE: [[f32; 48000]; 2] = {
        let mut rand: [[f32; 48000]; 2] = [[0.; 48000]; 2];

        for c in 0..2 {
            let mut dp = 0.0;
            for i in 0..(48000 - 1) {
                let r: i32 = rand::random();
                let d = r as f32 / 2147483647.0 - 0.5;
                rand[c][i] = d - dp;
                dp = d;
            }
            rand[c][48000 - 1] = 0.0 - dp;
        }
        rand
    };
}

impl Synth {
    fn one_block(&mut self, do_not_mix_fx_to_out: i32) {
        // clean the audio buffers
        {
            for i in 0..self.nbuf {
                self.left_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
                self.right_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
            }

            for i in 0..2 {
                self.fx_left_buf[i as usize]
                    .iter_mut()
                    .for_each(|v| *v = 0.0);
                self.fx_right_buf[i as usize]
                    .iter_mut()
                    .for_each(|v| *v = 0.0);
            }
        }

        /* Set up the reverb / chorus buffers only, when the effect is
         * enabled on synth level.  Nonexisting buffers are detected in the
         * DSP loop. Not sending the reverb / chorus signal saves some time
         * in that case. */

        /* call all playing synthesis processes */
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_playing() {
                /* The output associated with a MIDI channel is wrapped around
                 * using the number of audio groups as modulo divider.  This is
                 * typically the number of output channels on the 'sound card',
                 * as long as the LADSPA Fx unit is not used. In case of LADSPA
                 * unit, think of it as subgroups on a mixer.
                 *
                 * For example: Assume that the number of groups is set to 2.
                 * Then MIDI channel 1, 3, 5, 7 etc. go to output 1, channels 2,
                 * 4, 6, 8 etc to output 2.  Or assume 3 groups: Then MIDI
                 * channels 1, 4, 7, 10 etc go to output 1; 2, 5, 8, 11 etc to
                 * output 2, 3, 6, 9, 12 etc to output 3.
                 */
                let mut auchan = self.channel[voice.get_channel().unwrap().0].get_num();
                auchan %= self.settings.synth.audio_groups as u8;

                voice.write(
                    &self.channel,
                    self.min_note_length_ticks,
                    &mut self.left_buf[auchan as usize],
                    &mut self.right_buf[auchan as usize],
                    &mut self.fx_left_buf,
                    self.settings.synth.reverb_active,
                    self.settings.synth.chorus_active,
                );
            }
        }

        /* if multi channel output, don't mix the output of the chorus and
        reverb in the final output. The effects outputs are send
        separately. */
        if do_not_mix_fx_to_out != 0 {
            /* send to reverb */
            if self.settings.synth.reverb_active {
                self.reverb.process_replace(
                    // reverb_buf
                    &mut self.fx_left_buf[0],
                    &mut self.fx_right_buf[0],
                );
            }
            /* send to chorus */
            if self.settings.synth.chorus_active {
                self.chorus.process_replace(
                    // chorus_buf
                    &mut self.fx_left_buf[1],
                    &mut self.fx_right_buf[1],
                );
            }
        } else {
            /* send to reverb */
            if self.settings.synth.reverb_active {
                self.reverb.process_mix(
                    // reverb_buf
                    &mut self.fx_left_buf[0],
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
            /* send to chorus */
            if self.settings.synth.chorus_active {
                self.chorus.process_mix(
                    // chorus_buf
                    &mut self.fx_left_buf[1],
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
        }
        self.ticks = self.ticks.wrapping_add(64);
    }

    pub fn read_next(&mut self) -> (f32, f32) {
        if self.state != FLUID_SYNTH_PLAYING as u32 {
            (0.0, 0.0)
        } else {
            let mut l = self.cur;
            let mut i: usize = 0;
            let len = 1;

            let mut out = (0.0, 0.0);

            while i < len {
                /* fill up the buffers as needed */
                if l == 64 {
                    self.one_block(0);
                    l = 0;
                }

                out = (self.left_buf[0][l], self.right_buf[0][l]);

                i += 1;
                l += 1;
            }
            self.cur = l;

            out
        }
    }

    pub fn write<F: FnMut(usize, f32, f32)>(&mut self, len: usize, incr: usize, mut cb: F) {
        /* make sure we're playing */
        if self.state != FLUID_SYNTH_PLAYING as u32 {
            return;
        }

        let mut l = self.cur;
        let mut i: usize = 0;

        let mut out_id = 0;
        while i < len {
            /* fill up the buffers as needed */
            if l == 64 {
                self.one_block(0);
                l = 0;
            }

            cb(out_id, self.left_buf[0][l], self.right_buf[0][l]);

            out_id += incr;

            i += 1;
            l += 1;
        }
        self.cur = l;
    }

    pub fn write_f32(
        &mut self,
        len: usize,
        left_out: &mut [f32],
        loff: usize,
        lincr: usize,
        right_out: &mut [f32],
        roff: usize,
        rincr: usize,
    ) {
        /* make sure we're playing */
        if self.state != FLUID_SYNTH_PLAYING as u32 {
            return;
        }

        let mut l = self.cur;
        let mut i: usize = 0;
        let mut j = loff;
        let mut k = roff;

        while i < len {
            /* fill up the buffers as needed */
            if l == 64 {
                self.one_block(0);
                l = 0;
            }

            left_out[j] = self.left_buf[0][l];
            right_out[k] = self.right_buf[0][l];

            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
    }

    pub fn write_f64(
        &mut self,
        len: usize,
        left_out: &mut [f64],
        loff: usize,
        lincr: usize,
        right_out: &mut [f64],
        roff: usize,
        rincr: usize,
    ) {
        /* make sure we're playing */
        if self.state != FLUID_SYNTH_PLAYING as u32 {
            return;
        }

        let mut l = self.cur;
        let mut i: usize = 0;
        let mut j = loff;
        let mut k = roff;

        while i < len {
            /* fill up the buffers as needed */
            if l == 64 {
                self.one_block(0 as i32);
                l = 0;
            }

            left_out[j] = self.left_buf[0][l] as f64;
            right_out[k] = self.right_buf[0][l] as f64;

            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
    }

    pub fn write_s16(
        &mut self,
        len: usize,
        left_out: &mut [i16],
        loff: usize,
        lincr: usize,
        right_out: &mut [i16],
        roff: usize,
        rincr: usize,
    ) {
        let mut di: i32 = self.dither_index;

        /* make sure we're playing */
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return;
        }

        let mut cur = self.cur;
        let mut i: usize = 0;
        let mut j = loff;
        let mut k = roff;
        while i < len {
            /* fill up the buffers as needed */
            if cur == 64 {
                self.one_block(0 as i32);
                cur = 0;
            }
            /*
             * Converts stereo floating point sample data to signed 16 bit data with
             * dithering.
             */

            let mut left_sample = roundi(
                self.left_buf[0][cur as usize] * 32766.0f32
                    + RAND_TABLE[0 as i32 as usize][di as usize],
            ) as f32;
            let mut right_sample = roundi(
                self.right_buf[0][cur as usize] * 32766.0f32
                    + RAND_TABLE[1 as i32 as usize][di as usize],
            ) as f32;

            di += 1;
            if di >= 48000 as i32 {
                di = 0 as i32
            }

            /* digital clipping */
            if left_sample > 32767.0f32 {
                left_sample = 32767.0f32
            }
            if left_sample < -32768.0f32 {
                left_sample = -32768.0f32
            }
            if right_sample > 32767.0f32 {
                right_sample = 32767.0f32
            }
            if right_sample < -32768.0f32 {
                right_sample = -32768.0f32
            }

            left_out[j as usize] = left_sample as i16;
            right_out[k as usize] = right_sample as i16;

            i += 1;
            cur += 1;
            j += lincr;
            k += rincr
        }
        self.cur = cur;
        /* keep dither buffer continous */
        self.dither_index = di;
    }
}

/* A portable replacement for roundf(), seems it may actually be faster too! */
fn roundi(x: f32) -> i32 {
    if x >= 0.0f32 {
        (x + 0.5f32) as i32
    } else {
        (x - 0.5f32) as i32
    }
}
