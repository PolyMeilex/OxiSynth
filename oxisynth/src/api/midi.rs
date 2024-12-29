use crate::{
    core::midi,
    error::{range_check, OxiError},
    MidiEvent, SoundFontId, Synth,
};

/// MIDI related
impl Synth {
    pub fn send_event(&mut self, event: MidiEvent) -> Result<(), OxiError> {
        crate::core::midi::handle_event(&mut self.core, event)
    }

    /// Returns the number of MIDI channels that the synthesizer uses internally
    pub fn channel_count(&self) -> usize {
        self.core.channels.len()
    }

    /// Get a control value.
    pub fn cc(&self, channel: u8, ctrl: u16) -> Result<u8, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;

        range_check(0..=127, &ctrl, OxiError::CtrlOutOfRange)?;

        Ok(channel.cc(ctrl as usize))
    }

    /// Get the pitch bend value.
    pub fn pitch_bend(&self, channel: u8) -> Result<u16, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;
        Ok(channel.pitch_bend())
    }

    /// Set the pitch wheel sensitivity.
    pub fn set_pitch_wheel_sensitivity(&mut self, channel: u8, val: u8) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        midi::pitch_wheel_sens(channel, &mut self.core.voices, val);
        Ok(())
    }

    /// Get the pitch wheel sensitivity.
    pub fn pitch_wheel_sensitivity(&self, channel: u8) -> Result<u8, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;
        Ok(channel.pitch_wheel_sensitivity())
    }

    /// Select a bank.
    pub fn select_bank(&mut self, channel: u8, bank: u32) -> Result<(), OxiError> {
        self.core
            .channels
            .get_mut(channel as usize)?
            .set_banknum(bank);
        Ok(())
    }

    /// Select a preset for a channel. The preset is specified by the
    /// SoundFont ID, the bank number, and the preset number. This
    /// allows any preset to be selected and circumvents preset masking
    /// due to previously loaded SoundFonts on the SoundFont stack.
    pub fn select_program(
        &mut self,
        channel: u8,
        sfont_id: SoundFontId,
        bank_id: u32,
        preset_id: u8,
    ) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        let preset = self.core.font_bank.preset(sfont_id, bank_id, preset_id);

        if preset.is_none() {
            log::error!(
                "There is no preset with bank number {} and preset number {} in SoundFont {:?}",
                bank_id,
                preset_id,
                sfont_id
            );
            Err(OxiError::PresetNotFound {
                bank_id,
                preset_id,
                sfont_id,
            })
        } else {
            channel.set_sfontnum(Some(sfont_id));
            channel.set_banknum(bank_id);
            channel.set_prognum(preset_id);
            channel.set_preset(preset);
            Ok(())
        }
    }

    /// Returns the program, bank, and SoundFont number of the preset on a given channel.
    pub fn program(&self, channel: u8) -> Result<(Option<SoundFontId>, u32, u32), OxiError> {
        let channel = self.core.channels.get(channel as usize)?;

        Ok((
            channel.sfontnum(),
            channel.banknum(),
            channel.prognum() as u32,
        ))
    }

    /// Send a bank select and a program change to every channel to reinitialize the preset of the channel.
    ///
    /// This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
    pub fn reset_program(&mut self) {
        for channel in self.core.channels.iter_mut() {
            midi::program_change(
                channel,
                &self.core.font_bank,
                channel.prognum(),
                self.core.settings.drums_channel_active,
            );
        }
    }
}
