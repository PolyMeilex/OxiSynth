use crate::Reader;
use riff::Chunk;

#[derive(Debug)]
pub struct SFVersion {
    major: u16,
    minor: u16,
}

/// Supplemental Information
#[derive(Debug)]
pub struct SFInfo {
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
    pub fn read(info: &Chunk, file: &mut std::fs::File) -> SFInfo {
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