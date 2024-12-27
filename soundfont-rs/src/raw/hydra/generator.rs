use super::super::utils::Reader;
use crate::error::ParseError;
use crate::riff::{Chunk, ChunkId, ScratchReader};
use std::convert::{TryFrom, TryInto};
use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub enum GeneratorAmount {
    I16(i16),
    U16(u16),
    Range(GeneratorAmountRange),
}

pub union GeneratorAmountUnion {
    pub sword: i16,
    pub uword: u16,
    pub range: GeneratorAmountRange,
}

impl GeneratorAmount {
    pub fn get_union(&self) -> GeneratorAmountUnion {
        match self.clone() {
            GeneratorAmount::I16(sword) => GeneratorAmountUnion { sword },
            GeneratorAmount::U16(uword) => GeneratorAmountUnion { uword },
            GeneratorAmount::Range(range) => GeneratorAmountUnion { range },
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

    pub fn as_range(&self) -> Option<&GeneratorAmountRange> {
        if let Self::Range(r) = self {
            Some(r)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GeneratorAmountRange {
    pub low: u8,
    pub high: u8,
}

#[derive(Debug, Clone)]
pub struct Generator {
    pub ty: GeneratorType,
    pub amount: GeneratorAmount,
}

impl Generator {
    pub fn read(reader: &mut Reader) -> Result<Self, ParseError> {
        let id: u16 = reader.read_u16()?;

        let ty: GeneratorType = id.try_into()?;

        let amount = match ty {
            GeneratorType::KeyRange | GeneratorType::VelRange => {
                GeneratorAmount::Range(GeneratorAmountRange {
                    low: reader.read_u8()?,
                    high: reader.read_u8()?,
                })
            }
            GeneratorType::Instrument | GeneratorType::SampleID => {
                GeneratorAmount::U16(reader.read_u16()?)
            }
            _ => GeneratorAmount::I16(reader.read_i16()?),
        };

        Ok(Self { ty, amount })
    }

    pub fn read_all(
        pmod: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Vec<Self>, ParseError> {
        assert!(pmod.id() == ChunkId::pgen || pmod.id() == ChunkId::igen);

        let size = pmod.len();
        if size % 4 != 0 || size == 0 {
            Err(ParseError::InvalidGeneratorChunkSize(size))
        } else {
            let amount = size / 4;

            let data = pmod.read_contents(file)?;
            let mut reader = Reader::new(data);

            Ok((0..amount)
                .filter_map(|_| Self::read(&mut reader).ok())
                .collect())
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u16)]
pub enum GeneratorType {
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
    /// Filter Q
    InitialFilterQ = 9,
    /// Modulation LFO to filter cutoff
    ModLfoToFilterFc = 10,
    /// Modulation envelope to filter cutoff
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

impl TryFrom<u16> for GeneratorType {
    type Error = ParseError;
    fn try_from(id: u16) -> Result<Self, Self::Error> {
        if id <= 60 {
            Ok(unsafe { std::mem::transmute::<u16, Self>(id) })
        } else {
            Err(ParseError::UnknownGeneratorType(id))
        }
    }
}

#[cfg(test)]
mod test {
    use super::GeneratorType;
    #[test]
    fn gen_enum() {
        assert_eq!(GeneratorType::Unused5 as u16, 59);
        assert_eq!(GeneratorType::EndOper as u16, 60);
    }
}
