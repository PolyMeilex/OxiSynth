use crate::core::OxiError;
use crate::SoundFontId;
use crate::Synth;

/**
MIDI channel messages
 */
impl Synth {
    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: u8, ctrl: u16) -> Result<u8, OxiError> {
        self.core.get_cc(chan, ctrl)
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: u8) -> Result<u16, OxiError> {
        self.core.get_pitch_bend(chan)
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, chan: u8, val: u8) -> Result<(), OxiError> {
        self.core.pitch_wheel_sens(chan, val)
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: u8) -> Result<u8, OxiError> {
        self.core.get_pitch_wheel_sens(chan)
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: u8, bank: u32) -> Result<(), OxiError> {
        self.core.bank_select(chan, bank)
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: u8, sfont_id: SoundFontId) -> Result<(), OxiError> {
        self.core.sfont_select(chan, sfont_id)
    }

    /**
    Select a preset for a channel. The preset is specified by the
    SoundFont ID, the bank number, and the preset number. This
    allows any preset to be selected and circumvents preset masking
    due to previously loaded SoundFonts on the SoundFont stack.
     */
    pub fn program_select(
        &mut self,
        chan: u8,
        sfont_id: SoundFontId,
        bank_num: u32,
        preset_num: u8,
    ) -> Result<(), OxiError> {
        self.core
            .program_select(chan, sfont_id, bank_num, preset_num)
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: u8) -> Result<(Option<SoundFontId>, u32, u32), OxiError> {
        self.core.get_program(chan)
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) {
        self.core.program_reset()
    }
}
