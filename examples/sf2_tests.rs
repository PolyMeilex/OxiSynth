use riff::{Chunk, ChunkId};
use std::convert::TryInto;

struct Parser {
    data: Vec<u8>,
    curr: usize,
}
impl Parser {
    fn new(data: Vec<u8>) -> Self {
        Self { data, curr: 0 }
    }

    fn read(&mut self, len: usize) -> &[u8] {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        let out = &self.data[start..end];
        out
    }

    fn read_string(&mut self, len: usize) -> String {
        let start = self.curr;
        let end = self.curr + len;
        self.curr = end;

        let data = &self.data[start..end];
        let name = unsafe { std::ffi::CStr::from_ptr(data.as_ptr() as _) };
        let name = name.to_str().unwrap();
        let name = name.to_owned();

        name
    }

    fn read_u16(&mut self) -> u16 {
        let start = self.curr;
        let end = self.curr + 2;
        self.curr = end;

        let out: [u8; 2] = self.data[start..end].try_into().unwrap();
        u16::from_be_bytes(out)
    }

    fn read_u32(&mut self) -> u32 {
        let start = self.curr;
        let end = self.curr + 4;
        self.curr = end;

        let out: [u8; 4] = self.data[start..end].try_into().unwrap();
        u32::from_be_bytes(out)
    }
}

fn info(info: &Chunk, file: &mut std::fs::File) -> (SFVersion, SFVersion) {
    let id = info.id();
    assert_eq!(id.as_str(), "LIST");
    let ty = info.read_type(file).unwrap();
    assert_eq!(ty.as_str(), "INFO");

    let children: Vec<Chunk> = info.iter(file).collect();

    let mut version = SFVersion::default();
    let mut romver = SFVersion::default();

    for ch in children.iter() {
        let id = ch.id();

        match id.as_str() {
            "ifil" => {
                let data = ch.read_contents(file).unwrap();
                let data: [u8; 4] = data.try_into().unwrap();
                let data: SFVersion = unsafe { std::mem::transmute(data) };
                version = data;
            }
            "iver" => {
                let data = ch.read_contents(file).unwrap();
                let data: [u8; 4] = data.try_into().unwrap();
                let data: SFVersion = unsafe { std::mem::transmute(data) };
                romver = data;
            }
            "INAM" => {
                let data = ch.read_contents(file).unwrap();

                let name = unsafe { std::ffi::CStr::from_ptr(data.as_ptr() as _) };
                let name = name.to_str().unwrap();
                let name = name.to_owned();

                println!("{:?}", name);
            }
            _ => {}
        }

        // SF2Chunk { id, ty, data: () }
    }

    // println!("{:#?}, {:#?}", ty, content);

    // SF2Chunk { id, ty, data }
    (version, romver)
}

fn sdta(sdta: &Chunk, file: &mut std::fs::File) -> (u32, u64) {
    let id = sdta.id();
    assert_eq!(id.as_str(), "LIST");
    let ty = sdta.read_type(file).unwrap();
    assert_eq!(ty.as_str(), "sdta");

    let chunks: Vec<_> = sdta.iter(file).collect();

    for ch in chunks.iter() {
        let id = ch.id();

        match id.as_str() {
            "smpl" => {
                return (ch.len(), ch.offset());
            }
            _ => {}
        }
    }

    panic!("`smpl` not found");
}

fn pdta(pdta: &Chunk, file: &mut std::fs::File) -> () {
    fn phdr(phdr: &Chunk, file: &mut std::fs::File) {
        let size = phdr.len();
        if size % 38 != 0 || size == 0 {
            panic!("Preset header chunk size is invalid");
        }

        let amount = size / 38;
        println!("amount: {}", amount);

        let data = phdr.read_contents(file).unwrap();
        let mut parser = Parser::new(data);

        for _ in 0..amount {
            let name: String = parser.read_string(20);
            let preset: u16 = parser.read_u16();
            let bank: u16 = parser.read_u16();
            let bag: u16 = parser.read_u16();

            let liblary: u32 = parser.read_u32();
            let genre: u32 = parser.read_u32();
            let morphology: u32 = parser.read_u32();

            let preset = SFPreset {
                name,
                prenum: preset,
                bank,
                bag,
                libr: liblary,
                genre,
                morph: morphology,
            };

            println!("{:#?}", preset);
        }
    }
    fn pbag(pbag: &Chunk, file: &mut std::fs::File) {}
    fn pmod(pmod: &Chunk, file: &mut std::fs::File) {}
    fn pgen(pgen: &Chunk, file: &mut std::fs::File) {}

    let id = pdta.id();
    assert_eq!(id.as_str(), "LIST");
    let ty = pdta.read_type(file).unwrap();
    assert_eq!(ty.as_str(), "pdta");

    let chunks: Vec<_> = pdta.iter(file).collect();
    for ch in chunks.iter() {
        let id = ch.id();
        // let ty = ch.read_type(file);
        println!("{:?}", id);

        match id.as_str() {
            "phdr" => {
                phdr(ch, file);
            }
            "pbag" => {
                pbag(ch, file);
            }
            "pmod" => {
                pmod(ch, file);
            }
            "pgen" => {
                pgen(ch, file);
            }
            _ => {}
        }
    }
}

fn main() {
    let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();

    let chunk = riff::Chunk::read(&mut file, 0).unwrap();

    // root
    let chunks: Vec<_> = chunk.iter(&mut file).collect();

    // for ch in chunks.iter() {
    //     let ty = ch.read_type(&mut file).unwrap();

    //     println!("{:?}", ty);
    // }

    // let root = Root { info };

    let (version, romver) = info(&chunks[0], &mut file);
    let (samplepos, samplesize) = sdta(&chunks[1], &mut file);
    let () = pdta(&chunks[2], &mut file);

    let data = SFData {
        version,
        romver,

        samplepos,
        samplesize,

        ..Default::default()
    };

    println!("{:#?}", data);

    // for chunk in chunks.iter() {
    //     let ch_type = chunk.read_type(&mut file).unwrap();
    //     println!("{:#?}", ch_type);
    // }

    // let info = &chunks[0];

    // let info_vec: Vec<_> = info.iter(&mut file).collect();
    // println!("{:#?}", info_vec);
    // let name_raw = info_vec[2].read_contents(&mut file).unwrap();

    // let name = unsafe { std::ffi::CStr::from_ptr(name_raw.as_ptr() as _) };
    // let name = name.to_str().unwrap();
    // let name = name.to_owned();

    // println!("{:#?}", name);
}

#[derive(Default, Debug)]
struct SFData {
    version: SFVersion,
    romver: SFVersion,
    samplepos: u32,
    samplesize: u64,
    fname: Vec<u8>,
    sffd: Option<std::fs::File>,
    info: Vec<Vec<u8>>,
    preset: Vec<*mut SFPreset>,
    inst: Vec<*mut SFInst>,
    sample: Vec<*mut SFSample>,
}

#[repr(C)]
#[derive(Default, Debug)]
struct SFVersion {
    major: u16,
    minor: u16,
}

#[derive(Default)]
struct SFInst {
    name: [u8; 21],
    zone: Vec<*mut SFZone>,
}

enum InstSamp {
    Inst(*mut SFInst),
    Sample(*mut SFSample),
    Int(i32),
    None,
}

struct SFZone {
    instsamp: InstSamp,
    gen: Vec<*mut SFGen>,
    mod_0: Vec<*mut SFMod>,
}

#[derive(Default, Debug)]
struct SFPreset {
    /// [u8;21]
    name: String,
    prenum: u16,
    bank: u16,
    bag: u16,
    libr: u32,
    genre: u32,
    morph: u32,
    // zone: Vec<*mut SFZone>,
}

#[derive(Default)]
struct SFMod {
    src: u16,
    dest: u16,
    amount: i16,
    amtsrc: u16,
    trans: u16,
}

#[derive(Default)]
struct SFSample {
    name: [u8; 21],
    samfile: u8,
    start: u32,
    end: u32,
    loopstart: u32,
    loopend: u32,
    samplerate: u32,
    origpitch: u8,
    pitchadj: i8,
    sampletype: u16,
}

#[derive(Default)]
struct SFGen {
    id: u16,
    amount: SFGenAmount,
}

#[derive(Default)]
struct SFGenAmount {
    sword: i16,
    uword: u16,
    range: SFGenAmountRange,
}

#[derive(Default)]
struct SFGenAmountRange {
    lo: u8,
    hi: u8,
}

#[derive(Default)]
struct SFChunk {
    id: u32,
    size: u32,
}
