use std::sync::Arc;

use super::generator::{GeneratorList, GeneratorType};
use super::modulator::Mod;
use super::{instrument::Instrument, Sample};

const GEN_SET: u32 = 1;

pub struct Preset {
    name: String,
    bank: u32,
    num: u32,
    global_zone: Option<PresetZone>,
    zones: Vec<PresetZone>,
}

impl Preset {
    pub fn import(
        sf2: &soundfont::SoundFont2,
        preset: &soundfont::Preset,
        samples: &[Arc<Sample>],
    ) -> Result<Self, ()> {
        let name = if !preset.header.name.is_empty() {
            preset.header.name.clone()
        } else {
            format!("Bank:{},Preset{}", preset.header.bank, preset.header.preset)
        };

        let mut global_zone = None;
        let mut zones = Vec::new();

        for (id, sfzone) in preset.zones.iter().enumerate() {
            let name = format!("{}/{}", preset.header.name, id);
            let zone = PresetZone::import(name, sf2, sfzone, samples)?;

            if id == 0 && zone.inst.is_none() {
                global_zone = Some(zone);
            } else {
                zones.push(zone);
            }
        }

        Ok(Self {
            name,
            bank: preset.header.bank as u32,
            num: preset.header.preset as u32,
            global_zone,
            zones,
        })
    }

    pub(crate) fn global_zone(&self) -> Option<&PresetZone> {
        self.global_zone.as_ref()
    }

    pub(crate) fn zones(&self) -> &[PresetZone] {
        &self.zones
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn banknum(&self) -> u32 {
        self.bank
    }

    pub fn num(&self) -> u32 {
        self.num
    }
}

pub struct PresetZone {
    #[allow(dead_code)]
    pub name: String,
    pub inst: Option<Instrument>,
    pub key_low: u8,
    pub key_high: u8,
    pub vel_low: u8,
    pub vel_high: u8,
    pub gen: GeneratorList,
    pub mods: Vec<Mod>,
}

impl PresetZone {
    pub fn import(
        name: String,
        sf2: &soundfont::SoundFont2,
        zone: &soundfont::Zone,
        samples: &[Arc<Sample>],
    ) -> Result<Self, ()> {
        let mut key_low = 0;
        let mut key_high = 128;
        let mut vel_low = 0;
        let mut vel_high = 128;

        let mut gen = GeneratorList::default();

        for sfgen in zone
            .gen_list
            .iter()
            .filter(|g| g.ty != soundfont::data::GeneratorType::Instrument)
        {
            match sfgen.ty {
                soundfont::data::GeneratorType::KeyRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    key_low = amount.low;
                    key_high = amount.high;
                }
                soundfont::data::GeneratorType::VelRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    vel_low = amount.low;
                    vel_high = amount.high;
                }
                _ => {
                    let ty = GeneratorType::from(sfgen.ty);

                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[ty].val = *sfgen.amount.as_i16().unwrap() as f64;
                    gen[ty].flags = GEN_SET as u8;
                }
            }
        }

        let inst = if let Some(id) = zone.instrument() {
            let i = Instrument::import(sf2, &sf2.instruments[*id as usize], samples)?;
            Some(i)
        } else {
            None
        };

        // Import the modulators (only SF2.1 and higher)
        let mods: Vec<_> = zone
            .mod_list
            .iter()
            .map(|mod_src| {
                /* Store the new modulator in the zone The order of modulators
                 * will make a difference, at least in an instrument context: The
                 * second modulator overwrites the first one, if they only differ
                 * in amount. */
                Mod::from(mod_src)
            })
            .collect();

        Ok(Self {
            name,
            inst,
            key_low,
            key_high,
            vel_low,
            vel_high,
            gen,
            mods,
        })
    }
}
