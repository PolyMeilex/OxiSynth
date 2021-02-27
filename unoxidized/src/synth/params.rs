#![forbid(unsafe_code)]

use crate::synth::BankOffset;
use crate::synth::InterpMethod;
use crate::synth::Preset;
use crate::synth::Synth;
use crate::synth::FLUID_FAILED;
use crate::synth::FLUID_OK;
use crate::voice::fluid_voice_off;
use crate::voice::fluid_voice_set_gain;
use crate::voice::FLUID_VOICE_ON;
use crate::voice::FLUID_VOICE_SUSTAINED;

impl Synth {
    /**
    Set the master gain
     */
    pub fn set_gain(&mut self, mut gain: f64) {
        let mut i;
        gain = if gain < 0.0 {
            0.0
        } else if gain > 10.0 {
            10.0
        } else {
            gain
        };
        self.gain = gain;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_set_gain(voice, gain);
            }
            i += 1
        }
    }

    /**
    Get the master gain
     */
    pub fn get_gain(&self) -> f64 {
        self.gain
    }

    /**
    Set the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn set_polyphony(&mut self, polyphony: i32) -> i32 {
        let mut i;
        if polyphony < 1 as i32 || polyphony > self.nvoice {
            return FLUID_FAILED as i32;
        }
        i = polyphony;
        while i < self.nvoice {
            let voice = &mut self.voices[i as usize];
            if voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        self.settings.synth.polyphony = polyphony;
        return FLUID_OK as i32;
    }

    /**
    Get the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn get_polyphony(&self) -> u32 {
        self.settings.synth.polyphony as u32
    }

    /**
    Get the internal buffer size. The internal buffer size if not the
    same thing as the buffer size specified in the
    settings. Internally, the synth *always* uses a specific buffer
    size independent of the buffer size used by the audio driver. The
    internal buffer size is normally 64 samples. The reason why it
    uses an internal buffer size is to allow audio drivers to call the
    synthesizer with a variable buffer length. The internal buffer
    size is useful for client who want to optimize their buffer sizes.
     */
    pub fn get_internal_bufsize(&self) -> usize {
        64
    }

    /**
     * Set the interpolation method for one channel (`Some(chan)`) or all channels (`None`)
     */
    pub fn set_interp_method(&mut self, chan: Option<u8>, interp_method: InterpMethod) {
        if let Some(chan) = chan {
            let ch = self
                .channel
                .iter_mut()
                .take(self.settings.synth.midi_channels as usize)
                .find(|ch| ch.get_num() == chan);

            if let Some(ch) = ch {
                ch.set_interp_method(interp_method);
            }
        } else {
            for ch in self
                .channel
                .iter_mut()
                .take(self.settings.synth.midi_channels as usize)
            {
                ch.set_interp_method(interp_method);
            }
        }
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get_bank_offset(&self, sfont_id: u32) -> Option<&BankOffset> {
        self.bank_offsets.iter().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get_bank_offset_mut(&mut self, sfont_id: u32) -> Option<&mut BankOffset> {
        self.bank_offsets
            .iter_mut()
            .find(|x| x.sfont_id == sfont_id)
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occured (out of memory or negative offset)
     */
    pub fn set_bank_offset(&mut self, sfont_id: u32, offset: u32) {
        let bank_offset = self.get_bank_offset_mut(sfont_id);

        if let Some(mut bank_offset) = bank_offset {
            bank_offset.offset = offset
        } else {
            let bank_offset = BankOffset { sfont_id, offset };
            self.bank_offsets.insert(0, bank_offset);
        }
    }

    pub fn remove_bank_offset(&mut self, sfont_id: u32) {
        self.bank_offsets.retain(|x| x.sfont_id != sfont_id);
    }

    pub fn get_channel_preset(&mut self, chan: u8) -> Option<&mut Preset> {
        if chan < self.settings.synth.midi_channels {
            self.channel[chan as usize].get_preset()
        } else {
            None
        }
    }
}
