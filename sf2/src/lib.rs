mod utils;
use utils::Reader;

mod info;
use info::SFInfo;

mod sample_data;
use sample_data::SFSampleData;

mod hydra;
use hydra::SFHydra;

#[derive(Debug)]
struct SFFile {
    info: SFInfo,
    sample_data: SFSampleData,
    hydra: SFHydra,
}

pub fn main() {
    let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();

    let sfbk = riff::Chunk::read(&mut file, 0).unwrap();
    assert_eq!(sfbk.id().as_str(), "RIFF");
    assert_eq!(sfbk.read_type(&mut file).unwrap().as_str(), "sfbk");

    let chunks: Vec<_> = sfbk.iter(&mut file).collect();

    let mut info = None;
    let mut sample_data = None;
    let mut hydra = None;

    for ch in chunks.iter() {
        assert_eq!(ch.id().as_str(), "LIST");
        let ty = ch.read_type(&mut file).unwrap();

        match ty.as_str() {
            "INFO" => {
                info = Some(SFInfo::read(ch, &mut file));
            }
            "sdta" => {
                sample_data = Some(SFSampleData::read(ch, &mut file));
            }
            "pdta" => {
                hydra = Some(SFHydra::read(ch, &mut file));
            }
            unknown => {
                panic!("Unexpected: {} in sfbk", unknown);
            }
        }
    }

    let mut hydra = hydra.unwrap();
    hydra.pop_terminators();

    let sf_file = SFFile {
        info: info.unwrap(),
        sample_data: sample_data.unwrap(),
        hydra,
    };

    println!("{:#?}", sf_file);
}

mod unox {
    // #[derive(Default, Debug)]
    // struct SFData {
    //     version: SFVersion,
    //     romver: SFVersion,
    //     samplepos: u32,
    //     samplesize: u64,
    //     fname: Vec<u8>,
    //     sffd: Option<std::fs::File>,
    //     info: Vec<Vec<u8>>,
    //     preset: Vec<*mut SFPreset>,
    //     inst: Vec<*mut SFInst>,
    //     sample: Vec<*mut SFSample>,
    // }

    // #[repr(C)]
    // #[derive(Default, Debug)]
    // struct SFVersion {
    //     major: u16,
    //     minor: u16,
    // }

    // #[derive(Default)]
    // struct SFInst {
    //     name: [u8; 21],
    //     zone: Vec<*mut SFZone>,
    // }

    // enum InstSamp {
    //     Inst(*mut SFInst),
    //     Sample(*mut SFSample),
    //     Int(i32),
    //     None,
    // }

    // struct SFZone {
    //     instsamp: InstSamp,
    //     gen: Vec<*mut SFGen>,
    //     mod_0: Vec<*mut SFMod>,
    // }

    // #[derive(Default, Debug)]
    // struct SFPreset {
    //     /// [u8;21]
    //     name: String,
    //     prenum: u16,
    //     bank: u16,
    //     bag: u16,
    //     libr: u32,
    //     genre: u32,
    //     morph: u32,
    //     // zone: Vec<*mut SFZone>,
    // }

    // #[derive(Default)]
    // struct SFMod {
    //     src: u16,
    //     dest: u16,
    //     amount: i16,
    //     amtsrc: u16,
    //     trans: u16,
    // }

    // #[derive(Default)]
    // struct SFSample {
    //     name: [u8; 21],
    //     samfile: u8,
    //     start: u32,
    //     end: u32,
    //     loopstart: u32,
    //     loopend: u32,
    //     samplerate: u32,
    //     origpitch: u8,
    //     pitchadj: i8,
    //     sampletype: u16,
    // }

    // #[derive(Default)]
    // struct SFGen {
    //     id: u16,
    //     amount: SFGenAmount,
    // }

    // #[derive(Default)]
    // struct SFGenAmount {
    //     sword: i16,
    //     uword: u16,
    //     range: SFGenAmountRange,
    // }

    // #[derive(Default)]
    // struct SFGenAmountRange {
    //     lo: u8,
    //     hi: u8,
    // }

    // #[derive(Default)]
    // struct SFChunk {
    //     id: u32,
    //     size: u32,
    // }
}
