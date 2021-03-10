use crate::channel::Channel;
use crate::gen::GenParam;
use crate::voice::{Voice, VoiceEnvelope, VoiceId, VoiceStatus};

pub struct VoicePool {
    voices: Vec<Voice>,
    polyphony_limit: usize,
}

impl VoicePool {
    pub(crate) fn new(len: usize, output_rate: f32) -> Self {
        let mut voices = Vec::new();

        for _ in 0..len {
            voices.push(Voice::new(output_rate))
        }

        Self {
            voices,
            polyphony_limit: len,
        }
    }

    pub(crate) fn set_sample_rate(&mut self, sample_rate: f32) {
        for i in 0..self.voices.len() {
            self.voices[i as usize] = Voice::new(sample_rate);
        }
    }

    /// Set the polyphony limit
    pub(crate) fn set_polyphony_limit(&mut self, polyphony: usize) -> Result<(), ()> {
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

    pub(crate) fn damp_voices(
        &mut self,
        channels: &[Channel],
        chan: u8,
        min_note_length_ticks: u32,
    ) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan && voice.status == VoiceStatus::Sustained {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub(crate) fn modulate_voices(
        &mut self,
        channels: &[Channel],
        chan: u8,
        is_cc: i32,
        ctrl: u16,
    ) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan {
                voice.modulate(channels, is_cc, ctrl);
            }
        }
    }

    pub(crate) fn modulate_voices_all(&mut self, channels: &[Channel], chan: u8) {
        for voice in self.voices.iter_mut().take(self.polyphony_limit) {
            if voice.chan == chan {
                voice.modulate_all(channels);
            }
        }
    }

    pub(crate) fn free_voice_by_kill(&mut self, noteid: usize) -> Option<VoiceId> {
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

    pub(crate) fn kill_by_exclusive_class(&mut self, new_voice: VoiceId) {
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

    pub(crate) fn start_voice(&mut self, channels: &[Channel], voice_id: VoiceId) {
        self.kill_by_exclusive_class(voice_id);
        self.voices[voice_id.0].start(channels);
    }

    pub(crate) fn release_voice_on_same_note(
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
