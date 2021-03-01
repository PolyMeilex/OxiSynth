pub mod chorus;
pub mod count;
pub mod font;
pub mod gen;
pub mod midi;
pub mod misc;
pub mod params;
pub mod reverb;
pub mod tuning;
pub mod write;

use crate::voice::VoiceId;
use crate::voice::VoiceStatus;
use crate::voice_pool::VoicePool;
use std::rc::Rc;

use super::chorus::Chorus;
use super::modulator::Mod;
use super::reverb::ReverbModel;
use super::settings::Settings;
use super::soundfont::Preset;
use super::soundfont::Sample;
use super::soundfont::SoundFont;
use super::tuning::Tuning;
use super::voice::FluidVoiceAddMod;
use super::voice::Voice;
use super::{
    channel::{Channel, InterpMethod},
    chorus::ChorusMode,
};

use crate::gen::GenParam;

static mut FLUID_ERRBUF: [u8; 512] = [0; 512];

pub const FLUID_OK: C2RustUnnamed = 0;

#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: u32,
    pub offset: u32,
}

type ModFlags = u32;
type ModSrc = u32;
type GenType = u32;
type C2RustUnnamed = i32;

const FLUID_SYNTH_STOPPED: SynthStatus = 3;
const FLUID_FAILED: C2RustUnnamed = -1;
const FLUID_SYNTH_PLAYING: SynthStatus = 1;

const GEN_PITCH: GenType = 59;
const FLUID_MOD_POSITIVE: ModFlags = 0;
const FLUID_MOD_UNIPOLAR: ModFlags = 0;
const FLUID_MOD_LINEAR: ModFlags = 0;
const FLUID_MOD_GC: ModFlags = 0;
const FLUID_MOD_PITCHWHEELSENS: ModSrc = 16;
const FLUID_MOD_BIPOLAR: ModFlags = 2;
const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
const GEN_CHORUSSEND: GenType = 15;
const FLUID_MOD_CC: ModFlags = 16;
const GEN_REVERBSEND: GenType = 16;
const GEN_ATTENUATION: GenType = 48;
const FLUID_MOD_NEGATIVE: ModFlags = 1;
const FLUID_MOD_CONCAVE: ModFlags = 4;
const GEN_PAN: GenType = 17;
const GEN_VIBLFOTOPITCH: GenType = 6;
const FLUID_MOD_CHANNELPRESSURE: ModSrc = 13;
const GEN_FILTERFC: GenType = 8;
const FLUID_MOD_SWITCH: ModFlags = 12;
const FLUID_MOD_VELOCITY: ModSrc = 2;
const FLUID_MOD_KEYPRESSURE: ModSrc = 10;
const GEN_LAST: GenType = 60;
const FLUID_VOICE_DEFAULT: FluidVoiceAddMod = 2;
const FLUID_VOICE_ENVATTACK: VoiceEnvelopeIndex = 1;
const GEN_EXCLUSIVECLASS: GenType = 57;

#[derive(Copy, Clone)]
struct ReverbModelPreset {
    pub name: *mut i8,
    pub roomsize: f32,
    pub damp: f32,
    pub width: f32,
    pub level: f32,
}

type VoiceEnvelopeIndex = u32;
type SynthStatus = u32;

static mut FLUID_SYNTH_INITIALIZED: i32 = 0 as i32;

static mut DEFAULT_VEL2ATT_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_VEL2FILTER_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_AT2VIBLFO_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_MOD2VIBLFO_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_ATT_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_PAN_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_EXPR_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_REVERB_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_CHORUS_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

static mut DEFAULT_PITCH_BEND_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
};

pub struct Synth {
    state: u32,
    ticks: u32,
    sfont: Vec<SoundFont>,
    sfont_id: u32,
    bank_offsets: Vec<BankOffset>,
    gain: f64,
    channel: Vec<Channel>,
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
    cur: i32,
    dither_index: i32,
    tuning: Vec<Vec<Option<Tuning>>>,
    cur_tuning: Option<Tuning>,
    pub(crate) min_note_length_ticks: u32,

    pub(crate) settings: Settings,
}

impl Synth {
    pub fn new(mut settings: Settings) -> Self {
        unsafe {
            if FLUID_SYNTH_INITIALIZED == 0 as i32 {
                Synth::init();
            }
        }

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
            cur_tuning: None,
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
        for i in 0..self.voices.len() {
            self.voices[i as usize] = Voice::new(self.settings.synth.sample_rate as f32);
        }
        self.chorus.delete();
        self.chorus = Chorus::new(self.settings.synth.sample_rate as f32);
    }

    pub(crate) fn damp_voices(&mut self, chan: u8) -> i32 {
        for i in 0..self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan && voice.status == VoiceStatus::Sustained {
                voice.noteoff(self.min_note_length_ticks);
            }
        }
        return FLUID_OK as i32;
    }

    pub(crate) unsafe fn modulate_voices(&mut self, chan: u8, is_cc: i32, ctrl: i32) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan {
                voice.modulate(is_cc, ctrl);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub(crate) unsafe fn modulate_voices_all(&mut self, chan: u8) -> i32 {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.chan == chan {
                voice.modulate_all();
            }
            i += 1
        }
        return FLUID_OK as i32;
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

    pub(crate) unsafe fn free_voice_by_kill(&mut self) -> Option<VoiceId> {
        let mut best_prio: f32 = 999999.0f32;
        let mut this_voice_prio;
        let mut best_voice_index: Option<usize> = None;

        {
            for (id, voice) in self
                .voices
                .iter_mut()
                .take(self.settings.synth.polyphony as usize)
                .enumerate()
            {
                if voice.is_available() {
                    return Some(VoiceId(id));
                }
                this_voice_prio = 10000.0f32;
                if voice.chan as i32 == 0xff as i32 {
                    this_voice_prio = (this_voice_prio as f64 - 2000.0f64) as f32
                }
                if voice.status == VoiceStatus::Sustained {
                    this_voice_prio -= 1000 as i32 as f32
                }
                this_voice_prio -= self.noteid.wrapping_sub(voice.id) as f32;
                if voice.volenv_section != FLUID_VOICE_ENVATTACK as i32 {
                    this_voice_prio =
                        (this_voice_prio as f64 + voice.volenv_val as f64 * 1000.0f64) as f32
                }
                if this_voice_prio < best_prio {
                    best_voice_index = Some(id);
                    best_prio = this_voice_prio
                }
            }
        }

        if let Some(id) = best_voice_index {
            let voice = &mut self.voices[id];
            voice.off();
            Some(VoiceId(id))
        } else {
            None
        }
    }

    pub(crate) unsafe fn alloc_voice(
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
            voice_id = self.free_voice_by_kill()
        }

        if let Some(voice_id) = voice_id {
            if self.settings.synth.verbose {
                let mut k = 0;
                for i in 0..self.settings.synth.polyphony {
                    if !self.voices[i as usize].is_available() {
                        k += 1
                    }
                }
                log::info!(
                    "noteon\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                    chan,
                    key,
                    vel,
                    self.storeid,
                    (self.ticks as f32 / 44100.0f32) as f64,
                    0.0f32 as f64,
                    k
                );
            }

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

            voice.add_mod(&mut DEFAULT_VEL2ATT_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_VEL2FILTER_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_AT2VIBLFO_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_MOD2VIBLFO_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_ATT_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_PAN_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_EXPR_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_REVERB_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_CHORUS_MOD, FLUID_VOICE_DEFAULT as i32);
            voice.add_mod(&mut DEFAULT_PITCH_BEND_MOD, FLUID_VOICE_DEFAULT as i32);

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

    pub(crate) fn kill_by_exclusive_class(&mut self, new_voice: VoiceId) {
        let excl_class = {
            let new_voice = &mut self.voices[new_voice.0];
            let excl_class: i32 = (new_voice.gen[GenParam::ExclusiveClass as usize].val
                + new_voice.gen[GenParam::ExclusiveClass as usize].mod_0
                + new_voice.gen[GenParam::ExclusiveClass as usize].nrpn)
                as i32;
            excl_class
        };

        if excl_class != 0 {
            for i in 0..self.settings.synth.polyphony {
                let new_voice = &self.voices[new_voice.0];
                let existing_voice = &self.voices[i as usize];

                if existing_voice.is_playing() {
                    if !(existing_voice.chan as i32 != new_voice.chan as i32) {
                        if !((existing_voice.gen[GEN_EXCLUSIVECLASS as i32 as usize].val as f32
                            + existing_voice.gen[GEN_EXCLUSIVECLASS as i32 as usize].mod_0 as f32
                            + existing_voice.gen[GEN_EXCLUSIVECLASS as i32 as usize].nrpn as f32)
                            as i32
                            != excl_class)
                        {
                            if !(existing_voice.id == new_voice.id) {
                                self.voices[i as usize].kill_excl();
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) unsafe fn start_voice(&mut self, voice_id: VoiceId) {
        self.kill_by_exclusive_class(voice_id);
        self.voices[voice_id.0].start();
    }

    pub(crate) fn release_voice_on_same_note(&mut self, chan: u8, key: u8) {
        let mut i = 0;
        while i < self.settings.synth.polyphony {
            let voice = &mut self.voices[i as usize];
            if voice.is_playing()
                && voice.chan == chan
                && voice.key == key
                && voice.id != self.noteid
            {
                voice.noteoff(self.min_note_length_ticks);
            }
            i += 1
        }
    }

    pub(crate) fn start(
        &mut self,
        id: u32,
        _audio_chan: i32,
        midi_chan: u8,
        key: u8,
        vel: i32,
    ) -> Result<(), ()> {
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

            // TODO: proper borrowing...
            let preset = self.channel[midi_chan as usize].preset.as_mut().unwrap() as *mut Preset;

            if unsafe { *preset }.noteon(self, midi_chan, key, vel) == FLUID_OK {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub(crate) fn update_presets(&mut self) {
        let mut chan = 0;
        while chan < self.settings.synth.midi_channels {
            let sfontnum = self.channel[chan as usize].get_sfontnum();
            let banknum = self.channel[chan as usize].get_banknum();
            let prognum = self.channel[chan as usize].get_prognum() as u32;
            let preset = self.get_preset(sfontnum, banknum, prognum);
            self.channel[chan as usize].set_preset(preset);
            chan += 1
        }
    }

    unsafe fn init() {
        FLUID_SYNTH_INITIALIZED += 1;
        init_dither();
        DEFAULT_VEL2ATT_MOD.set_source1(
            FLUID_MOD_VELOCITY as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_CONCAVE as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_NEGATIVE as i32,
        );
        DEFAULT_VEL2ATT_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_VEL2ATT_MOD.set_dest(GEN_ATTENUATION as i32);
        DEFAULT_VEL2ATT_MOD.set_amount(960.0f64);
        DEFAULT_VEL2FILTER_MOD.set_source1(
            FLUID_MOD_VELOCITY as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_NEGATIVE as i32,
        );
        DEFAULT_VEL2FILTER_MOD.set_source2(
            FLUID_MOD_VELOCITY as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_SWITCH as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_VEL2FILTER_MOD.set_dest(GEN_FILTERFC as i32);
        DEFAULT_VEL2FILTER_MOD.set_amount(-(2400 as i32) as f64);
        DEFAULT_AT2VIBLFO_MOD.set_source1(
            FLUID_MOD_CHANNELPRESSURE as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_AT2VIBLFO_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_AT2VIBLFO_MOD.set_dest(GEN_VIBLFOTOPITCH as i32);
        DEFAULT_AT2VIBLFO_MOD.set_amount(50 as i32 as f64);
        DEFAULT_MOD2VIBLFO_MOD.set_source1(
            1 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_MOD2VIBLFO_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_MOD2VIBLFO_MOD.set_dest(GEN_VIBLFOTOPITCH as i32);
        DEFAULT_MOD2VIBLFO_MOD.set_amount(50 as i32 as f64);
        DEFAULT_ATT_MOD.set_source1(
            7 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_CONCAVE as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_NEGATIVE as i32,
        );
        DEFAULT_ATT_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_ATT_MOD.set_dest(GEN_ATTENUATION as i32);
        DEFAULT_ATT_MOD.set_amount(960.0f64);
        DEFAULT_PAN_MOD.set_source1(
            10 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_BIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_PAN_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_PAN_MOD.set_dest(GEN_PAN as i32);
        DEFAULT_PAN_MOD.set_amount(500.0f64);
        DEFAULT_EXPR_MOD.set_source1(
            11 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_CONCAVE as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_NEGATIVE as i32,
        );
        DEFAULT_EXPR_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_EXPR_MOD.set_dest(GEN_ATTENUATION as i32);
        DEFAULT_EXPR_MOD.set_amount(960.0f64);
        DEFAULT_REVERB_MOD.set_source1(
            91 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_REVERB_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_REVERB_MOD.set_dest(GEN_REVERBSEND as i32);
        DEFAULT_REVERB_MOD.set_amount(200 as i32 as f64);
        DEFAULT_CHORUS_MOD.set_source1(
            93 as i32,
            FLUID_MOD_CC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_CHORUS_MOD.set_source2(0 as i32, 0 as i32);
        DEFAULT_CHORUS_MOD.set_dest(GEN_CHORUSSEND as i32);
        DEFAULT_CHORUS_MOD.set_amount(200 as i32 as f64);
        DEFAULT_PITCH_BEND_MOD.set_source1(
            FLUID_MOD_PITCHWHEEL as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_BIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_PITCH_BEND_MOD.set_source2(
            FLUID_MOD_PITCHWHEELSENS as i32,
            FLUID_MOD_GC as i32
                | FLUID_MOD_LINEAR as i32
                | FLUID_MOD_UNIPOLAR as i32
                | FLUID_MOD_POSITIVE as i32,
        );
        DEFAULT_PITCH_BEND_MOD.set_dest(GEN_PITCH as i32);
        DEFAULT_PITCH_BEND_MOD.set_amount(12700.0f64);
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        self.state = FLUID_SYNTH_STOPPED as i32 as u32;
        for voice in self.voices.iter_mut() {
            voice.off();
        }
        self.bank_offsets.clear();
        self.voices.clear();
        self.chorus.delete();
    }
}

static mut RAND_TABLE: [[f32; 48000]; 2] = [[0.; 48000]; 2];
unsafe fn init_dither() {
    let mut d;
    let mut dp;
    let mut c;
    let mut i;
    c = 0 as i32;
    while c < 2 as i32 {
        dp = 0 as i32 as f32;
        i = 0 as i32;
        while i < 48000 as i32 - 1 as i32 {
            d = libc::rand() as f32 / 2147483647 as i32 as f32 - 0.5f32;
            RAND_TABLE[c as usize][i as usize] = d - dp;
            dp = d;
            i += 1
        }
        RAND_TABLE[c as usize][(48000 as i32 - 1 as i32) as usize] = 0 as i32 as f32 - dp;
        c += 1
    }
}
