use std::{
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

use soundfont::raw::SampleChunk;

#[derive(Debug, Clone)]
pub(crate) struct SampleData(Arc<[i16]>);

impl SampleData {
    #[cfg_attr(not(feature = "sf3"), allow(dead_code))]
    pub fn new(data: Arc<[i16]>) -> Self {
        Self(data)
    }

    pub fn load<F: Read + Seek>(file: &mut F, smpl: &SampleChunk) -> Result<Self, ()> {
        let sample_pos = smpl.offset;
        let sample_size = smpl.len as usize;

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

        Ok(Self(data.into()))
    }

    #[cfg_attr(not(feature = "sf3"), allow(dead_code))]
    pub fn as_byte_slice(&self) -> &[u8] {
        let slice: &[i16] = &self.0;
        let len = std::mem::size_of_val(slice);
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
    }
}

impl std::ops::Deref for SampleData {
    type Target = [i16];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
