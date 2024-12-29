mod voice;

use soundfont::raw::GeneralPalette;
pub(crate) use voice::{EnvelopeStep, ModulateCtrl, Voice, VoiceAddMode, VoiceDescriptor};

use super::channel_pool::Channel;
use super::soundfont::generator::GeneratorType;

#[derive(Copy, Clone)]
struct VoiceId(pub(crate) usize);

pub struct VoicePool {
    voices: Vec<Voice>,
    sample_rate: f32,
    polyphony_limit: usize,

    noteid: usize,
    storeid: usize,
}

impl VoicePool {
    pub fn new(len: usize, sample_rate: f32) -> Self {
        Self {
            voices: Vec::new(),
            sample_rate,
            polyphony_limit: len,

            noteid: 0,
            storeid: 0,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Voice> {
        self.voices.iter_mut()
    }

    pub fn noteid_add(&mut self) {
        self.storeid = self.noteid;
        self.noteid += 1;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.voices.clear();
        self.sample_rate = sample_rate;
    }

    /// Set the polyphony limit
    pub fn set_polyphony_limit(&mut self, polyphony: usize) {
        // remove any voices above the new limit
        self.voices.truncate(polyphony);
        self.polyphony_limit = polyphony;
    }

    pub fn set_gen(&mut self, chan: usize, param: GeneratorType, value: f32) {
        for voice in self.voices.iter_mut().filter(|v| v.channel_id() == chan) {
            voice.set_param(param, value, 0);
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        for voice in self.voices.iter_mut().filter(|v| v.is_playing()) {
            voice.set_gain(gain);
        }
    }

    pub fn noteoff(&mut self, channel: &Channel, min_note_length_ticks: usize, key: u8) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.is_on())
            .filter(|v| v.channel_id() == channel.id())
            .filter(|v| v.key() == key)
        {
            log::trace!(
                "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t",
                voice.channel_id(),
                voice.key(),
                0,
                voice.get_note_id(),
                (voice.start_time() + voice.ticks()) as f32 / 44100.0,
                voice.ticks() as f32 / 44100.0,
            );
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn all_notes_off(&mut self, channel: &Channel, min_note_length_ticks: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
            .filter(|v| v.is_playing())
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn all_sounds_off(&mut self, chan: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == chan)
            .filter(|v| v.is_playing())
        {
            voice.off();
        }
    }

    /// Reset turns all the voices off
    pub fn system_reset(&mut self) {
        self.voices.iter_mut().for_each(|v| v.off())
    }

    pub fn key_pressure(&mut self, channel: &Channel, key: u8) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
            .filter(|v| v.key() == key)
        {
            voice.modulate(channel, ModulateCtrl::SF(GeneralPalette::PolyPressure));
        }
    }

    pub fn damp_voices(&mut self, channel: &Channel, min_note_length_ticks: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
            .filter(|v| v.is_sustained())
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn modulate_voices(&mut self, channel: &Channel, ctrl: ModulateCtrl) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
        {
            voice.modulate(channel, ctrl);
        }
    }

    pub fn modulate_voices_all(&mut self, channel: &Channel) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
        {
            voice.modulate_all(channel);
        }
    }

    fn free_voice_by_kill(&mut self, noteid: usize) -> Option<VoiceId> {
        let mut best_prio = 999999.0;
        let mut best_voice_index: Option<usize> = None;

        for (id, voice) in self.voices.iter_mut().enumerate() {
            if voice.is_available() {
                return Some(VoiceId(id));
            }
            let mut this_voice_prio = 10000.0;
            if voice.channel_id() == 0xff {
                this_voice_prio -= 2000.0;
            }
            if voice.is_sustained() {
                this_voice_prio -= 1000.0;
            }
            this_voice_prio -= noteid.wrapping_sub(voice.get_note_id()) as f32;
            if voice.volenv_section() != EnvelopeStep::Attack {
                this_voice_prio += voice.volenv_val() * 1000.0;
            }
            if this_voice_prio < best_prio {
                best_voice_index = Some(id);
                best_prio = this_voice_prio
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

    fn kill_by_exclusive_class(&mut self, new_voice: VoiceId) {
        let new_class_sum = self.voices[new_voice.0].exclusive_class_sum();

        if new_class_sum.trunc() == 0.0 {
            return;
        }

        for i in 0..self.voices.len() {
            let new_voice = &self.voices[new_voice.0];
            let existing_voice = &self.voices[i];

            if !existing_voice.is_playing() {
                continue;
            }

            if existing_voice.channel_id() != new_voice.channel_id() {
                continue;
            }

            if existing_voice.exclusive_class_sum() != new_class_sum {
                continue;
            }

            if existing_voice.get_note_id() != new_voice.get_note_id() {
                self.voices[i].kill_excl();
            }
        }
    }

    pub fn release_voice_on_same_note(
        &mut self,
        channel: &Channel,
        key: u8,
        min_note_length_ticks: usize,
    ) {
        let noteid = self.noteid;
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.channel_id() == channel.id())
            .filter(|v| v.is_playing())
            .filter(|v| v.key() == key)
            .filter(|v| v.get_note_id() != noteid)
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }
}

impl VoicePool {
    fn start_voice(&mut self, channel: &Channel, voice_id: VoiceId) {
        self.kill_by_exclusive_class(voice_id);

        let v = &mut self.voices[voice_id.0];
        v.start(channel);
    }

    pub fn request_new_voice<A: FnOnce(&mut Voice)>(
        &mut self,
        desc: VoiceDescriptor,
        after: A,
    ) -> Result<(), ()> {
        // find free synthesis process
        let voice_id = self
            .voices
            .iter()
            .enumerate()
            .find(|(_, v)| v.is_available())
            .map(|(id, _)| VoiceId(id));

        let channel = desc.channel;

        let voice_id = match voice_id {
            Some(id) => {
                self.voices[id.0] = Voice::new(self.sample_rate, desc, self.storeid);
                Some(id)
            }
            // If none free voice was found:
            None => {
                // Check if we can add a new voice
                if self.voices.len() < self.polyphony_limit {
                    // If we can we do...
                    self.voices
                        .push(Voice::new(self.sample_rate, desc, self.storeid));
                    Some(VoiceId(self.voices.len() - 1))
                } else {
                    // If we can't we free already existing one...
                    let id = self.free_voice_by_kill(self.noteid);
                    if let Some(id) = id {
                        self.voices[id.0] = Voice::new(self.sample_rate, desc, self.storeid);
                    }
                    id
                }
            }
        };

        if let Some(id) = voice_id {
            after(&mut self.voices[id.0]);

            // add the synthesis process to the synthesis loop.
            self.start_voice(channel, id);
            Ok(())
        } else {
            Err(())
        }
    }
}
