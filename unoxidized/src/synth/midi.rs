use crate::synth::Synth;

impl Synth {
    /**
    Send a noteon message.
     */
    pub fn noteon(&mut self, midi_chan: u8, key: u8, vel: u8) -> Result<(), &str> {
        if key >= 128 {
            log::error!("Key out of range");
            Err("Key out of range")
        } else if vel >= 128 {
            log::error!("Velocity out of range");
            Err("Velocity out of range")
        } else if let Some(channel) = self.channels.get_mut(midi_chan as usize) {
            if vel == 0 {
                self.noteoff(midi_chan, key);
                Ok(())
            } else if channel.preset.is_none() {
                log::warn!(
                    "noteon\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}\t{}",
                    midi_chan,
                    key,
                    vel,
                    0,
                    (self.ticks as f32 / 44100.0f32),
                    0.0f32,
                    0,
                    "channel has no preset"
                );
                Err("Channel has no preset")
            } else {
                self.voices.release_voice_on_same_note(
                    &self.channels,
                    midi_chan,
                    key,
                    self.noteid,
                    self.min_note_length_ticks,
                );

                let id = self.noteid;
                self.noteid = self.noteid.wrapping_add(1);

                self.storeid = id;
                self.sf_noteon(midi_chan, key, vel);
                Ok(())
            }
        } else {
            log::error!("Channel out of range");
            Err("Channel out of range")
        }
    }

    /**
    Send a noteoff message.
     */
    pub fn noteoff(&mut self, chan: u8, key: u8) {
        self.voices
            .noteoff(&self.channels, self.min_note_length_ticks, chan, key)
    }

    /**
    Send a control change message.
     */
    pub fn cc(&mut self, chan: u8, num: u16, val: u16) -> Result<(), ()> {
        if chan as usize >= self.channels.len() {
            log::warn!("Channel out of range",);
            return Err(());
        }
        if num >= 128 {
            log::warn!("Ctrl out of range",);
            return Err(());
        }
        if val >= 128 {
            log::warn!("Value out of range",);
            return Err(());
        }

        log::trace!("cc\t{}\t{}\t{}", chan, num, val);

        self.channel_cc(chan as usize, num, val);

        Ok(())
    }

    /**
    Get a control value.
     */
    pub fn get_cc(&self, chan: u8, num: u16) -> Result<u8, &str> {
        if let Some(channel) = self.channels.get(chan as usize) {
            if num >= 128 {
                log::warn!("Ctrl out of range");
                Err("Ctrl out of range")
            } else {
                let pval = channel.cc[num as usize];
                Ok(pval)
            }
        } else {
            log::warn!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    pub fn all_notes_off(&mut self, chan: u8) {
        self.voices
            .all_notes_off(&self.channels, self.min_note_length_ticks, chan)
    }

    pub fn all_sounds_off(&mut self, chan: u8) {
        self.voices.all_sounds_off(chan)
    }

    /**
    Send a pitch bend message.
     */
    pub fn pitch_bend(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            log::trace!("pitchb\t{}\t{}", chan, val);

            const FLUID_MOD_PITCHWHEEL: u16 = 14;

            channel.pitch_bend = val as i16;

            let channum = channel.channum;
            self.voices
                .modulate_voices(&self.channels, channum, 0, FLUID_MOD_PITCHWHEEL);

            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Get the pitch bend value.
     */
    pub fn get_pitch_bend(&self, chan: u8) -> Result<i16, &str> {
        if let Some(channel) = self.channels.get(chan as usize) {
            let pitch_bend = channel.pitch_bend;
            Ok(pitch_bend)
        } else {
            log::warn!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Set the pitch wheel sensitivity.
     */
    pub fn pitch_wheel_sens(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            log::trace!("pitchsens\t{}\t{}", chan, val);

            const FLUID_MOD_PITCHWHEELSENS: u16 = 16;

            channel.pitch_wheel_sensitivity = val;

            let channum = channel.channum;

            self.voices
                .modulate_voices(&self.channels, channum, 0, FLUID_MOD_PITCHWHEELSENS);

            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: u8) -> Result<u32, &str> {
        if let Some(channel) = self.channels.get(chan as usize) {
            Ok(channel.pitch_wheel_sensitivity as u32)
        } else {
            log::warn!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: u8, prognum: u8) -> Result<(), ()> {
        let mut preset;
        let banknum;
        let sfont_id;
        let mut subst_bank;
        let mut subst_prog;

        if prognum >= 128 || chan as usize >= self.channels.len() {
            log::error!("Index out of range (chan={}, prog={})", chan, prognum);
            return Err(());
        }

        banknum = self.channels[chan as usize].get_banknum();
        self.channels[chan as usize].set_prognum(prognum);

        log::trace!("prog\t{}\t{}\t{}", chan, banknum, prognum);

        if self.channels[chan as usize].channum == 9 && self.settings.drums_channel_active {
            preset = self.find_preset(128, prognum)
        } else {
            preset = self.find_preset(banknum, prognum)
        }

        if preset.is_none() {
            subst_bank = banknum as i32;
            subst_prog = prognum;
            if banknum != 128 {
                subst_bank = 0;
                preset = self.find_preset(0, prognum);
                if preset.is_none() && prognum != 0 {
                    preset = self.find_preset(0, 0);
                    subst_prog = 0;
                }
            } else {
                preset = self.find_preset(128, 0);
                subst_prog = 0;
            }
            if preset.is_none() {
                log::warn!(
                        "Instrument not found on channel {} [bank={} prog={}], substituted [bank={} prog={}]",
                        chan, banknum, prognum,
                        subst_bank, subst_prog);
            }
        }
        sfont_id = if let Some(preset) = &preset {
            preset.sfont_id
        } else {
            0
        };
        self.channels[chan as usize].set_sfontnum(sfont_id);
        self.channels[chan as usize].set_preset(preset);

        Ok(())
    }

    /**
    Set channel pressure
     */
    pub fn channel_pressure(&mut self, chan: u8, val: u16) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            log::trace!("channelpressure\t{}\t{}", chan, val);

            const FLUID_MOD_CHANNELPRESSURE: u16 = 13;
            channel.channel_pressure = val as i16;

            self.voices.modulate_voices(
                &self.channels,
                self.channels[chan as usize].channum,
                0,
                FLUID_MOD_CHANNELPRESSURE,
            );
            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Set key pressure (aftertouch)
     */
    pub fn key_pressure(&mut self, chan: u8, key: u8, val: u8) -> Result<(), ()> {
        if key > 127 {
            return Err(());
        }
        if val > 127 {
            return Err(());
        }

        log::trace!("keypressure\t{}\t{}\t{}", chan, key, val);

        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.key_pressure[key as usize] = val as i8;

            self.voices.key_pressure(&self.channels, chan, key);
            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err(())
        }
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: u8, bank: u32) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.set_banknum(bank);
            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: u8, sfont_id: usize) -> Result<(), &str> {
        if let Some(channel) = self.channels.get_mut(chan as usize) {
            channel.set_sfontnum(sfont_id);
            Ok(())
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
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
        sfont_id: usize,
        bank_num: u32,
        preset_num: u8,
    ) -> Result<(), &str> {
        let preset = self.get_preset(sfont_id, bank_num, preset_num);

        if let Some(channel) = self.channels.get_mut(chan as usize) {
            if preset.is_none() {
                log::error!(
                    "There is no preset with bank number {} and preset number {} in SoundFont {}",
                    bank_num,
                    preset_num,
                    sfont_id
                );
                Err("This preset does not exist")
            } else {
                channel.set_sfontnum(sfont_id);
                channel.set_banknum(bank_num);
                channel.set_prognum(preset_num);
                channel.set_preset(preset);
                Ok(())
            }
        } else {
            log::error!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: u8) -> Result<(usize, u32, u32), &str> {
        if let Some(channel) = self.channels.get(chan as usize) {
            Ok((
                channel.get_sfontnum(),
                channel.get_banknum(),
                channel.get_prognum() as u32,
            ))
        } else {
            log::warn!("Channel out of range",);
            Err("Channel out of range")
        }
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) {
        for id in 0..self.channels.len() {
            let preset = self.channels[id].get_prognum();
            self.program_change(id as u8, preset).ok();
        }
    }

    /**
    Send a reset.

    A reset turns all the notes off and resets the controller values.

    Purpose:
    Respond to the MIDI command 'system reset' (0xFF, big red 'panic' button)
     */
    pub fn system_reset(&mut self) {
        self.voices.system_reset();

        let preset = self.find_preset(0, 0);
        for channel in self.channels.iter_mut() {
            channel.init(preset.clone());
            channel.init_ctrl(0);
        }

        self.chorus.reset();
        self.reverb.reset();
    }
}
