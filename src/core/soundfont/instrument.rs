use std::sync::Arc;

use crate::GeneratorType;

use super::generator::GeneratorList;
use super::modulator::Mod;
use super::Sample;

const GEN_SET: u32 = 1;

#[derive(Clone, Debug)]
pub struct Instrument {
    _name: String,
    global_zone: Option<InstrumentZone>,
    zones: Vec<InstrumentZone>,
}

impl Instrument {
    pub fn import(
        sf2: &soundfont::SoundFont2,
        inst: &soundfont::Instrument,
        samples: &[Arc<Sample>],
    ) -> Result<Self, ()> {
        let name = if !inst.header.name.is_empty() {
            inst.header.name.clone()
        } else {
            "<untitled>".into()
        };

        let mut global_zone = None;
        let mut zones = Vec::new();

        for (id, zone) in inst.zones.iter().enumerate() {
            let name = format!("{}/{}", inst.header.name, id);
            let zone = InstrumentZone::import(name, sf2, zone, samples)?;
            if id == 0 && zone.sample.is_none() {
                global_zone = Some(zone);
            } else {
                zones.push(zone);
            }
        }

        Ok(Self {
            _name: name,
            global_zone,
            zones,
        })
    }

    pub fn global_zone(&self) -> Option<&InstrumentZone> {
        self.global_zone.as_ref()
    }

    pub fn zones(&self) -> &[InstrumentZone] {
        &self.zones
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct InstrumentZone {
    pub name: String,
    pub sample: Option<Arc<Sample>>,
    pub key_low: u8,
    pub key_high: u8,
    pub vel_low: u8,
    pub vel_high: u8,
    pub gen: GeneratorList,
    pub mods: Vec<Mod>,
}

impl InstrumentZone {
    fn import(
        name: String,
        sf2: &soundfont::SoundFont2,
        zone: &soundfont::Zone,
        samples: &[Arc<Sample>],
    ) -> Result<InstrumentZone, ()> {
        let mut key_low = 0;
        let mut key_high = 128;
        let mut vel_low = 0;
        let mut vel_high = 128;

        let mut gen = GeneratorList::default();

        for new_gen in zone
            .gen_list
            .iter()
            .filter(|g| g.ty != soundfont::data::GeneratorType::SampleID)
        {
            match new_gen.ty {
                soundfont::data::GeneratorType::KeyRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    key_low = amount.low;
                    key_high = amount.high;
                }
                soundfont::data::GeneratorType::VelRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    vel_low = amount.low;
                    vel_high = amount.high;
                }
                _ => {
                    let ty = GeneratorType::from(new_gen.ty);
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[ty].val = *new_gen.amount.as_i16().unwrap() as f64;
                    gen[ty].flags = GEN_SET as u8;
                }
            }
        }

        let sample = if let Some(sample_id) = zone.sample() {
            let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();
            let name = &sample.name;

            // Find Sample by name:
            let sample = samples.iter().find(|sample| &sample.name == name).cloned();

            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }

            sample
        } else {
            None
        };

        let mods = zone
            .mod_list
            .iter()
            .map(|new_mod| {
                /* Store the new modulator in the zone
                 * The order of modulators will make a difference, at least in an instrument context:
                 * The second modulator overwrites the first one, if they only differ in amount. */
                Mod::from(new_mod)
            })
            .collect();

        Ok(Self {
            name,
            sample,
            key_low,
            key_high,
            vel_low,
            vel_high,
            gen,
            mods,
        })
    }
}
