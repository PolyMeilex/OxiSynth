use crate::core::Core;

#[cfg(feature = "i16-out")]
mod i16_write;
#[cfg(feature = "i16-out")]
pub use i16_write::write_i16;

use super::BUFSIZE;

#[derive(Clone)]
pub(crate) struct FxBuf {
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

    #[allow(clippy::too_many_arguments)]
    fn write_voices(&mut self) {
        for voice in self.voices.iter_mut().filter(|v| v.is_playing()) {
            // The output associated with a MIDI channel is wrapped around
            // using the number of audio groups as modulo divider.  This is
            // typically the number of output channels on the 'sound card',
            // as long as the LADSPA Fx unit is not used. In case of LADSPA
            // unit, think of it as subgroups on a mixer.
            //
            // For example: Assume that the number of groups is set to 2.
            // Then MIDI channel 1, 3, 5, 7 etc. go to output 1, channels 2,
            // 4, 6, 8 etc to output 2.  Or assume 3 groups: Then MIDI
            // channels 1, 4, 7, 10 etc go to output 1; 2, 5, 8, 11 etc to
            // output 2, 3, 6, 9, 12 etc to output 3.
            let mut auchan = voice.channel_id();
            auchan %= self.settings.audio_groups as usize;

            voice.write(
                &self.channels[voice.channel_id()],
                self.settings.min_note_length_ticks,
                (
                    &mut self.output.left_buf[auchan],
                    &mut self.output.right_buf[auchan],
                ),
                &mut self.output.fx_left_buf,
                self.settings.reverb_active,
                self.settings.chorus_active,
            );
        }
    }

    fn one_block(&mut self, do_not_mix_fx_to_out: bool) {
        self.clear_buffers();

        // Set up the reverb / chorus buffers only, when the effect is
        // enabled on synth level.  Nonexisting buffers are detected in the
        // DSP loop. Not sending the reverb / chorus signal saves some time
        // in that case.

        // call all playing synthesis processes
        self.write_voices();

        // if multi channel output, don't mix the output of the chorus and
        // reverb in the final output. The effects outputs are send
        // separately.
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

        self.ticks += BUFSIZE;
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
