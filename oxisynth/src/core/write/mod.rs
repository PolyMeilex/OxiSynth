use crate::core::Core;

#[cfg(feature = "i16-out")]
mod i16_write;
#[cfg(feature = "i16-out")]
pub use i16_write::write_i16;

#[derive(Clone)]
pub(super) struct FxBuf {
    pub reverb: [f32; 64],
    pub chorus: [f32; 64],
}

pub(crate) struct OutputBuffer {
    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,

    fx_left_buf: FxBuf,
    fx_right_buf: FxBuf,

    cur: usize,

    #[cfg(feature = "i16-out")]
    i16_output: i16_write::I16OutputState,
}

impl OutputBuffer {
    pub(crate) fn new(nbuf: usize) -> Self {
        Self {
            left_buf: vec![[0.0; 64]; nbuf],
            right_buf: vec![[0.0; 64]; nbuf],
            fx_left_buf: FxBuf {
                reverb: [0.0; 64],
                chorus: [0.0; 64],
            },
            fx_right_buf: FxBuf {
                reverb: [0.0; 64],
                chorus: [0.0; 64],
            },
            cur: 64,
            #[cfg(feature = "i16-out")]
            i16_output: Default::default(),
        }
    }
}

impl Core {
    /// clean the audio buffers
    fn clear_buffers(&mut self) {
        self.output
            .left_buf
            .iter_mut()
            .for_each(|buff| buff.fill(0.0));
        self.output
            .right_buf
            .iter_mut()
            .for_each(|buff| buff.fill(0.0));

        self.output.fx_left_buf.reverb.fill(0.0);
        self.output.fx_left_buf.chorus.fill(0.0);

        self.output.fx_right_buf.reverb.fill(0.0);
        self.output.fx_right_buf.chorus.fill(0.0);
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
            self.settings.min_note_length_ticks,
            self.settings.audio_groups,
            (&mut self.output.left_buf, &mut self.output.right_buf),
            &mut self.output.fx_left_buf,
            self.settings.reverb_active,
            self.settings.chorus_active,
        );

        /* if multi channel output, don't mix the output of the chorus and
        reverb in the final output. The effects outputs are send
        separately. */
        if do_not_mix_fx_to_out {
            // send to reverb
            if self.settings.reverb_active {
                self.reverb.process_replace(
                    &mut self.output.fx_left_buf.reverb,
                    &mut self.output.fx_right_buf.reverb,
                );
            }

            // send to chorus
            if self.settings.chorus_active {
                self.chorus.process_replace(
                    &mut self.output.fx_left_buf.chorus,
                    &mut self.output.fx_right_buf.chorus,
                );
            }
        } else {
            // send to reverb
            if self.settings.reverb_active {
                self.reverb.process_mix(
                    &mut self.output.fx_left_buf.reverb,
                    &mut self.output.left_buf[0],
                    &mut self.output.right_buf[0],
                );
            }

            // send to chorus
            if self.settings.chorus_active {
                self.chorus.process_mix(
                    &mut self.output.fx_left_buf.chorus,
                    &mut self.output.left_buf[0],
                    &mut self.output.right_buf[0],
                );
            }
        }

        self.ticks += 64;
    }

    #[inline]
    pub fn read_next(&mut self) -> (f32, f32) {
        if self.output.cur == 64 {
            self.one_block(false);
            self.output.cur = 0;
        }

        let out = (
            self.output.left_buf[0][self.output.cur],
            self.output.right_buf[0][self.output.cur],
        );
        self.output.cur += 1;
        out
    }
}
