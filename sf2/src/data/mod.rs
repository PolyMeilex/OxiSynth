mod utils;

pub mod hydra;
pub mod info;
pub mod sample_data;

pub use hydra::*;
pub use info::*;
pub use sample_data::*;

#[derive(Debug)]
pub struct SFData {
    pub info: SFInfo,
    pub sample_data: SFSampleData,
    pub hydra: SFHydra,
}

impl SFData {
    pub fn load(file: &mut std::fs::File) -> SFData {
        let sfbk = riff::Chunk::read(file, 0).unwrap();
        assert_eq!(sfbk.id().as_str(), "RIFF");
        assert_eq!(sfbk.read_type(file).unwrap().as_str(), "sfbk");
        let chunks: Vec<_> = sfbk.iter(file).collect();
        let mut info = None;
        let mut sample_data = None;
        let mut hydra = None;
        for ch in chunks.iter() {
            assert_eq!(ch.id().as_str(), "LIST");
            let ty = ch.read_type(file).unwrap();
            match ty.as_str() {
                "INFO" => {
                    info = Some(SFInfo::read(ch, file));
                }
                "sdta" => {
                    sample_data = Some(SFSampleData::read(ch, file));
                }
                "pdta" => {
                    hydra = Some(SFHydra::read(ch, file));
                }
                unknown => {
                    panic!("Unexpected: {} in sfbk", unknown);
                }
            }
        }

        SFData {
            info: info.unwrap(),
            sample_data: sample_data.unwrap(),
            hydra: hydra.unwrap(),
        }
    }
}
