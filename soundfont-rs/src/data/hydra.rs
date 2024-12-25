pub mod generator;
pub use generator::{Generator, GeneratorAmount, GeneratorAmountRange, GeneratorType};

pub mod modulator;
pub use modulator::Modulator;

pub mod bag;
pub use bag::Bag;

pub mod preset;
pub use preset::PresetHeader;

pub mod instrument;
pub use instrument::InstrumentHeader;

pub mod sample;
pub use sample::SampleHeader;

use crate::error::ParseError;
use riff::Chunk;

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
    pub fn read<F: Read + Seek>(pdta: &Chunk, file: &mut F) -> Result<Self, ParseError> {
        assert_eq!(pdta.id().as_str(), "LIST");
        assert_eq!(pdta.read_type(file).unwrap().as_str(), "pdta");

        let chunks: Vec<_> = pdta.iter(file).collect();

        let mut preset_headers = None;
        let mut preset_bags = None;
        let mut preset_modulators = None;
        let mut preset_generators = None;

        let mut instrument_headers = None;
        let mut instrument_bags = None;
        let mut instrument_modulators = None;
        let mut instrument_generators = None;

        let mut sample_headers = None;

        for ch in chunks.into_iter() {
            let ch = ch?;
            let id = ch.id();

            match id.as_str() {
                // The Preset Headers
                "phdr" => preset_headers = Some(PresetHeader::read_all(&ch, file)?),
                // The Preset Index list
                "pbag" => preset_bags = Some(Bag::read_all(&ch, file)?),
                // The Preset Modulator list
                "pmod" => preset_modulators = Some(Modulator::read_all(&ch, file)?),
                // The Preset Generator list
                "pgen" => preset_generators = Some(Generator::read_all(&ch, file)?),
                // The Instrument Names and Indices
                "inst" => instrument_headers = Some(InstrumentHeader::read_all(&ch, file)?),
                // The Instrument Index list
                "ibag" => instrument_bags = Some(Bag::read_all(&ch, file)?),
                // The Instrument Modulator list
                "imod" => instrument_modulators = Some(Modulator::read_all(&ch, file)?),
                // The Instrument Generator list
                "igen" => instrument_generators = Some(Generator::read_all(&ch, file)?),
                // The Sample Headers
                "shdr" => sample_headers = Some(SampleHeader::read_all(&ch, file)?),
                _ => {
                    return Err(ParseError::UnexpectedMemeberOfHydra(ch));
                }
            }
        }

        Ok(Self {
            preset_headers: preset_headers.unwrap(),
            preset_bags: preset_bags.unwrap(),
            preset_modulators: preset_modulators.unwrap(),
            preset_generators: preset_generators.unwrap(),

            instrument_headers: instrument_headers.unwrap(),
            instrument_bags: instrument_bags.unwrap(),
            instrument_modulators: instrument_modulators.unwrap(),
            instrument_generators: instrument_generators.unwrap(),

            sample_headers: sample_headers.unwrap(),
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
