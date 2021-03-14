mod voice;

pub(crate) use voice::{Voice, VoiceAddMode, VoiceDescriptor, VoiceEnvelope, VoiceId, VoiceStatus};

use crate::channel::Channel;
use crate::generator::GenParam;
use crate::synth::FxBuf;

pub(crate) struct VoicePool {
    voices: Vec<Voice>,
    sample_rate: f32,
    polyphony_limit: usize,
}

impl VoicePool {
    pub fn new(len: usize, sample_rate: f32) -> Self {
        Self {
            voices: Vec::new(),
            sample_rate,
            polyphony_limit: len,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.voices.clear();
        self.sample_rate = sample_rate;
    }

    /// Set the polyphony limit
    pub fn set_polyphony_limit(&mut self, polyphony: usize) {
        /* remove any voices above the new limit */
        self.voices.truncate(polyphony);
        self.polyphony_limit = polyphony;
    }

    pub fn set_gen(&mut self, chan: u8, param: GenParam, value: f32) {
        for voice in self.voices.iter_mut() {
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
        for voice in self.voices.iter_mut() {
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
        for voice in self.voices.iter_mut() {
            if voice.is_playing() && voice.chan == chan {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn all_sounds_off(&mut self, chan: u8) {
        for voice in self.voices.iter_mut() {
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
        const MOD_KEYPRESSURE: u16 = 10;

        for voice in self.voices.iter_mut() {
            if voice.chan == chan && voice.key == key {
                voice.modulate(channels, 0, MOD_KEYPRESSURE);
            }
        }
    }

    pub fn damp_voices(&mut self, channels: &[Channel], chan: u8, min_note_length_ticks: u32) {
        for voice in self.voices.iter_mut() {
            if voice.chan == chan && voice.status == VoiceStatus::Sustained {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub fn modulate_voices(&mut self, channels: &[Channel], chan: u8, is_cc: i32, ctrl: u16) {
        for voice in self.voices.iter_mut() {
            if voice.chan == chan {
                voice.modulate(channels, is_cc, ctrl);
            }
        }
    }

    pub fn modulate_voices_all(&mut self, channels: &[Channel], chan: u8) {
        for voice in self.voices.iter_mut() {
            if voice.chan == chan {
                voice.modulate_all(channels);
            }
        }
    }

    pub fn free_voice_by_kill(&mut self, noteid: usize) -> Option<VoiceId> {
        let mut best_prio: f32 = 999999.0f32;
        let mut best_voice_index: Option<usize> = None;

        {
            for (id, voice) in self.voices.iter_mut().enumerate() {
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
            for i in 0..self.voices.len() {
                let new_voice = &self.voices[new_voice.0];
                let existing_voice = &self.voices[i as usize];

                if existing_voice.is_playing() {
                    if existing_voice.chan as i32 == new_voice.chan as i32 {
                        if (existing_voice.gen[GenParam::ExclusiveClass as usize].val
                            + existing_voice.gen[GenParam::ExclusiveClass as usize].mod_0
                            + existing_voice.gen[GenParam::ExclusiveClass as usize].nrpn)
                            as i32
                            == excl_class
                        {
                            if existing_voice.id != new_voice.id {
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
        for voice in self.voices.iter_mut() {
            if voice.is_playing() && voice.chan == chan && voice.key == key && voice.id != noteid {
                voice.noteoff(channels, min_note_length_ticks);
            }
        }
    }

    pub(super) fn write_voices(
        &mut self,
        channels: &[Channel],
        min_note_length_ticks: u32,
        audio_groups: u8,
        dsp_left_buf: &mut [[f32; 64]],
        dsp_right_buf: &mut [[f32; 64]],
        fx_left_buf: &mut FxBuf,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        for voice in self.voices.iter_mut() {
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
                let mut auchan = channels[voice.get_channel().0].get_num();
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
    pub fn request_new_voice<A: FnOnce(&mut Voice)>(
        &mut self,
        noteid: usize,
        desc: VoiceDescriptor,
        after: A,
    ) -> Result<VoiceId, ()> {
        // find free synthesis process
        let voice_id = self
            .voices
            .iter()
            .enumerate()
            .find(|(_, v)| v.is_available())
            .map(|(id, _)| VoiceId(id));

        let voice_id = match voice_id {
            Some(id) => {
                self.voices[id.0].reinit(desc);
                Some(id)
            }
            // If none free voice was found:
            None => {
                // Check if we can add a new voice
                if self.voices.len() < self.polyphony_limit {
                    // If we can we do...
                    self.voices.push(Voice::new(self.sample_rate, desc));
                    Some(VoiceId(self.voices.len() - 1))
                } else {
                    // If we can't we free already existing one...
                    let id = self.free_voice_by_kill(noteid);
                    if let Some(id) = id {
                        self.voices[id.0].reinit(desc);
                    }
                    id
                }
            }
        };

        if let Some(id) = voice_id {
            after(&mut self.voices[id.0]);
            Ok(id)
        } else {
            Err(())
        }
    }
}
