use crate::error::OxiError;
use crate::soundfont::SoundFont;
use crate::synth::{internal, Synth};
use crate::utils::{RangeCheck, TypedIndex};

impl Synth {
    /**
    Get a control value.
     */
    pub fn get_cc(&self, channel_id: u8, num: u16) -> Result<u8, OxiError> {
        let channel = self.channels.get(channel_id as usize)?;

        RangeCheck::check(0..=127, &num, OxiError::CtrlOutOfRange)?;

        Ok(channel.cc(num as usize))
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, channel_id: u8) -> Result<i16, OxiError> {
        let channel = self.channels.get(channel_id as usize)?;

        Ok(channel.pitch_bend())
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, channel_id: u8, val: u8) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(channel_id as usize)?;

        internal::pitch_wheel_sens(channel, &mut self.voices, val);
        Ok(())
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, channel_id: u8) -> Result<u8, OxiError> {
        let channel = self.channels.get(channel_id as usize)?;

        Ok(channel.pitch_wheel_sensitivity())
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, channel_id: u8, bank: u32) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(channel_id as usize)?;

        internal::midi::bank_select(channel, bank);
        Ok(())
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(
        &mut self,
        channel_id: u8,
        sfont_id: TypedIndex<SoundFont>,
    ) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(channel_id as usize)?;

        internal::midi::sfont_select(channel, sfont_id);
        Ok(())
    }

    /**
    Select a preset for a channel. The preset is specified by the
    SoundFont ID, the bank number, and the preset number. This
    allows any preset to be selected and circumvents preset masking
    due to previously loaded SoundFonts on the SoundFont stack.
     */
    pub fn program_select(
        &mut self,
        channel_id: u8,
        sfont_id: TypedIndex<SoundFont>,
        bank_id: u32,
        preset_id: u8,
    ) -> Result<(), OxiError> {
        let channel = self.channels.get_mut(channel_id as usize)?;
        internal::midi::program_select(channel, &self.font_bank, sfont_id, bank_id, preset_id)
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(
        &self,
        channel_id: u8,
    ) -> Result<(Option<TypedIndex<SoundFont>>, u32, u32), OxiError> {
        let channel = self.channels.get(channel_id as usize)?;

        Ok(internal::midi::get_program(channel))
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) {
        internal::midi::program_reset(
            &mut self.channels,
            &self.font_bank,
            self.settings.drums_channel_active,
        )
    }
}
