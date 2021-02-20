use crate::Reader;
use riff::Chunk;

#[derive(Debug, Clone)]
pub enum SFGeneratorAmount {
    I16(i16),
    U16(u16),
    Range(SFGeneratorAmountRange),
}

pub union SFGeneratorAmountUnion {
    sword: i16,
    uword: u16,
    range: SFGeneratorAmountRange,
}

impl SFGeneratorAmount {
    pub fn get_union(&self) -> SFGeneratorAmountUnion {
        match self.clone() {
            SFGeneratorAmount::I16(sword) => SFGeneratorAmountUnion { sword },
            SFGeneratorAmount::U16(uword) => SFGeneratorAmountUnion { uword },
            SFGeneratorAmount::Range(range) => SFGeneratorAmountUnion { range },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SFGeneratorAmountRange {
    low: u8,
    high: u8,
}

#[derive(Debug)]
pub struct SFGenerator {
    ty: SFGeneratorType,
    amount: SFGeneratorAmount,
}

impl SFGenerator {
    pub fn read(reader: &mut Reader) -> Self {
        let id: u16 = reader.read_u16();

        let ty: SFGeneratorType = if id <= 60 {
            unsafe { std::mem::transmute(id) }
        } else {
            panic!("Unknown Generator Type: {}", id);
        };

        let amount = match ty {
            SFGeneratorType::KeyRange | SFGeneratorType::VelRange => {
                SFGeneratorAmount::Range(SFGeneratorAmountRange {
                    low: reader.read_u8(),
                    high: reader.read_u8(),
                })
            }
            SFGeneratorType::Instrument => SFGeneratorAmount::U16(reader.read_u16()),
            _ => SFGeneratorAmount::I16(reader.read_i16()),
        };

        Self { ty, amount }
    }

    pub fn read_all(pmod: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
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

#[derive(Debug)]
#[repr(u16)]
pub enum SFGeneratorType {
    StartAddrsOffset,
    EndAddrsOffset,

    StartloopAddrsOffset,
    EndloopAddrsOffset,
    StartAddrsCoarseOffset,
    ModLfoToPitch,
    VibLfoToPitch,
    ModEnvToPitch,
    InitialFilterFc,
    InitialFilterQ,
    ModLfoToFilterFc,
    ModEnvToFilterFc,
    EndAddrsCoarseOffset,
    ModLfoToVolume,
    Unused1,
    ChorusEffectsSend,
    ReverbEffectsSend,
    Pan,

    Unused2,
    Unused3,
    Unused4,

    DelayModLFO,
    FreqModLFO,
    DelayVibLFO,
    FreqVibLFO,

    DelayModEnv,
    AttackModEnv,
    HoldModEnv,
    DecayModEnv,
    SustainModEnv,
    ReleaseModEnv,

    KeynumToModEnvHold,
    KeynumToModEnvDecay,

    DelayVolEnv,
    AttackVolEnv,
    HoldVolEnv,
    DecayVolEnv,

    SustainVolEnv,
    ReleaseVolEnv,
    KeynumToVolEnvHold,
    KeynumToVolEnvDecay,
    Instrument,
    Reserved1,

    KeyRange,
    VelRange,
    StartloopAddrsCoarseOffset,
    Keynum,
    Velocity,
    InitialAttenuation,
    Reserved2,
    EndloopAddrsCoarseOffset,
    CoarseTune,
    FineTune,

    SampleID,
    SampleModes,

    Reserved3,
    ScaleTuning,
    ExclusiveClass,
    OverridingRootKey,
    Unused5,
    EndOper,
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
