use crate::synth::Synth;
use crate::synth::FLUID_SYNTH_PLAYING;
use crate::synth::RAND_TABLE;
use crate::voice::FLUID_VOICE_ON;
use crate::voice::FLUID_VOICE_SUSTAINED;

impl Synth {
    unsafe fn one_block(&mut self, do_not_mix_fx_to_out: i32) -> i32 {
        // clean the audio buffers
        {
            for i in 0..self.nbuf {
                self.left_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
                self.right_buf[i as usize].iter_mut().for_each(|v| *v = 0.0);
            }

            for i in 0..self.settings.synth.effects_channels {
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
        let reverb_buf = if self.settings.synth.reverb_active {
            self.fx_left_buf[0].as_mut_ptr()
        } else {
            0 as *mut f32
        };
        let chorus_buf = if self.settings.synth.chorus_active {
            self.fx_left_buf[1].as_mut_ptr()
        } else {
            0 as *mut f32
        };

        /* call all playing synthesis processes */
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
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
                let mut auchan = voice.get_channel().as_ref().unwrap().get_num();
                auchan %= self.settings.synth.audio_groups as u8;

                voice.write(
                    self.min_note_length_ticks,
                    &mut self.left_buf[auchan as usize],
                    &mut self.right_buf[auchan as usize],
                    reverb_buf,
                    chorus_buf,
                );
            }
            i += 1
        }

        /* if multi channel output, don't mix the output of the chorus and
        reverb in the final output. The effects outputs are send
        separately. */
        if do_not_mix_fx_to_out != 0 {
            /* send to reverb */
            if !reverb_buf.is_null() {
                self.reverb.process_replace(
                    reverb_buf,
                    self.fx_left_buf[0].as_mut_ptr(),
                    self.fx_right_buf[0].as_mut_ptr(),
                );
            }
            /* send to chorus */
            if !chorus_buf.is_null() {
                self.chorus.process_replace(
                    chorus_buf,
                    self.fx_left_buf[1].as_mut_ptr(),
                    self.fx_right_buf[1].as_mut_ptr(),
                );
            }
        } else {
            /* send to reverb */
            if !reverb_buf.is_null() {
                self.reverb.process_mix(
                    reverb_buf,
                    self.left_buf[0].as_mut_ptr(),
                    self.right_buf[0].as_mut_ptr(),
                );
            }
            /* send to chorus */
            if !chorus_buf.is_null() {
                self.chorus.process_mix(
                    chorus_buf,
                    self.left_buf[0].as_mut_ptr(),
                    self.right_buf[0].as_mut_ptr(),
                );
            }
        }
        self.ticks = self.ticks.wrapping_add(64);
        return 0 as i32;
    }

    pub unsafe fn write_f32(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f32 = lout as *mut f32;
        let right_out: *mut f32 = rout as *mut f32;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize);
            *right_out.offset(k as isize) = *right_in.offset(l as isize);
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_f64(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f64 = lout as *mut f64;
        let right_out: *mut f64 = rout as *mut f64;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize) as f64;
            *right_out.offset(k as isize) = *right_in.offset(l as isize) as f64;
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_s16(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut cur;
        let left_out: *mut i16 = lout as *mut i16;
        let right_out: *mut i16 = rout as *mut i16;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        let mut left_sample;
        let mut right_sample;
        let mut di: i32 = self.dither_index;
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        cur = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if cur == 64 as i32 {
                self.one_block(0 as i32);
                cur = 0 as i32
            }
            left_sample = roundi(
                *left_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[0 as i32 as usize][di as usize],
            ) as f32;
            right_sample = roundi(
                *right_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[1 as i32 as usize][di as usize],
            ) as f32;
            di += 1;
            if di >= 48000 as i32 {
                di = 0 as i32
            }
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
            *left_out.offset(j as isize) = left_sample as i16;
            *right_out.offset(k as isize) = right_sample as i16;
            i += 1;
            cur += 1;
            j += lincr;
            k += rincr
        }
        self.cur = cur;
        self.dither_index = di;
        return 0 as i32;
    }
}

fn roundi(x: f32) -> i32 {
    if x >= 0.0f32 {
        (x + 0.5f32) as i32
    } else {
        (x - 0.5f32) as i32
    }
}
