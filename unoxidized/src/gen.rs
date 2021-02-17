use super::channel::Channel;

/**
Generator (effect) numbers

See also _SoundFont 2.01 specifications section 8.1.3_
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
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

#[derive(Copy, Clone)]
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
    pub num: i8,
    pub init: i8,
    pub nrpn_scale: i8,
    pub min: f32,
    pub max: f32,
    pub def: f32,
}
pub type C2RustUnnamed = i32;

pub static mut GEN_INFO: [GenInfo; 60] = [
    GenInfo {
        num: GenParam::StartAddrOfs as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndAddrOfs as i8,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartLoopAddrOfs as i8,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndLoopAddrOfs as i8,
        init: 1,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartAddrCoarseOfs as i8,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToPitch as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoToPitch as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvToPitch as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::FilterFc as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: 1500.0f32,
        max: 13500.0f32,
        def: 13500.0f32,
    },
    GenInfo {
        num: GenParam::FilterQ as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 960.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToFilterFc as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvToFilterFc as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 12000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndAddrCoarseOfs as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoToVol as i8,
        init: 1,
        nrpn_scale: 1,
        min: -960.0f32,
        max: 960.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ChorusSend as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ReverbSend as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Pan as i8,
        init: 1,
        nrpn_scale: 1,
        min: -500.0f32,
        max: 500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused2 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused3 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Unused4 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoDelay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModLfoFreq as i8,
        init: 1,
        nrpn_scale: 4 as i8,
        min: -16000.0f32,
        max: 4500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoDelay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VibLfoFreq as i8,
        init: 1,
        nrpn_scale: 4 as i8,
        min: -16000.0f32,
        max: 4500.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvDelay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvAttack as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvHold as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvDecay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvSustain as i8,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1000.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ModEnvRelease as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::KeyToModEnvHold as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyToModEnvDecay as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvDelay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvAttack as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvHold as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 5000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvDecay as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvSustain as i8,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VolEnvRelease as i8,
        init: 1,
        nrpn_scale: 2 as i8,
        min: -12000.0f32,
        max: 8000.0f32,
        def: -12000.0f32,
    },
    GenInfo {
        num: GenParam::KeyToVolEnvHold as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyToVolEnvDecay as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1200.0f32,
        max: 1200.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Instrument as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved1 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyRange as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::VelRange as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::StartLoopAddrCoarseOfs as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::KeyNum as i8,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Velocity as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Attenuation as i8,
        init: 1,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved2 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::EndLoopAddrCoarseOfs as i8,
        init: 0,
        nrpn_scale: 1,
        min: -1e10f32,
        max: 1e10f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::CoarseTune as i8,
        init: 0,
        nrpn_scale: 1,
        min: -120.0f32,
        max: 120.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::FineTune as i8,
        init: 0,
        nrpn_scale: 1,
        min: -99.0f32,
        max: 99.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::SampleId as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::SampleMode as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::Reserved3 as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::ScaleTune as i8,
        init: 0,
        nrpn_scale: 1,
        min: 0.0f32,
        max: 1200.0f32,
        def: 100.0f32,
    },
    GenInfo {
        num: GenParam::ExclusiveClass as i8,
        init: 0,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 0.0f32,
        def: 0.0f32,
    },
    GenInfo {
        num: GenParam::OverrideRootKey as i8,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: -1.0f32,
    },
    GenInfo {
        num: GenParam::Pitch as i8,
        init: 1,
        nrpn_scale: 0,
        min: 0.0f32,
        max: 127.0f32,
        def: 0.0f32,
    },
];

pub unsafe fn fluid_gen_set_default_values(gen: *mut Gen) -> i32 {
    let mut i: i32;
    i = 0 as i32;
    while i < GenParam::Last as i32 {
        (*gen.offset(i as isize)).flags = GEN_UNUSED as i32 as u8;
        (*gen.offset(i as isize)).mod_0 = 0.0f64;
        (*gen.offset(i as isize)).nrpn = 0.0f64;
        (*gen.offset(i as isize)).val = GEN_INFO[i as usize].def as f64;
        i += 1
    }
    return FLUID_OK as i32;
}

pub unsafe fn fluid_gen_init(gen: *mut Gen, channel: *mut Channel) -> i32 {
    let mut i: i32;
    fluid_gen_set_default_values(gen);
    i = 0 as i32;
    while i < GenParam::Last as i32 {
        (*gen.offset(i as isize)).nrpn = (*channel).gen[i as usize] as f64;
        if (*channel).gen_abs[i as usize] != 0 {
            (*gen.offset(i as isize)).flags = GEN_ABS_NRPN as i32 as u8
        }
        i += 1
    }
    return FLUID_OK as i32;
}

pub unsafe fn fluid_gen_scale_nrpn(gen: i16, data: i32) -> f32 {
    let mut value: f32 = data as f32 - 8192.0f32;
    value = if value < -(8192 as i32) as f32 {
        -(8192 as i32) as f32
    } else if value > 8192 as i32 as f32 {
        8192 as i32 as f32
    } else {
        value
    };
    return value * GEN_INFO[gen as usize].nrpn_scale as f32;
}
