pub mod chorus;
pub mod count;
pub mod font;
pub mod gen;
pub mod midi;
pub mod params;
pub mod reverb;
pub mod tuning;
pub mod write;

use crate::voice::VoiceId;
use crate::voice_pool::VoicePool;
use std::rc::Rc;

use crate::gen::GenParam;

use super::chorus::Chorus;
use super::modulator::Mod;
use super::reverb::ReverbModel;
use super::settings::Settings;
use super::soundfont::Preset;
use super::soundfont::Sample;
use super::soundfont::SoundFont;
use super::tuning::Tuning;
use super::voice::FluidVoiceAddMod;
use super::{
    channel::{Channel, InterpMethod},
    chorus::ChorusMode,
};

pub const FLUID_OK: C2RustUnnamed = 0;

type ModFlags = u8;
type ModSrc = u8;
type GenType = u8;
type C2RustUnnamed = i32;

const FLUID_FAILED: C2RustUnnamed = -1;
const FLUID_SYNTH_PLAYING: SynthStatus = 1;

const FLUID_MOD_POSITIVE: ModFlags = 0;
const FLUID_MOD_UNIPOLAR: ModFlags = 0;
const FLUID_MOD_LINEAR: ModFlags = 0;
const FLUID_MOD_GC: ModFlags = 0;
const FLUID_MOD_PITCHWHEELSENS: ModSrc = 16;
const FLUID_MOD_BIPOLAR: ModFlags = 2;
const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
const FLUID_MOD_CC: ModFlags = 16;
const FLUID_MOD_NEGATIVE: ModFlags = 1;
const FLUID_MOD_CONCAVE: ModFlags = 4;
const FLUID_MOD_CHANNELPRESSURE: ModSrc = 13;
const FLUID_MOD_SWITCH: ModFlags = 12;
const FLUID_MOD_VELOCITY: ModSrc = 2;
const FLUID_MOD_KEYPRESSURE: ModSrc = 10;
const GEN_LAST: GenType = 60;

type SynthStatus = u32;

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: u32,
    pub offset: u32,
}

pub struct Synth {
    state: u32,
    ticks: u32,
    sfont: Vec<SoundFont>,
    sfont_id: u32,
    bank_offsets: Vec<BankOffset>,
    gain: f64,
    pub(crate) channel: Vec<Channel>,
    pub(crate) voices: VoicePool,
    noteid: u32,
    storeid: u32,
    nbuf: i32,
    left_buf: Vec<[f32; 64]>,
    right_buf: Vec<[f32; 64]>,
    fx_left_buf: Vec<[f32; 64]>,
    fx_right_buf: Vec<[f32; 64]>,
    reverb: ReverbModel,
    chorus: Chorus,
    cur: usize,
    dither_index: i32,
    tuning: Vec<Vec<Option<Tuning>>>,
    pub(crate) min_note_length_ticks: u32,

    pub(crate) settings: Settings,
}

impl Synth {
    pub fn new(mut settings: Settings) -> Self {
        let min_note_length_ticks =
            (settings.synth.min_note_length as f32 * settings.synth.sample_rate / 1000.0) as u32;

        if settings.synth.midi_channels % 16 != 0 {
            log::warn!("Requested number of MIDI channels is not a multiple of 16. I\'ll increase the number of channels to the next multiple.");
            let n = settings.synth.midi_channels / 16;
            let midi_channels = (n + 1) * 16;
            settings.synth.midi_channels = midi_channels;
        }

        if settings.synth.audio_channels < 1 {
            log::warn!(
                "Requested number of audio channels is smaller than 1. Changing this setting to 1."
            );
            settings.synth.audio_channels = 1;
        } else if settings.synth.audio_channels > 128 {
            log::warn!(
                "Requested number of audio channels is too big ({}). Limiting this setting to 128.",
                settings.synth.audio_channels
            );
            settings.synth.audio_channels = 128;
        }

        if settings.synth.audio_groups < 1 {
            log::warn!(
                "Requested number of audio groups is smaller than 1. Changing this setting to 1."
            );
            settings.synth.audio_groups = 1;
        } else if settings.synth.audio_groups > 128 {
            log::warn!(
                "Requested number of audio groups is too big ({}). Limiting this setting to 128.",
                settings.synth.audio_groups
            );
            settings.synth.audio_groups = 128;
        }

        if settings.synth.effects_channels != 2 {
            log::warn!(
                "Invalid number of effects channels ({}).Setting effects channels to 2.",
                settings.synth.effects_channels
            );
            settings.synth.effects_channels = 2;
        }

        let nbuf = {
            let nbuf = settings.synth.audio_channels;
            if settings.synth.audio_groups > nbuf {
                settings.synth.audio_groups
            } else {
                nbuf
            }
        };

        let mut synth = Self {
            state: FLUID_SYNTH_PLAYING,
            ticks: 0,
            sfont: Vec::new(),
            sfont_id: 0 as _,
            bank_offsets: Vec::new(),
            gain: settings.synth.gain,
            channel: Vec::new(),
            voices: VoicePool::new(
                settings.synth.polyphony as usize,
                settings.synth.sample_rate,
            ),
            noteid: 0,
            storeid: 0 as _,
            nbuf,
            left_buf: Vec::new(),
            right_buf: Vec::new(),
            fx_left_buf: Vec::new(),
            fx_right_buf: Vec::new(),
            reverb: ReverbModel::new(),
            chorus: Chorus::new(settings.synth.sample_rate as f32),
            cur: 64,
            dither_index: 0,
            tuning: Vec::new(),
            min_note_length_ticks,

            settings,
        };

        for i in 0..synth.settings.synth.midi_channels {
            synth.channel.push(Channel::new(&synth, i));
        }

        synth.left_buf.resize(synth.nbuf as usize, [0f32; 64]);
        synth.right_buf.resize(synth.nbuf as usize, [0f32; 64]);
        synth
            .fx_left_buf
            .resize(synth.settings.synth.effects_channels as usize, [0f32; 64]);
        synth
            .fx_right_buf
            .resize(synth.settings.synth.effects_channels as usize, [0f32; 64]);

        synth.set_reverb_params(0.2, 0.0, 0.5, 0.9);

        if synth.settings.synth.drums_channel_active {
            synth.bank_select(9, 128).ok();
        }

        synth
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.settings.synth.sample_rate = sample_rate;
        self.voices.set_sample_rate(sample_rate);

        self.chorus = Chorus::new(sample_rate);
    }

    pub(crate) fn get_preset(
        &mut self,
        sfontnum: u32,
        banknum: u32,
        prognum: u32,
    ) -> Option<Preset> {
        let sfont = self.get_sfont_by_id(sfontnum);
        if let Some(sfont) = sfont {
            let offset = self
                .get_bank_offset(sfontnum)
                .map(|o| o.offset)
                .unwrap_or_default();
            let preset = sfont.get_preset(banknum.wrapping_sub(offset as u32), prognum);
            preset
        } else {
            None
        }
    }

    pub(crate) fn find_preset(&self, banknum: u32, prognum: u32) -> Option<Preset> {
        for sfont in self.sfont.iter() {
            let offset = self
                .get_bank_offset(sfont.id)
                .map(|o| o.offset)
                .unwrap_or_default();

            let preset = sfont.get_preset(banknum.wrapping_sub(offset as u32), prognum);
            if let Some(preset) = preset {
                return Some(preset);
            }
        }
        return None;
    }

    pub(crate) fn alloc_voice(
        &mut self,
        sample: Rc<Sample>,
        chan: u8,
        key: u8,
        vel: i32,
    ) -> Option<VoiceId> {
        /* check if there's an available synthesis process */
        let mut voice_id = self
            .voices
            .iter()
            .take(self.settings.synth.polyphony as usize)
            .enumerate()
            .find(|(_, v)| v.is_available())
            .map(|(id, _)| VoiceId(id));

        if voice_id.is_none() {
            voice_id = self
                .voices
                .free_voice_by_kill(self.settings.synth.polyphony, self.noteid);
        }

        if let Some(voice_id) = voice_id {
            log::trace!(
                "noteon\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                chan,
                key,
                vel,
                self.storeid,
                self.ticks as f32 / 44100.0,
                0.0,
                {
                    let mut k = 0;
                    for i in 0..self.settings.synth.polyphony {
                        if !self.voices[i as usize].is_available() {
                            k += 1
                        }
                    }
                    k
                }
            );

            let channel = &mut self.channel[chan as usize];

            let voice = &mut self.voices[voice_id.0];
            voice.init(
                sample,
                channel,
                key,
                vel,
                self.storeid,
                self.ticks,
                self.gain as f32,
            );

            const FLUID_VOICE_DEFAULT: FluidVoiceAddMod = 2;

            voice.add_mod(&DEFAULT_VEL2ATT_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_VEL2FILTER_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_AT2VIBLFO_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_MOD2VIBLFO_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_ATT_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_PAN_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_EXPR_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_REVERB_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_CHORUS_MOD, FLUID_VOICE_DEFAULT);
            voice.add_mod(&DEFAULT_PITCH_BEND_MOD, FLUID_VOICE_DEFAULT);

            Some(voice_id)
        } else {
            log::warn!(
                "Failed to allocate a synthesis process. (chan={},key={})",
                chan,
                key
            );
            None
        }
    }

    pub(crate) fn update_presets(&mut self) {
        for chan in 0..(self.settings.synth.midi_channels as usize) {
            let sfontnum = self.channel[chan].get_sfontnum();
            let banknum = self.channel[chan].get_banknum();
            let prognum = self.channel[chan].get_prognum() as u32;
            let preset = self.get_preset(sfontnum, banknum, prognum);
            self.channel[chan].set_preset(preset);
        }
    }
}

lazy_static! {
    static ref RAND_TABLE: [[f32; 48000]; 2] = {
        let mut rand: [[f32; 48000]; 2] = [[0.; 48000]; 2];

        for c in 0..2 {
            let mut dp = 0.0;
            for i in 0..(48000 - 1) {
                let r: i32 = rand::random();
                let d = r as f32 / 2147483647.0 - 0.5;
                rand[c][i] = d - dp;
                dp = d;
            }
            rand[c][48000 - 1] = 0.0 - dp;
        }
        rand
    };
}

static DEFAULT_VEL2ATT_MOD: Mod = Mod {
    dest: GenParam::Attenuation,
    amount: 960.0,

    src1: FLUID_MOD_VELOCITY,
    flags1: FLUID_MOD_GC | FLUID_MOD_CONCAVE | FLUID_MOD_UNIPOLAR | FLUID_MOD_NEGATIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_VEL2FILTER_MOD: Mod = Mod {
    dest: GenParam::FilterFc,
    amount: -2400.0,

    src1: FLUID_MOD_VELOCITY,
    flags1: FLUID_MOD_GC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_NEGATIVE,

    src2: FLUID_MOD_VELOCITY,
    flags2: FLUID_MOD_GC | FLUID_MOD_SWITCH | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,
};

static DEFAULT_AT2VIBLFO_MOD: Mod = Mod {
    dest: GenParam::VibLfoToPitch,
    amount: 50.0,

    src1: FLUID_MOD_CHANNELPRESSURE,
    flags1: FLUID_MOD_GC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_MOD2VIBLFO_MOD: Mod = Mod {
    dest: GenParam::VibLfoToPitch,
    amount: 50.0,

    src1: 1,
    flags1: FLUID_MOD_CC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_ATT_MOD: Mod = Mod {
    dest: GenParam::Attenuation,
    amount: 960.0,

    src1: 7,
    flags1: FLUID_MOD_CC | FLUID_MOD_CONCAVE | FLUID_MOD_UNIPOLAR | FLUID_MOD_NEGATIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_PAN_MOD: Mod = Mod {
    amount: 500.0,
    dest: GenParam::Pan,

    src1: 10,
    flags1: FLUID_MOD_CC | FLUID_MOD_LINEAR | FLUID_MOD_BIPOLAR | FLUID_MOD_POSITIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_EXPR_MOD: Mod = Mod {
    amount: 960.0,
    dest: GenParam::Attenuation,

    src1: 11,
    flags1: FLUID_MOD_CC | FLUID_MOD_CONCAVE | FLUID_MOD_UNIPOLAR | FLUID_MOD_NEGATIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_REVERB_MOD: Mod = Mod {
    amount: 200.0,
    dest: GenParam::ReverbSend,

    src1: 91,
    flags1: FLUID_MOD_CC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_CHORUS_MOD: Mod = Mod {
    amount: 200.0,
    dest: GenParam::ChorusSend,

    src1: 93,
    flags1: FLUID_MOD_CC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,

    src2: 0,
    flags2: 0,
};

static DEFAULT_PITCH_BEND_MOD: Mod = Mod {
    amount: 12700.0,
    dest: GenParam::Pitch,

    src1: FLUID_MOD_PITCHWHEEL,
    flags1: FLUID_MOD_GC | FLUID_MOD_LINEAR | FLUID_MOD_BIPOLAR | FLUID_MOD_POSITIVE,

    src2: FLUID_MOD_PITCHWHEELSENS,
    flags2: FLUID_MOD_GC | FLUID_MOD_LINEAR | FLUID_MOD_UNIPOLAR | FLUID_MOD_POSITIVE,
};
