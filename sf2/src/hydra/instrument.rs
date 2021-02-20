use crate::Reader;
use riff::Chunk;

#[derive(Debug)]
pub struct SFInstrumentHeader {
    name: String,
    bag_id: u16,
}

impl SFInstrumentHeader {
    pub fn read(reader: &mut Reader) -> Self {
        let name: String = reader.read_string(20);
        let bag_id: u16 = reader.read_u16();

        Self { name, bag_id }
    }

    pub fn read_all(phdr: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(phdr.id().as_str(), "inst");

        let size = phdr.len();
        if size % 22 != 0 || size == 0 {
            panic!("Instrument header chunk size is invalid");
        }

        let amount = size / 22;

        let data = phdr.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}
