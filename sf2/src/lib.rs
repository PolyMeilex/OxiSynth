mod data;

use data::{
    SFBag, SFData, SFGenerator, SFGeneratorAmountRange, SFGeneratorType, SFInstrumentHeader,
    SFModulator, SFPresetHeader, SFSampleData, SFSampleHeader,
};

pub fn main() {
    let mut file = std::fs::File::open("./testdata/test.sf2").unwrap();

    let data = SFData::load(&mut file);
    let sf2 = SoundFont2::from_data(data);

    for p in sf2.presets.iter() {
        println!("====== Preset =======");
        println!("Name: {}", p.header.name);
        let instruments: Vec<_> = p
            .zones
            .iter()
            .map(|z| {
                let id = z.instrument().unwrap();
                let instrument = &sf2.instruments[*id as usize];

                let mut samples = Vec::new();
                for z in instrument.zones.iter() {
                    let sample_id = z.sample().unwrap();
                    samples.push(sf2.sample_headers[*sample_id as usize].clone());
                }

                (instrument.header.name.clone(), samples.len())
            })
            .collect();
        println!("Instruments: {:?}", instruments);
        println!("");
    }
}

#[derive(Debug)]
pub struct Preset {
    header: SFPresetHeader,
    zones: Vec<Zone>,
}

#[derive(Debug)]
pub struct Instrument {
    header: SFInstrumentHeader,
    zones: Vec<Zone>,
}

#[derive(Debug)]
pub struct SoundFont2 {
    presets: Vec<Preset>,
    instruments: Vec<Instrument>,
    sample_headers: Vec<SFSampleHeader>,
    sample_data: SFSampleData,
}

impl SoundFont2 {
    fn from_data(data: SFData) -> Self {
        fn get_zones(
            zones: &[SFBag],
            modulators: &[SFModulator],
            generators: &[SFGenerator],
            start: usize,
            end: usize,
        ) -> Vec<Zone> {
            let mut zone_items = Vec::new();
            let mut j = start;
            while j < end {
                let curr = zones.get(j).unwrap();
                let next = zones.get(j + 1);

                let start = curr.generator_id as usize;

                let end = if let Some(next) = next {
                    next.generator_id as usize
                } else {
                    zones.len()
                };

                let mod_list = {
                    let mut list = Vec::new();

                    let mut i = start;
                    while i < end {
                        let item = modulators.get(i);
                        if let Some(item) = item {
                            list.push(item.to_owned());
                        }
                        i += 1;
                    }
                    list
                };

                let gen_list = {
                    let mut list = Vec::new();

                    let mut i = start;
                    while i < end {
                        let item = generators.get(i);
                        if let Some(item) = item {
                            list.push(item.to_owned());
                        }
                        i += 1;
                    }
                    list
                };

                zone_items.push(Zone { mod_list, gen_list });

                j += 1
            }
            zone_items
        }

        let instruments = {
            let headers = &data.hydra.instrument_headers;
            let zones = &data.hydra.instrument_bags;
            let modulators = &data.hydra.instrument_modulators;
            let generators = &data.hydra.instrument_generators;

            let iter = headers.iter();
            let mut iter_peek = headers.iter();
            iter_peek.next();

            let mut list = Vec::new();
            for header in iter {
                let curr = header;
                let next = iter_peek.next();

                let start = curr.bag_id as usize;

                let end = if let Some(next) = next {
                    next.bag_id as usize
                } else {
                    zones.len()
                };

                let zone_items = get_zones(&zones, &modulators, &generators, start, end);
                let zone_items: Vec<_> = zone_items
                    .into_iter()
                    .filter(|zone| {
                        zone.gen_list
                            .iter()
                            .find(|g| g.ty == SFGeneratorType::SampleID)
                            .is_some()
                    })
                    .collect();

                list.push(Instrument {
                    header: header.clone(),
                    zones: zone_items,
                })
            }
            list
        };

        let presets = {
            let headers = &data.hydra.preset_headers;
            let zones = &data.hydra.preset_bags;
            let modulators = &data.hydra.preset_modulators;
            let generators = &data.hydra.preset_generators;

            let iter = headers.iter();
            let mut iter_peek = headers.iter();
            iter_peek.next();

            let mut list = Vec::new();
            for header in iter {
                let curr = header;
                let next = iter_peek.next();

                let start = curr.bag_id as usize;

                let end = if let Some(next) = next {
                    next.bag_id as usize
                } else {
                    zones.len()
                };

                let zone_items = get_zones(&zones, &modulators, &generators, start, end);

                let zone_items: Vec<_> = zone_items
                    .into_iter()
                    .filter(|zone| {
                        zone.gen_list
                            .iter()
                            .find(|g| g.ty == SFGeneratorType::Instrument)
                            .is_some()
                    })
                    .collect();

                list.push(Preset {
                    header: header.clone(),
                    zones: zone_items,
                })
            }
            list
        };

        Self {
            presets,
            instruments,
            sample_headers: data.hydra.sample_headers,
            sample_data: data.sample_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Zone {
    mod_list: Vec<SFModulator>,
    gen_list: Vec<SFGenerator>,
}

impl Zone {
    fn key_range(&self) -> Option<&i16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::KeyRange)
            .map(|g| g.amount.as_i16().unwrap())
    }
    fn vel_range(&self) -> Option<&SFGeneratorAmountRange> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::VelRange)
            .map(|g| g.amount.as_range().unwrap())
    }
    fn instrument(&self) -> Option<&u16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::Instrument)
            .map(|g| g.amount.as_u16().unwrap())
    }
    fn sample(&self) -> Option<&u16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::SampleID)
            .map(|g| g.amount.as_u16().unwrap())
    }
}
