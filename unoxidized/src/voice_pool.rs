use crate::voice::{Voice, VoiceId, VoiceStatus, FLUID_VOICE_ENVATTACK};

pub struct VoicePool {
    voices: Vec<Voice>,
}

impl VoicePool {
    pub(crate) fn new(len: usize, output_rate: f32) -> Self {
        let mut voices = Vec::new();

        for _ in 0..len {
            voices.push(Voice::new(output_rate))
        }

        Self { voices }
    }

    pub(crate) fn set_sample_rate(&mut self, sample_rate: f32) {
        for i in 0..self.voices.len() {
            self.voices[i as usize] = Voice::new(sample_rate);
        }
    }

    pub(crate) fn damp_voices(&mut self, chan: u8, polyphony: u16, min_note_length_ticks: u32) {
        for i in 0..polyphony {
            let voice = &mut self.voices[i as usize];

            if voice.chan == chan && voice.status == VoiceStatus::Sustained {
                voice.noteoff(min_note_length_ticks);
            }
        }
    }

    pub(crate) fn modulate_voices(&mut self, chan: u8, is_cc: i32, ctrl: u16, polyphony: u16) {
        for i in 0..polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan {
                voice.modulate(is_cc, ctrl);
            }
        }
    }

    pub(crate) fn modulate_voices_all(&mut self, chan: u8, polyphony: u16) {
        for i in 0..polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan {
                voice.modulate_all();
            }
        }
    }

    pub(crate) fn free_voice_by_kill(&mut self, polyphony: u16, noteid: u32) -> Option<VoiceId> {
        let mut best_prio: f32 = 999999.0f32;
        let mut best_voice_index: Option<usize> = None;

        {
            for (id, voice) in self.voices.iter_mut().take(polyphony as usize).enumerate() {
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
                if voice.volenv_section != FLUID_VOICE_ENVATTACK as i32 {
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
}

impl VoicePool {
    pub fn len(&self) -> usize {
        self.voices.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Voice> {
        self.voices.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Voice> {
        self.voices.iter_mut()
    }

    pub fn clear(&mut self) {
        self.voices.clear();
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
