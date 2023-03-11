use soundfont::{data::SFData, SoundFont2};

fn main() {
    let mut file = std::fs::File::open("./testdata/sin.sf2").unwrap();

    // Load from memory
    // use std::io::Cursor;
    // let mut file = Cursor::new(include_bytes!("../testdata/sin.sf2"));

    let data = SFData::load(&mut file).unwrap();
    let sf2 = SoundFont2::from_data(data);

    for p in sf2.presets.iter() {
        println!("====== Preset =======");
        println!("Name: {}", p.header.name);
        let instruments: Vec<_> = p
            .zones
            .iter()
            .filter_map(|z| {
                z.instrument().map(|id| {
                    let instrument = &sf2.instruments[*id as usize];

                    let mut samples = Vec::new();
                    for z in instrument.zones.iter() {
                        if let Some(sample_id) = z.sample() {
                            samples.push(sf2.sample_headers[*sample_id as usize].clone());
                        }
                    }

                    (instrument.header.name.clone(), samples.len())
                })
            })
            .collect();
        println!("Instruments: {:?}", instruments);
        println!();
    }
}
