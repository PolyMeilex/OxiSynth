use super::channel::Channel;

use num_derive::FromPrimitive;

/**
Generator (effect) numbers

See also _SoundFont 2.01 specifications section 8.1.3_
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
#[repr(u8)]
pub enum GenParam {
    /** Sample start address offset (0-32767) */
    StartAddrOfs = 0,
    /**< Sample end address offset (-32767-0) */
    EndAddrOfs = 1,
    /**< Sample loop start address offset (-32767-32767) */
    StartLoopAddrOfs = 2,
    /**< Sample loop end address offset (-32767-32767) */
    EndLoopAddrOfs = 3,
    /** Sample start address coarse offset (X 32768) */
    StartAddrCoarseOfs = 4,
    /** Modulation LFO to pitch */
    ModLfoToPitch = 5,
    /** Vibrato LFO to pitch */
    VibLfoToPitch = 6,
    /** Modulation envelope to pitch */
    ModEnvToPitch = 7,
    /** Filter cutoff */
    FilterFc = 8,
    /** Filter Q */
    FilterQ = 9,
    /** Modulation LFO to filter cutoff */
    ModLfoToFilterFc = 10,
    /** Modulation envelope to filter cutoff */
    ModEnvToFilterFc = 11,
    /** Sample end address coarse offset (X 32768) */
    EndAddrCoarseOfs = 12,
    /** Modulation LFO to volume */
    ModLfoToVol = 13,
    /** Unused */
    Unused = 14,
    /** Chorus send amount */
    ChorusSend = 15,
    /** Reverb send amount */
    ReverbSend = 16,
    /** Stereo panning */
    Pan = 17,
    /** Unused */
    Unused2 = 18,
    /** Unused */
    Unused3 = 19,
    /** Unused */
    Unused4 = 20,
    /** Modulation LFO delay */
    ModLfoDelay = 21,
    /** Modulation LFO frequency */
    ModLfoFreq = 22,
    /** Vibrato LFO delay */
    VibLfoDelay = 23,
    /** Vibrato LFO frequency */
    VibLfoFreq = 24,
    /** Modulation envelope delay */
    ModEnvDelay = 25,
    /** Modulation envelope attack */
    ModEnvAttack = 26,
    /** Modulation envelope hold */
    ModEnvHold = 27,
    /** Modulation envelope decay */
    ModEnvDecay = 28,
    /** Modulation envelope sustain */
    ModEnvSustain = 29,
    /** Modulation envelope release */
    ModEnvRelease = 30,
    /** Key to modulation envelope hold */
    KeyToModEnvHold = 31,
    /** Key to modulation envelope decay */
    KeyToModEnvDecay = 32,
    /** Volume envelope delay */
    VolEnvDelay = 33,
    /** Volume envelope attack */
    VolEnvAttack = 34,
    /** Volume envelope hold */
    VolEnvHold = 35,
    /** Volume envelope decay */
    VolEnvDecay = 36,
    /** Volume envelope sustain */
    VolEnvSustain = 37,
    /** Volume envelope release */
    VolEnvRelease = 38,
    /** Key to volume envelope hold */
    KeyToVolEnvHold = 39,
    /** Key to volume envelope decay */
    KeyToVolEnvDecay = 40,
    /** Instrument ID (shouldn't be set by user) */
    Instrument = 41,
    /** Reserved */
    Reserved1 = 42,
    /** MIDI note range */
    KeyRange = 43,
    /** MIDI velocity range */
    VelRange = 44,
    /** Sample start loop address coarse offset (X 32768) */
    StartLoopAddrCoarseOfs = 45,
    /** Fixed MIDI note number */
    KeyNum = 46,
    /** Fixed MIDI velocity value */
    Velocity = 47,
    /** Initial volume attenuation */
    Attenuation = 48,
    /** Reserved */
    Reserved2 = 49,
    /** Sample end loop address coarse offset (X 32768) */
    EndLoopAddrCoarseOfs = 50,
    /** Coarse tuning */
    CoarseTune = 51,
    /** Fine tuning */
    FineTune = 52,
    /** Sample ID (shouldn't be set by user) */
    SampleId = 53,
    /** Sample mode flags */
    SampleMode = 54,
    /** Reserved */
    Reserved3 = 55,
    /** Scale tuning */
    ScaleTune = 56,
    /** Exclusive class number */
    ExclusiveClass = 57,
    /** Sample root note override */
    OverrideRootKey = 58,
    /** Pitch (NOTE: Not a real SoundFont generator)

    The initial pitch is not a "standard" generator. It is not
    mentioned in the list of generator in the SF2 specifications. It
    is used, however, as the destination for the default pitch wheel
    modulator.
    */
    Pitch = 59,

    Last = 60,
}

#[derive(Copy, Default, Debug, PartialEq, Clone)]
pub struct Gen {
    pub(crate) flags: u8,
    pub(crate) val: f64,
    pub(crate) mod_0: f64,
    pub(crate) nrpn: f64,
}
pub type GenFlags = u32;
pub const GEN_ABS_NRPN: GenFlags = 2;
pub const GEN_UNUSED: GenFlags = 0;
pub const FLUID_OK: C2RustUnnamed = 0;
#[derive(Copy, Clone)]
pub struct GenInfo {
    pub num: GenParam,
    pub init: i8,
    pub nrpn_scale: i8,
    pub min: f32,
    pub max: f32,
    pub def: f32,
}
pub type C2RustUnnamed = i32;

pub static GEN_INFO: [GenInfo; 60] = [
    GenInfo {
        num: GenParam::StartAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartLoopAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndLoopAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToPitch,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoToPitch,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvToPitch,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::FilterFc,
        init: 1,
        nrpn_scale: 2 as i8,
        min: 1500.0f32,
        max: 13500.0f32,
        def: 13500.0f32,
    },
    GenInfo {
        num: GenParam::FilterQ,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 960.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToFilterFc,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvToFilterFc,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToVol,
        init: 1,
        nrpn_scale: 1,
        min: -960.0f32,
        max: 960.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ChorusSend,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ReverbSend,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Pan,
        init: 1,
        nrpn_scale: 1,
        min: -500.0f32,
        max: 500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused2,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused3,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused4,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoDelay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoFreq,
        init: 1,
        nrpn_scale: 4 as i8,
        min: -16000.0f32,
        max: 4500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoDelay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoFreq,
        init: 1,
        nrpn_scale: 4 as i8,
        min: -16000.0f32,
        max: 4500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvDelay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvAttack,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvHold,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvDecay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvSustain,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvRelease,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::KeyToModEnvHold,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyToModEnvDecay,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvDelay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvAttack,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvHold,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvDecay,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvSustain,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvRelease,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::KeyToVolEnvHold,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyToVolEnvDecay,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Instrument,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved1,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyRange,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VelRange,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartLoopAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyNum,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Velocity,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Attenuation,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved2,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndLoopAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::CoarseTune,
        init: 0,
        nrpn_scale: 1,
        min: -120.0f32,
        max: 120.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::FineTune,
        init: 0,
        nrpn_scale: 1,
        min: -99.0f32,
        max: 99.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::SampleId,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::SampleMode,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved3,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ScaleTune,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1200.0f32,
        def: 100.0f32,
    },
    GenInfo {
        num: GenParam::ExclusiveClass,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::OverrideRootKey,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Pitch,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
];

/// Flag the generators as unused.
///
/// This also sets the generator values to default, but they will be overwritten anyway, if used.
pub fn get_default_values() -> [Gen; 60] {
    let mut out = [Gen::default(); 60];

    for (id, gen) in out.iter_mut().enumerate() {
        gen.flags = GEN_UNUSED as i32 as u8;
        gen.mod_0 = 0.0;
        gen.nrpn = 0.0;
        gen.val = GEN_INFO[id].def as f64;
    }

    out
}

pub fn gen_init(channel: &Channel) -> [Gen; 60] {
    let mut out = get_default_values();

    for (id, gen) in out.iter_mut().enumerate() {
        gen.nrpn = channel.gen[id] as f64;
        if channel.gen_abs[id] != 0 {
            gen.flags = GEN_ABS_NRPN as i32 as u8
        }
    }

    out
}

pub fn fluid_gen_scale_nrpn(gen: i16, data: i32) -> f32 {
    let mut value: f32 = data as f32 - 8192.0f32;
    value = if value < -(8192 as i32) as f32 {
        -(8192 as i32) as f32
    } else if value > 8192 as i32 as f32 {
        8192 as i32 as f32
    } else {
        value
    };
    value * GEN_INFO[gen as usize].nrpn_scale as f32
}
