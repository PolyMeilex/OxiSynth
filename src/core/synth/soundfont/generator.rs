use super::super::Channel;

use num_derive::FromPrimitive;

/**
Generator (effect) numbers

See also _SoundFont 2.01 specifications section 8.1.3_
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
#[repr(u8)]
pub enum GeneratorType {
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
pub struct Generator {
    pub flags: u8,
    pub val: f64,
    pub mod_0: f64,
    pub nrpn: f64,
}

const GEN_ABS_NRPN: u8 = 2;
const GEN_UNUSED: u8 = 0;

#[derive(Copy, Clone)]
pub(crate) struct GenInfo {
    pub num: GeneratorType,
    pub init: i8,
    pub nrpn_scale: i8,
    pub min: f32,
    pub max: f32,
    pub def: f32,
}

pub(crate) static GEN_INFO: [GenInfo; 60] = [
    GenInfo {
        num: GeneratorType::StartAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::EndAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::StartLoopAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::EndLoopAddrOfs,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::StartAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: 0.0,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModLfoToPitch,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::VibLfoToPitch,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvToPitch,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::FilterFc,
        init: 1,
        nrpn_scale: 2,
        min: 1500.0,
        max: 13500.0,
        def: 13500.0,
    },
    GenInfo {
        num: GeneratorType::FilterQ,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 960.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModLfoToFilterFc,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvToFilterFc,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::EndAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModLfoToVol,
        init: 1,
        nrpn_scale: 1,
        min: -960.0,
        max: 960.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Unused,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ChorusSend,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ReverbSend,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Pan,
        init: 1,
        nrpn_scale: 1,
        min: -500.0,
        max: 500.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Unused2,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Unused3,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Unused4,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModLfoDelay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::ModLfoFreq,
        init: 1,
        nrpn_scale: 4,
        min: -16000.0,
        max: 4500.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::VibLfoDelay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::VibLfoFreq,
        init: 1,
        nrpn_scale: 4,
        min: -16000.0,
        max: 4500.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvDelay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvAttack,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvHold,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvDecay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvSustain,
        init: 0,
        nrpn_scale: 1,
        min: 0.0,
        max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ModEnvRelease,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::KeyToModEnvHold,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0,
        max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::KeyToModEnvDecay,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0,
        max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::VolEnvDelay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::VolEnvAttack,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::VolEnvHold,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::VolEnvDecay,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::VolEnvSustain,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GeneratorType::VolEnvRelease,
        init: 1,
        nrpn_scale: 2,
        min: -12000.0,
        max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        num: GeneratorType::KeyToVolEnvHold,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0,
        max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::KeyToVolEnvDecay,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0,
        max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Instrument,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Reserved1,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::KeyRange,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 127.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::VelRange,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 127.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::StartLoopAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::KeyNum,
        init: 1,
        nrpn_scale: 0,
        min: 0.0,
        max: 127.0,
        def: -1.0,
    },
    GenInfo {
        num: GeneratorType::Velocity,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 127.0,
        def: -1.0,
    },
    GenInfo {
        num: GeneratorType::Attenuation,
        init: 1,
        nrpn_scale: 1,
        min: 0.0,
        max: 1440.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Reserved2,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::EndLoopAddrCoarseOfs,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::CoarseTune,
        init: 0,
        nrpn_scale: 1,
        min: -120.0,
        max: 120.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::FineTune,
        init: 0,
        nrpn_scale: 1,
        min: -99.0,
        max: 99.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::SampleId,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::SampleMode,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::Reserved3,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::ScaleTune,
        init: 0,
        nrpn_scale: 1,
        min: 0.0,
        max: 1200.0,
        def: 100.0,
    },
    GenInfo {
        num: GeneratorType::ExclusiveClass,
        init: 0,
        nrpn_scale: 0,
        min: 0.0,
        max: 0.0,
        def: 0.0,
    },
    GenInfo {
        num: GeneratorType::OverrideRootKey,
        init: 1,
        nrpn_scale: 0,
        min: 0.0,
        max: 127.0,
        def: -1.0,
    },
    GenInfo {
        num: GeneratorType::Pitch,
        init: 1,
        nrpn_scale: 0,
        min: 0.0,
        max: 127.0,
        def: 0.0,
    },
];

/// Flag the generators as unused.
///
/// This also sets the generator values to default, but they will be overwritten anyway, if used.
pub(crate) fn get_default_values() -> [Generator; 60] {
    let mut out = [Generator::default(); 60];

    for (id, gen) in out.iter_mut().enumerate() {
        gen.flags = GEN_UNUSED;
        gen.mod_0 = 0.0;
        gen.nrpn = 0.0;
        gen.val = GEN_INFO[id].def as f64;
    }

    out
}

pub(crate) fn gen_init(channel: &Channel) -> [Generator; 60] {
    let mut out = get_default_values();

    for (id, gen) in out.iter_mut().enumerate() {
        gen.nrpn = channel.gen(id) as f64;
        if channel.gen_abs(id) != 0 {
            gen.flags = GEN_ABS_NRPN;
        }
    }

    out
}

pub(crate) fn gen_scale_nrpn(gen: i16, data: i32) -> f32 {
    let value = data as f32 - 8192.0;
    let value = if value < -8192.0 {
        -8192.0
    } else if value > 8192.0 {
        8192.0
    } else {
        value
    };
    value * GEN_INFO[gen as usize].nrpn_scale as f32
}
