use crate::channel::Channel;
use crate::gen::GenParam;
use crate::voice::{Voice, VoiceEnvelope, VoiceId, VoiceStatus};

pub(crate) struct VoicePool {
    voices: Vec<Voice>,
    polyphony_limit: usize,
}

impl VoicePool {
    pub fn new(len: usize, output_rate: f32) -> Self {
        let mut voices = Vec::new();

        for _ in 0..len {
            voices.push(Voice::new(output_rate))
        }

        Self {
            voices,
            polyphony_limit: len,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for voice in self.voices.iter_mut() {
            *voice = Voice::new(sample_rate);
        }
    }

    /// Set the polyphony limit
    pub fn set_polyphony_limit(&mut self, polyphony: usize) -> Result<(), ()> {
        if polyphony < 1 || polyphony > self.voices.len() {
            Err(())
        } else {
            self.polyphony_limit = polyphony;

            /* turn off any voices above the new limit */
            for i in polyphony..self.voices.len() {
                let voice = &mut self.voices[i];
                if voice.is_playing() {
                    voice.off();
                }
            }

            Ok(())
        }
    }

    pub fn set_gen(&mut self, chan: u8, param: GenParam, value: f32) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan as u8 {
                voice.set_param(param, value, 0);
            }
        }
    }
    pub fn set_gain(&mut self, gain: f32) {
        for voice in self.voices.iter_mut() {
            if voice.is_playing() {
                voice.set_gain(gain);
            }
        }
    }

    pub fn noteoff(&mut self, channels: &[Channel], min_note_length_ticks: u32, chan: u8, key: u8) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.is_on() && voice.chan == chan && voice.key == key {
                log::trace!(
                    "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t",
                    voice.chan,
                    voice.key,
                    0,
                    voice.id,
                    voice.start_time.wrapping_add(voice.ticks) as f32 / 44100.0,
                    voice.ticks as f32 / 44100.0,
                );
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn all_notes_off(&mut self, channels: &[Channel], min_note_length_ticks: u32, chan: u8) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.is_playing() && voice.chan == chan {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn all_sounds_off(&mut self, chan: u8) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.is_playing() && voice.chan == chan {
                voice.off();
            }
        }
    }

    /// Reset turns all the voices off
    pub fn system_reset(&mut self) {
        self.voices.iter_mut().for_each(|v| v.off())
    }

    pub fn key_pressure(&mut self, channels: &[Channel], chan: u8, key: u8) {
        use crate::synth::FLUID_MOD_KEYPRESSURE;

        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan && voice.key == key {
                voice.modulate(channels, 0, FLUID_MOD_KEYPRESSURE as u16);
            }
        }
    }

    pub fn damp_voices(&mut self, channels: &[Channel], chan: u8, min_note_length_ticks: u32) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan && voice.status == VoiceStatus::Sustained {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn modulate_voices(&mut self, channels: &[Channel], chan: u8, is_cc: i32, ctrl: u16) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan {
                voice.modulate(channels, is_cc, ctrl);
            }
        }
    }

    pub fn modulate_voices_all(&mut self, channels: &[Channel], chan: u8) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan {
                voice.modulate_all(channels);
            }
        }
    }

    pub fn free_voice_by_kill(&mut self, noteid: usize) -> Option<VoiceId> {
        let mut best_prio: f32 = 999999.0f32;
        let mut best_voice_index: Option<usize> = None;

        {
            for (id, voice) in self
                .voices
                .iter_mut()
                .take(self.polyphony_limit)
                .enumerate()
            {
                if voice.is_available() {
                    return Some(VoiceId(id));
                }
                let mut this_voice_prio = 10000.0;
                if voice.chan == 0xff {
                    this_voice_prio -= 2000.0;
                }
                if voice.status == VoiceStatus::Sustained {
                    this_voice_prio -= 1000.0;
                }
                this_voice_prio -= noteid.wrapping_sub(voice.id) as f32;
                if voice.volenv_section != VoiceEnvelope::Attack as i32 {
                    this_voice_prio =
                        (this_voice_prio as f64 + voice.volenv_val as f64 * 1000.0f64) as f32
                }
                if this_voice_prio < best_prio {
                    best_voice_index = Some(id);
                    best_prio = this_voice_prio
                }
            }
        }

        if let Some(id) = best_voice_index {
            let voice = &mut self.voices[id];
            voice.off();
            Some(VoiceId(id))
        } else {
            None
        }
    }

    pub fn kill_by_exclusive_class(&mut self, new_voice: VoiceId) {
        let excl_class = {
            let new_voice = &mut self.voices[new_voice.0];
            let excl_class: i32 = (new_voice.gen[GenParam::ExclusiveClass as usize].val
                + new_voice.gen[GenParam::ExclusiveClass as usize].mod_0
                + new_voice.gen[GenParam::ExclusiveClass as usize].nrpn)
                as i32;
            excl_class
        };

        if excl_class != 0 {
            for i in 0..self.polyphony_limit {
                let new_voice = &self.voices[new_voice.0];
                let existing_voice = &self.voices[i as usize];

                if existing_voice.is_playing() {
                    if !(existing_voice.chan as i32 != new_voice.chan as i32) {
                        if !((existing_voice.gen[GenParam::ExclusiveClass as usize].val as f32
                            + existing_voice.gen[GenParam::ExclusiveClass as usize].mod_0 as f32
                            + existing_voice.gen[GenParam::ExclusiveClass as usize].nrpn as f32)
                            as i32
                            != excl_class)
                        {
                            if !(existing_voice.id == new_voice.id) {
                                self.voices[i as usize].kill_excl();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn start_voice(&mut self, channels: &[Channel], voice_id: VoiceId) {
        self.kill_by_exclusive_class(voice_id);
        self.voices[voice_id.0].start(channels);
    }

    pub fn release_voice_on_same_note(
        &mut self,
        channels: &[Channel],
        chan: u8,
        key: u8,
        noteid: usize,
        min_note_length_ticks: u32,
    ) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.is_playing() && voice.chan == chan && voice.key == key && voice.id != noteid {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn write_voices(
        &mut self,
        channels: &[Channel],
        min_note_length_ticks: u32,
        audio_groups: u8,
        dsp_left_buf: &mut [[f32; 64]],
        dsp_right_buf: &mut [[f32; 64]],
        fx_left_buf: &mut [[f32; 64]; 2],
        reverb_active: bool,
        chorus_active: bool,
    ) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
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
                let mut auchan = channels[voice.get_channel().unwrap().0].get_num();
                auchan %= audio_groups as u8;

                voice.write(
                    channels,
                    min_note_length_ticks,
                    &mut dsp_left_buf[auchan as usize],
                    &mut dsp_right_buf[auchan as usize],
                    fx_left_buf,
                    reverb_active,
                    chorus_active,
                );
            }
        }
    }
}

impl VoicePool {
    pub fn request_new_voice(&mut self, noteid: usize) -> Option<VoiceId> {
        // find free synthesis process
        let voice_id = self
            .voices
            .iter()
            .take(self.polyphony_limit)
            .enumerate()
            .find(|(_, v)| v.is_available())
            .map(|(id, _)| VoiceId(id));

        match voice_id {
            Some(id) => Some(id),
            // If none was found, free one by kill
            None => self.free_voice_by_kill(noteid),
        }
    }
}

impl VoicePool {
    // pub fn len(&self) -> usize {
    //     self.voices.len()
    // }

    pub fn iter(&self) -> std::slice::Iter<'_, Voice> {
        self.voices.iter()
    }
}

impl std::ops::Index<usize> for VoicePool {
    type Output = Voice;
    fn index(&self, id: usize) -> &Self::Output {
        &self.voices[id]
    }
}

impl std::ops::IndexMut<usize> for VoicePool {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        &mut self.voices[id]
    }
}
