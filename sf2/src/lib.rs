pub mod data;

use data::{
    SFBag, SFData, SFGenerator, SFGeneratorAmountRange, SFGeneratorType, SFInfo,
    SFInstrumentHeader, SFModulator, SFPresetHeader, SFSampleData, SFSampleHeader,
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
        println!("");
    }
}

#[derive(Debug)]
pub struct Preset {
    pub header: SFPresetHeader,
    pub zones: Vec<Zone>,
}

#[derive(Debug)]
pub struct Instrument {
    pub header: SFInstrumentHeader,
    pub zones: Vec<Zone>,
}

#[derive(Debug)]
pub struct SoundFont2 {
    pub info: SFInfo,
    pub presets: Vec<Preset>,
    pub instruments: Vec<Instrument>,
    pub sample_headers: Vec<SFSampleHeader>,
    pub sample_data: SFSampleData,
}

impl SoundFont2 {
    pub fn from_data(data: SFData) -> Self {
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

                let mod_list = {
                    let start = curr.modulator_id as usize;
                    let end = if let Some(next) = next {
                        next.modulator_id as usize
                    } else {
                        zones.len()
                    };

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
                    let start = curr.generator_id as usize;
                    let end = if let Some(next) = next {
                        next.generator_id as usize
                    } else {
                        zones.len()
                    };

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
            let mut i = 0;
            while i < headers.len() - 1 {
                let start = headers[i].bag_id as usize;
                let end = headers[i + 1].bag_id as usize;

                let zone_items = get_zones(&zones, &modulators, &generators, start, end);

                if headers[i].name != "EOI" {
                    list.push(Instrument {
                        header: headers[i].clone(),
                        zones: zone_items,
                    })
                }

                i += 1;
            }

            // for header in iter {
            //     let curr = header;
            //     let next = iter_peek.next();

            //     let start = curr.bag_id as usize;

            //     let end = if let Some(next) = next {
            //         next.bag_id as usize
            //     } else {
            //         zones.len()
            //     };

            //     let zone_items = get_zones(&zones, &modulators, &generators, start, end);
            //     // let zone_items: Vec<_> = zone_items
            //     //     .into_iter()
            //     //     .filter(|zone| {
            //     //         zone.gen_list
            //     //             .iter()
            //     //             .find(|g| g.ty == SFGeneratorType::SampleID)
            //     //             .is_some()
            //     //     })
            //     //     .collect();

            //     if header.name != "EOS" {
            //         list.push(Instrument {
            //             header: header.clone(),
            //             zones: zone_items,
            //         })
            //     }
            // }
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

                let mut zone_items = get_zones(&zones, &modulators, &generators, start, end);

                if header.name != "EOP" {
                    list.push(Preset {
                        header: header.clone(),
                        zones: zone_items,
                    })
                }
            }

            // let re = {
            //     let headers = data.hydra.preset_headers.clone();
            //     // headers.pop();

            //     let mut n = 0;

            //     let mut i = headers.len() - 1;
            //     println!("{}", i);

            //     let mut pr: Option<usize> = None;
            //     let mut zndx: u16 = 0;
            //     let mut pzndx: u16 = 0;
            //     let mut presets = Vec::new();

            //     while i > 0 {
            //         presets.push(Vec::new());

            //         zndx = headers[n].bag_id;

            //         if let Some(pr) = &pr {
            //             if (zndx as i32) < pzndx as i32 {
            //                 panic!("Preset header indices not monotonic");
            //             }
            //             let mut i2 = zndx as i32 - pzndx as i32;
            //             loop {
            //                 let fresh6 = i2;
            //                 i2 = i2 - 1;
            //                 if !(fresh6 != 0) {
            //                     break;
            //                 }
            //                 presets[*pr].insert(0, ());
            //             }
            //         } else if zndx > 0 {
            //             panic!("{} preset zones not referenced, discarding", zndx);
            //         }
            //         pr = Some(presets.len() - 1);
            //         pzndx = zndx;

            //         i -= 1;
            //         n += 1;
            //     }
            //     // println!("Zndx: {}", zndx);
            //     zndx = headers[n].bag_id;
            //     // println!("Zndx: {}", zndx);

            //     if zndx < pzndx {
            //         panic!("Preset header indices not monotonic");
            //     }
            //     let mut i2 = zndx as i32 - pzndx as i32;
            //     loop {
            //         let fresh7 = i2;
            //         i2 = i2 - 1;
            //         if !(fresh7 != 0) {
            //             break;
            //         }
            //         if let Some(pr) = &pr {
            //             presets[*pr].insert(0, ());
            //         }
            //     }
            //     // println!("list: {}", presets.len());
            //     presets
            // };

            // println!("zones: {}", re[12].len());
            // println!("my-zones: {:?}", list[12].zones.len());

            // println!("gen: {:?}", list[1].zones[0].gen_list.len());

            // for (id, my) in list.iter().enumerate() {
            //     let re = &re[id];

            //     let re_len = re.len();
            //     let my_len = my.zones.len();

            //     assert_eq!(re_len, my_len);
            // }

            list
        };

        let mut sum = 0;
        for p in presets.iter() {
            for z in p.zones.iter() {
                if let Some(i) = z.instrument() {
                    sum += 1;
                }
            }
        }
        println!("SUM {}", sum);
        Self {
            info: data.info,
            presets,
            instruments,
            sample_headers: data
                .hydra
                .sample_headers
                .into_iter()
                .filter(|h| h.name != "EOS")
                .collect(),
            sample_data: data.sample_data,
        }
    }

    pub fn sort_presets(&mut self) {
        self.presets.sort_by(|a, b| {
            let aval = (a.header.bank as i32) << 16 | a.header.preset as i32;
            let bbal = (b.header.bank as i32) << 16 | b.header.preset as i32;
            let cmp = aval - bbal;

            if cmp < 0 {
                std::cmp::Ordering::Less
            } else if cmp > 0 {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub mod_list: Vec<SFModulator>,
    pub gen_list: Vec<SFGenerator>,
}

impl Zone {
    pub fn key_range(&self) -> Option<&i16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::KeyRange)
            .map(|g| g.amount.as_i16().unwrap())
    }
    pub fn vel_range(&self) -> Option<&SFGeneratorAmountRange> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::VelRange)
            .map(|g| g.amount.as_range().unwrap())
    }
    pub fn instrument(&self) -> Option<&u16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::Instrument)
            .map(|g| g.amount.as_u16().unwrap())
    }
    pub fn sample(&self) -> Option<&u16> {
        self.gen_list
            .iter()
            .find(|g| g.ty == SFGeneratorType::SampleID)
            .map(|g| g.amount.as_u16().unwrap())
    }
}
