use std::rc::Rc;

use crate::generator::{gen_scale_nrpn, GenParam};
use crate::soundfont::Preset;
use crate::soundfont::SoundFont;
use crate::synth::Synth;
use crate::tuning::Tuning;
use crate::utils::TypedIndex;

type GenType = u32;
const GEN_LAST: GenType = 60;

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
const DATA_ENTRY_LSB: MidiControlChange = 38;
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
    channel_pressure: i16,

    pitch_bend: i16,
    pitch_wheel_sensitivity: u16,

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

    pub fn channel_pressure(&self) -> i16 {
        self.channel_pressure
    }

    pub fn set_channel_pressure(&mut self, val: i16) {
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

    pub fn pitch_wheel_sensitivity(&self) -> u16 {
        self.pitch_wheel_sensitivity
    }

    pub fn set_pitch_wheel_sensitivity(&mut self, val: u16) {
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

impl Synth {
    // TODO: writing self.channel[id] every time is stupid, there has to be a better way
    pub(crate) fn channel_cc(&mut self, chan_id: usize, num: u16, value: u16) {
        self.channels[chan_id].cc[num as usize] = value as u8;

        match num {
            // SUSTAIN_SWITCH
            64 => {
                if value < 64 {
                    // sustain off
                    self.voices
                        .damp_voices(&self.channels[chan_id], self.min_note_length_ticks)
                } else {
                    // sustain on
                }
            }

            // BANK_SELECT_MSB
            0 => {
                if chan_id == 9 && self.settings.drums_channel_active {
                    // ignored
                    return;
                }

                let chan = &mut self.channels[chan_id];

                chan.bank_msb = (value & 0x7f) as u8;

                /* I fixed the handling of a MIDI bank select controller 0,
                e.g., bank select MSB (or "coarse" bank select according to
                my spec).  Prior to this fix a channel's bank number was only
                changed upon reception of MIDI bank select controller 32,
                e.g, bank select LSB (or "fine" bank-select according to my
                spec). [KLE]
                FIXME: is this correct? [PH] */
                chan.banknum = (value & 0x7f) as u32;
            }

            // BANK_SELECT_LSB
            32 => {
                if chan_id == 9 && self.settings.drums_channel_active {
                    // ignored
                    return;
                }

                let chan = &mut self.channels[chan_id];

                /* FIXME: according to the Downloadable Sounds II specification,
                bit 31 should be set when we receive the message on channel
                10 (drum channel) */
                chan.banknum =
                    (value as u32 & 0x7f).wrapping_add((chan.bank_msb as u32) << 7 as i32);
            }

            // ALL_NOTES_OFF
            123 => {
                self.all_notes_off(chan_id);
            }

            // ALL_SOUND_OFF
            120 => {
                self.all_sounds_off(chan_id);
            }

            // ALL_CTRL_OFF
            121 => {
                self.channels[chan_id].init_ctrl(1);
                self.voices.modulate_voices_all(&self.channels[chan_id]);
            }

            // DATA_ENTRY_MSB
            6 => {
                let data: i32 = ((value as i32) << 7 as i32)
                    + self.channels[chan_id].cc[DATA_ENTRY_LSB as usize] as i32;

                if self.channels[chan_id].nrpn_active != 0 {
                    let (channum, nrpn_select, nrpn_msb, nrpn_lsb) = {
                        let channel = &self.channels[chan_id];
                        (
                            channel.id(),
                            channel.nrpn_select,
                            channel.cc[NRPN_MSB as usize],
                            channel.cc[NRPN_LSB as usize],
                        )
                    };

                    // SontFont 2.01 NRPN Message (Sect. 9.6, p. 74)
                    if nrpn_msb == 120 && nrpn_lsb < 100 {
                        if (nrpn_select as i32) < GEN_LAST as i32 {
                            use num_traits::FromPrimitive;

                            let val: f32 = gen_scale_nrpn(nrpn_select, data);

                            let param = FromPrimitive::from_u8(nrpn_select as u8).unwrap();
                            self.set_gen(channum, param, val).unwrap();
                        }

                        self.channels[chan_id].nrpn_select = 0; // Reset to 0
                    }
                }
                /* RPN is active: MSB = 0? */
                else if self.channels[chan_id].cc[RPN_MSB as usize] == 0 {
                    match self.channels[chan_id].cc[RPN_LSB as usize] {
                        // RPN_PITCH_BEND_RANGE
                        0 => {
                            self.pitch_wheel_sens(chan_id, value).ok();
                        }
                        // RPN_CHANNEL_FINE_TUNE
                        1 => {
                            self.set_gen(
                                self.channels[chan_id].id(),
                                GenParam::FineTune,
                                ((data - 8192 as i32) as f64 / 8192.0f64 * 100.0f64) as f32,
                            )
                            .unwrap();
                        }
                        // RPN_CHANNEL_COARSE_TUNE
                        2 => {
                            self.set_gen(
                                self.channels[chan_id].id(),
                                GenParam::CoarseTune,
                                (value - 64) as f32,
                            )
                            .unwrap();
                        }
                        // RPN_TUNING_PROGRAM_CHANGE | RPN_TUNING_BANK_SELECT | RPN_MODULATION_DEPTH_RANGE
                        3 | 4 | 5 | _ => {}
                    }
                }
            }

            // NRPN_MSB
            99 => {
                let chan = &mut self.channels[chan_id];
                chan.cc[NRPN_LSB as usize] = 0;
                chan.nrpn_select = 0;
                chan.nrpn_active = 1;
            }

            // NRPN_LSB
            98 => {
                let chan = &mut self.channels[chan_id];
                // SontFont 2.01 NRPN Message (Sect. 9.6, p. 74)
                if chan.cc[NRPN_MSB as usize] == 120 {
                    if value == 100 {
                        chan.nrpn_select = chan.nrpn_select + 100;
                    } else if value == 101 {
                        chan.nrpn_select = chan.nrpn_select + 1000;
                    } else if value == 102 {
                        chan.nrpn_select = chan.nrpn_select + 10000;
                    } else if value < 100 {
                        chan.nrpn_select = chan.nrpn_select + value as i16;
                    }
                }
                chan.nrpn_active = 1 as i32 as i16
            }

            // RPN_MSB | RPN_LSB
            101 | 100 => self.channels[chan_id].nrpn_active = 0,
            _ => self.voices.modulate_voices(&self.channels[chan_id], 1, num),
        }
    }
}
