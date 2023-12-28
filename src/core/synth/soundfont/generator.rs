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

impl From<soundfont::data::GeneratorType> for GeneratorType {
    fn from(value: soundfont::data::GeneratorType) -> Self {
        num_traits::FromPrimitive::from_u8(value as u8).unwrap()
    }
}

#[derive(Copy, Default, Debug, PartialEq, Clone)]
pub struct Generator {
    pub flags: u8,
    pub val: f64,
    pub mod_0: f64,
    pub nrpn: f64,
}

#[derive(Clone, Debug)]
pub struct GeneratorList([Generator; 60]);

impl Default for GeneratorList {
    fn default() -> Self {
        Self(get_default_values())
    }
}

impl GeneratorList {
    pub fn new(channel: &Channel) -> Self {
        let mut out = Self::default();

        for (id, gen) in out.0.iter_mut().enumerate() {
            gen.nrpn = channel.gen(id) as f64;
            if channel.gen_abs(id) != 0 {
                gen.flags = GEN_ABS_NRPN;
            }
        }

        out
    }
}

impl std::ops::Index<GeneratorType> for GeneratorList {
    type Output = Generator;

    fn index(&self, index: GeneratorType) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl std::ops::IndexMut<GeneratorType> for GeneratorList {
    fn index_mut(&mut self, index: GeneratorType) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

const GEN_ABS_NRPN: u8 = 2;
const GEN_UNUSED: u8 = 0;

#[derive(Copy, Clone)]
struct GenInfo {
    _num: GeneratorType,
    _init: i8,
    nrpn_scale: i8,
    _min: f32,
    _max: f32,
    def: f32,
}

static GEN_INFO: [GenInfo; 60] = [
    GenInfo {
        _num: GeneratorType::StartAddrOfs,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::EndAddrOfs,
        _init: 1,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::StartLoopAddrOfs,
        _init: 1,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::EndLoopAddrOfs,
        _init: 1,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::StartAddrCoarseOfs,
        _init: 0,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModLfoToPitch,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::VibLfoToPitch,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvToPitch,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::FilterFc,
        _init: 1,
        nrpn_scale: 2,
        _min: 1500.0,
        _max: 13500.0,
        def: 13500.0,
    },
    GenInfo {
        _num: GeneratorType::FilterQ,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 960.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModLfoToFilterFc,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvToFilterFc,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 12000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::EndAddrCoarseOfs,
        _init: 0,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModLfoToVol,
        _init: 1,
        nrpn_scale: 1,
        _min: -960.0,
        _max: 960.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Unused,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ChorusSend,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ReverbSend,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Pan,
        _init: 1,
        nrpn_scale: 1,
        _min: -500.0,
        _max: 500.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Unused2,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Unused3,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Unused4,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModLfoDelay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::ModLfoFreq,
        _init: 1,
        nrpn_scale: 4,
        _min: -16000.0,
        _max: 4500.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::VibLfoDelay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::VibLfoFreq,
        _init: 1,
        nrpn_scale: 4,
        _min: -16000.0,
        _max: 4500.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvDelay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvAttack,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvHold,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvDecay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvSustain,
        _init: 0,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1000.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ModEnvRelease,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::KeyToModEnvHold,
        _init: 0,
        nrpn_scale: 1,
        _min: -1200.0,
        _max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::KeyToModEnvDecay,
        _init: 0,
        nrpn_scale: 1,
        _min: -1200.0,
        _max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::VolEnvDelay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::VolEnvAttack,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::VolEnvHold,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 5000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::VolEnvDecay,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::VolEnvSustain,
        _init: 0,
        nrpn_scale: 1,
        _min: 0.0f32,
        _max: 1440.0f32,
        def: 0.0f32,
    },
    GenInfo {
        _num: GeneratorType::VolEnvRelease,
        _init: 1,
        nrpn_scale: 2,
        _min: -12000.0,
        _max: 8000.0,
        def: -12000.0,
    },
    GenInfo {
        _num: GeneratorType::KeyToVolEnvHold,
        _init: 0,
        nrpn_scale: 1,
        _min: -1200.0,
        _max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::KeyToVolEnvDecay,
        _init: 0,
        nrpn_scale: 1,
        _min: -1200.0,
        _max: 1200.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Instrument,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Reserved1,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::KeyRange,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 127.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::VelRange,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 127.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::StartLoopAddrCoarseOfs,
        _init: 0,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::KeyNum,
        _init: 1,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 127.0,
        def: -1.0,
    },
    GenInfo {
        _num: GeneratorType::Velocity,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 127.0,
        def: -1.0,
    },
    GenInfo {
        _num: GeneratorType::Attenuation,
        _init: 1,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1440.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Reserved2,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::EndLoopAddrCoarseOfs,
        _init: 0,
        nrpn_scale: 1,
        _min: -1e10f32,
        _max: 1e10f32,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::CoarseTune,
        _init: 0,
        nrpn_scale: 1,
        _min: -120.0,
        _max: 120.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::FineTune,
        _init: 0,
        nrpn_scale: 1,
        _min: -99.0,
        _max: 99.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::SampleId,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::SampleMode,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::Reserved3,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::ScaleTune,
        _init: 0,
        nrpn_scale: 1,
        _min: 0.0,
        _max: 1200.0,
        def: 100.0,
    },
    GenInfo {
        _num: GeneratorType::ExclusiveClass,
        _init: 0,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 0.0,
        def: 0.0,
    },
    GenInfo {
        _num: GeneratorType::OverrideRootKey,
        _init: 1,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 127.0,
        def: -1.0,
    },
    GenInfo {
        _num: GeneratorType::Pitch,
        _init: 1,
        nrpn_scale: 0,
        _min: 0.0,
        _max: 127.0,
        def: 0.0,
    },
];

/// Flag the generators as unused.
///
/// This also sets the generator values to default, but they will be overwritten anyway, if used.
fn get_default_values() -> [Generator; 60] {
    let mut out = [Generator::default(); 60];

    for (id, gen) in out.iter_mut().enumerate() {
        gen.flags = GEN_UNUSED;
        gen.mod_0 = 0.0;
        gen.nrpn = 0.0;
        gen.val = GEN_INFO[id].def as f64;
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
