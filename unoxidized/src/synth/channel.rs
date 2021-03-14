use crate::generator::{gen_scale_nrpn, GenParam};
use crate::soundfont::Preset;
use crate::synth::Synth;
use crate::tuning::Tuning;

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

#[derive(Clone, Copy)]
pub(crate) struct ChannelId(pub usize);

#[derive(Clone)]
pub(crate) struct Channel {
    pub(crate) channum: u8,

    sfontnum: usize,

    banknum: u32,
    prognum: u8,

    pub(crate) preset: Option<Preset>,

    pub(crate) key_pressure: [i8; 128],
    pub(crate) channel_pressure: i16,

    pub(crate) pitch_bend: i16,
    pub(crate) pitch_wheel_sensitivity: u16,

    pub(crate) cc: [u8; 128],
    bank_msb: u8,

    interp_method: InterpolationMethod,
    pub(crate) tuning: Option<Tuning>,

    nrpn_select: i16,
    nrpn_active: i16,

    pub(crate) gen: [f32; 60],
    pub(crate) gen_abs: [i8; 60],
}

impl Channel {
    pub fn new(synth: &Synth, num: u8) -> Self {
        let mut chan = Self {
            channum: num,
            sfontnum: 0,
            banknum: 0,
            prognum: 0,

            preset: None,

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
        chan.init(synth.find_preset(chan.banknum, chan.prognum));
        chan.init_ctrl(0);
        return chan;
    }

    pub fn init(&mut self, preset: Option<Preset>) {
        self.prognum = 0;
        self.banknum = 0;
        self.sfontnum = 0;

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

    pub fn set_preset(&mut self, preset: Option<Preset>) {
        self.preset = preset;
    }

    pub fn get_preset(&self) -> Option<&Preset> {
        self.preset.as_ref()
    }

    pub fn get_banknum(&self) -> u32 {
        self.banknum
    }

    pub fn set_prognum(&mut self, prognum: u8) {
        self.prognum = prognum;
    }

    pub fn get_prognum(&self) -> u8 {
        self.prognum
    }

    pub fn set_banknum(&mut self, banknum: u32) {
        self.banknum = banknum;
    }

    pub fn get_cc(&self, num: i32) -> u8 {
        if num >= 0 && num < 128 {
            self.cc[num as usize]
        } else {
            0
        }
    }

    pub fn get_num(&self) -> u8 {
        self.channum
    }

    pub fn set_interp_method(&mut self, new_method: InterpolationMethod) {
        self.interp_method = new_method;
    }

    pub fn get_interp_method(&self) -> InterpolationMethod {
        self.interp_method
    }

    pub fn get_sfontnum(&self) -> usize {
        self.sfontnum
    }

    pub fn set_sfontnum(&mut self, sfontnum: usize) {
        self.sfontnum = sfontnum;
    }
}

impl Synth {
    // TODO: writing self.channel[id] every time is stupid, there has to be a better way
    pub(crate) fn channel_cc(&mut self, chan_id: usize, num: u16, value: u16) {
        self.channels[chan_id].cc[num as usize] = value as u8;

        let channum = self.channels[chan_id].channum;

        match num {
            // SUSTAIN_SWITCH
            64 => {
                if value < 64 {
                    // sustain off
                    self.voices
                        .damp_voices(&self.channels, channum, self.min_note_length_ticks)
                } else {
                    // sustain on
                }
            }

            // BANK_SELECT_MSB
            0 => {
                if channum == 9 && self.settings.drums_channel_active {
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
                if channum == 9 && self.settings.drums_channel_active {
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
                self.all_notes_off(channum);
            }

            // ALL_SOUND_OFF
            120 => {
                self.all_sounds_off(channum);
            }

            // ALL_CTRL_OFF
            121 => {
                self.channels[chan_id].init_ctrl(1);
                self.voices.modulate_voices_all(&self.channels, channum);
            }

            // DATA_ENTRY_MSB
            6 => {
                let data: i32 = ((value as i32) << 7 as i32)
                    + self.channels[chan_id].cc[DATA_ENTRY_LSB as usize] as i32;

                if self.channels[chan_id].nrpn_active != 0 {
                    let (channum, nrpn_select, nrpn_msb, nrpn_lsb) = {
                        let channel = &self.channels[chan_id];
                        (
                            channel.channum,
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
                            self.pitch_wheel_sens(chan_id as u8, value).ok();
                        }
                        // RPN_CHANNEL_FINE_TUNE
                        1 => {
                            self.set_gen(
                                self.channels[chan_id].channum,
                                GenParam::FineTune,
                                ((data - 8192 as i32) as f64 / 8192.0f64 * 100.0f64) as f32,
                            )
                            .unwrap();
                        }
                        // RPN_CHANNEL_COARSE_TUNE
                        2 => {
                            self.set_gen(
                                self.channels[chan_id].channum,
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
            _ => {
                self.voices
                    .modulate_voices(&self.channels, self.channels[chan_id].channum, 1, num)
            }
        }
    }
}
