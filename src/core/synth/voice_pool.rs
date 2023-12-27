mod voice;

pub(crate) use voice::{EnvelopeStep, Voice, VoiceAddMode, VoiceDescriptor, VoiceStatus};

use super::channel_pool::Channel;
use super::soundfont::generator::GeneratorType;
use super::FxBuf;

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
        /* remove any voices above the new limit */
        self.voices.truncate(polyphony);
        self.polyphony_limit = polyphony;
    }

    pub fn set_gen(&mut self, chan: usize, param: GeneratorType, value: f32) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == chan)
        {
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
            .filter(|v| v.get_channel_id() == channel.id())
            .filter(|v| v.key == key)
        {
            log::trace!(
                "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t",
                voice.get_channel_id(),
                voice.key,
                0,
                voice.get_note_id(),
                voice.start_time.wrapping_add(voice.ticks) as f32 / 44100.0,
                voice.ticks as f32 / 44100.0,
            );
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn all_notes_off(&mut self, channel: &Channel, min_note_length_ticks: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == channel.id())
            .filter(|v| v.is_playing())
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn all_sounds_off(&mut self, chan: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == chan)
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
        const MOD_KEYPRESSURE: u8 = 10;

        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == channel.id())
            .filter(|v| v.key == key)
        {
            voice.modulate(channel, false, MOD_KEYPRESSURE);
        }
    }

    pub fn damp_voices(&mut self, channel: &Channel, min_note_length_ticks: usize) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == channel.id())
            .filter(|v| v.status == VoiceStatus::Sustained)
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub fn modulate_voices(&mut self, channel: &Channel, is_cc: bool, ctrl: u8) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == channel.id())
        {
            voice.modulate(channel, is_cc, ctrl);
        }
    }

    pub fn modulate_voices_all(&mut self, channel: &Channel) {
        for voice in self
            .voices
            .iter_mut()
            .filter(|v| v.get_channel_id() == channel.id())
        {
            voice.modulate_all(channel);
        }
    }

    fn free_voice_by_kill(&mut self, noteid: usize) -> Option<VoiceId> {
        let mut best_prio: f32 = 999999.0f32;
        let mut best_voice_index: Option<usize> = None;

        for (id, voice) in self.voices.iter_mut().enumerate() {
            if voice.is_available() {
                return Some(VoiceId(id));
            }
            let mut this_voice_prio = 10000.0;
            if voice.get_channel_id() == 0xff {
                this_voice_prio -= 2000.0;
            }
            if voice.status == VoiceStatus::Sustained {
                this_voice_prio -= 1000.0;
            }
            this_voice_prio -= noteid.wrapping_sub(voice.get_note_id()) as f32;
            if voice.volenv_section != EnvelopeStep::Attack {
                this_voice_prio =
                    (this_voice_prio as f64 + voice.volenv_val as f64 * 1000.0f64) as f32
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
        let excl_class = {
            let new_voice = &mut self.voices[new_voice.0];
            let excl_class: i32 = (new_voice.gen[GeneratorType::ExclusiveClass as usize].val
                + new_voice.gen[GeneratorType::ExclusiveClass as usize].mod_0
                + new_voice.gen[GeneratorType::ExclusiveClass as usize].nrpn)
                as i32;
            excl_class
        };

        if excl_class != 0 {
            for i in 0..self.voices.len() {
                let new_voice = &self.voices[new_voice.0];
                let existing_voice = &self.voices[i as usize];

                if existing_voice.is_playing() {
                    if existing_voice.get_channel_id() == new_voice.get_channel_id() {
                        if (existing_voice.gen[GeneratorType::ExclusiveClass as usize].val
                            + existing_voice.gen[GeneratorType::ExclusiveClass as usize].mod_0
                            + existing_voice.gen[GeneratorType::ExclusiveClass as usize].nrpn)
                            as i32
                            == excl_class
                        {
                            if existing_voice.get_note_id() != new_voice.get_note_id() {
                                self.voices[i as usize].kill_excl();
                            }
                        }
                    }
                }
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
            .filter(|v| v.get_channel_id() == channel.id())
            .filter(|v| v.is_playing())
            .filter(|v| v.key == key)
            .filter(|v| v.get_note_id() != noteid)
        {
            voice.noteoff(channel, min_note_length_ticks);
        }
    }

    pub(super) fn write_voices(
        &mut self,
        channels: &[Channel],
        min_note_length_ticks: usize,
        audio_groups: u8,
        dsp_left_buf: &mut [[f32; 64]],
        dsp_right_buf: &mut [[f32; 64]],
        fx_left_buf: &mut FxBuf,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        for voice in self.voices.iter_mut().filter(|v| v.is_playing()) {
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
            let mut auchan = voice.get_channel_id();
            auchan %= audio_groups as usize;

            voice.write(
                &channels[voice.get_channel_id()],
                min_note_length_ticks,
                &mut dsp_left_buf[auchan],
                &mut dsp_right_buf[auchan],
                fx_left_buf,
                reverb_active,
                chorus_active,
            );
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
