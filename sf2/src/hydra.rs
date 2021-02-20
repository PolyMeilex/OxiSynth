mod generator;
pub use generator::SFGenerator;

mod modulator;
pub use modulator::SFModulator;

mod preset;
pub use preset::{SFPresetBag, SFPresetHeader};

use riff::Chunk;

#[derive(Debug)]
pub struct SFHydra {
    preset_headers: Vec<SFPresetHeader>,
    preset_bags: Vec<SFPresetBag>,
    modulators: Vec<SFModulator>,
    generators: Vec<SFGenerator>,
}

impl SFHydra {
    pub fn read(pdta: &Chunk, file: &mut std::fs::File) -> Self {
        assert_eq!(pdta.id().as_str(), "LIST");
        assert_eq!(pdta.read_type(file).unwrap().as_str(), "pdta");

        let chunks: Vec<_> = pdta.iter(file).collect();

        let mut preset_headers = None;
        let mut preset_bags = None;
        let mut modulators = None;
        let mut generators = None;

        for ch in chunks.iter() {
            let id = ch.id();

            match id.as_str() {
                "phdr" => preset_headers = Some(SFPresetHeader::read_all(ch, file)),
                "pbag" => preset_bags = Some(SFPresetBag::read_all(ch, file)),
                "pmod" => modulators = Some(SFModulator::read_all(ch, file)),
                "pgen" => generators = Some(SFGenerator::read_all(ch, file)),
                _ => {}
            }
        }

        Self {
            preset_headers: preset_headers.unwrap(),
            preset_bags: preset_bags.unwrap(),
            modulators: modulators.unwrap(),
            generators: generators.unwrap(),
        }
    }
}
