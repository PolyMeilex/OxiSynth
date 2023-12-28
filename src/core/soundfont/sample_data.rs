use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct SampleData(Vec<i16>);

impl SampleData {
    pub fn new(data: Vec<i16>) -> Self {
        Self(data)
    }

    pub fn load<F: Read + Seek>(
        file: &mut F,
        sample_pos: u64,
        sample_size: usize,
    ) -> Result<Self, ()> {
        if file.seek(SeekFrom::Start(sample_pos)).is_err() {
            log::error!("Failed to seek position in data file",);
            return Err(());
        }

        use byteorder::{LittleEndian, ReadBytesExt};

        let mut data = vec![0i16; sample_size / 2];
        if file.read_i16_into::<LittleEndian>(&mut data).is_err() {
            log::error!("Failed to read sample data");
            return Err(());
        }

        Ok(Self(data))
    }
}

impl std::ops::Deref for SampleData {
    type Target = Vec<i16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
