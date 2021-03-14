use crate::synth::Synth;

impl Synth {
    fn one_block(&mut self, do_not_mix_fx_to_out: i32) {
        // clean the audio buffers
        {
            for i in 0..self.nbuf {
                self.left_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
                self.right_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
            }

            {
                self.fx_left_buf.reverb.iter_mut().for_each(|v| *v = 0.0);
                self.fx_left_buf.chorus.iter_mut().for_each(|v| *v = 0.0);

                self.fx_right_buf.reverb.iter_mut().for_each(|v| *v = 0.0);
                self.fx_right_buf.chorus.iter_mut().for_each(|v| *v = 0.0);
            }
        }

        /* Set up the reverb / chorus buffers only, when the effect is
         * enabled on synth level.  Nonexisting buffers are detected in the
         * DSP loop. Not sending the reverb / chorus signal saves some time
         * in that case. */

        /* call all playing synthesis processes */
        self.voices.write_voices(
            &self.channel,
            self.min_note_length_ticks,
            self.settings.audio_groups,
            &mut self.left_buf,
            &mut self.right_buf,
            &mut self.fx_left_buf,
            self.settings.reverb_active,
            self.settings.chorus_active,
        );

        /* if multi channel output, don't mix the output of the chorus and
        reverb in the final output. The effects outputs are send
        separately. */
        if do_not_mix_fx_to_out != 0 {
            /* send to reverb */
            if self.settings.reverb_active {
                self.reverb
                    .process_replace(&mut self.fx_left_buf.reverb, &mut self.fx_right_buf.reverb);
            }
            /* send to chorus */
            if self.settings.chorus_active {
                self.chorus
                    .process_replace(&mut self.fx_left_buf.chorus, &mut self.fx_right_buf.chorus);
            }
        } else {
            /* send to reverb */
            if self.settings.reverb_active {
                self.reverb.process_mix(
                    &mut self.fx_left_buf.reverb,
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
            /* send to chorus */
            if self.settings.chorus_active {
                self.chorus.process_mix(
                    &mut self.fx_left_buf.chorus,
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
        }
        self.ticks = self.ticks.wrapping_add(64);
    }

    pub fn read_next(&mut self) -> (f32, f32) {
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

    pub fn write<F: FnMut(usize, f32, f32)>(&mut self, len: usize, incr: usize, mut cb: F) {
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

    #[cfg(feature = "i16-out")]
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

            let mut left_sample = f32::round(
                self.left_buf[0][cur as usize] * 32766.0f32
                    + RAND_TABLE[0 as i32 as usize][di as usize],
            );
            let mut right_sample = f32::round(
                self.right_buf[0][cur as usize] * 32766.0f32
                    + RAND_TABLE[1 as i32 as usize][di as usize],
            );

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

#[cfg(feature = "i16-out")]
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
