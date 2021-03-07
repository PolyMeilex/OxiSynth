#![forbid(unsafe_code)]

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

pub struct ChannelId(pub usize);

#[derive(Clone)]
pub struct Channel {
    pub(crate) channum: u8,
    sfontnum: u32,
    banknum: u32,
    prognum: u32,
    pub(crate) preset: Option<Preset>,
    pub(crate) key_pressure: [i8; 128],
    pub(crate) channel_pressure: i16,
    pub(crate) pitch_bend: i16,
    pub(crate) pitch_wheel_sensitivity: u16,
    pub(crate) cc: [u8; 128],
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
    pub fn new(synth: &Synth, num: u8) -> Self {
        let mut chan = Self {
            channum: num,
            sfontnum: 0 as _,
            banknum: 0 as _,
            prognum: 0 as _,
            preset: None,
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
        chan.init(synth.find_preset(chan.banknum, chan.prognum));
        chan.init_ctrl(0);
        return chan;
    }

    pub fn init(&mut self, preset: Option<Preset>) {
        self.prognum = 0 as i32 as u32;
        self.banknum = 0 as i32 as u32;
        self.sfontnum = 0 as i32 as u32;

        self.preset = preset;
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
                            self.cc[i as usize] = 0;
                        }
                    }
                }
                i += 1
            }
        } else {
            i = 0 as i32;
            while i < 128 as i32 {
                self.cc[i as usize] = 0;
                i += 1
            }
        }
        i = 0 as i32;
        while i < 128 as i32 {
            self.key_pressure[i as usize] = 0 as i32 as i8;
            i += 1
        }
        self.cc[RPN_LSB as i32 as usize] = 127;
        self.cc[RPN_MSB as i32 as usize] = 127;
        self.cc[NRPN_LSB as i32 as usize] = 127;
        self.cc[NRPN_MSB as i32 as usize] = 127;
        self.cc[EXPRESSION_MSB as i32 as usize] = 127;
        self.cc[EXPRESSION_LSB as i32 as usize] = 127;
        if is_all_ctrl_off == 0 {
            self.pitch_wheel_sensitivity = 2;
            i = SOUND_CTRL1 as i32;
            while i <= SOUND_CTRL10 as i32 {
                self.cc[i as usize] = 64;
                i += 1
            }
            self.cc[VOLUME_MSB as i32 as usize] = 100;
            self.cc[VOLUME_LSB as i32 as usize] = 0;
            self.cc[PAN_MSB as i32 as usize] = 64;
            self.cc[PAN_LSB as i32 as usize] = 0;
        };
    }

    pub fn set_preset(&mut self, preset: Option<Preset>) -> i32 {
        self.preset = preset;
        return FLUID_OK as i32;
    }

    pub fn get_preset(&mut self) -> Option<&mut Preset> {
        self.preset.as_mut()
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

    pub fn set_banknum(&mut self, banknum: u32) {
        self.banknum = banknum;
    }

    pub fn get_cc(&self, num: i32) -> i32 {
        return if num >= 0 as i32 && num < 128 as i32 {
            self.cc[num as usize] as i32
        } else {
            0 as i32
        };
    }

    pub fn get_num(&self) -> u8 {
        self.channum
    }

    pub fn set_interp_method(&mut self, new_method: InterpMethod) {
        self.interp_method = new_method;
    }

    pub fn get_interp_method(&self) -> InterpMethod {
        self.interp_method
    }

    pub fn get_sfontnum(&self) -> u32 {
        self.sfontnum
    }

    pub fn set_sfontnum(&mut self, sfontnum: u32) {
        self.sfontnum = sfontnum;
    }
}

impl Synth {
    // TODO: writing self.channel[id] every time is stupid, there has to be a better way
    pub(crate) fn channel_cc(&mut self, chan_id: usize, num: u16, value: u16) {
        {
            let chan = &mut self.channel[chan_id];
            chan.cc[num as usize] = value as u8;
        }

        let channum = self.channel[chan_id].channum;

        match num {
            // SUSTAIN_SWITCH
            64 => {
                if value < 64 {
                    // sustain off
                    self.voices.damp_voices(
                        &self.channel,
                        channum,
                        self.settings.synth.polyphony,
                        self.min_note_length_ticks,
                    )
                } else {
                    // sustain on
                }
            }

            // BANK_SELECT_MSB
            0 => {
                let chan = &mut self.channel[chan_id];
                if channum == 9 && self.settings.synth.drums_channel_active {
                    // ignored
                    return;
                }
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
                let chan = &mut self.channel[chan_id];
                if channum == 9 && self.settings.synth.drums_channel_active {
                    // ignored
                    return;
                }

                /* FIXME: according to the Downloadable Sounds II specification,
                bit 31 should be set when we receive the message on channel
                10 (drum channel) */
                chan.banknum =
                    (value as u32 & 0x7f).wrapping_add((chan.bank_msb as u32) << 7 as i32);
            }

            // ALL_NOTES_OFF
            123 => {
                self.all_notes_off(channum);
            }

            // ALL_SOUND_OFF
            120 => {
                self.all_sounds_off(channum);
            }

            // ALL_CTRL_OFF
            121 => {
                let chan = &mut self.channel[chan_id];
                chan.init_ctrl(1);
                self.voices.modulate_voices_all(
                    &self.channel,
                    channum,
                    self.settings.synth.polyphony,
                );
            }

            // DATA_ENTRY_MSB
            6 => {
                let data: i32 = ((value as i32) << 7 as i32)
                    + self.channel[chan_id].cc[DATA_ENTRY_LSB as usize] as i32;
                if self.channel[chan_id].nrpn_active != 0 {
                    // SontFont 2.01 NRPN Message (Sect. 9.6, p. 74)
                    if self.channel[chan_id].cc[NRPN_MSB as usize] == 120
                        && self.channel[chan_id].cc[NRPN_LSB as usize] < 100
                    {
                        if (self.channel[chan_id].nrpn_select as i32) < GEN_LAST as i32 {
                            use num_traits::FromPrimitive;

                            let val: f32 =
                                fluid_gen_scale_nrpn(self.channel[chan_id].nrpn_select, data);

                            let param =
                                FromPrimitive::from_u8(self.channel[chan_id].nrpn_select as u8)
                                    .unwrap();
                            self.set_gen(self.channel[chan_id].channum, param, val)
                                .unwrap();
                        }
                        self.channel[chan_id].nrpn_select = 0; // Reset to 0
                    }
                }
                /* RPN is active: MSB = 0? */
                else if self.channel[chan_id].cc[RPN_MSB as usize] == 0 {
                    match self.channel[chan_id].cc[RPN_LSB as usize] {
                        // RPN_PITCH_BEND_RANGE
                        0 => {
                            self.pitch_wheel_sens(chan_id as u8, value).ok();
                        }
                        // RPN_CHANNEL_FINE_TUNE
                        1 => {
                            self.set_gen(
                                self.channel[chan_id].channum,
                                GenParam::FineTune,
                                ((data - 8192 as i32) as f64 / 8192.0f64 * 100.0f64) as f32,
                            )
                            .unwrap();
                        }
                        // RPN_CHANNEL_COARSE_TUNE
                        2 => {
                            self.set_gen(
                                self.channel[chan_id].channum,
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
                let chan = &mut self.channel[chan_id];
                chan.cc[NRPN_LSB as usize] = 0;
                chan.nrpn_select = 0;
                chan.nrpn_active = 1;
            }

            // NRPN_LSB
            98 => {
                let chan = &mut self.channel[chan_id];
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
            101 | 100 => self.channel[chan_id].nrpn_active = 0,
            _ => self.voices.modulate_voices(
                &self.channel,
                self.channel[chan_id].channum,
                1,
                num,
                self.settings.synth.polyphony,
            ),
        }
    }
}
