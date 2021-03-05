use crate::{Bank, Chan, Ctrl, FontId, Key, PresetId, Prog, Synth, Val, Vel};

/**
MIDI channel messages
 */
impl Synth {
    /**
    Send a noteon message.
     */
    pub fn note_on(&mut self, chan: Chan, key: Key, vel: Vel) -> Result<(), ()> {
        self.handle.noteon(chan as _, key as _, vel as _)
    }

    /**
    Send a noteoff message.
     */
    pub fn note_off(&mut self, chan: Chan, key: Key) -> Result<(), ()> {
        self.handle.noteoff(chan, key)
    }

    /**
    Send a control change message.
     */
    pub fn cc(&mut self, chan: Chan, ctrl: Ctrl, val: Val) -> Result<(), ()> {
        self.handle.cc(chan as _, ctrl as _, val as _)
    }

    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: Chan, ctrl: Ctrl) -> Result<u8, ()> {
        self.handle.get_cc(chan, ctrl)
    }

    /**
    Send a pitch bend message.
     */
    pub fn pitch_bend(&mut self, chan: Chan, val: Val) -> Result<(), ()> {
        self.handle.pitch_bend(chan as _, val as _)
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: Chan) -> Result<i16, ()> {
        self.handle.get_pitch_bend(chan)
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, chan: Chan, val: Val) -> Result<(), ()> {
        self.handle.pitch_wheel_sens(chan as _, val as _)
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: Chan) -> Result<u32, ()> {
        self.handle.get_pitch_wheel_sens(chan)
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: Chan, prog: Prog) -> Result<(), ()> {
        self.handle.program_change(chan as _, prog as _)
    }

    /**
    Set channel pressure
     */
    pub fn channel_pressure(&mut self, chan: Chan, val: Val) -> Result<(), ()> {
        self.handle.channel_pressure(chan as _, val as _)
    }

    /**
    Set key pressure (aftertouch)
     */
    pub fn key_pressure(&mut self, chan: Chan, key: Key, val: Val) -> Result<(), ()> {
        self.handle.key_pressure(chan as _, key as _, val as _)
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: Chan, bank: Bank) -> Result<(), ()> {
        self.handle.bank_select(chan as _, bank)
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: Chan, sfont_id: FontId) -> Result<(), ()> {
        self.handle.sfont_select(chan as _, sfont_id)
    }

    /**
    Select a preset for a channel. The preset is specified by the
    SoundFont ID, the bank number, and the preset number. This
    allows any preset to be selected and circumvents preset masking
    due to previously loaded SoundFonts on the SoundFont stack.
     */
    pub fn program_select(
        &mut self,
        chan: Chan,
        sfont_id: FontId,
        bank_num: Bank,
        preset_num: PresetId,
    ) -> Result<(), ()> {
        self.handle
            .program_select(chan as _, sfont_id, bank_num, preset_num)
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: Chan) -> Result<(FontId, Bank, PresetId), ()> {
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
