use std::ffi::CStr;

use super::chorus::Chorus;
use super::dsp_float::fluid_dsp_float_config;
use super::modulator::Mod;
use super::reverb::ReverbModel;
use super::settings::{self, Settings};
use super::sfloader::new_fluid_defsfloader;
use super::soundfont::Preset;
use super::soundfont::Sample;
use super::soundfont::SoundFont;
use super::soundfont::SoundFontLoader;
use super::tuning::Tuning;
use super::voice::delete_fluid_voice;
use super::voice::fluid_voice_add_mod;
use super::voice::fluid_voice_get_channel;
use super::voice::fluid_voice_get_id;
use super::voice::fluid_voice_init;
use super::voice::fluid_voice_kill_excl;
use super::voice::fluid_voice_modulate;
use super::voice::fluid_voice_modulate_all;
use super::voice::fluid_voice_noteoff;
use super::voice::fluid_voice_off;
use super::voice::fluid_voice_set_gain;
use super::voice::fluid_voice_set_param;
use super::voice::fluid_voice_start;
use super::voice::fluid_voice_write;
use super::voice::new_fluid_voice;
use super::voice::FluidVoiceAddMod;
use super::voice::Voice;
use super::{
    channel::{Channel, InterpMethod},
    chorus::ChorusMode,
};

static mut FLUID_ERRBUF: [u8; 512] = [0; 512];

pub struct Synth {
    // polyphony: i32,
    // with_reverb: bool,
    with_chorus: bool,
    verbose: bool,
    dump: bool,
    sample_rate: f64,
    midi_channels: i32,
    audio_channels: i32,
    audio_groups: i32,
    effects_channels: i32,
    state: u32,
    ticks: u32,
    loaders: Vec<*mut SoundFontLoader>,
    sfont: Vec<SoundFont>,
    sfont_id: u32,
    bank_offsets: Vec<*mut BankOffset>,
    gain: f64,
    channel: Vec<Channel>,
    nvoice: i32,
    voice: Vec<*mut Voice>,
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

    pub settings: settings::new::Settings,
}

pub const FLUID_OK: C2RustUnnamed = 0;
#[derive(Copy, Clone)]
pub struct BankOffset {
    pub sfont_id: i32,
    pub offset: i32,
}
pub const FLUID_SYNTH_STOPPED: SynthStatus = 3;
pub const FLUID_FAILED: C2RustUnnamed = -1;
pub const FLUID_SYNTH_PLAYING: SynthStatus = 1;
pub type IntUpdateFn = Option<unsafe fn(_: *mut libc::c_void, _: &str, _: i32) -> i32>;
pub const FLUID_VOICE_SUSTAINED: VoiceStatus = 2;
pub const FLUID_VOICE_ON: VoiceStatus = 1;
pub type NumUpdateFn = Option<unsafe fn(_: *mut libc::c_void, _: &str, _: f64) -> i32>;
pub const GEN_PITCH: GenType = 59;
pub const FLUID_MOD_POSITIVE: ModFlags = 0;
pub const FLUID_MOD_UNIPOLAR: ModFlags = 0;
pub const FLUID_MOD_LINEAR: ModFlags = 0;
pub const FLUID_MOD_GC: ModFlags = 0;
pub const FLUID_MOD_PITCHWHEELSENS: ModSrc = 16;
pub const FLUID_MOD_BIPOLAR: ModFlags = 2;
pub const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
pub const GEN_CHORUSSEND: GenType = 15;
pub const FLUID_MOD_CC: ModFlags = 16;
pub const GEN_REVERBSEND: GenType = 16;
pub const GEN_ATTENUATION: GenType = 48;
pub const FLUID_MOD_NEGATIVE: ModFlags = 1;
pub const FLUID_MOD_CONCAVE: ModFlags = 4;
pub const GEN_PAN: GenType = 17;
pub const GEN_VIBLFOTOPITCH: GenType = 6;
pub const FLUID_MOD_CHANNELPRESSURE: ModSrc = 13;
pub const GEN_FILTERFC: GenType = 8;
pub const FLUID_MOD_SWITCH: ModFlags = 12;
pub const FLUID_MOD_VELOCITY: ModSrc = 2;
pub const FLUID_VOICE_OFF: VoiceStatus = 3;
pub const FLUID_VOICE_CLEAN: VoiceStatus = 0;
pub const FLUID_VOICE_ENVRELEASE: VoiceEnvelopeIndex = 5;
pub const FLUID_MOD_KEYPRESSURE: ModSrc = 10;
pub const GEN_LAST: GenType = 60;
pub const FLUID_VOICE_DEFAULT: FluidVoiceAddMod = 2;
pub const FLUID_VOICE_ENVATTACK: VoiceEnvelopeIndex = 1;
pub const GEN_EXCLUSIVECLASS: GenType = 57;
pub type ModFlags = u32;
pub type ModSrc = u32;
pub type GenType = u32;
pub type C2RustUnnamed = i32;
#[derive(Copy, Clone)]
pub struct ReverbModelPreset {
    pub name: *mut i8,
    pub roomsize: f32,
    pub damp: f32,
    pub width: f32,
    pub level: f32,
}
pub type VoiceStatus = u32;
pub type VoiceEnvelopeIndex = u32;
pub type SynthStatus = u32;
static mut FLUID_SYNTH_INITIALIZED: i32 = 0 as i32;

pub static mut DEFAULT_VEL2ATT_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_VEL2FILTER_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_AT2VIBLFO_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_MOD2VIBLFO_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_ATT_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_PAN_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_EXPR_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_REVERB_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_CHORUS_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

pub static mut DEFAULT_PITCH_BEND_MOD: Mod = Mod {
    dest: 0,
    src1: 0,
    flags1: 0,
    src2: 0,
    flags2: 0,
    amount: 0.,
    next: 0 as *const Mod as *mut Mod,
};

impl Synth {
    pub fn new(mut settings: settings::new::Settings) -> Result<Self, &'static str> {
        unsafe {
            if FLUID_SYNTH_INITIALIZED == 0 as i32 {
                Synth::init();
            }

            let sample_rate = settings.synth.sample_rate;

            let min_note_length_ticks =
                (settings.synth.min_note_length as f64 * sample_rate / 1000.0) as u32;

            let midi_channels = {
                let midi_channels = settings.synth.midi_channels;
                if midi_channels % 16 != 0 {
                    log::warn!("Requested number of MIDI channels is not a multiple of 16. I\'ll increase the number of channels to the next multiple.");
                    let n = midi_channels / 16;
                    let midi_channels = (n + 1) * 16;
                    settings.synth.midi_channels = midi_channels;
                    midi_channels
                } else {
                    midi_channels
                }
            };

            let audio_channels = {
                let audio_channels = settings.synth.audio_channels;
                if audio_channels < 1 {
                    log::warn!("Requested number of audio channels is smaller than 1. Changing this setting to 1.");
                    1
                } else if audio_channels > 128 {
                    log::warn!("Requested number of audio channels is too big ({}). Limiting this setting to 128.", audio_channels);
                    128
                } else {
                    audio_channels
                }
            };

            let audio_groups = {
                let audio_groups = settings.synth.audio_groups;
                if audio_groups < 1 as i32 {
                    log::warn!("Requested number of audio groups is smaller than 1. Changing this setting to 1.");
                    1
                } else if audio_groups > 128 as i32 {
                    log::warn!("Requested number of audio groups is too big ({}). Limiting this setting to 128.", audio_groups);
                    128
                } else {
                    audio_groups
                }
            };

            let effects_channels = {
                let effects_channels = settings.synth.effects_channels;
                if effects_channels != 2 as i32 {
                    log::warn!(
                        "Invalid number of effects channels ({}).Setting effects channels to 2.",
                        effects_channels
                    );
                    2
                } else {
                    effects_channels
                }
            };

            let nbuf = {
                let nbuf = audio_channels;
                if audio_groups > nbuf {
                    audio_groups
                } else {
                    nbuf
                }
            };

            let mut synth = Self {
                // polyphony: settings.synth.polyphony,
                // with_reverb: settings.synth.reverb_active,
                with_chorus: settings.synth.chorus_active,
                verbose: settings.synth.verbose,
                dump: settings.synth.dump,
                sample_rate,
                midi_channels,
                audio_channels,
                audio_groups,
                effects_channels,
                state: FLUID_SYNTH_PLAYING,
                ticks: 0,
                loaders: Vec::new(),
                sfont: Vec::new(),
                sfont_id: 0 as _,
                bank_offsets: Vec::new(),
                gain: settings.synth.gain,
                channel: Vec::new(),
                nvoice: 0 as _,
                voice: Vec::new(),
                noteid: 0,
                storeid: 0 as _,
                nbuf,
                left_buf: Vec::new(),
                right_buf: Vec::new(),
                fx_left_buf: Vec::new(),
                fx_right_buf: Vec::new(),
                reverb: ReverbModel::new(),
                chorus: Chorus::new(44100f32),
                cur: 0 as _,
                dither_index: 0 as _,
                tuning: Vec::new(),
                cur_tuning: None,
                min_note_length_ticks,

                settings,
            };

            // settings.register_num(
            //     "synth.gain",
            //     0.2f32 as f64,
            //     0.0f32 as f64,
            //     10.0f32 as f64,
            //     0 as i32,
            //     ::std::mem::transmute::<
            //         Option<unsafe fn(_: &mut Synth, _: &str, _: f64) -> i32>,
            //         NumUpdateFn,
            //     >(Some(
            //         Synth::update_gain as unsafe fn(_: &mut Synth, _: &str, _: f64) -> i32,
            //     )),
            //     &mut synth as *mut Self as *mut libc::c_void,
            // );
            // settings.register_int(
            //     "synth.polyphony",
            //     synth.polyphony,
            //     16 as i32,
            //     4096 as i32,
            //     0 as i32,
            //     ::std::mem::transmute::<
            //         Option<unsafe fn(_: &mut Synth, _: &str, _: i32) -> i32>,
            //         IntUpdateFn,
            //     >(Some(
            //         Synth::update_polyphony as unsafe fn(_: &mut Synth, _: &str, _: i32) -> i32,
            //     )),
            //     &mut synth as *mut Self as *mut libc::c_void,
            // );

            let loader = new_fluid_defsfloader();
            if loader.is_null() {
                log::warn!("Failed to create the default SoundFont loader",);
            } else {
                synth.add_sfloader(loader);
            }

            for i in 0..synth.midi_channels {
                synth.channel.push(Channel::new(&synth, i));
            }

            synth.nvoice = synth.settings.synth.polyphony;
            for _ in 0..synth.nvoice {
                synth.voice.push(new_fluid_voice(synth.sample_rate as f32));
            }

            synth.left_buf.resize(synth.nbuf as usize, [0f32; 64]);
            synth.right_buf.resize(synth.nbuf as usize, [0f32; 64]);
            synth
                .fx_left_buf
                .resize(synth.effects_channels as usize, [0f32; 64]);
            synth
                .fx_right_buf
                .resize(synth.effects_channels as usize, [0f32; 64]);
            synth.cur = 64 as i32;
            synth.dither_index = 0 as i32;
            synth.reverb = ReverbModel::new();
            synth.set_reverb_params(0.2f32 as f64, 0.0f32 as f64, 0.5f32 as f64, 0.9f32 as f64);
            synth.chorus = Chorus::new(synth.sample_rate as f32);
            if synth.settings.synth.drums_channel_active {
                synth.bank_select(9 as i32, 128 as i32 as u32);
            }

            return Ok(synth);
        }
    }

    pub unsafe fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f64;
        for i in 0..self.nvoice {
            delete_fluid_voice(self.voice[i as usize]);
            self.voice[i as usize] = new_fluid_voice(self.sample_rate as f32);
        }
        self.chorus.delete();
        self.chorus = Chorus::new(self.sample_rate as f32);
    }

    pub unsafe fn noteon(&mut self, chan: i32, key: i32, vel: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if vel == 0 as i32 {
            return self.noteoff(chan, key);
        }
        if self.channel[chan as usize].preset.is_null() {
            if self.verbose {
                log::info!(
                    "noteon\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}\t{}",
                    chan,
                    key,
                    vel,
                    0,
                    (self.ticks as f32 / 44100.0f32),
                    0.0f32,
                    0,
                    "channel has no preset"
                );
            }
            return FLUID_FAILED as i32;
        }
        self.release_voice_on_same_note(chan, key);
        let fresh7 = self.noteid;
        self.noteid = self.noteid.wrapping_add(1);
        return self.start(
            fresh7,
            self.channel[chan as usize].preset,
            0 as i32,
            chan,
            key,
            vel,
        );
    }

    pub unsafe fn noteoff(&mut self, chan: i32, key: i32) -> i32 {
        let mut i;
        let mut voice;
        let mut status: i32 = FLUID_FAILED as i32;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_ON as i32
                && (*voice).volenv_section < FLUID_VOICE_ENVRELEASE as i32
                && (*voice).chan as i32 == chan
                && (*voice).key as i32 == key
            {
                if self.verbose {
                    let mut used_voices: i32 = 0 as i32;
                    let mut k;
                    k = 0 as i32;
                    while k < self.settings.synth.polyphony {
                        if !((*self.voice[i as usize]).status as i32 == FLUID_VOICE_CLEAN as i32
                            || (*self.voice[i as usize]).status as i32 == FLUID_VOICE_OFF as i32)
                        {
                            used_voices += 1
                        }
                        k += 1
                    }
                    log::info!(
                        "noteoff\t{}\t{}\t{}\t{}\t{}\t\t{}\t{}",
                        (*voice).chan,
                        (*voice).key,
                        0 as i32,
                        (*voice).id,
                        ((*voice).start_time.wrapping_add((*voice).ticks) as f32 / 44100.0f32)
                            as f64,
                        ((*voice).ticks as f32 / 44100.0f32) as f64,
                        used_voices
                    );
                }
                fluid_voice_noteoff(voice, &*self);
                status = FLUID_OK as i32
            }
            i += 1
        }
        return status;
    }

    pub unsafe fn damp_voices(&mut self, chan: i32) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).chan as i32 == chan
                && (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_noteoff(voice, &*self);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn cc(&mut self, chan: i32, num: i32, val: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if num < 0 as i32 || num >= 128 as i32 {
            log::warn!("Ctrl out of range",);
            return FLUID_FAILED as i32;
        }
        if val < 0 as i32 || val >= 128 as i32 {
            log::warn!("Value out of range",);
            return FLUID_FAILED as i32;
        }
        if self.verbose {
            log::info!("cc\t{}\t{}\t{}", chan, num, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize].cc(
            synth_ptr.as_mut().unwrap(),
            num,
            val,
        );
        return FLUID_OK as i32;
    }

    pub unsafe fn get_cc(&self, chan: i32, num: i32, pval: *mut i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if num < 0 as i32 || num >= 128 as i32 {
            log::warn!("Ctrl out of range",);
            return FLUID_FAILED as i32;
        }
        *pval = self.channel[chan as usize].cc[num as usize] as i32;
        return FLUID_OK as i32;
    }

    pub unsafe fn all_notes_off(&mut self, chan: i32) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if ((*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32)
                && (*voice).chan as i32 == chan
            {
                fluid_voice_noteoff(voice, &*self);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn all_sounds_off(&mut self, chan: i32) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if ((*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32)
                && (*voice).chan as i32 == chan
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn system_reset(&mut self) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        i = 0 as i32;
        while i < self.midi_channels {
            // TODO: double borrow
            let synth_ptr = self as *mut Synth;
            synth_ptr.as_mut().unwrap().channel[i as usize].reset(synth_ptr.as_mut().unwrap());
            i += 1
        }
        self.chorus.reset();
        self.reverb.reset();
        return FLUID_OK as i32;
    }

    pub unsafe fn modulate_voices(&mut self, chan: i32, is_cc: i32, ctrl: i32) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).chan as i32 == chan {
                fluid_voice_modulate(voice, is_cc, ctrl);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn modulate_voices_all(&mut self, chan: i32) -> i32 {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).chan as i32 == chan {
                fluid_voice_modulate_all(voice);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn channel_pressure(&mut self, chan: i32, val: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.verbose {
            log::info!("channelpressure\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pressure(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    pub unsafe fn key_pressure(&mut self, chan: i32, key: i32, val: i32) -> i32 {
        let mut result: i32 = FLUID_OK as i32;
        if key < 0 as i32 || key > 127 as i32 {
            return FLUID_FAILED as i32;
        }
        if val < 0 as i32 || val > 127 as i32 {
            return FLUID_FAILED as i32;
        }
        if self.verbose {
            log::info!("keypressure\t{}\t{}\t{}", chan, key, val);
        }
        self.channel[chan as usize].key_pressure[key as usize] = val as i8;
        let mut voice;
        let mut i;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).chan as i32 == chan && (*voice).key as i32 == key {
                result = fluid_voice_modulate(voice, 0 as i32, FLUID_MOD_KEYPRESSURE as i32);
                if result != FLUID_OK as i32 {
                    break;
                }
            }
            i += 1
        }
        return result;
    }

    pub unsafe fn pitch_bend(&mut self, chan: i32, val: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.verbose {
            log::info!("pitchb\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pitch_bend(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    pub unsafe fn get_pitch_bend(&self, chan: i32, ppitch_bend: *mut i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        *ppitch_bend = self.channel[chan as usize].pitch_bend as i32;
        return FLUID_OK as i32;
    }

    pub unsafe fn pitch_wheel_sens(&mut self, chan: i32, val: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if self.verbose {
            log::info!("pitchsens\t{}\t{}", chan, val);
        }
        // TODO: double borrow
        let synth_ptr = self as *mut Synth;
        synth_ptr.as_mut().unwrap().channel[chan as usize]
            .pitch_wheel_sens(synth_ptr.as_mut().unwrap(), val);
        return FLUID_OK as i32;
    }

    pub unsafe fn get_pitch_wheel_sens(&self, chan: i32, pval: *mut i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        *pval = self.channel[chan as usize].pitch_wheel_sensitivity as i32;
        return FLUID_OK as i32;
    }

    pub unsafe fn get_preset(&mut self, sfontnum: u32, banknum: u32, prognum: u32) -> *mut Preset {
        let preset;
        let sfont;
        let offset;
        sfont = self.get_sfont_by_id(sfontnum);
        if !sfont.is_null() {
            offset = self.get_bank_offset(sfontnum as i32);
            preset = Some((*sfont).get_preset.expect("non-null function pointer"))
                .expect("non-null function pointer")(
                sfont,
                banknum.wrapping_sub(offset as u32),
                prognum,
            );
            if !preset.is_null() {
                return preset;
            }
        }
        return 0 as *mut Preset;
    }

    pub unsafe fn find_preset(&self, banknum: u32, prognum: u32) -> *mut Preset {
        for sfont in self.sfont.iter() {
            let offset = self.get_bank_offset(sfont.id as i32);
            let preset = Some(sfont.get_preset.expect("non-null function pointer"))
                .expect("non-null function pointer")(
                sfont,
                banknum.wrapping_sub(offset as u32),
                prognum,
            );
            if !preset.is_null() {
                (*preset).sfont = sfont;
                return preset;
            }
        }
        return 0 as *mut Preset;
    }

    pub unsafe fn program_change(&mut self, chan: i32, prognum: i32) -> i32 {
        let mut preset;
        let banknum;
        let sfont_id;
        let mut subst_bank;
        let mut subst_prog;
        if prognum < 0 as i32
            || prognum >= 128 as i32
            || chan < 0 as i32
            || chan >= self.midi_channels
        {
            log::error!("Index out of range (chan={}, prog={})", chan, prognum);
            return FLUID_FAILED as i32;
        }
        banknum = self.channel[chan as usize].get_banknum();
        self.channel[chan as usize].set_prognum(prognum);
        if self.verbose {
            log::info!("prog\t{}\t{}\t{}", chan, banknum, prognum);
        }
        if self.channel[chan as usize].channum == 9 as i32
            && self.settings.synth.drums_channel_active
        {
            preset = self.find_preset(128 as i32 as u32, prognum as u32)
        } else {
            preset = self.find_preset(banknum, prognum as u32)
        }
        if preset.is_null() {
            subst_bank = banknum as i32;
            subst_prog = prognum;
            if banknum != 128 as i32 as u32 {
                subst_bank = 0 as i32;
                preset = self.find_preset(0 as i32 as u32, prognum as u32);
                if preset.is_null() && prognum != 0 as i32 {
                    preset = self.find_preset(0 as i32 as u32, 0 as i32 as u32);
                    subst_prog = 0 as i32
                }
            } else {
                preset = self.find_preset(128 as i32 as u32, 0 as i32 as u32);
                subst_prog = 0 as i32
            }
            if !preset.is_null() {
                log::warn!(
                        "Instrument not found on channel {} [bank={} prog={}], substituted [bank={} prog={}]",
                        chan, banknum, prognum,
                        subst_bank, subst_prog);
            }
        }
        sfont_id = if !preset.is_null() {
            (*(*preset).sfont).id
        } else {
            0 as i32 as u32
        };
        self.channel[chan as usize].set_sfontnum(sfont_id);
        self.channel[chan as usize].set_preset(preset);
        return FLUID_OK as i32;
    }

    pub fn bank_select(&mut self, chan: i32, bank: u32) -> i32 {
        if chan >= 0 as i32 && chan < self.midi_channels {
            self.channel[chan as usize].set_banknum(bank);
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    pub unsafe fn sfont_select(&mut self, chan: i32, sfont_id: u32) -> i32 {
        if chan >= 0 as i32 && chan < self.midi_channels {
            self.channel[chan as usize].set_sfontnum(sfont_id);
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    pub unsafe fn get_program(
        &self,
        chan: i32,
        sfont_id: *mut u32,
        bank_num: *mut u32,
        preset_num: *mut u32,
    ) -> i32 {
        let channel;
        if chan >= 0 as i32 && chan < self.midi_channels {
            channel = &self.channel[chan as usize];
            *sfont_id = channel.get_sfontnum();
            *bank_num = channel.get_banknum();
            *preset_num = channel.get_prognum() as u32;
            return FLUID_OK as i32;
        }
        return FLUID_FAILED as i32;
    }

    pub unsafe fn program_select(
        &mut self,
        chan: i32,
        sfont_id: u32,
        bank_num: u32,
        preset_num: u32,
    ) -> i32 {
        let preset;
        let channel;
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::error!("Channel number out of range (chan={})", chan);
            return FLUID_FAILED as i32;
        }
        preset = self.get_preset(sfont_id, bank_num, preset_num);
        if preset.is_null() {
            log::error!(
                "There is no preset with bank number {} and preset number {} in SoundFont {}",
                bank_num,
                preset_num,
                sfont_id
            );
            return FLUID_FAILED as i32;
        }
        channel = &mut self.channel[chan as usize];
        channel.set_sfontnum(sfont_id);
        channel.set_banknum(bank_num);
        channel.set_prognum(preset_num as i32);
        channel.set_preset(preset);
        return FLUID_OK as i32;
    }

    pub unsafe fn update_presets(&mut self) {
        let mut chan;
        chan = 0 as i32;
        while chan < self.midi_channels {
            let sfontnum = self.channel[chan as usize].get_sfontnum();
            let banknum = self.channel[chan as usize].get_banknum();
            let prognum = self.channel[chan as usize].get_prognum() as u32;
            let preset = self.get_preset(sfontnum, banknum, prognum);
            self.channel[chan as usize].set_preset(preset);
            chan += 1
        }
    }

    pub unsafe fn update_gain(&mut self, _name: &str, value: f64) -> i32 {
        self.set_gain(value as f32);
        return 0 as i32;
    }

    pub unsafe fn set_gain(&mut self, mut gain: f32) {
        let mut i;
        gain = if gain < 0.0f32 {
            0.0f32
        } else if gain > 10.0f32 {
            10.0f32
        } else {
            gain
        };
        self.gain = gain as f64;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            let voice: *mut Voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_set_gain(voice, gain);
            }
            i += 1
        }
    }

    pub unsafe fn get_gain(&self) -> f32 {
        return self.gain as f32;
    }

    pub unsafe fn update_polyphony(&mut self, _name: &str, value: i32) -> i32 {
        self.set_polyphony(value);
        return 0 as i32;
    }

    pub unsafe fn set_polyphony(&mut self, polyphony: i32) -> i32 {
        let mut i;
        if polyphony < 1 as i32 || polyphony > self.nvoice {
            return FLUID_FAILED as i32;
        }
        i = polyphony;
        while i < self.nvoice {
            let voice: *mut Voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                fluid_voice_off(voice);
            }
            i += 1
        }
        self.settings.synth.polyphony = polyphony;
        return FLUID_OK as i32;
    }

    pub unsafe fn get_polyphony(&self) -> i32 {
        return self.settings.synth.polyphony;
    }

    pub unsafe fn get_internal_bufsize(&self) -> i32 {
        return 64 as i32;
    }

    pub unsafe fn program_reset(&mut self) -> i32 {
        let mut i;
        i = 0 as i32;
        while i < self.midi_channels {
            self.program_change(i, self.channel[i as usize].get_prognum());
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub fn set_reverb_params(&mut self, roomsize: f64, damping: f64, width: f64, level: f64) {
        self.reverb.set_room_size(roomsize as f32);
        self.reverb.set_damp(damping as f32);
        self.reverb.set_width(width as f32);
        self.reverb.set_level(level as f32);
    }

    pub fn set_chorus_params(
        &mut self,
        nr: i32,
        level: f64,
        speed: f64,
        depth_ms: f64,
        type_0: ChorusMode,
    ) {
        self.chorus.set_nr(nr);
        self.chorus.set_level(level as f32);
        self.chorus.set_speed_hz(speed as f32);
        self.chorus.set_depth_ms(depth_ms as f32);
        self.chorus.set_type(type_0);
        self.chorus.update();
    }

    pub unsafe fn write_f32(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f32 = lout as *mut f32;
        let right_out: *mut f32 = rout as *mut f32;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize);
            *right_out.offset(k as isize) = *right_in.offset(l as isize);
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_f64(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut l;
        let left_out: *mut f64 = lout as *mut f64;
        let right_out: *mut f64 = rout as *mut f64;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        l = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if l == 64 as i32 {
                self.one_block(0 as i32);
                l = 0 as i32
            }
            *left_out.offset(j as isize) = *left_in.offset(l as isize) as f64;
            *right_out.offset(k as isize) = *right_in.offset(l as isize) as f64;
            i += 1;
            l += 1;
            j += lincr;
            k += rincr
        }
        self.cur = l;
        return 0 as i32;
    }

    pub unsafe fn write_s16(
        &mut self,
        len: i32,
        lout: *mut libc::c_void,
        loff: i32,
        lincr: i32,
        rout: *mut libc::c_void,
        roff: i32,
        rincr: i32,
    ) -> i32 {
        let mut i;
        let mut j;
        let mut k;
        let mut cur;
        let left_out: *mut i16 = lout as *mut i16;
        let right_out: *mut i16 = rout as *mut i16;
        let left_in: *mut f32 = self.left_buf[0].as_mut_ptr();
        let right_in: *mut f32 = self.right_buf[0].as_mut_ptr();
        let mut left_sample;
        let mut right_sample;
        let mut di: i32 = self.dither_index;
        if self.state != FLUID_SYNTH_PLAYING as i32 as u32 {
            return 0 as i32;
        }
        cur = self.cur;
        i = 0 as i32;
        j = loff;
        k = roff;
        while i < len {
            if cur == 64 as i32 {
                self.one_block(0 as i32);
                cur = 0 as i32
            }
            left_sample = roundi(
                *left_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[0 as i32 as usize][di as usize],
            ) as f32;
            right_sample = roundi(
                *right_in.offset(cur as isize) * 32766.0f32
                    + RAND_TABLE[1 as i32 as usize][di as usize],
            ) as f32;
            di += 1;
            if di >= 48000 as i32 {
                di = 0 as i32
            }
            if left_sample > 32767.0f32 {
                left_sample = 32767.0f32
            }
            if left_sample < -32768.0f32 {
                left_sample = -32768.0f32
            }
            if right_sample > 32767.0f32 {
                right_sample = 32767.0f32
            }
            if right_sample < -32768.0f32 {
                right_sample = -32768.0f32
            }
            *left_out.offset(j as isize) = left_sample as i16;
            *right_out.offset(k as isize) = right_sample as i16;
            i += 1;
            cur += 1;
            j += lincr;
            k += rincr
        }
        self.cur = cur;
        self.dither_index = di;
        return 0 as i32;
    }

    pub unsafe fn one_block(&mut self, do_not_mix_fx_to_out: i32) -> i32 {
        let mut i;
        let mut auchan;
        let mut voice;
        let mut left_buf;
        let mut right_buf;
        let reverb_buf;
        let chorus_buf;
        let byte_size: i32 = (64 as i32 as libc::size_t)
            .wrapping_mul(::std::mem::size_of::<f32>() as libc::size_t)
            as i32;
        i = 0 as i32;
        while i < self.nbuf {
            libc::memset(
                self.left_buf[i as usize].as_mut_ptr() as *mut libc::c_void,
                0 as i32,
                byte_size as libc::size_t,
            );
            libc::memset(
                self.right_buf[i as usize].as_mut_ptr() as *mut libc::c_void,
                0 as i32,
                byte_size as libc::size_t,
            );
            i += 1
        }
        i = 0 as i32;
        while i < self.effects_channels {
            libc::memset(
                self.fx_left_buf[i as usize].as_mut_ptr() as *mut libc::c_void,
                0 as i32,
                byte_size as libc::size_t,
            );
            libc::memset(
                self.fx_right_buf[i as usize].as_mut_ptr() as *mut libc::c_void,
                0 as i32,
                byte_size as libc::size_t,
            );
            i += 1
        }
        reverb_buf = if self.settings.synth.reverb_active {
            self.fx_left_buf[0].as_mut_ptr()
        } else {
            0 as *mut f32
        };
        chorus_buf = if self.with_chorus {
            self.fx_left_buf[1].as_mut_ptr()
        } else {
            0 as *mut f32
        };
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                auchan = fluid_voice_get_channel(voice).as_ref().unwrap().get_num();
                auchan %= self.audio_groups;
                left_buf = self.left_buf[auchan as usize].as_mut_ptr();
                right_buf = self.right_buf[auchan as usize].as_mut_ptr();
                fluid_voice_write(voice, &*self, left_buf, right_buf, reverb_buf, chorus_buf);
            }
            i += 1
        }
        if do_not_mix_fx_to_out != 0 {
            if !reverb_buf.is_null() {
                self.reverb.process_replace(
                    reverb_buf,
                    self.fx_left_buf[0].as_mut_ptr(),
                    self.fx_right_buf[0].as_mut_ptr(),
                );
            }
            if !chorus_buf.is_null() {
                self.chorus.process_replace(
                    chorus_buf,
                    self.fx_left_buf[1].as_mut_ptr(),
                    self.fx_right_buf[1].as_mut_ptr(),
                );
            }
        } else {
            if !reverb_buf.is_null() {
                self.reverb.process_mix(
                    reverb_buf,
                    self.left_buf[0].as_mut_ptr(),
                    self.right_buf[0].as_mut_ptr(),
                );
            }
            if !chorus_buf.is_null() {
                self.chorus.process_mix(
                    chorus_buf,
                    self.left_buf[0].as_mut_ptr(),
                    self.right_buf[0].as_mut_ptr(),
                );
            }
        }
        self.ticks = self.ticks.wrapping_add(64);
        return 0 as i32;
    }

    pub unsafe fn free_voice_by_kill(&mut self) -> *mut Voice {
        let mut i;
        let mut best_prio: f32 = 999999.0f32;
        let mut this_voice_prio;
        let mut voice;
        let mut best_voice_index: i32 = -(1 as i32);
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).status as i32 == FLUID_VOICE_CLEAN as i32
                || (*voice).status as i32 == FLUID_VOICE_OFF as i32
            {
                return voice;
            }
            this_voice_prio = 10000.0f32;
            if (*voice).chan as i32 == 0xff as i32 {
                this_voice_prio = (this_voice_prio as f64 - 2000.0f64) as f32
            }
            if (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32 {
                this_voice_prio -= 1000 as i32 as f32
            }
            this_voice_prio -= self.noteid.wrapping_sub(fluid_voice_get_id(voice)) as f32;
            if (*voice).volenv_section != FLUID_VOICE_ENVATTACK as i32 {
                this_voice_prio =
                    (this_voice_prio as f64 + (*voice).volenv_val as f64 * 1000.0f64) as f32
            }
            if this_voice_prio < best_prio {
                best_voice_index = i;
                best_prio = this_voice_prio
            }
            i += 1
        }
        if best_voice_index < 0 as i32 {
            return 0 as *mut Voice;
        }
        voice = self.voice[best_voice_index as usize];
        fluid_voice_off(voice);
        return voice;
    }

    pub unsafe fn alloc_voice(
        &mut self,
        sample: *mut Sample,
        chan: i32,
        key: i32,
        vel: i32,
    ) -> *mut Voice {
        let mut i;
        let mut k;
        let mut voice: *mut Voice = 0 as *mut Voice;
        let channel;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            if (*self.voice[i as usize]).status as i32 == FLUID_VOICE_CLEAN as i32
                || (*self.voice[i as usize]).status as i32 == FLUID_VOICE_OFF as i32
            {
                voice = self.voice[i as usize];
                break;
            } else {
                i += 1
            }
        }
        if voice.is_null() {
            voice = self.free_voice_by_kill()
        }
        if voice.is_null() {
            log::warn!(
                "Failed to allocate a synthesis process. (chan={},key={})",
                chan,
                key
            );
            return 0 as *mut Voice;
        }
        if self.verbose {
            k = 0 as i32;
            i = 0 as i32;
            while i < self.settings.synth.polyphony {
                if !((*self.voice[i as usize]).status as i32 == FLUID_VOICE_CLEAN as i32
                    || (*self.voice[i as usize]).status as i32 == FLUID_VOICE_OFF as i32)
                {
                    k += 1
                }
                i += 1
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
        if chan >= 0 as i32 {
            channel = &mut self.channel[chan as usize]
        } else {
            log::warn!("Channel should be valid",);
            return 0 as *mut Voice;
        }
        if fluid_voice_init(
            voice,
            sample,
            channel,
            key,
            vel,
            self.storeid,
            self.ticks,
            self.gain as f32,
        ) != FLUID_OK as i32
        {
            log::warn!("Failed to initialize voice",);
            return 0 as *mut Voice;
        }
        fluid_voice_add_mod(voice, &mut DEFAULT_VEL2ATT_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(
            voice,
            &mut DEFAULT_VEL2FILTER_MOD,
            FLUID_VOICE_DEFAULT as i32,
        );
        fluid_voice_add_mod(
            voice,
            &mut DEFAULT_AT2VIBLFO_MOD,
            FLUID_VOICE_DEFAULT as i32,
        );
        fluid_voice_add_mod(
            voice,
            &mut DEFAULT_MOD2VIBLFO_MOD,
            FLUID_VOICE_DEFAULT as i32,
        );
        fluid_voice_add_mod(voice, &mut DEFAULT_ATT_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(voice, &mut DEFAULT_PAN_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(voice, &mut DEFAULT_EXPR_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(voice, &mut DEFAULT_REVERB_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(voice, &mut DEFAULT_CHORUS_MOD, FLUID_VOICE_DEFAULT as i32);
        fluid_voice_add_mod(
            voice,
            &mut DEFAULT_PITCH_BEND_MOD,
            FLUID_VOICE_DEFAULT as i32,
        );
        return voice;
    }

    pub unsafe fn kill_by_exclusive_class(&mut self, new_voice: *mut Voice) {
        let mut i;
        let excl_class: i32 = ((*new_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].val as f32
            + (*new_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].mod_0 as f32
            + (*new_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].nrpn as f32)
            as i32;
        if excl_class == 0 as i32 {
            return;
        }
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            let existing_voice: *mut Voice = self.voice[i as usize];
            if (*existing_voice).status as i32 == FLUID_VOICE_ON as i32
                || (*existing_voice).status as i32 == FLUID_VOICE_SUSTAINED as i32
            {
                if !((*existing_voice).chan as i32 != (*new_voice).chan as i32) {
                    if !(((*existing_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].val as f32
                        + (*existing_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].mod_0 as f32
                        + (*existing_voice).gen[GEN_EXCLUSIVECLASS as i32 as usize].nrpn as f32)
                        as i32
                        != excl_class)
                    {
                        if !(fluid_voice_get_id(existing_voice) == fluid_voice_get_id(new_voice)) {
                            fluid_voice_kill_excl(existing_voice);
                        }
                    }
                }
            }
            i += 1
        }
    }

    pub unsafe fn start_voice(&mut self, voice: *mut Voice) {
        self.kill_by_exclusive_class(voice);
        fluid_voice_start(voice);
    }

    pub fn add_sfloader(&mut self, loader: *mut SoundFontLoader) {
        self.loaders.insert(0, loader);
    }

    pub unsafe fn sfload(&mut self, filename: &[u8], reset_presets: i32) -> i32 {
        for loader in self.loaders.iter() {
            let sfont = Some((*(*loader)).load.expect("non-null function pointer"))
                .expect("non-null function pointer")(*loader, filename);
            match sfont {
                Some(mut sfont) => {
                    self.sfont_id = self.sfont_id.wrapping_add(1);
                    sfont.id = self.sfont_id;
                    self.sfont.insert(0, sfont);
                    if reset_presets != 0 {
                        self.program_reset();
                    }
                    return self.sfont_id as i32;
                }
                None => {
                    return -(1 as i32);
                }
            }
        }
        log::error!(
            "Failed to load SoundFont \"{}\"",
            CStr::from_ptr(filename.as_ptr() as *const i8)
                .to_str()
                .unwrap()
        );
        return -(1 as i32);
    }

    pub unsafe fn sfunload(&mut self, id: u32, reset_presets: i32) -> i32 {
        let sfont: *mut SoundFont = self.get_sfont_by_id(id);
        if sfont.is_null() {
            log::error!("No SoundFont with id = {}", id);
            return FLUID_FAILED as i32;
        }
        self.sfont.retain(|s| s.id != (*sfont).id);
        if reset_presets != 0 {
            self.program_reset();
        } else {
            self.update_presets();
        }
        if (if !sfont.is_null() && (*sfont).free.is_some() {
            Some((*sfont).free.expect("non-null function pointer"))
                .expect("non-null function pointer")(sfont)
        } else {
            0 as i32
        }) != 0 as i32
        {
            let r: i32 = if !sfont.is_null() && (*sfont).free.is_some() {
                Some((*sfont).free.expect("non-null function pointer"))
                    .expect("non-null function pointer")(sfont)
            } else {
                0 as i32
            };
            if r == 0 as i32 {
                log::debug!("Unloaded SoundFont",);
            }
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn sfreload(&mut self, id: u32) -> i32 {
        let sfont;
        let index;
        index = self
            .sfont
            .iter()
            .position(|x| x.id == id)
            .expect("SoundFont with ID");
        sfont = &self.sfont[index];
        let filename = sfont.get_name.expect("non-null function pointer")(sfont);
        if self.sfunload(id, 0 as i32) != FLUID_OK as i32 {
            return FLUID_FAILED as i32;
        }
        for loader in self.loaders.iter() {
            match Some((*(*loader)).load.expect("non-null function pointer"))
                .expect("non-null function pointer")(
                *loader, &filename.clone().expect("filename")
            ) {
                Some(mut sfont) => {
                    sfont.id = id;
                    self.sfont.insert(index, sfont);
                    self.update_presets();
                    return id as _;
                }
                None => {}
            }
        }
        log::error!(
            "Failed to load SoundFont {:?}",
            CStr::from_ptr(filename.unwrap_or(b"(null)\x00".to_vec()).as_ptr() as *const i8)
                .to_str()
                .unwrap()
        );
        return -(1 as i32);
    }

    pub unsafe fn remove_sfont(&mut self, sfont: *mut SoundFont) {
        let sfont_id: i32 = (*sfont).id as i32;
        self.sfont.retain(|s| s as *const SoundFont != sfont);
        self.remove_bank_offset(sfont_id);
        self.program_reset();
    }

    pub unsafe fn sfcount(&self) -> i32 {
        return self.sfont.len() as _;
    }

    pub unsafe fn get_sfont(&mut self, num: u32) -> *mut SoundFont {
        return match self.sfont.get_mut(num as usize) {
            Some(sfont) => sfont,
            None => 0 as _,
        };
    }

    pub unsafe fn get_sfont_by_id(&mut self, id: u32) -> *mut SoundFont {
        return match self.sfont.iter_mut().find(|x| x.id == id) {
            Some(sfont) => sfont,
            None => 0 as _,
        };
    }

    pub unsafe fn get_channel_preset(&self, chan: i32) -> *mut Preset {
        if chan >= 0 as i32 && chan < self.midi_channels {
            return self.channel[chan as usize].get_preset();
        }
        return 0 as *mut Preset;
    }

    pub fn set_reverb_on(&mut self, on: bool) {
        self.settings.synth.reverb_active = on;
    }

    pub fn set_chorus_on(&mut self, on: bool) {
        self.with_chorus = on;
    }

    pub fn get_chorus_nr(&self) -> i32 {
        return self.chorus.get_nr();
    }

    pub fn get_chorus_level(&self) -> f64 {
        return self.chorus.get_level() as f64;
    }

    pub fn get_chorus_speed_hz(&self) -> f64 {
        return self.chorus.get_speed_hz() as f64;
    }

    pub fn get_chorus_depth_ms(&self) -> f64 {
        return self.chorus.get_depth_ms() as f64;
    }

    pub fn get_chorus_type(&self) -> ChorusMode {
        return self.chorus.get_type();
    }

    pub fn get_reverb_roomsize(&self) -> f64 {
        return self.reverb.get_room_size() as f64;
    }

    pub fn get_reverb_damp(&self) -> f64 {
        return self.reverb.get_damp() as f64;
    }

    pub fn get_reverb_level(&self) -> f64 {
        return self.reverb.get_level() as f64;
    }

    pub fn get_reverb_width(&self) -> f64 {
        return self.reverb.get_width() as f64;
    }

    pub unsafe fn release_voice_on_same_note(&mut self, chan: i32, key: i32) {
        let mut i;
        let mut voice;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if ((*voice).status as i32 == FLUID_VOICE_ON as i32
                || (*voice).status as i32 == FLUID_VOICE_SUSTAINED as i32)
                && (*voice).chan as i32 == chan
                && (*voice).key as i32 == key
                && fluid_voice_get_id(voice) != self.noteid
            {
                fluid_voice_noteoff(voice, &*self);
            }
            i += 1
        }
    }

    pub unsafe fn set_interp_method(&mut self, chan: i32, interp_method: InterpMethod) -> i32 {
        let mut i;
        i = 0 as i32;
        while i < self.midi_channels {
            if chan < 0 as i32 || self.channel[chan as usize].get_num() == chan {
                self.channel[chan as usize].set_interp_method(interp_method);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub fn count_midi_channels(&self) -> i32 {
        return self.midi_channels;
    }

    pub fn count_audio_channels(&self) -> i32 {
        return self.audio_channels;
    }

    pub fn count_audio_groups(&self) -> i32 {
        return self.audio_groups;
    }

    pub fn count_effects_channels(&self) -> i32 {
        return self.effects_channels;
    }
    fn get_tuning(&self, bank: i32, prog: i32) -> Option<&Tuning> {
        if bank < 0 as i32 || bank >= 128 as i32 {
            log::warn!("Bank number out of range",);
            return None;
        }
        if prog < 0 as i32 || prog >= 128 as i32 {
            log::warn!("Program number out of range",);
            return None;
        }
        return self.tuning[bank as usize][prog as usize].as_ref();
    }
    unsafe fn create_tuning<'a>(
        &'a mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
    ) -> Option<&'a mut Tuning> {
        if bank < 0 as i32 || bank >= 128 as i32 {
            log::warn!("Bank number out of range",);
            return None;
        }
        if prog < 0 as i32 || prog >= 128 as i32 {
            log::warn!("Program number out of range",);
            return None;
        }
        let tuning = self.tuning[bank as usize][prog as usize]
            .get_or_insert_with(|| Tuning::new(name, bank, prog));
        if libc::strcmp(tuning.get_name().as_ptr() as _, name.as_ptr() as _) != 0 {
            tuning.set_name(name);
        }
        return Some(tuning);
    }

    pub unsafe fn create_key_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 128],
    ) -> i32 {
        return match self.create_tuning(bank, prog, name) {
            Some(tuning) => {
                tuning.set_all(pitch);
                FLUID_OK as i32
            }
            None => FLUID_FAILED as i32,
        };
    }

    pub unsafe fn create_octave_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 12],
    ) -> i32 {
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        return match self.create_tuning(bank, prog, name) {
            Some(tuning) => {
                tuning.set_octave(pitch);
                FLUID_OK as i32
            }
            None => FLUID_FAILED as i32,
        };
    }

    pub unsafe fn activate_octave_tuning(
        &mut self,
        bank: i32,
        prog: i32,
        name: &[u8],
        pitch: &[f64; 12],
        _apply: i32,
    ) -> i32 {
        return self.create_octave_tuning(bank, prog, name, pitch);
    }

    pub unsafe fn tune_notes(
        &mut self,
        bank: i32,
        prog: i32,
        len: i32,
        key: *mut i32,
        pitch: *mut f64,
        _apply: i32,
    ) -> i32 {
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(len > 0 as i32) {
            return FLUID_FAILED as i32;
        }
        if key.is_null() {
            return FLUID_FAILED as i32;
        }
        if pitch.is_null() {
            return FLUID_FAILED as i32;
        }
        match self.create_tuning(bank, prog, b"Unnamed\x00") {
            Some(tuning) => {
                for i in 0..len {
                    tuning.set_pitch(*key.offset(i as isize), *pitch.offset(i as isize));
                }
                return FLUID_OK as i32;
            }
            None => {
                return FLUID_FAILED as i32;
            }
        }
    }

    pub unsafe fn select_tuning(&mut self, chan: i32, bank: i32, prog: i32) -> i32 {
        let tuning;
        if !(bank >= 0 as i32 && bank < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        if !(prog >= 0 as i32 && prog < 128 as i32) {
            return FLUID_FAILED as i32;
        }
        tuning = self.get_tuning(bank, prog);
        if tuning.is_none() {
            return FLUID_FAILED as i32;
        }
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].tuning = Some(tuning.unwrap().clone());
        return FLUID_OK as i32;
    }

    pub unsafe fn activate_tuning(&mut self, chan: i32, bank: i32, prog: i32, _apply: i32) -> i32 {
        return self.select_tuning(chan, bank, prog);
    }

    pub unsafe fn reset_tuning(&mut self, chan: i32) -> i32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].tuning = None;
        return FLUID_OK as i32;
    }

    pub unsafe fn tuning_iteration_start(&mut self) {
        self.cur_tuning = None;
    }

    pub unsafe fn tuning_iteration_next(&mut self, bank: *mut i32, prog: *mut i32) -> i32 {
        let mut b = 0;
        let mut p = 0;
        match self.cur_tuning.as_ref() {
            Some(tuning) => {
                b = tuning.bank;
                p = tuning.prog + 1;
                if p >= 128 {
                    p = 0;
                    b += 1
                }
            }
            None => {}
        }
        while b < 128 {
            while p < 128 {
                match self.tuning[b as usize][p as usize] {
                    Some(_) => {
                        *bank = b;
                        *prog = p;
                        return 1;
                    }
                    None => {}
                }
                p += 1
            }
            p = 0 as i32;
            b += 1
        }
        return 0 as i32;
    }

    pub unsafe fn tuning_dump(
        &self,
        bank: i32,
        prog: i32,
        name: *mut i8,
        len: i32,
        pitch: *mut f64,
    ) -> i32 {
        match self.get_tuning(bank, prog) {
            Some(tuning) => {
                if !name.is_null() {
                    libc::strncpy(
                        name,
                        tuning.get_name().as_ptr() as _,
                        (len - 1 as i32) as libc::size_t,
                    );
                    *name.offset((len - 1 as i32) as isize) = 0 as i32 as i8
                }
                if !pitch.is_null() {
                    libc::memcpy(
                        pitch as *mut libc::c_void,
                        tuning.pitch.as_ptr().offset(0 as i32 as isize) as *mut f64
                            as *const libc::c_void,
                        (128 as i32 as libc::size_t)
                            .wrapping_mul(::std::mem::size_of::<f64>() as libc::size_t),
                    );
                }
                return FLUID_OK as i32;
            }
            None => {
                return FLUID_FAILED as i32;
            }
        }
    }

    pub unsafe fn set_gen(&mut self, chan: i32, param: i16, value: f32) -> i32 {
        let mut i;
        let mut voice;
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        self.channel[chan as usize].gen[param as usize] = value;
        self.channel[chan as usize].gen_abs[param as usize] = 0 as i32 as i8;
        i = 0 as i32;
        while i < self.settings.synth.polyphony {
            voice = self.voice[i as usize];
            if (*voice).chan as i32 == chan {
                fluid_voice_set_param(voice, param, value, 0 as i32);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub unsafe fn get_gen(&self, chan: i32, param: i32) -> f32 {
        if chan < 0 as i32 || chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return 0.0f32;
        }
        if param < 0 as i32 || param >= GEN_LAST as i32 {
            log::warn!("Parameter number out of range",);
            return 0.0f32;
        }
        return self.channel[chan as usize].gen[param as usize];
    }

    pub unsafe fn start(
        &mut self,
        id: u32,
        preset: *mut Preset,
        _audio_chan: i32,
        midi_chan: i32,
        key: i32,
        vel: i32,
    ) -> i32 {
        let r;
        if midi_chan < 0 as i32 || midi_chan >= self.midi_channels {
            log::warn!("Channel out of range",);
            return FLUID_FAILED as i32;
        }
        if key < 0 as i32 || key >= 128 as i32 {
            log::warn!("Key out of range",);
            return FLUID_FAILED as i32;
        }
        if vel <= 0 as i32 || vel >= 128 as i32 {
            log::warn!("Velocity out of range",);
            return FLUID_FAILED as i32;
        }
        self.storeid = id;
        r = Some((*preset).noteon.expect("non-null function pointer"))
            .expect("non-null function pointer")(preset, self, midi_chan, key, vel);
        return r;
    }

    pub unsafe fn get_bank_offset0(&self, sfont_id: i32) -> *const BankOffset {
        return self
            .bank_offsets
            .iter()
            .find(|x| (*(*(*x))).sfont_id == sfont_id)
            .map(|x| *x as *const BankOffset)
            .unwrap_or(0 as _);
    }

    pub unsafe fn get_mut_bank_offset0(&mut self, sfont_id: i32) -> *mut BankOffset {
        return self
            .bank_offsets
            .iter_mut()
            .find(|x| (*(*(*x))).sfont_id == sfont_id)
            .map(|x| *x as *mut BankOffset)
            .unwrap_or(0 as _);
    }

    pub unsafe fn set_bank_offset(&mut self, sfont_id: i32, offset: i32) -> i32 {
        let mut bank_offset;
        bank_offset = self.get_mut_bank_offset0(sfont_id);
        if bank_offset.is_null() {
            bank_offset = libc::malloc(::std::mem::size_of::<BankOffset>() as libc::size_t)
                as *mut BankOffset;
            if bank_offset.is_null() {
                return -(1 as i32);
            }
            (*bank_offset).sfont_id = sfont_id;
            (*bank_offset).offset = offset;
            self.bank_offsets.insert(0, bank_offset);
        } else {
            (*bank_offset).offset = offset
        }
        return 0 as i32;
    }

    pub unsafe fn get_bank_offset(&self, sfont_id: i32) -> i32 {
        let bank_offset;
        bank_offset = self.get_bank_offset0(sfont_id);
        return if bank_offset.is_null() {
            0 as i32
        } else {
            (*bank_offset).offset
        };
    }

    pub unsafe fn remove_bank_offset(&mut self, sfont_id: i32) {
        self.bank_offsets.retain(|x| (*(*x)).sfont_id != sfont_id);
    }

    pub(crate) unsafe fn register_settings(settings: &mut Settings) {
        settings.register_str(
            "synth.verbose",
            "no",
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_str("synth.dump", "no", 0 as i32, None, 0 as *mut libc::c_void);
        settings.register_str(
            "synth.reverb.active",
            "yes",
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_str(
            "synth.chorus.active",
            "yes",
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_str(
            "synth.ladspa.active",
            "no",
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_str("midi.portname", "", 0 as i32, None, 0 as *mut libc::c_void);
        settings.register_str(
            "synth.drums-channel.active",
            "yes",
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.polyphony",
            256 as i32,
            16 as i32,
            4096 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.midi-channels",
            16 as i32,
            16 as i32,
            256 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_num(
            "synth.gain",
            0.2f32 as f64,
            0.0f32 as f64,
            10.0f32 as f64,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.audio-channels",
            1 as i32,
            1 as i32,
            256 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.audio-groups",
            1 as i32,
            1 as i32,
            256 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.effects-channels",
            2 as i32,
            2 as i32,
            2 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_num(
            "synth.sample-rate",
            44100.0f32 as f64,
            22050.0f32 as f64,
            96000.0f32 as f64,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
        settings.register_int(
            "synth.min-note-length",
            10 as i32,
            0 as i32,
            65535 as i32,
            0 as i32,
            None,
            0 as *mut libc::c_void,
        );
    }

    unsafe fn init() {
        FLUID_SYNTH_INITIALIZED += 1;
        fluid_dsp_float_config();
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
        unsafe {
            self.state = FLUID_SYNTH_STOPPED as i32 as u32;
            for voice in self.voice.iter() {
                fluid_voice_off(*voice);
            }
            for bank_offset in self.bank_offsets.iter() {
                libc::free(*bank_offset as *mut libc::c_void);
            }
            self.bank_offsets.clear();
            for loader in self.loaders.iter() {
                if !(*loader).is_null() {
                    if (*(*loader)).free.is_some() {
                        Some((*(*loader)).free.expect("non-null function pointer"))
                            .expect("non-null function pointer")(*loader);
                    }
                }
            }
            self.loaders.clear();
            for voice in self.voice.iter_mut() {
                delete_fluid_voice(*voice);
            }
            self.voice.clear();
            self.chorus.delete();
        }
    }
}

pub unsafe fn error() -> *mut u8 {
    return FLUID_ERRBUF.as_mut_ptr();
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
unsafe fn roundi(x: f32) -> i32 {
    if x >= 0.0f32 {
        return (x + 0.5f32) as i32;
    } else {
        return (x - 0.5f32) as i32;
    };
}
