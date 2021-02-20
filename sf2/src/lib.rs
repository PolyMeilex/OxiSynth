use riff::Chunk;

mod reader;
use reader::Reader;

#[derive(Debug)]
struct SFVersion {
    major: u16,
    minor: u16,
}

/// Supplemental Information
#[derive(Debug)]
struct SFInfo {
    /// Refers to the version of the Sound Font RIFF file
    version: SFVersion,
    /// Refers to the target Sound Engine
    sound_engine: String,
    /// Refers to the Sound Font Bank Name
    bank_name: String,

    /// Refers to the Sound ROM Name
    rom_name: Option<String>,
    /// Refers to the Sound ROM Version
    rom_version: Option<SFVersion>,

    /// Refers to the Date of Creation of the Bank
    creation_date: Option<String>,
    /// Sound Designers and Engineers for the Bank
    engineers: Option<String>,
    /// Product for which the Bank was intended
    product: Option<String>,
    /// Contains any Copyright message
    copyright: Option<String>,
    /// Contains any Comments on the Bank
    comments: Option<String>,
    /// The SoundFont tools used to create and alter the bank
    software: Option<String>,
}

impl SFInfo {
    fn read(info: &Chunk, file: &mut std::fs::File) -> SFInfo {
        assert_eq!(info.id().as_str(), "LIST");
        assert_eq!(info.read_type(file).unwrap().as_str(), "INFO");

        let children: Vec<Chunk> = info.iter(file).collect();

        let mut version = None;
        let mut sound_engine = None;
        let mut bank_name = None;

        let mut rom_name = None;
        let mut rom_version = None;

        let mut creation_date = None;
        let mut engineers = None;
        let mut product = None;
        let mut copyright = None;
        let mut comments = None;
        let mut software = None;

        for ch in children.iter() {
            let id = ch.id();

            match id.as_str() {
                // <ifil-ck> Refers to the version of the Sound Font RIFF file
                "ifil" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    version = Some(SFVersion {
                        major: data.read_u16(),
                        minor: data.read_u16(),
                    });
                }
                // <isng-ck> Refers to the target Sound Engine
                "isng" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    sound_engine = Some(data.read_string(ch.len() as usize));
                }
                // <INAM-ck> Refers to the Sound Font Bank Name
                "INAM" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    bank_name = Some(data.read_string(ch.len() as usize));
                }

                // [<irom-ck>] Refers to the Sound ROM Name
                "irom" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    rom_name = Some(data.read_string(ch.len() as usize));
                }
                // [<iver-ck>] Refers to the Sound ROM Version
                "iver" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    rom_version = Some(SFVersion {
                        major: data.read_u16(),
                        minor: data.read_u16(),
                    });
                }
                // [<ICRD-ck>] Refers to the Date of Creation of the Bank
                "ICRD" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    creation_date = Some(data.read_string(ch.len() as usize));
                }
                // [<IENG-ck>] Sound Designers and Engineers for the Bank
                "IENG" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    engineers = Some(data.read_string(ch.len() as usize));
                }
                // [<IPRD-ck>] Product for which the Bank was intended
                "IPRD" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    product = Some(data.read_string(ch.len() as usize));
                }
                // [<ICOP-ck>] Contains any Copyright message
                "ICOP" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    copyright = Some(data.read_string(ch.len() as usize));
                }
                // [<ICMT-ck>] Contains any Comments on the Bank
                "ICMT" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    comments = Some(data.read_string(ch.len() as usize));
                }
                // [<ISFT-ck>] The SoundFont tools used to create and alter the bank
                "ISFT" => {
                    let mut data = Reader::new(ch.read_contents(file).unwrap());
                    software = Some(data.read_string(ch.len() as usize));
                }
                unknown => {
                    panic!("Unexpected: {} in 'info'", unknown);
                }
            }
        }

        SFInfo {
            version: version.unwrap(),
            sound_engine: sound_engine.unwrap(),
            bank_name: bank_name.unwrap(),

            rom_name,
            rom_version,

            creation_date,
            engineers,
            product,
            copyright,
            comments,
            software,
        }
    }
}

/// The Sample Binary Data
#[derive(Debug)]
struct SFSampleData {
    /// The smpl sub-chunk, if present, contains one or more “samples” of digital audio information in the form of linearly coded sixteen bit, signed, little endian (least significant byte first) words.  Each sample is followed by a minimum of forty-six zero valued sample data points.  These zero valued data points are necessary to guarantee that any reasonable upward pitch shift using any reasonable interpolator can loop on zero data at the end of the sound.
    smpl: Option<Chunk>,
    /// The sm24 sub-chunk, if present, contains the least significant byte counterparts to each sample data point contained in the smpl chunk. Note this means for every two bytes in the [smpl] sub-chunk there is a 1-byte counterpart in [sm24] sub-chunk.
    ///
    /// These sample waveform points are to be combined with the sample waveform points in the smpl sub-chunk, to collectively create a single sample data pool with up to 24 bits of resolution.
    ///
    /// If the smpl Sub-chunk is not present, the sm24 sub-chunk should be ignored. If the ifil version of the format is less than thatwhich represents 2.04, the sm24 sub-chunk should be ignored. If the size of the sm24 chunk is not exactly equal to the ½ the size of the smpl chunk (+ 1 byte in the case that ½ the size of smpl chunk is an odd value), the sm24 sub-chunk should be ignored.  
    ///
    /// In any and all cases where the sm24 sub-chunk is ignored, the synthesizer should render only those samples contained within the smpl sub-chunk.
    sm24: Option<Chunk>,
}

impl SFSampleData {
    fn read(sdta: &Chunk, file: &mut std::fs::File) -> Self {
        assert_eq!(sdta.id().as_str(), "LIST");
        assert_eq!(sdta.read_type(file).unwrap().as_str(), "sdta");

        let mut smpl = None;
        let mut sm24 = None;

        for ch in sdta.iter(file) {
            let id = ch.id();

            match id.as_str() {
                // [<smpl-ck>] The Digital Audio Samples for the upper 16 bits
                "smpl" => {
                    smpl = Some(ch);
                }
                // [<sm24-ck>] The Digital Audio Samples for the lower 8 bits
                "sm23" => {
                    sm24 = Some(ch);
                }
                unknown => {
                    panic!("Unexpected: {} in sdta", unknown);
                }
            }
        }

        SFSampleData { smpl, sm24 }
    }
}

#[derive(Debug)]
struct SFPresetHeader {
    name: String,
    preset: u16,
    bank: u16,
    bag_id: u16,
    library: u32,
    genre: u32,
    morphology: u32,
}

impl SFPresetHeader {
    fn read(reader: &mut Reader) -> Self {
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

    fn read_all(phdr: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
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
struct SFPresetBag {
    generator_id: u16,
    modulator_id: u16,
}

impl SFPresetBag {
    fn read(reader: &mut Reader) -> Self {
        let generator_id: u16 = reader.read_u16();
        let modulator_id: u16 = reader.read_u16();

        Self {
            generator_id,
            modulator_id,
        }
    }

    fn read_all(pbag: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
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

#[derive(Debug)]
struct SFModulator {
    src: u16,
    dest: u16,
    amount: i16,
    amt_src: u16,
    transform: u16,
}

impl SFModulator {
    fn read(reader: &mut Reader) -> Self {
        let src: u16 = reader.read_u16();
        let dest: u16 = reader.read_u16();
        let amount: i16 = reader.read_i16();
        let amt_src: u16 = reader.read_u16();
        let transform: u16 = reader.read_u16();

        Self {
            src,
            dest,
            amount,
            amt_src,
            transform,
        }
    }

    fn read_all(pmod: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(pmod.id().as_str(), "pmod");

        let size = pmod.len();
        if size % 10 != 0 || size == 0 {
            panic!("Preset modulator chunk size mismatch");
        }

        let amount = size / 10;

        let data = pmod.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}

// #[derive(Debug)]
union SFGeneratorAmount {
    sword: i16,
    uword: u16,
    range: SFGeneratorAmountRange,
}
impl std::fmt::Debug for SFGeneratorAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "...")
    }
}

#[derive(Debug, Clone, Copy)]
struct SFGeneratorAmountRange {
    low: u8,
    high: u8,
}

#[derive(Debug)]
struct SFGenerator {
    id: u16,
    amount: SFGeneratorAmount,
}

impl SFGenerator {
    fn read(reader: &mut Reader) -> Self {
        let id: u16 = reader.read_u16();

        let amount = SFGeneratorAmount {
            sword: reader.read_i16(),
            // uword: reader.read_u16(),
            // range: SFGeneratorAmountRange {
            //     low: reader.read_u8(),
            //     high: reader.read_u8(),
            // },
        };

        Self { id, amount }
    }

    fn read_all(pmod: &Chunk, file: &mut std::fs::File) -> Vec<Self> {
        assert_eq!(pmod.id().as_str(), "pgen");

        let size = pmod.len();
        if size % 4 != 0 || size == 0 {
            panic!("Preset generator chunk size mismatch");
        }

        let amount = size / 4;

        let data = pmod.read_contents(file).unwrap();
        let mut reader = Reader::new(data);

        (0..amount).map(|_| Self::read(&mut reader)).collect()
    }
}

#[derive(Debug)]
struct SFHydra {
    preset_headers: Vec<SFPresetHeader>,
    preset_bags: Vec<SFPresetBag>,
    modulators: Vec<SFModulator>,
}

impl SFHydra {
    fn read(pdta: &Chunk, file: &mut std::fs::File) -> Self {
        // fn phdr(phdr: &Chunk, file: &mut std::fs::File) {}
        // fn pbag(pbag: &Chunk, file: &mut std::fs::File) {}
        // fn pmod(pmod: &Chunk, file: &mut std::fs::File) {}
        fn pgen(pgen: &Chunk, file: &mut std::fs::File) {}

        assert_eq!(pdta.id().as_str(), "LIST");
        assert_eq!(pdta.read_type(file).unwrap().as_str(), "pdta");

        let chunks: Vec<_> = pdta.iter(file).collect();

        let mut preset_headers = None;
        let mut preset_bags = None;
        let mut modulators = None;

        for ch in chunks.iter() {
            let id = ch.id();

            match id.as_str() {
                "phdr" => {
                    preset_headers = Some(SFPresetHeader::read_all(ch, file));
                }
                "pbag" => {
                    preset_bags = Some(SFPresetBag::read_all(ch, file));
                }
                "pmod" => modulators = Some(SFModulator::read_all(ch, file)),
                "pgen" => {
                    Some(SFGenerator::read_all(ch, file));
                }
                _ => {}
            }
        }

        Self {
            preset_headers: preset_headers.unwrap(),
            preset_bags: preset_bags.unwrap(),
            modulators: modulators.unwrap(),
        }
    }
}

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

    let sf_file = SFFile {
        info: info.unwrap(),
        sample_data: sample_data.unwrap(),
        hydra: hydra.unwrap(),
    };

    println!("{:#?}", sf_file);
}

mod unox {
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
}
