use crate::core::{synth::internal, utils::RangeCheck, OxiError};
use crate::SoundFontId;
use crate::Synth;

// MIDI channel messages
impl Synth {
    /// Get a control value.
    pub fn get_cc(&self, channel: u8, ctrl: u16) -> Result<u8, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;

        RangeCheck::check(0..=127, &ctrl, OxiError::CtrlOutOfRange)?;

        Ok(channel.cc(ctrl as usize))
    }

    /// Get the pitch bend value.
    pub fn get_pitch_bend(&self, channel: u8) -> Result<u16, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;
        Ok(channel.pitch_bend())
    }

    /// Set the pitch wheel sensitivity.
    pub fn pitch_wheel_sens(&mut self, channel: u8, val: u8) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        internal::midi::pitch_wheel_sens(channel, &mut self.core.voices, val);
        Ok(())
    }

    /// Get the pitch wheel sensitivity.
    pub fn get_pitch_wheel_sens(&self, channel: u8) -> Result<u8, OxiError> {
        let channel = self.core.channels.get(channel as usize)?;
        Ok(channel.pitch_wheel_sensitivity())
    }

    /// Select a bank.
    pub fn bank_select(&mut self, channel: u8, bank: u32) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        internal::midi::bank_select(channel, bank);
        Ok(())
    }

    /// Select a sfont.
    pub fn sfont_select(&mut self, channel: u8, sfont_id: SoundFontId) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        internal::midi::sfont_select(channel, sfont_id);
        Ok(())
    }

    /// Select a preset for a channel. The preset is specified by the
    /// SoundFont ID, the bank number, and the preset number. This
    /// allows any preset to be selected and circumvents preset masking
    /// due to previously loaded SoundFonts on the SoundFont stack.
    pub fn program_select(
        &mut self,
        channel: u8,
        sfont_id: SoundFontId,
        bank_id: u32,
        preset_id: u8,
    ) -> Result<(), OxiError> {
        let channel = self.core.channels.get_mut(channel as usize)?;
        internal::midi::program_select(channel, &self.core.font_bank, sfont_id, bank_id, preset_id)
    }

    /// Returns the program, bank, and SoundFont number of the preset on a given channel.
    pub fn get_program(&self, channel: u8) -> Result<(Option<SoundFontId>, u32, u32), OxiError> {
        let channel = self.core.channels.get(channel as usize)?;
        Ok(internal::midi::get_program(channel))
    }

    /// Send a bank select and a program change to every channel to reinitialize the preset of the channel.
    ///
    /// This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
    pub fn program_reset(&mut self) {
        internal::midi::program_reset(
            &mut self.core.channels,
            &self.core.font_bank,
            self.core.settings.drums_channel_active,
        )
    }
}
