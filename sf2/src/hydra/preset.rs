use crate::Reader;
use riff::Chunk;

#[derive(Debug)]
pub struct SFPresetHeader {
    name: String,
    preset: u16,
    bank: u16,
    bag_id: u16,
    library: u32,
    genre: u32,
    morphology: u32,
}

impl SFPresetHeader {
    pub fn read(reader: &mut Reader) -> Self {
        let name: String = reader.read_string(20);
        let preset: u16 = reader.read_u16();
        let bank: u16 = reader.read_u16();
        let bag_id: u16 = reader.read_u16();

        let library: u32 = reader.read_u32();
        let genre: u32 = reader.read_u32();
        let morphology: u32 = reader.read_u32();

        Self {
            name,
            preset,
            bank,
            bag_id,
            library,
            genre,
            morphology,
        }
    }

    pub fn read_all(phdr: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(phdr.id().as_str(), "phdr");

        let size = phdr.len();
        if size % 38 != 0 || size == 0 {
            panic!("Preset header chunk size is invalid");
        }

        let amount = size / 38;

        let data = phdr.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}

#[derive(Debug)]
pub struct SFPresetBag {
    generator_id: u16,
    modulator_id: u16,
}

impl SFPresetBag {
    pub fn read(reader: &mut Reader) -> Self {
        let generator_id: u16 = reader.read_u16();
        let modulator_id: u16 = reader.read_u16();

        Self {
            generator_id,
            modulator_id,
        }
    }

    pub fn read_all(pbag: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(pbag.id().as_str(), "pbag");

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
