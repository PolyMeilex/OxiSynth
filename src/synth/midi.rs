use crate::oxi::SoundFontId;
use crate::Synth;

/**
MIDI channel messages
 */
impl Synth {
    /**
    Send a noteon message.
     */
    pub fn note_on(&mut self, chan: u8, key: u8, vel: u8) -> Result<(), &str> {
        self.handle.noteon(chan, key, vel)
    }

    /**
    Send a noteoff message.
     */
    pub fn note_off(&mut self, chan: u8, key: u8) {
        self.handle.noteoff(chan, key)
    }

    /**
    Send a control change message.
     */
    pub fn cc(&mut self, chan: u8, ctrl: u16, val: u16) -> Result<(), ()> {
        self.handle.cc(chan, ctrl, val)
    }

    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: u8, ctrl: u16) -> Result<u8, &str> {
        self.handle.get_cc(chan, ctrl)
    }

    /**
    Send a pitch bend message.
     */
    pub fn pitch_bend(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        self.handle.pitch_bend(chan, val)
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: u8) -> Result<i16, &str> {
        self.handle.get_pitch_bend(chan)
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        self.handle.pitch_wheel_sens(chan, val)
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: u8) -> Result<u32, &str> {
        self.handle.get_pitch_wheel_sens(chan)
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: u8, prog: u8) -> Result<(), ()> {
        self.handle.program_change(chan, prog)
    }

    /**
    Set channel pressure
     */
    pub fn channel_pressure(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        self.handle.channel_pressure(chan, val)
    }

    /**
    Set key pressure (aftertouch)
     */
    pub fn key_pressure(&mut self, chan: u8, key: u8, val: u8) -> Result<(), ()> {
        self.handle.key_pressure(chan, key, val)
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: u8, bank: u32) -> Result<(), &str> {
        self.handle.bank_select(chan, bank)
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: u8, sfont_id: SoundFontId) -> Result<(), &str> {
        self.handle.sfont_select(chan, sfont_id)
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
    ) -> Result<(), &str> {
        self.handle
            .program_select(chan, sfont_id, bank_num, preset_num)
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: u8) -> Result<(Option<SoundFontId>, u32, u32), &str> {
        self.handle.get_program(chan)
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) {
        self.handle.program_reset()
    }

    /**
    Send a reset.

    A reset turns all the notes off and resets the controller values.
     */
    pub fn system_reset(&mut self) {
        self.handle.system_reset()
    }
}
