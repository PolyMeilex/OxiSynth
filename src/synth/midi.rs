use crate::{Bank, Chan, Ctrl, FontId, Key, PresetId, Prog, Result, Status, Synth, Val, Vel};
use std::mem::MaybeUninit;

/**
MIDI channel messages
 */
impl Synth {
    /**
    Send a noteon message.
     */
    pub fn note_on(&mut self, chan: Chan, key: Key, vel: Vel) -> Status {
        Synth::zero_ok(unsafe { self.handle.noteon(chan as _, key as _, vel as _) })
    }

    /**
    Send a noteoff message.
     */
    pub fn note_off(&mut self, chan: Chan, key: Key) -> Status {
        Synth::zero_ok(unsafe { self.handle.noteoff(chan as _, key as _) })
    }

    /**
    Send a control change message.
     */
    pub fn cc(&mut self, chan: Chan, ctrl: Ctrl, val: Val) -> Status {
        Synth::zero_ok(unsafe { self.handle.cc(chan as _, ctrl as _, val as _) })
    }

    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: Chan, ctrl: Ctrl) -> Result<Val> {
        let mut val = MaybeUninit::uninit();

        Synth::zero_ok(unsafe { self.handle.get_cc(chan as _, ctrl as _, val.as_mut_ptr()) })
            .map(|_| unsafe { val.assume_init() as _ })
    }

    /**
    Send a pitch bend message.
     */
    pub fn pitch_bend(&mut self, chan: Chan, val: Val) -> Status {
        Synth::zero_ok(unsafe { self.handle.pitch_bend(chan as _, val as _) })
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: Chan) -> Result<Val> {
        let mut pitch_bend = MaybeUninit::uninit();

        Synth::zero_ok(unsafe {
            self.handle
                .get_pitch_bend(chan as _, pitch_bend.as_mut_ptr())
        })
        .map(|_| unsafe { pitch_bend.assume_init() as _ })
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, chan: Chan, val: Val) -> Status {
        Synth::zero_ok(unsafe { self.handle.pitch_wheel_sens(chan as _, val as _) })
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: Chan) -> Result<Val> {
        let mut val = MaybeUninit::uninit();

        Synth::zero_ok(unsafe {
            self.handle
                .get_pitch_wheel_sens(chan as _, val.as_mut_ptr())
        })
        .map(|_| unsafe { val.assume_init() as _ })
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: Chan, prog: Prog) -> Status {
        Synth::zero_ok(unsafe { self.handle.program_change(chan as _, prog as _) })
    }

    /**
    Set channel pressure
     */
    pub fn channel_pressure(&mut self, chan: Chan, val: Val) -> Status {
        Synth::zero_ok(unsafe { self.handle.channel_pressure(chan as _, val as _) })
    }

    /**
    Set key pressure (aftertouch)
     */
    pub fn key_pressure(&mut self, chan: Chan, key: Key, val: Val) -> Status {
        Synth::zero_ok(unsafe { self.handle.key_pressure(chan as _, key as _, val as _) })
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: Chan, bank: Bank) -> Status {
        Synth::zero_ok(self.handle.bank_select(chan as _, bank))
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: Chan, sfont_id: FontId) -> Status {
        Synth::zero_ok(unsafe { self.handle.sfont_select(chan as _, sfont_id) })
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
    ) -> Status {
        Synth::zero_ok(unsafe {
            self.handle
                .program_select(chan as _, sfont_id, bank_num, preset_num)
        })
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: Chan) -> Result<(FontId, Bank, PresetId)> {
        let mut sfont_id = MaybeUninit::uninit();
        let mut bank_num = MaybeUninit::uninit();
        let mut preset_num = MaybeUninit::uninit();

        Synth::zero_ok(unsafe {
            self.handle.get_program(
                chan as _,
                sfont_id.as_mut_ptr(),
                bank_num.as_mut_ptr(),
                preset_num.as_mut_ptr(),
            )
        })
        .map(|_| unsafe {
            (
                sfont_id.assume_init(),
                bank_num.assume_init(),
                preset_num.assume_init(),
            )
        })
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) -> Status {
        Synth::zero_ok(unsafe { self.handle.program_reset() })
    }

    /**
    Send a reset.

    A reset turns all the notes off and resets the controller values.
     */
    pub fn system_reset(&mut self) -> Status {
        Synth::zero_ok(unsafe { self.handle.system_reset() })
    }
}
