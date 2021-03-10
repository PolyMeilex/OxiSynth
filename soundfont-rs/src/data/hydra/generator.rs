use super::super::utils::Reader;
use crate::error::ParseError;
use riff::Chunk;
use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub enum SFGeneratorAmount {
    I16(i16),
    U16(u16),
    Range(SFGeneratorAmountRange),
}

pub union SFGeneratorAmountUnion {
    pub sword: i16,
    pub uword: u16,
    pub range: SFGeneratorAmountRange,
}

impl SFGeneratorAmount {
    pub fn get_union(&self) -> SFGeneratorAmountUnion {
        match self.clone() {
            SFGeneratorAmount::I16(sword) => SFGeneratorAmountUnion { sword },
            SFGeneratorAmount::U16(uword) => SFGeneratorAmountUnion { uword },
            SFGeneratorAmount::Range(range) => SFGeneratorAmountUnion { range },
        }
    }

    pub fn as_i16(&self) -> Option<&i16> {
        if let Self::I16(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_u16(&self) -> Option<&u16> {
        if let Self::U16(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_range(&self) -> Option<&SFGeneratorAmountRange> {
        if let Self::Range(r) = self {
            Some(r)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SFGeneratorAmountRange {
    pub low: u8,
    pub high: u8,
}

#[derive(Debug, Clone)]
pub struct SFGenerator {
    pub ty: SFGeneratorType,
    pub amount: SFGeneratorAmount,
}

impl SFGenerator {
    pub fn read(reader: &mut Reader) -> Result<Self, ParseError> {
        let id: u16 = reader.read_u16()?;

        let ty: SFGeneratorType = if id <= 60 {
            unsafe { std::mem::transmute(id) }
        } else {
            panic!("Unknown Generator Type: {}", id);
        };

        let amount = match ty {
            SFGeneratorType::KeyRange | SFGeneratorType::VelRange => {
                SFGeneratorAmount::Range(SFGeneratorAmountRange {
                    low: reader.read_u8()?,
                    high: reader.read_u8()?,
                })
            }
            SFGeneratorType::Instrument | SFGeneratorType::SampleID => {
                SFGeneratorAmount::U16(reader.read_u16()?)
            }
            _ => SFGeneratorAmount::I16(reader.read_i16()?),
        };

        Ok(Self { ty, amount })
    }

    pub fn read_all<F: Read + Seek>(pmod: &Chunk, file: &mut F) -> Result<Vec<Self>, ParseError> {
        assert!(pmod.id().as_str() == "pgen" || pmod.id().as_str() == "igen");

        let size = pmod.len();
        if size % 4 != 0 || size == 0 {
            panic!("Preset generator chunk size mismatch");
        }

        let amount = size / 4;

        let data = pmod.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u16)]
pub enum SFGeneratorType {
    /// Sample start address offset (0-32767)
    StartAddrsOffset = 0,
    ///< Sample end address offset (-32767-0)
    EndAddrsOffset = 1,

    ///< Sample loop start address offset (-32767-32767)
    StartloopAddrsOffset = 2,
    ///< Sample loop end address offset (-32767-32767)
    EndloopAddrsOffset = 3,
    /// Sample start address coarse offset (X 32768)
    StartAddrsCoarseOffset = 4,
    /// Modulation LFO to pitch
    ModLfoToPitch = 5,
    /// Vibrato LFO to pitch
    VibLfoToPitch = 6,
    /// Modulation envelope to pitch
    ModEnvToPitch = 7,
    /// Filter cutoff
    InitialFilterFc = 8,
    /// Modulation envelope to filter cutoff
    InitialFilterQ = 9,
    /// Modulation envelope to filter cutoff
    ModLfoToFilterFc = 10,
    /// Modulation LFO to volume
    ModEnvToFilterFc = 11,
    /// Sample end address coarse offset (X 32768)
    EndAddrsCoarseOffset = 12,
    /// Modulation LFO to volume
    ModLfoToVolume = 13,
    /// Unused
    Unused1 = 14,
    /// Chorus send amount
    ChorusEffectsSend = 15,
    /// Reverb send amount
    ReverbEffectsSend = 16,
    /// Stereo panning
    Pan = 17,

    /// Unused
    Unused2 = 18,
    /// Unused
    Unused3 = 19,
    /// Unused
    Unused4 = 20,

    /// Modulation LFO delay
    DelayModLFO = 21,
    /// Modulation LFO frequency
    FreqModLFO = 22,
    /// Vibrato LFO delay
    DelayVibLFO = 23,
    /// Vibrato LFO frequency
    FreqVibLFO = 24,

    /// Modulation envelope delay
    DelayModEnv = 25,
    /// Modulation envelope attack
    AttackModEnv = 26,
    /// Modulation envelope hold
    HoldModEnv = 27,
    /// Modulation envelope decay
    DecayModEnv = 28,
    /// Modulation envelope sustain
    SustainModEnv = 29,
    /// Modulation envelope release
    ReleaseModEnv = 30,

    /// Key to modulation envelope hold
    KeynumToModEnvHold = 31,
    /// Key to modulation envelope decay
    KeynumToModEnvDecay = 32,

    /// Volume envelope delay
    DelayVolEnv = 33,
    /// Volume envelope attack
    AttackVolEnv = 34,
    /// Volume envelope hold
    HoldVolEnv = 35,
    /// Volume envelope decay
    DecayVolEnv = 36,

    /// Volume envelope sustain
    SustainVolEnv = 37,
    /// Volume envelope release
    ReleaseVolEnv = 38,
    /// Key to volume envelope hold
    KeynumToVolEnvHold = 39,
    /// Key to volume envelope decay
    KeynumToVolEnvDecay = 40,
    /// Instrument ID (shouldn't be set by user)
    Instrument = 41,
    /// Reserved
    Reserved1 = 42,

    /// MIDI note range
    KeyRange = 43,
    /// MIDI velocity range
    VelRange = 44,
    /// Sample start loop address coarse offset (X 32768)
    StartloopAddrsCoarseOffset = 45,
    /// Fixed MIDI note number
    Keynum = 46,
    /// Fixed MIDI velocity value
    Velocity = 47,
    /// Initial volume attenuation
    InitialAttenuation = 48,
    /// Reserved
    Reserved2 = 49,
    /// Sample end loop address coarse offset (X 32768)
    EndloopAddrsCoarseOffset = 50,
    /// Coarse tuning
    CoarseTune = 51,
    /// Fine tuning
    FineTune = 52,

    /// Sample ID (shouldn't be set by user)
    SampleID = 53,
    /// Sample mode flags
    SampleModes = 54,

    /// Reserved
    Reserved3 = 55,
    /// Scale tuning
    ScaleTuning = 56,
    /// Exclusive class number
    ExclusiveClass = 57,
    /// Sample root note override
    OverridingRootKey = 58,
    /// Unused
    Unused5 = 59,

    EndOper = 60,
}

#[cfg(test)]
mod test {
    use super::SFGeneratorEnum;
    #[test]
    fn gen_enum() {
        assert_eq!(SFGeneratorEnum::Unused5 as u16, 59);
        assert_eq!(SFGeneratorEnum::EndOper as u16, 60);
    }
}
