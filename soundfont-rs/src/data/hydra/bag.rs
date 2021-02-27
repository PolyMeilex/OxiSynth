use super::super::utils::Reader;
use riff::Chunk;

#[derive(Debug)]
pub struct SFBag {
    pub generator_id: u16,
    pub modulator_id: u16,
}

impl SFBag {
    pub fn read(reader: &mut Reader) -> Self {
        let generator_id: u16 = reader.read_u16();
        let modulator_id: u16 = reader.read_u16();

        Self {
            generator_id,
            modulator_id,
        }
    }

    pub fn read_all(pbag: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert!(pbag.id().as_str() == "pbag" || pbag.id().as_str() == "ibag");

        let size = pbag.len();
        if size % 4 != 0 || size == 0 {
            panic!("Preset bag chunk size is invalid");
        }

        let amount = size / 4;

        let data = pbag.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}
