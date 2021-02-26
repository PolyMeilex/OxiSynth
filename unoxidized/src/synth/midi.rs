use crate::synth::Synth;
use crate::synth::FLUID_FAILED;
use crate::synth::FLUID_MOD_KEYPRESSURE;
use crate::synth::FLUID_OK;
use crate::voice::fluid_voice_modulate;
use crate::voice::fluid_voice_noteoff;
use crate::voice::fluid_voice_off;
use crate::voice::FLUID_VOICE_CLEAN;
use crate::voice::FLUID_VOICE_ENVRELEASE;
use crate::voice::FLUID_VOICE_OFF;
use crate::voice::FLUID_VOICE_ON;
use crate::voice::FLUID_VOICE_SUSTAINED;

impl Synth {
    /**
    Send a noteon message.
     */
    pub unsafe fn noteon(&mut self, chan: u8, key: i32, vel: i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range");
            return FLUID_FAILED as i32;
        }
        if vel == 0 as i32 {
            return self.noteoff(chan, key);
        }
        if self.channel[chan as usize].preset.is_none() {
            if self.settings.synth.verbose {
                log::info!(
                    "noteon\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}\t{}",
                    chan,
                    key,
                    vel,
                    0,
                    (self.ticks as f32 / 44100.0f32),
                    0.0f32,
                    0,
                    "channel has no preset"
                );
            }
            return FLUID_FAILED as i32;
        }
        self.release_voice_on_same_note(chan, key);
        let fresh7 = self.noteid;
        self.noteid = self.noteid.wrapping_add(1);

        let preset_ptr = self.channel[chan as usize].preset.as_mut().unwrap() as *mut _;
        return self.start(fresh7, preset_ptr, 0, chan, key, vel);
    }

    /**
    Send a noteoff message.
     */
    pub unsafe fn noteoff(&mut self, chan: u8, key: i32) -> i32 {
        let mut i = 0;
        let mut status: i32 = FLUID_FAILED as i32;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.status as i32 == FLUID_VOICE_ON as i32
                && voice.volenv_section < FLUID_VOICE_ENVRELEASE as i32
                && voice.chan == chan
                && voice.key as i32 == key
            {
                if self.settings.synth.verbose {
                    let mut used_voices: i32 = 0 as i32;
                    let mut k;
                    k = 0 as i32;
                    while k < self.settings.synth.polyphony {
                        if !(voice.status as i32 == FLUID_VOICE_CLEAN as i32
                            || voice.status as i32 == FLUID_VOICE_OFF as i32)
                        {
                            used_voices += 1
                        }
                        k += 1
                    }
                    log::info!(
                        "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                        voice.chan,
                        voice.key,
                        0 as i32,
                        voice.id,
                        (voice.start_time.wrapping_add(voice.ticks) as f32 / 44100.0f32) as f64,
                        (voice.ticks as f32 / 44100.0f32) as f64,
                        used_voices
                    );
                }
                fluid_voice_noteoff(voice, self.min_note_length_ticks);
                status = FLUID_OK as i32
            }
            i += 1
        }
        return status;
    }

    /**
    Send a control change message.
     */
    pub unsafe fn cc(&mut self, chan: u8, num: i32, val: i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if num < 0 as i32 || num >= 128 as i32 {
            log::warn!("Ctrl out of range",);
            return FLUID_FAILED as i32;
        }
        if val < 0 as i32 || val >= 128 as i32 {
            log::warn!("Value out of range",);
            return FLUID_FAILED as i32;
        }
        if self.settings.synth.verbose {
            log::info!("cc\t{}\t{}\t{}", chan, num, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize].cc(
            synth_ptr.as_mut().unwrap(),
            num,
            val,
        );
        return FLUID_OK as i32;
    }

    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: u8, num: u32) -> Result<u8, ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            Err(())
        } else if num >= 128 {
            log::warn!("Ctrl out of range",);
            Err(())
        } else {
            let pval = self.channel[chan as usize].cc[num as usize];
            Ok(pval as u8)
        }
    }

    pub fn all_notes_off(&mut self, chan: u8) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if (voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32)
                && voice.chan == chan
            {
                unsafe {
                    fluid_voice_noteoff(voice, self.min_note_length_ticks);
                }
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn all_sounds_off(&mut self, chan: u8) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if (voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32)
                && voice.chan == chan
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    /**
    Send a pitch bend message.
     */
    pub unsafe fn pitch_bend(&mut self, chan: u8, val: i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.settings.synth.verbose {
            log::info!("pitchb\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pitch_bend(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: u8) -> Result<i16, ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            Err(())
        } else {
            let pitch_bend = self.channel[chan as usize].pitch_bend;
            Ok(pitch_bend)
        }
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub unsafe fn pitch_wheel_sens(&mut self, chan: u8, val: i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.settings.synth.verbose {
            log::info!("pitchsens\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pitch_wheel_sens(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub unsafe fn get_pitch_wheel_sens(&self, chan: u8, pval: *mut i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        *pval = self.channel[chan as usize].pitch_wheel_sensitivity as i32;
        return FLUID_OK as i32;
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: u8, prognum: i32) -> i32 {
        let mut preset;
        let banknum;
        let sfont_id;
        let mut subst_bank;
        let mut subst_prog;
        if prognum < 0 as i32 || prognum >= 128 as i32 || chan >= self.settings.synth.midi_channels
        {
            log::error!("Index out of range (chan={}, prog={})", chan, prognum);
            return FLUID_FAILED as i32;
        }
        banknum = self.channel[chan as usize].get_banknum();
        self.channel[chan as usize].set_prognum(prognum);
        if self.settings.synth.verbose {
            log::info!("prog\t{}\t{}\t{}", chan, banknum, prognum);
        }

        if self.channel[chan as usize].channum == 9 && self.settings.synth.drums_channel_active {
            preset = self.find_preset(128 as i32 as u32, prognum as u32)
        } else {
            preset = self.find_preset(banknum, prognum as u32)
        }

        if preset.is_none() {
            subst_bank = banknum as i32;
            subst_prog = prognum;
            if banknum != 128 as i32 as u32 {
                subst_bank = 0 as i32;
                preset = self.find_preset(0 as i32 as u32, prognum as u32);
                if preset.is_none() && prognum != 0 as i32 {
                    preset = self.find_preset(0 as i32 as u32, 0 as i32 as u32);
                    subst_prog = 0 as i32
                }
            } else {
                preset = self.find_preset(128 as i32 as u32, 0 as i32 as u32);
                subst_prog = 0 as i32
            }
            if preset.is_none() {
                log::warn!(
                        "Instrument not found on channel {} [bank={} prog={}], substituted [bank={} prog={}]",
                        chan, banknum, prognum,
                        subst_bank, subst_prog);
            }
        }
        sfont_id = if let Some(preset) = &preset {
            unsafe { (*preset.sfont).id }
        } else {
            0
        };
        self.channel[chan as usize].set_sfontnum(sfont_id);
        self.channel[chan as usize].set_preset(preset);
        return FLUID_OK as i32;
    }

    /**
    Set channel pressure
     */
    pub unsafe fn channel_pressure(&mut self, chan: u8, val: i32) -> i32 {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.settings.synth.verbose {
            log::info!("channelpressure\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pressure(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    /**
    Set key pressure (aftertouch)
     */
    pub unsafe fn key_pressure(&mut self, chan: i32, key: i32, val: i32) -> i32 {
        let mut result: i32 = FLUID_OK as i32;
        if key < 0 as i32 || key > 127 as i32 {
            return FLUID_FAILED as i32;
        }
        if val < 0 as i32 || val > 127 as i32 {
            return FLUID_FAILED as i32;
        }
        if self.settings.synth.verbose {
            log::info!("keypressure\t{}\t{}\t{}", chan, key, val);
        }
        self.channel[chan as usize].key_pressure[key as usize] = val as i8;
        let mut i;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan as i32 == chan && voice.key as i32 == key {
                result = fluid_voice_modulate(voice, 0 as i32, FLUID_MOD_KEYPRESSURE as i32);
                if result != FLUID_OK as i32 {
                    break;
                }
            }
            i += 1
        }
        return result;
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: u8, bank: u32) -> i32 {
        if chan < self.settings.synth.midi_channels {
            self.channel[chan as usize].set_banknum(bank);
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    /**
    Select a sfont.
     */
    pub unsafe fn sfont_select(&mut self, chan: u8, sfont_id: u32) -> i32 {
        if chan < self.settings.synth.midi_channels {
            self.channel[chan as usize].set_sfontnum(sfont_id);
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    /**
    Select a preset for a channel. The preset is specified by the
    SoundFont ID, the bank number, and the preset number. This
    allows any preset to be selected and circumvents preset masking
    due to previously loaded SoundFonts on the SoundFont stack.
     */
    pub unsafe fn program_select(
        &mut self,
        chan: u8,
        sfont_id: u32,
        bank_num: u32,
        preset_num: u32,
    ) -> i32 {
        let preset;
        let channel;
        if chan >= self.settings.synth.midi_channels {
            log::error!("Channel number out of range (chan={})", chan);
            return FLUID_FAILED as i32;
        }
        preset = self.get_preset(sfont_id, bank_num, preset_num);
        if preset.is_none() {
            log::error!(
                "There is no preset with bank number {} and preset number {} in SoundFont {}",
                bank_num,
                preset_num,
                sfont_id
            );
            return FLUID_FAILED as i32;
        }
        channel = &mut self.channel[chan as usize];
        channel.set_sfontnum(sfont_id);
        channel.set_banknum(bank_num);
        channel.set_prognum(preset_num as i32);
        channel.set_preset(preset);
        return FLUID_OK as i32;
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub unsafe fn get_program(
        &self,
        chan: u8,
        sfont_id: *mut u32,
        bank_num: *mut u32,
        preset_num: *mut u32,
    ) -> i32 {
        let channel;
        if chan < self.settings.synth.midi_channels {
            channel = &self.channel[chan as usize];
            *sfont_id = channel.get_sfontnum();
            *bank_num = channel.get_banknum();
            *preset_num = channel.get_prognum() as u32;
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.midi_channels {
            self.program_change(i, self.channel[i as usize].get_prognum());
            i += 1
        }
        return FLUID_OK as i32;
    }

    /**
    Send a reset.

    A reset turns all the notes off and resets the controller values.
     */
    pub unsafe fn system_reset(&mut self) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.status as i32 == FLUID_VOICE_ON as i32
                || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        let mut i = 0;
        while i < self.settings.synth.midi_channels {
            // TODO: double borrow
            let synth_ptr = self as *mut Synth;
            synth_ptr.as_mut().unwrap().channel[i as usize].reset(synth_ptr.as_mut().unwrap());
            i += 1
        }
        self.chorus.reset();
        self.reverb.reset();
        return FLUID_OK as i32;
    }
}
