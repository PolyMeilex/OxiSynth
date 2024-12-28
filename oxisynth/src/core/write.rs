use crate::core::Synth;

#[cfg(feature = "i16-out")]
pub(crate) mod i16_write;

impl Synth {
    /// clean the audio buffers
    fn clear_buffers(&mut self) {
        self.left_buf.iter_mut().for_each(|buff| buff.fill(0.0));
        self.right_buf.iter_mut().for_each(|buff| buff.fill(0.0));

        self.fx_left_buf.reverb.fill(0.0);
        self.fx_left_buf.chorus.fill(0.0);

        self.fx_right_buf.reverb.fill(0.0);
        self.fx_right_buf.chorus.fill(0.0);
    }

    fn one_block(&mut self, do_not_mix_fx_to_out: bool) {
        self.clear_buffers();

        /* Set up the reverb / chorus buffers only, when the effect is
         * enabled on synth level.  Nonexisting buffers are detected in the
         * DSP loop. Not sending the reverb / chorus signal saves some time
         * in that case. */

        /* call all playing synthesis processes */
        self.voices.write_voices(
            &self.channels,
            self.min_note_length_ticks,
            self.settings.audio_groups,
            (&mut self.left_buf, &mut self.right_buf),
            &mut self.fx_left_buf,
            self.reverb.active(),
            self.chorus.active(),
        );

        /* if multi channel output, don't mix the output of the chorus and
        reverb in the final output. The effects outputs are send
        separately. */
        if do_not_mix_fx_to_out {
            /* send to reverb */
            if self.reverb.active() {
                self.reverb
                    .process_replace(&mut self.fx_left_buf.reverb, &mut self.fx_right_buf.reverb);
            }
            /* send to chorus */
            if self.chorus.active() {
                self.chorus
                    .process_replace(&mut self.fx_left_buf.chorus, &mut self.fx_right_buf.chorus);
            }
        } else {
            /* send to reverb */
            if self.reverb.active() {
                self.reverb.process_mix(
                    &mut self.fx_left_buf.reverb,
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
            /* send to chorus */
            if self.chorus.active() {
                self.chorus.process_mix(
                    &mut self.fx_left_buf.chorus,
                    &mut self.left_buf[0],
                    &mut self.right_buf[0],
                );
            }
        }

        self.ticks += 64;
    }

    #[inline]
    pub fn read_next(&mut self) -> (f32, f32) {
        if self.cur == 64 {
            self.one_block(false);
            self.cur = 0;
        }

        let out = (self.left_buf[0][self.cur], self.right_buf[0][self.cur]);
        self.cur += 1;
        out
    }
}
