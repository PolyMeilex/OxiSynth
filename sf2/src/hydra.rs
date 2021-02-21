mod generator;
pub use generator::SFGenerator;

mod modulator;
pub use modulator::SFModulator;

mod bag;
use bag::SFBag;

mod preset;
pub use preset::SFPresetHeader;

mod instrument;
pub use instrument::SFInstrumentHeader;

mod sample;
use sample::SFSample;

use riff::Chunk;

#[derive(Debug)]
pub struct SFHydra {
    preset_headers: Vec<SFPresetHeader>,
    preset_bags: Vec<SFBag>,
    preset_modulators: Vec<SFModulator>,
    preset_generators: Vec<SFGenerator>,

    instrument_headers: Vec<SFInstrumentHeader>,
    instrument_bags: Vec<SFBag>,
    instrument_modulators: Vec<SFModulator>,
    instrument_generators: Vec<SFGenerator>,

    samples: Vec<SFSample>,
}

impl SFHydra {
    pub fn read(pdta: &Chunk, file: &mut std::fs::File) -> Self {
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
        let mut samples = None;

        for ch in chunks.iter() {
            let id = ch.id();

            match id.as_str() {
                // The Preset Headers
                "phdr" => preset_headers = Some(SFPresetHeader::read_all(ch, file)),
                // The Preset Index list
                "pbag" => preset_bags = Some(SFBag::read_all(ch, file)),
                // The Preset Modulator list
                "pmod" => preset_modulators = Some(SFModulator::read_all(ch, file)),
                // The Preset Generator list
                "pgen" => preset_generators = Some(SFGenerator::read_all(ch, file)),
                // The Instrument Names and Indices
                "inst" => instrument_headers = Some(SFInstrumentHeader::read_all(ch, file)),
                // The Instrument Index list
                "ibag" => instrument_bags = Some(SFBag::read_all(ch, file)),
                // The Instrument Modulator list
                "imod" => instrument_modulators = Some(SFModulator::read_all(ch, file)),
                // The Instrument Generator list
                "igen" => instrument_generators = Some(SFGenerator::read_all(ch, file)),
                // The Sample Headers
                "shdr" => samples = Some(SFSample::read_all(ch, file)),
                unknown => {
                    panic!("Unexpected: {} in hydra", unknown);
                }
            }
        }

        Self {
            preset_headers: preset_headers.unwrap(),
            preset_bags: preset_bags.unwrap(),
            preset_modulators: preset_modulators.unwrap(),
            preset_generators: preset_generators.unwrap(),

            instrument_headers: instrument_headers.unwrap(),
            instrument_bags: instrument_bags.unwrap(),
            instrument_modulators: instrument_modulators.unwrap(),
            instrument_generators: instrument_generators.unwrap(),

            samples: samples.unwrap(),
        }
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
        self.samples.pop().unwrap();
    }

    pub fn test(&self) {
        let mut items: Vec<()> = Vec::new();

        for header in self.preset_headers.chunks(2) {
            let curr = header.get(0).unwrap();
            let next = header.get(1);

            let start = curr.bag_id as usize;

            let end = if let Some(next) = next {
                next.bag_id as usize
            } else {
                self.preset_bags.len()
            };

            let mut bag_items: Vec<()> = Vec::new();

            for n in start..end {
                
            }
        }
    }
}
