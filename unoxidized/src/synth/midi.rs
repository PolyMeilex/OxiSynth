use crate::synth::Synth;
use crate::synth::FLUID_MOD_KEYPRESSURE;
use crate::synth::FLUID_OK;

impl Synth {
    /**
    Send a noteon message.
     */
    pub fn noteon(&mut self, midi_chan: u8, key: u8, vel: i32) -> Result<(), ()> {
        if midi_chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range");
            return Err(());
        }
        if vel == 0 as i32 {
            return self.noteoff(midi_chan, key);
        }
        if self.channel[midi_chan as usize].preset.is_none() {
            log::trace!(
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
            return Err(());
        }

        self.voices.release_voice_on_same_note(
            &self.channel,
            midi_chan,
            key,
            self.settings.synth.polyphony,
            self.noteid,
            self.min_note_length_ticks,
        );

        let id = self.noteid;
        self.noteid = self.noteid.wrapping_add(1);

        // Start
        if midi_chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            Err(())
        } else if key >= 128 {
            log::warn!("Key out of range",);
            Err(())
        } else if vel <= 0 || vel >= 128 {
            log::warn!("Velocity out of range",);
            Err(())
        } else {
            self.storeid = id;
            self.sf_noteon(midi_chan, key, vel)
        }
    }

    /**
    Send a noteoff message.
     */
    pub fn noteoff(&mut self, chan: u8, key: u8) -> Result<(), ()> {
        let mut status = Err(());
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_on() && voice.chan == chan && voice.key == key {
                log::trace!(
                    "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                    voice.chan,
                    voice.key,
                    0 as i32,
                    voice.id,
                    (voice.start_time.wrapping_add(voice.ticks) as f32 / 44100.0f32) as f64,
                    (voice.ticks as f32 / 44100.0f32) as f64,
                    {
                        let mut used_voices: i32 = 0 as i32;
                        for _ in 0..self.settings.synth.polyphony {
                            if !voice.is_available() {
                                used_voices += 1
                            }
                        }
                        used_voices
                    }
                );
                voice.noteoff(&self.channel, self.min_note_length_ticks);
                status = Ok(());
            }
        }
        return status;
    }

    /**
    Send a control change message.
     */
    pub fn cc(&mut self, chan: u8, num: u16, val: u16) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
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

    pub fn all_notes_off(&mut self, chan: u8) {
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_playing() && voice.chan == chan {
                voice.noteoff(&self.channel, self.min_note_length_ticks);
            }
        }
    }

    pub fn all_sounds_off(&mut self, chan: u8) {
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_playing() && voice.chan == chan {
                voice.off();
            }
        }
    }

    /**
    Send a pitch bend message.
     */
    pub fn pitch_bend(&mut self, chan: u8, val: i32) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return Err(());
        }

        log::trace!("pitchb\t{}\t{}", chan, val);

        // chan.pitch_bend()
        {
            const FLUID_MOD_PITCHWHEEL: u16 = 14;

            self.channel[chan as usize].pitch_bend = val as i16;
            self.voices.modulate_voices(
                &self.channel,
                self.channel[chan as usize].channum,
                0,
                FLUID_MOD_PITCHWHEEL,
                self.settings.synth.polyphony,
            )
        }

        Ok(())
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
    pub fn pitch_wheel_sens(&mut self, chan: u8, val: u16) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return Err(());
        }

        log::trace!("pitchsens\t{}\t{}", chan, val);

        // chan.pitch_wheel_sens()
        {
            const FLUID_MOD_PITCHWHEELSENS: u16 = 16;

            self.channel[chan as usize].pitch_wheel_sensitivity = val;
            self.voices.modulate_voices(
                &self.channel,
                self.channel[chan as usize].channum,
                0,
                FLUID_MOD_PITCHWHEELSENS,
                self.settings.synth.polyphony,
            );
        }

        return Ok(());
    }

    /**
    Get the pitch wheel sensitivity.
     */
    pub fn get_pitch_wheel_sens(&self, chan: u8) -> Result<u32, ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            Err(())
        } else {
            Ok(self.channel[chan as usize].pitch_wheel_sensitivity as u32)
        }
    }

    /**
    Send a program change message.
     */
    pub fn program_change(&mut self, chan: u8, prognum: i32) -> Result<(), ()> {
        let mut preset;
        let banknum;
        let sfont_id;
        let mut subst_bank;
        let mut subst_prog;
        if prognum < 0 as i32 || prognum >= 128 as i32 || chan >= self.settings.synth.midi_channels
        {
            log::error!("Index out of range (chan={}, prog={})", chan, prognum);
            return Err(());
        }
        banknum = self.channel[chan as usize].get_banknum();
        self.channel[chan as usize].set_prognum(prognum);

        log::trace!("prog\t{}\t{}\t{}", chan, banknum, prognum);

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
            preset.sfont_id
        } else {
            0
        };
        self.channel[chan as usize].set_sfontnum(sfont_id);
        self.channel[chan as usize].set_preset(preset);

        Ok(())
    }

    /**
    Set channel pressure
     */
    pub fn channel_pressure(&mut self, chan: u8, val: i32) -> Result<(), ()> {
        if chan >= self.settings.synth.midi_channels {
            log::warn!("Channel out of range",);
            return Err(());
        }

        log::trace!("channelpressure\t{}\t{}", chan, val);

        // channel.pressure()
        {
            const FLUID_MOD_CHANNELPRESSURE: u16 = 13;

            self.channel[chan as usize].channel_pressure = val as i16;
            self.voices.modulate_voices(
                &self.channel,
                self.channel[chan as usize].channum,
                0,
                FLUID_MOD_CHANNELPRESSURE,
                self.settings.synth.polyphony,
            );
        }

        Ok(())
    }

    /**
    Set key pressure (aftertouch)
     */
    pub fn key_pressure(&mut self, chan: i32, key: i32, val: i32) -> Result<(), ()> {
        let mut result: i32 = FLUID_OK as i32;
        if key < 0 as i32 || key > 127 as i32 {
            return Err(());
        }
        if val < 0 as i32 || val > 127 as i32 {
            return Err(());
        }

        log::trace!("keypressure\t{}\t{}\t{}", chan, key, val);

        self.channel[chan as usize].key_pressure[key as usize] = val as i8;
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan as i32 == chan && voice.key as i32 == key {
                result = voice.modulate(&self.channel, 0 as i32, FLUID_MOD_KEYPRESSURE as u16);
                if result != FLUID_OK as i32 {
                    break;
                }
            }
        }

        if result == FLUID_OK {
            Ok(())
        } else {
            Err(())
        }
    }

    /**
    Select a bank.
     */
    pub fn bank_select(&mut self, chan: u8, bank: u32) -> Result<(), ()> {
        if chan < self.settings.synth.midi_channels {
            self.channel[chan as usize].set_banknum(bank);
            return Ok(());
        }
        Err(())
    }

    /**
    Select a sfont.
     */
    pub fn sfont_select(&mut self, chan: u8, sfont_id: u32) -> Result<(), ()> {
        if chan < self.settings.synth.midi_channels {
            self.channel[chan as usize].set_sfontnum(sfont_id);
            return Ok(());
        }
        Err(())
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
        sfont_id: u32,
        bank_num: u32,
        preset_num: u32,
    ) -> Result<(), ()> {
        let preset;
        let channel;
        if chan >= self.settings.synth.midi_channels {
            log::error!("Channel number out of range (chan={})", chan);
            return Err(());
        }
        preset = self.get_preset(sfont_id, bank_num, preset_num);
        if preset.is_none() {
            log::error!(
                "There is no preset with bank number {} and preset number {} in SoundFont {}",
                bank_num,
                preset_num,
                sfont_id
            );
            return Err(());
        }
        channel = &mut self.channel[chan as usize];
        channel.set_sfontnum(sfont_id);
        channel.set_banknum(bank_num);
        channel.set_prognum(preset_num as i32);
        channel.set_preset(preset);

        Ok(())
    }

    /**
    Returns the program, bank, and SoundFont number of the preset on a given channel.
     */
    pub fn get_program(&self, chan: u8) -> Result<(u32, u32, u32), ()> {
        if chan < self.settings.synth.midi_channels {
            let channel = &self.channel[chan as usize];

            Ok((
                channel.get_sfontnum(),
                channel.get_banknum(),
                channel.get_prognum() as u32,
            ))
        } else {
            Err(())
        }
    }

    /**
    Send a bank select and a program change to every channel to reinitialize the preset of the channel.

    This function is useful mainly after a SoundFont has been loaded, unloaded or reloaded.
     */
    pub fn program_reset(&mut self) {
        let mut i = 0;
        while i < self.settings.synth.midi_channels {
            self.program_change(i, self.channel[i as usize].get_prognum())
                .ok();
            i += 1
        }
    }

    /**
    Send a reset.

    A reset turns all the notes off and resets the controller values.

    Purpose:
    Respond to the MIDI command 'system reset' (0xFF, big red 'panic' button)
     */
    pub fn system_reset(&mut self) {
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_playing() {
                voice.off();
            }
        }

        for i in 0..self.settings.synth.midi_channels {
            // channel.reset()
            let preset = self.find_preset(0, 0);
            self.channel[i as usize].init(preset);
            self.channel[i as usize].init_ctrl(0 as i32);
        }
        self.chorus.reset();
        self.reverb.reset();
    }
}
