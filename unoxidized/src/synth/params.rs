use crate::settings::Settings;
use crate::synth::BankOffset;
use crate::synth::InterpMethod;
use crate::synth::Preset;
use crate::synth::Synth;

impl Synth {
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    /**
    Set the master gain
     */
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = if gain < 0.0 {
            0.0
        } else if gain > 10.0 {
            10.0
        } else {
            gain
        };

        self.voices.set_gain(gain)
    }

    /**
    Get the master gain
     */
    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    /**
    Set the polyphony limit
     */
    pub fn set_polyphony(&mut self, polyphony: u16) -> Result<(), ()> {
        if polyphony < 1 {
            Err(())
        } else {
            self.settings.polyphony = polyphony;
            self.voices.set_polyphony_limit(polyphony as usize);

            Ok(())
        }
    }

    /**
    Get the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn get_polyphony(&self) -> u32 {
        self.settings.polyphony as u32
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
            let ch = self.channels.iter_mut().find(|ch| ch.get_num() == chan);

            if let Some(ch) = ch {
                ch.set_interp_method(interp_method);
            }
        } else {
            for ch in self.channels.iter_mut() {
                ch.set_interp_method(interp_method);
            }
        }
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    pub fn get_bank_offset(&self, sfont_id: usize) -> Option<&BankOffset> {
        self.bank_offsets.iter().find(|x| x.sfont_id == sfont_id)
    }

    /**
    Get the offset of the bank numbers in a SoundFont.
     */
    fn get_bank_offset_mut(&mut self, sfont_id: usize) -> Option<&mut BankOffset> {
        self.bank_offsets
            .iter_mut()
            .find(|x| x.sfont_id == sfont_id)
    }

    /**
    Offset the bank numbers in a SoundFont.
    Returns -1 if an error occured (out of memory or negative offset)
     */
    pub fn set_bank_offset(&mut self, sfont_id: usize, offset: u32) {
        let bank_offset = self.get_bank_offset_mut(sfont_id);

        if let Some(mut bank_offset) = bank_offset {
            bank_offset.offset = offset
        } else {
            let bank_offset = BankOffset { sfont_id, offset };
            self.bank_offsets.insert(0, bank_offset);
        }
    }

    pub fn remove_bank_offset(&mut self, sfont_id: usize) {
        self.bank_offsets.retain(|x| x.sfont_id != sfont_id);
    }

    pub fn get_channel_preset(&mut self, chan: u8) -> Option<&Preset> {
        if let Some(channel) = self.channels.get(chan as usize) {
            channel.get_preset()
        } else {
            log::warn!("Channel out of range");
            None
        }
    }
}
