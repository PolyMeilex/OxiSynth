use std::ops::RangeInclusive;
use std::sync::Arc;

use super::super::soundfont::{Preset, SoundFont};

use crate::arena::Index;
use crate::core::InterpolationMethod;
use crate::midi_event::ControlFunction;
use crate::GeneratorType;
use crate::Tuning;

#[derive(Clone)]
struct CcList([u8; 128]);

impl std::ops::Index<ControlFunction> for CcList {
    type Output = u8;

    fn index(&self, index: ControlFunction) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl std::ops::IndexMut<ControlFunction> for CcList {
    fn index_mut(&mut self, index: ControlFunction) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl std::ops::Index<RangeInclusive<ControlFunction>> for CcList {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<ControlFunction>) -> &Self::Output {
        let start = *index.start() as usize;
        let end = *index.end() as usize;
        &self.0[start..=end]
    }
}

impl std::ops::IndexMut<RangeInclusive<ControlFunction>> for CcList {
    fn index_mut(&mut self, index: RangeInclusive<ControlFunction>) -> &mut Self::Output {
        let start = *index.start() as usize;
        let end = *index.end() as usize;
        &mut self.0[start..=end]
    }
}

#[derive(Clone)]
pub struct Channel {
    id: usize,

    sfontnum: Option<Index<SoundFont>>,

    banknum: u32,
    prognum: u8,

    preset: Option<Arc<Preset>>,

    key_pressure: [i8; 128],
    channel_pressure: u8,

    pitch_bend: u16,
    pitch_wheel_sensitivity: u8,

    cc: CcList,
    bank_msb: u8,

    interp_method: InterpolationMethod,
    tuning: Option<Tuning>,

    nrpn_select: i16,
    nrpn_active: i16,

    gen: [f32; 60],
    gen_abs: [i8; 60],
}

impl Channel {
    pub fn new(id: usize) -> Self {
        let mut chan = Self {
            id,
            sfontnum: None,
            banknum: 0,
            prognum: 0,

            preset: None,

            key_pressure: [0; 128],
            channel_pressure: 0,

            pitch_bend: 0,
            pitch_wheel_sensitivity: 0,

            cc: CcList([0; 128]),
            bank_msb: 0,

            interp_method: InterpolationMethod::default(),
            tuning: None,

            nrpn_select: 0,
            nrpn_active: 0,

            gen: [0f32; 60],
            gen_abs: [0; 60],
        };
        chan.init_ctrl(false);
        chan
    }

    pub fn init(&mut self, preset: Option<Arc<Preset>>) {
        self.prognum = 0;
        self.banknum = 0;
        self.sfontnum = None;

        self.preset = preset;
        self.interp_method = Default::default();
        self.tuning = None;
        self.nrpn_select = 0;
        self.nrpn_active = 0;
    }

    pub fn init_ctrl(&mut self, is_all_ctrl_off: bool) {
        self.channel_pressure = 0;
        self.pitch_bend = 0x2000;

        self.gen.fill(0.0);
        self.gen_abs.fill(0);

        if is_all_ctrl_off {
            for i in ControlFunction::iter_range(ControlFunction::MIN..ControlFunction::AllSoundOff)
                .filter(|i| !i.is_effects_n_depth())
                .filter(|i| !i.is_sound_controller_n())
                .filter(|i| {
                    !matches!(
                        i,
                        ControlFunction::BankSelect | ControlFunction::BankSelectLsb
                    )
                })
                .filter(|i| {
                    !matches!(
                        i,
                        ControlFunction::ChannelVolume | ControlFunction::ChannelVolumeLsb
                    )
                })
                .filter(|i| !matches!(i, ControlFunction::Pan | ControlFunction::PanLsb))
            {
                self.cc[i] = 0;
            }
        } else {
            self.cc.0.fill(0);
        }

        self.key_pressure.fill(0);

        self.cc[ControlFunction::RegisteredParameterNumberLsb] = 127;
        self.cc[ControlFunction::RegisteredParameterNumberMsb] = 127;
        self.cc[ControlFunction::NonRegisteredParameterNumberLsb] = 127;
        self.cc[ControlFunction::NonRegisteredParameterNumberMsb] = 127;
        self.cc[ControlFunction::ExpressionController] = 127;
        self.cc[ControlFunction::ExpressionControllerLsb] = 127;

        if !is_all_ctrl_off {
            self.pitch_wheel_sensitivity = 2;

            self.cc[ControlFunction::SoundController1..=ControlFunction::SoundController10]
                .fill(64);
            self.cc[ControlFunction::ChannelVolume] = 100;
            self.cc[ControlFunction::ChannelVolumeLsb] = 0;
            self.cc[ControlFunction::Pan] = 64;
            self.cc[ControlFunction::PanLsb] = 0;
        };
    }
}

impl Channel {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn sfontnum(&self) -> Option<Index<SoundFont>> {
        self.sfontnum
    }

    pub fn set_sfontnum(&mut self, sfontnum: Option<Index<SoundFont>>) {
        self.sfontnum = sfontnum;
    }

    pub fn banknum(&self) -> u32 {
        self.banknum
    }

    pub fn set_banknum(&mut self, banknum: u32) {
        self.banknum = banknum;
    }

    pub fn prognum(&self) -> u8 {
        self.prognum
    }

    pub fn set_prognum(&mut self, prognum: u8) {
        self.prognum = prognum;
    }

    pub fn preset(&self) -> Option<&Arc<Preset>> {
        self.preset.as_ref()
    }

    pub fn set_preset(&mut self, preset: Option<Arc<Preset>>) {
        self.preset = preset;
    }

    pub fn key_pressure(&self, id: usize) -> i8 {
        self.key_pressure[id]
    }

    pub fn set_key_pressure(&mut self, id: usize, val: i8) {
        self.key_pressure[id] = val;
    }

    pub fn channel_pressure(&self) -> u8 {
        self.channel_pressure
    }

    pub fn set_channel_pressure(&mut self, val: u8) {
        self.channel_pressure = val;
    }

    pub fn pitch_bend(&self) -> u16 {
        self.pitch_bend
    }

    pub fn set_pitch_bend(&mut self, val: u16) {
        self.pitch_bend = val;
    }

    pub fn pitch_wheel_sensitivity(&self) -> u8 {
        self.pitch_wheel_sensitivity
    }

    pub fn set_pitch_wheel_sensitivity(&mut self, val: u8) {
        self.pitch_wheel_sensitivity = val;
    }

    pub fn cc(&self, id: usize) -> u8 {
        self.cc.0.get(id).copied().unwrap_or(0)
    }

    pub fn cc_mut(&mut self, id: usize) -> &mut u8 {
        &mut self.cc.0[id]
    }

    pub fn bank_msb(&self) -> u8 {
        self.bank_msb
    }

    pub fn set_bank_msb(&mut self, val: u8) {
        self.bank_msb = val;
    }

    pub fn interp_method(&self) -> InterpolationMethod {
        self.interp_method
    }

    pub fn set_interp_method(&mut self, new_method: InterpolationMethod) {
        self.interp_method = new_method;
    }

    pub fn tuning(&self) -> Option<&Tuning> {
        self.tuning.as_ref()
    }

    pub fn set_tuning(&mut self, val: Option<Tuning>) {
        self.tuning = val;
    }

    pub fn nrpn_select(&self) -> i16 {
        self.nrpn_select
    }

    pub fn set_nrpn_select(&mut self, value: i16) {
        self.nrpn_select = value;
    }

    pub fn nrpn_active(&self) -> i16 {
        self.nrpn_active
    }

    pub fn set_nrpn_active(&mut self, value: i16) {
        self.nrpn_active = value;
    }

    /// Retrieve the value of a generator. This function returns the value
    /// set by a previous call 'set_gen()' or by an NRPN message.
    ///
    /// Returns the value of the generator.
    pub fn gen(&self, id: GeneratorType) -> f32 {
        self.gen[id as usize]
    }

    pub fn set_gen(&mut self, id: GeneratorType, val: f32) {
        self.gen[id as usize] = val;
    }

    pub fn gen_abs(&self, id: GeneratorType) -> i8 {
        self.gen_abs[id as usize]
    }

    pub fn set_gen_abs(&mut self, id: GeneratorType, val: i8) {
        self.gen_abs[id as usize] = val;
    }
}
