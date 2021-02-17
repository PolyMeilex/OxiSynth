use super::gen::{fluid_gen_scale_nrpn, GenParam};
use super::soundfont::Preset;
use super::synth::Synth;
use super::tuning::Tuning;
/* Flags to choose the interpolation method */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum InterpMethod {
    /**
    No interpolation: Fastest, but questionable audio quality
     */
    None = 0,
    /**
    Straight-line interpolation: A bit slower, reasonable audio quality
     */
    Linear = 1,
    /**
    Fourth-order interpolation: Requires 50% of the whole DSP processing time, good quality
    (default)
     */
    FourthOrder = 4,
    /**
    Seventh-order interpolation
     */
    SeventhOrder = 7,
}

impl Default for InterpMethod {
    fn default() -> Self {
        Self::FourthOrder
    }
}

#[derive(Clone)]
pub struct Channel {
    pub(crate) channum: i32,
    sfontnum: u32,
    banknum: u32,
    prognum: u32,
    pub(crate) preset: *mut Preset,
    pub(crate) key_pressure: [i8; 128],
    pub(crate) channel_pressure: i16,
    pub(crate) pitch_bend: i16,
    pub(crate) pitch_wheel_sensitivity: i16,
    pub(crate) cc: [i16; 128],
    bank_msb: u8,
    interp_method: InterpMethod,
    pub(crate) tuning: Option<Tuning>,
    nrpn_select: i16,
    nrpn_active: i16,
    pub(crate) gen: [f32; 60],
    pub(crate) gen_abs: [i8; 60],
}

pub type ModSrc = u32;
pub const FLUID_MOD_PITCHWHEELSENS: ModSrc = 16;
pub const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
pub const FLUID_MOD_CHANNELPRESSURE: ModSrc = 13;
pub type GenType = u32;
pub const GEN_LAST: GenType = 60;
pub const FLUID_OK: i32 = 0;
pub type MidiControlChange = u32;
pub const ALL_SOUND_OFF: MidiControlChange = 120;
pub const RPN_MSB: MidiControlChange = 101;
pub const RPN_LSB: MidiControlChange = 100;
pub const NRPN_MSB: MidiControlChange = 99;
pub const NRPN_LSB: MidiControlChange = 98;
pub const EFFECTS_DEPTH5: MidiControlChange = 95;
pub const EFFECTS_DEPTH1: MidiControlChange = 91;
pub const SOUND_CTRL10: MidiControlChange = 79;
pub const SOUND_CTRL1: MidiControlChange = 70;
pub const EXPRESSION_LSB: MidiControlChange = 43;
pub const PAN_LSB: MidiControlChange = 42;
pub const VOLUME_LSB: MidiControlChange = 39;
pub const DATA_ENTRY_LSB: MidiControlChange = 38;
pub const BANK_SELECT_LSB: MidiControlChange = 32;
pub const EXPRESSION_MSB: MidiControlChange = 11;
pub const PAN_MSB: MidiControlChange = 10;
pub const VOLUME_MSB: MidiControlChange = 7;
pub const BANK_SELECT_MSB: MidiControlChange = 0;

impl Channel {
    pub fn new(synth: &Synth, num: i32) -> Self {
        let mut chan = Self {
            channum: num,
            sfontnum: 0 as _,
            banknum: 0 as _,
            prognum: 0 as _,
            preset: 0 as _,
            key_pressure: [0; 128],
            channel_pressure: 0 as _,
            pitch_bend: 0 as _,
            pitch_wheel_sensitivity: 0 as _,
            cc: [0; 128],
            bank_msb: 0 as _,
            interp_method: Default::default(),
            tuning: None,
            nrpn_select: 0 as _,
            nrpn_active: 0 as _,
            gen: [0f32; 60],
            gen_abs: [0; 60],
        };
        chan.init(synth);
        chan.init_ctrl(0);
        return chan;
    }

    pub fn init(&mut self, synth: &Synth) {
        self.prognum = 0 as i32 as u32;
        self.banknum = 0 as i32 as u32;
        self.sfontnum = 0 as i32 as u32;
        match unsafe { self.preset.as_ref() } {
            Some(preset) => match preset.free {
                Some(free) => unsafe {
                    free(self.preset);
                },
                _ => {}
            },
            _ => {}
        }
        self.preset = unsafe { synth.find_preset(self.banknum, self.prognum) };
        self.interp_method = Default::default();
        self.tuning = None;
        self.nrpn_select = 0 as _;
        self.nrpn_active = 0 as _;
    }

    pub fn init_ctrl(&mut self, is_all_ctrl_off: i32) {
        self.channel_pressure = 0 as i32 as i16;
        self.pitch_bend = 0x2000 as i32 as i16;
        let mut i = 0 as i32;
        while i < GEN_LAST as i32 {
            self.gen[i as usize] = 0.0f32;
            self.gen_abs[i as usize] = 0 as i32 as i8;
            i += 1
        }
        if is_all_ctrl_off != 0 {
            i = 0 as i32;
            while i < ALL_SOUND_OFF as i32 {
                if !(i >= EFFECTS_DEPTH1 as i32 && i <= EFFECTS_DEPTH5 as i32) {
                    if !(i >= SOUND_CTRL1 as i32 && i <= SOUND_CTRL10 as i32) {
                        if !(i == BANK_SELECT_MSB as i32
                            || i == BANK_SELECT_LSB as i32
                            || i == VOLUME_MSB as i32
                            || i == VOLUME_LSB as i32
                            || i == PAN_MSB as i32
                            || i == PAN_LSB as i32)
                        {
                            self.cc[i as usize] = 0 as i32 as i16
                        }
                    }
                }
                i += 1
            }
        } else {
            i = 0 as i32;
            while i < 128 as i32 {
                self.cc[i as usize] = 0 as i32 as i16;
                i += 1
            }
        }
        i = 0 as i32;
        while i < 128 as i32 {
            self.key_pressure[i as usize] = 0 as i32 as i8;
            i += 1
        }
        self.cc[RPN_LSB as i32 as usize] = 127 as i32 as i16;
        self.cc[RPN_MSB as i32 as usize] = 127 as i32 as i16;
        self.cc[NRPN_LSB as i32 as usize] = 127 as i32 as i16;
        self.cc[NRPN_MSB as i32 as usize] = 127 as i32 as i16;
        self.cc[EXPRESSION_MSB as i32 as usize] = 127 as i32 as i16;
        self.cc[EXPRESSION_LSB as i32 as usize] = 127 as i32 as i16;
        if is_all_ctrl_off == 0 {
            self.pitch_wheel_sensitivity = 2 as i32 as i16;
            i = SOUND_CTRL1 as i32;
            while i <= SOUND_CTRL10 as i32 {
                self.cc[i as usize] = 64 as i32 as i16;
                i += 1
            }
            self.cc[VOLUME_MSB as i32 as usize] = 100 as i32 as i16;
            self.cc[VOLUME_LSB as i32 as usize] = 0 as i32 as i16;
            self.cc[PAN_MSB as i32 as usize] = 64 as i32 as i16;
            self.cc[PAN_LSB as i32 as usize] = 0 as i32 as i16
        };
    }

    pub fn reset(&mut self, synth: &Synth) {
        self.init(synth);
        self.init_ctrl(0 as i32);
    }

    pub fn set_preset(&mut self, preset: *mut Preset) -> i32 {
        unsafe {
            if !self.preset.is_null() {
                if !self.preset.is_null() && (*self.preset).free.is_some() {
                    Some((*self.preset).free.expect("non-null function pointer"))
                        .expect("non-null function pointer")(self.preset);
                }
            }
        }
        self.preset = preset;
        return FLUID_OK as i32;
    }

    pub fn get_preset(&self) -> *mut Preset {
        return self.preset;
    }

    pub fn get_banknum(&self) -> u32 {
        return self.banknum;
    }

    pub fn set_prognum(&mut self, prognum: i32) -> i32 {
        self.prognum = prognum as u32;
        return FLUID_OK as i32;
    }

    pub fn get_prognum(&self) -> i32 {
        return self.prognum as i32;
    }

    pub fn set_banknum(&mut self, banknum: u32) -> i32 {
        self.banknum = banknum;
        return FLUID_OK as i32;
    }

    pub fn cc(&mut self, synth: &mut Synth, num: i32, value: i32) -> i32 {
        unsafe {
            self.cc[num as usize] = value as i16;
            match num {
                64 => {
                    if value < 64 as i32 {
                        synth.damp_voices(self.channum);
                    }
                }
                0 => {
                    if self.channum == 9 as i32
                        && synth
                            .settings
                            .str_equal("synth.drums-channel.active", "yes")
                            != false
                    {
                        return FLUID_OK as i32;
                    }
                    self.bank_msb = (value & 0x7f as i32) as u8;
                    self.set_banknum((value & 0x7f as i32) as u32);
                }
                32 => {
                    if self.channum == 9 as i32
                        && synth
                            .settings
                            .str_equal("synth.drums-channel.active", "yes")
                            != false
                    {
                        return FLUID_OK as i32;
                    }
                    self.set_banknum(
                        (value as u32 & 0x7f as i32 as u32)
                            .wrapping_add((self.bank_msb as u32) << 7 as i32),
                    );
                }
                123 => {
                    synth.all_notes_off(self.channum);
                }
                120 => {
                    synth.all_sounds_off(self.channum);
                }
                121 => {
                    self.init_ctrl(1 as i32);
                    synth.modulate_voices_all(self.channum);
                }
                6 => {
                    let data: i32 =
                        (value << 7 as i32) + self.cc[DATA_ENTRY_LSB as i32 as usize] as i32;
                    if self.nrpn_active != 0 {
                        if self.cc[NRPN_MSB as i32 as usize] as i32 == 120 as i32
                            && (self.cc[NRPN_LSB as i32 as usize] as i32) < 100 as i32
                        {
                            if (self.nrpn_select as i32) < GEN_LAST as i32 {
                                let val: f32 = fluid_gen_scale_nrpn(self.nrpn_select, data);
                                synth.set_gen(self.channum, self.nrpn_select, val);
                            }
                            self.nrpn_select = 0
                        }
                    } else if self.cc[RPN_MSB as i32 as usize] as i32 == 0 as i32 {
                        match self.cc[RPN_LSB as i32 as usize] as i32 {
                            0 => {
                                self.pitch_wheel_sens(synth, value);
                            }
                            1 => {
                                synth.set_gen(
                                    self.channum,
                                    GenParam::FineTune as i16,
                                    ((data - 8192 as i32) as f64 / 8192.0f64 * 100.0f64) as f32,
                                );
                            }
                            2 => {
                                synth.set_gen(
                                    self.channum,
                                    GenParam::CoarseTune as i16,
                                    (value - 64 as i32) as f32,
                                );
                            }
                            3 | 4 | 5 | _ => {}
                        }
                    }
                }
                99 => {
                    self.cc[NRPN_LSB as i32 as usize] = 0 as i32 as i16;
                    self.nrpn_select = 0 as _;
                    self.nrpn_active = 1 as _
                }
                98 => {
                    if self.cc[NRPN_MSB as i32 as usize] as i32 == 120 as i32 {
                        if value == 100 as i32 {
                            self.nrpn_select = (self.nrpn_select as i32 + 100 as i32) as i16
                        } else if value == 101 as i32 {
                            self.nrpn_select = (self.nrpn_select as i32 + 1000 as i32) as i16
                        } else if value == 102 as i32 {
                            self.nrpn_select = (self.nrpn_select as i32 + 10000 as i32) as i16
                        } else if value < 100 as i32 {
                            self.nrpn_select = (self.nrpn_select as i32 + value) as i16
                        }
                    }
                    self.nrpn_active = 1 as i32 as i16
                }
                101 | 100 => self.nrpn_active = 0 as i32 as i16,
                _ => {
                    synth.modulate_voices(self.channum, 1 as i32, num);
                }
            }
        }
        return FLUID_OK as i32;
    }

    pub fn get_cc(&self, num: i32) -> i32 {
        return if num >= 0 as i32 && num < 128 as i32 {
            self.cc[num as usize] as i32
        } else {
            0 as i32
        };
    }

    pub fn pressure(&mut self, synth: &mut Synth, val: i32) -> i32 {
        self.channel_pressure = val as i16;
        unsafe {
            synth.modulate_voices(self.channum, 0 as i32, FLUID_MOD_CHANNELPRESSURE as i32);
        }
        return FLUID_OK as i32;
    }

    pub fn pitch_bend(&mut self, synth: &mut Synth, val: i32) -> i32 {
        self.pitch_bend = val as i16;
        unsafe {
            synth.modulate_voices(self.channum, 0 as i32, FLUID_MOD_PITCHWHEEL as i32);
        }
        return FLUID_OK as i32;
    }

    pub fn pitch_wheel_sens(&mut self, synth: &mut Synth, val: i32) -> i32 {
        self.pitch_wheel_sensitivity = val as i16;
        unsafe {
            synth.modulate_voices(self.channum, 0 as i32, FLUID_MOD_PITCHWHEELSENS as i32);
        }
        return FLUID_OK as i32;
    }

    pub fn get_num(&self) -> i32 {
        return self.channum;
    }

    pub fn set_interp_method(&mut self, new_method: InterpMethod) {
        self.interp_method = new_method;
    }

    pub fn get_interp_method(&self) -> InterpMethod {
        return self.interp_method;
    }

    pub fn get_sfontnum(&self) -> u32 {
        return self.sfontnum;
    }

    pub fn set_sfontnum(&mut self, sfontnum: u32) -> i32 {
        self.sfontnum = sfontnum;
        return FLUID_OK as i32;
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        match unsafe { self.preset.as_ref() } {
            Some(preset) => match preset.free {
                Some(free) => unsafe {
                    free(self.preset);
                },
                _ => {}
            },
            _ => {}
        }
    }
}
