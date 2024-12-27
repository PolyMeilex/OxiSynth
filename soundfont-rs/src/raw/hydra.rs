mod generator;
pub use generator::{
    Generator, GeneratorAmount, GeneratorAmountRange, GeneratorAmountUnion, GeneratorType,
};

mod modulator;
pub use modulator::{
    default_modulators, ControllerPalette, GeneralPalette, Modulator, ModulatorSource,
    ModulatorTransform, SourceDirection, SourcePolarity, SourceType,
};

mod bag;
pub use bag::Bag;

mod preset;
pub use preset::PresetHeader;

mod instrument;
pub use instrument::InstrumentHeader;

mod sample;
pub use sample::{SampleHeader, SampleLink};

#[allow(unused_imports)]
pub use bag::*;
#[allow(unused_imports)]
pub use generator::*;
#[allow(unused_imports)]
pub use instrument::*;
#[allow(unused_imports)]
pub use modulator::*;
#[allow(unused_imports)]
pub use preset::*;
#[allow(unused_imports)]
pub use sample::*;

use crate::error::MissingChunk;
use crate::riff::{Chunk, ScratchReader};
use crate::{error::ParseError, riff::ChunkId};

use std::io::{Read, Seek};

#[derive(Debug)]
pub struct Hydra {
    pub preset_headers: Vec<PresetHeader>,
    pub preset_bags: Vec<Bag>,
    pub preset_modulators: Vec<Modulator>,
    pub preset_generators: Vec<Generator>,

    pub instrument_headers: Vec<InstrumentHeader>,
    pub instrument_bags: Vec<Bag>,
    pub instrument_modulators: Vec<Modulator>,
    pub instrument_generators: Vec<Generator>,

    pub sample_headers: Vec<SampleHeader>,
}

impl Hydra {
    pub fn read(
        pdta: &Chunk,
        file: &mut ScratchReader<impl Read + Seek>,
    ) -> Result<Self, ParseError> {
        assert_eq!(pdta.id(), ChunkId::LIST);
        assert_eq!(pdta.read_type(file)?, ChunkId::pdta);

        let mut preset_headers = None;
        let mut preset_bags = None;
        let mut preset_modulators = None;
        let mut preset_generators = None;

        let mut instrument_headers = None;
        let mut instrument_bags = None;
        let mut instrument_modulators = None;
        let mut instrument_generators = None;

        let mut sample_headers = None;

        let mut iter = pdta.iter();
        while let Some(ch) = iter.next(file) {
            let ch = ch?;

            match ch.id() {
                // The Preset Headers
                ChunkId::phdr => preset_headers = Some(PresetHeader::read_all(&ch, file)?),
                // The Preset Index list
                ChunkId::pbag => preset_bags = Some(Bag::read_all(&ch, file)?),
                // The Preset Modulator list
                ChunkId::pmod => preset_modulators = Some(Modulator::read_all(&ch, file)?),
                // The Preset Generator list
                ChunkId::pgen => preset_generators = Some(Generator::read_all(&ch, file)?),
                // The Instrument Names and Indices
                ChunkId::inst => instrument_headers = Some(InstrumentHeader::read_all(&ch, file)?),
                // The Instrument Index list
                ChunkId::ibag => instrument_bags = Some(Bag::read_all(&ch, file)?),
                // The Instrument Modulator list
                ChunkId::imod => instrument_modulators = Some(Modulator::read_all(&ch, file)?),
                // The Instrument Generator list
                ChunkId::igen => instrument_generators = Some(Generator::read_all(&ch, file)?),
                // The Sample Headers
                ChunkId::shdr => sample_headers = Some(SampleHeader::read_all(&ch, file)?),
                _ => {
                    return Err(ParseError::UnexpectedMemberOfHydra(ch));
                }
            }
        }

        use MissingChunk::*;
        Ok(Self {
            preset_headers: preset_headers.ok_or(PresetHeaders)?,
            preset_bags: preset_bags.ok_or(PresetBags)?,
            preset_modulators: preset_modulators.ok_or(PresetModulators)?,
            preset_generators: preset_generators.ok_or(PresetGenerators)?,

            instrument_headers: instrument_headers.ok_or(InstrumentHeaders)?,
            instrument_bags: instrument_bags.ok_or(InstrumentBags)?,
            instrument_modulators: instrument_modulators.ok_or(InstrumentModulators)?,
            instrument_generators: instrument_generators.ok_or(InstrumentGenerators)?,

            sample_headers: sample_headers.ok_or(SampleHeaders)?,
        })
    }

    pub fn pop_terminators(&mut self) {
        self.preset_headers.pop().unwrap();
        self.preset_bags.pop().unwrap();
        self.preset_modulators.pop().unwrap();
        self.preset_generators.pop().unwrap();

        self.instrument_headers.pop().unwrap();
        self.instrument_bags.pop().unwrap();
        self.instrument_modulators.pop().unwrap();
        self.instrument_generators.pop().unwrap();
        self.sample_headers.pop().unwrap();
    }
}
