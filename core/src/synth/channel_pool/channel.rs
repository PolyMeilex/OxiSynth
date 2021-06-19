use std::rc::Rc;

use super::super::soundfont::{Preset, SoundFont};

use crate::tuning::Tuning;
use crate::utils::TypedIndex;

type MidiControlChange = u32;
const ALL_SOUND_OFF: MidiControlChange = 120;
const RPN_MSB: MidiControlChange = 101;
const RPN_LSB: MidiControlChange = 100;
const NRPN_MSB: MidiControlChange = 99;
const NRPN_LSB: MidiControlChange = 98;
const EFFECTS_DEPTH5: MidiControlChange = 95;
const EFFECTS_DEPTH1: MidiControlChange = 91;
const SOUND_CTRL10: MidiControlChange = 79;
const SOUND_CTRL1: MidiControlChange = 70;
const EXPRESSION_LSB: MidiControlChange = 43;
const PAN_LSB: MidiControlChange = 42;
const VOLUME_LSB: MidiControlChange = 39;
const BANK_SELECT_LSB: MidiControlChange = 32;
const EXPRESSION_MSB: MidiControlChange = 11;
const PAN_MSB: MidiControlChange = 10;
const VOLUME_MSB: MidiControlChange = 7;
const BANK_SELECT_MSB: MidiControlChange = 0;

/* Flags to choose the interpolation method */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterpolationMethod {
    /// No interpolation: Fastest, but questionable audio quality
    None = 0,
    /// Straight-line interpolation: A bit slower, reasonable audio quality
    Linear = 1,
    /// Fourth-order interpolation: Requires 50% of the whole DSP processing time, good quality (default)
    FourthOrder = 4,
    /// Seventh-order interpolation
    SeventhOrder = 7,
}

impl Default for InterpolationMethod {
    fn default() -> Self {
        Self::FourthOrder
    }
}

#[derive(Clone)]
pub struct Channel {
    id: usize,

    sfontnum: Option<TypedIndex<SoundFont>>,

    banknum: u32,
    prognum: u8,

    preset: Option<Rc<Preset>>,

    key_pressure: [i8; 128],
    channel_pressure: u8,

    pitch_bend: i16,
    pitch_wheel_sensitivity: u8,

    cc: [u8; 128],
    bank_msb: u8,

    interp_method: InterpolationMethod,
    tuning: Option<Tuning>,

    nrpn_select: i16,
    nrpn_active: i16,

    gen: [f32; 60],
    gen_abs: [i8; 60],
}

impl Channel {
    pub fn new(id: usize, preset: Option<Rc<Preset>>) -> Self {
        let mut chan = Self {
            id,
            sfontnum: None,
            banknum: 0,
            prognum: 0,

            preset,

            key_pressure: [0; 128],
            channel_pressure: 0,

            pitch_bend: 0,
            pitch_wheel_sensitivity: 0,

            cc: [0; 128],
            bank_msb: 0,

            interp_method: Default::default(),
            tuning: None,

            nrpn_select: 0,
            nrpn_active: 0,

            gen: [0f32; 60],
            gen_abs: [0; 60],
        };
        chan.init_ctrl(0);
        chan
    }

    pub fn init(&mut self, preset: Option<Rc<Preset>>) {
        self.prognum = 0;
        self.banknum = 0;
        self.sfontnum = None;

        self.preset = preset;
        self.interp_method = Default::default();
        self.tuning = None;
        self.nrpn_select = 0;
        self.nrpn_active = 0;
    }

    pub fn init_ctrl(&mut self, is_all_ctrl_off: i32) {
        self.channel_pressure = 0;
        self.pitch_bend = 0x2000;

        for i in 0..60 {
            self.gen[i as usize] = 0.0;
            self.gen_abs[i as usize] = 0;
        }

        if is_all_ctrl_off != 0 {
            for i in 0..ALL_SOUND_OFF {
                if !(i >= EFFECTS_DEPTH1 && i <= EFFECTS_DEPTH5) {
                    if !(i >= SOUND_CTRL1 && i <= SOUND_CTRL10) {
                        if !(i == BANK_SELECT_MSB
                            || i == BANK_SELECT_LSB
                            || i == VOLUME_MSB
                            || i == VOLUME_LSB
                            || i == PAN_MSB
                            || i == PAN_LSB)
                        {
                            self.cc[i as usize] = 0;
                        }
                    }
                }
            }
        } else {
            for i in 0..128 {
                self.cc[i] = 0;
            }
        }

        for i in 0..128 {
            self.key_pressure[i] = 0;
        }

        self.cc[RPN_LSB as usize] = 127;
        self.cc[RPN_MSB as usize] = 127;
        self.cc[NRPN_LSB as usize] = 127;
        self.cc[NRPN_MSB as usize] = 127;
        self.cc[EXPRESSION_MSB as usize] = 127;
        self.cc[EXPRESSION_LSB as usize] = 127;

        if is_all_ctrl_off == 0 {
            self.pitch_wheel_sensitivity = 2;

            let mut i = SOUND_CTRL1;
            while i <= SOUND_CTRL10 {
                self.cc[i as usize] = 64;
                i += 1
            }

            self.cc[VOLUME_MSB as usize] = 100;
            self.cc[VOLUME_LSB as usize] = 0;
            self.cc[PAN_MSB as usize] = 64;
            self.cc[PAN_LSB as usize] = 0;
        };
    }
}

impl Channel {
    pub fn id(&self) -> usize {
        self.id
    }

    //

    pub fn sfontnum(&self) -> Option<TypedIndex<SoundFont>> {
        self.sfontnum
    }

    pub fn set_sfontnum(&mut self, sfontnum: Option<TypedIndex<SoundFont>>) {
        self.sfontnum = sfontnum;
    }

    //

    pub fn banknum(&self) -> u32 {
        self.banknum
    }

    pub fn set_banknum(&mut self, banknum: u32) {
        self.banknum = banknum;
    }

    //

    pub fn prognum(&self) -> u8 {
        self.prognum
    }

    pub fn set_prognum(&mut self, prognum: u8) {
        self.prognum = prognum;
    }

    //

    pub fn preset(&self) -> Option<&Rc<Preset>> {
        self.preset.as_ref()
    }

    pub fn set_preset(&mut self, preset: Option<Rc<Preset>>) {
        self.preset = preset;
    }

    //

    pub fn key_pressure(&self, id: usize) -> i8 {
        self.key_pressure[id]
    }

    pub fn set_key_pressure(&mut self, id: usize, val: i8) {
        self.key_pressure[id] = val;
    }

    //

    pub fn channel_pressure(&self) -> u8 {
        self.channel_pressure
    }

    pub fn set_channel_pressure(&mut self, val: u8) {
        self.channel_pressure = val;
    }

    //

    pub fn pitch_bend(&self) -> i16 {
        self.pitch_bend
    }

    pub fn set_pitch_bend(&mut self, val: i16) {
        self.pitch_bend = val;
    }

    //

    pub fn pitch_wheel_sensitivity(&self) -> u8 {
        self.pitch_wheel_sensitivity
    }

    pub fn set_pitch_wheel_sensitivity(&mut self, val: u8) {
        self.pitch_wheel_sensitivity = val;
    }

    //

    pub fn cc(&self, id: usize) -> u8 {
        if id < 128 {
            self.cc[id]
        } else {
            0
        }
    }

    pub fn cc_mut(&mut self, id: usize) -> &mut u8 {
        &mut self.cc[id]
    }

    //

    pub fn bank_msb(&self) -> u8 {
        self.bank_msb
    }

    pub fn set_bank_msb(&mut self, val: u8) {
        self.bank_msb = val;
    }

    //

    pub fn interp_method(&self) -> InterpolationMethod {
        self.interp_method
    }

    pub fn set_interp_method(&mut self, new_method: InterpolationMethod) {
        self.interp_method = new_method;
    }

    //

    pub fn tuning(&self) -> Option<&Tuning> {
        self.tuning.as_ref()
    }

    pub fn set_tuning(&mut self, val: Option<Tuning>) {
        self.tuning = val;
    }

    //

    pub fn nrpn_select(&self) -> i16 {
        self.nrpn_select
    }

    pub fn set_nrpn_select(&mut self, value: i16) {
        self.nrpn_select = value;
    }

    //

    pub fn nrpn_active(&self) -> i16 {
        self.nrpn_active
    }

    pub fn set_nrpn_active(&mut self, value: i16) {
        self.nrpn_active = value;
    }

    //

    pub fn gen(&self, id: usize) -> f32 {
        self.gen[id]
    }

    pub fn set_gen(&mut self, id: usize, val: f32) {
        self.gen[id] = val;
    }

    //

    pub fn gen_abs(&self, id: usize) -> i8 {
        self.gen_abs[id]
    }

    pub fn set_gen_abs(&mut self, id: usize, val: i8) {
        self.gen_abs[id] = val;
    }
}
