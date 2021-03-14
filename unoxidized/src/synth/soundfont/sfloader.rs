use crate::channel::ChannelId;
use soundfont as sf2;
use std::rc::Rc;

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::generator::{self, Gen};
use crate::modulator::Mod;
use crate::soundfont::Sample;
use crate::synth::voice_pool::{Voice, VoiceAddMode, VoiceDescriptor};
use crate::synth::Synth;

const GEN_SET: u32 = 1;

pub(super) struct DefaultSoundFont {
    pub filename: PathBuf,
    pub presets: Vec<Rc<DefaultPreset>>,
}

impl DefaultSoundFont {
    pub fn load<F: Read + Seek>(file: &mut F) -> Result<Self, ()> {
        let data = sf2::data::SFData::load(file);

        let data = match data {
            Ok(data) => data,
            Err(err) => {
                log::error!("{:#?}", err);
                return Err(());
            }
        };

        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let sample_pos = smpl.offset() + 8;
        let sample_size = smpl.len() as usize;

        let sample_data = Rc::new(Self::load_sampledata(file, sample_pos, sample_size)?);

        let mut samples = Vec::new();

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import_sfont(sfsample, sample_data.clone())?;
            let mut sample = sample;

            sample.optimize_sample();

            samples.push(Rc::new(sample));
        }

        let mut presets = Vec::new();
        for sfpreset in sf2.presets.iter() {
            let preset = DefaultPreset::import_sfont(&sf2, sfpreset, &samples)?;
            presets.push(Rc::new(preset));
        }

        Ok(DefaultSoundFont {
            filename: PathBuf::from(""),
            presets,
        })
    }

    fn load_sampledata<F: Read + Seek>(
        file: &mut F,
        sample_pos: u64,
        sample_size: usize,
    ) -> Result<Vec<i16>, ()> {
        if file.seek(SeekFrom::Start(sample_pos)).is_err() {
            log::error!("Failed to seek position in data file",);
            return Err(());
        }

        let mut sample_data = vec![0u8; sample_size];
        if file.read_exact(&mut sample_data).is_err() {
            log::error!("Failed to read sample data");
            return Err(());
        }

        let sample_data: Vec<i16> = sample_data
            .chunks(2)
            .map(|num| {
                if num.len() == 2 {
                    i16::from_le_bytes([num[0], num[1]])
                } else {
                    log::error!("Wrong sample data");
                    0
                }
            })
            .collect();

        Ok(sample_data)
    }
}

pub(super) struct DefaultPreset {
    pub name: String,
    pub bank: u32,
    pub num: u32,
    global_zone: Option<PresetZone>,
    zones: Vec<PresetZone>,
}

impl DefaultPreset {
    fn import_sfont(
        sf2: &sf2::SoundFont2,
        sfpreset: &sf2::Preset,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut preset = DefaultPreset {
            name: String::new(),
            bank: 0 as i32 as u32,
            num: 0 as i32 as u32,
            global_zone: None,
            zones: Vec::new(),
        };

        if sfpreset.header.name.len() != 0 {
            preset.name = sfpreset.header.name.clone();
        } else {
            preset.name = format!(
                "Bank:{},Preset{}",
                sfpreset.header.bank, sfpreset.header.preset
            );
        }

        preset.bank = sfpreset.header.bank as u32;
        preset.num = sfpreset.header.preset as u32;

        for (id, sfzone) in sfpreset.zones.iter().enumerate() {
            let name = format!("{}/{}", sfpreset.header.name, id);
            let zone = PresetZone::import_sfont(name, sf2, sfzone, samples)?;

            if id == 0 && zone.inst.is_none() {
                preset.global_zone = Some(zone);
            } else {
                preset.zones.push(zone);
            }
        }

        Ok(preset)
    }
}

struct PresetZone {
    #[allow(dead_code)]
    name: String,
    inst: Option<Instrument>,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mods: Vec<Mod>,
}

impl PresetZone {
    fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        sfzone: &sf2::Zone,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut zone = Self {
            name,
            inst: None,
            keylo: 0,
            keyhi: 128,
            vello: 0 as i32,
            velhi: 128 as i32,
            gen: generator::get_default_values(),
            mods: Vec::new(),
        };

        for sfgen in sfzone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::Instrument)
        {
            match sfgen.ty {
                sf2::data::GeneratorType::KeyRange | sf2::data::GeneratorType::VelRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    zone.keylo = amount.low;
                    zone.keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    zone.gen[sfgen.ty as usize].val = *sfgen.amount.as_i16().unwrap() as f64;
                    zone.gen[sfgen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }
        if let Some(id) = sfzone.instrument() {
            let inst = Instrument::import_sfont(sf2, &sf2.instruments[*id as usize], samples)?;

            zone.inst = Some(inst);
        }
        // Import the modulators (only SF2.1 and higher)
        for mod_src in sfzone.mod_list.iter() {
            let mod_dest = Mod::from(mod_src);

            /* Store the new modulator in the zone The order of modulators
             * will make a difference, at least in an instrument context: The
             * second modulator overwrites the first one, if they only differ
             * in amount. */
            zone.mods.push(mod_dest);
        }

        Ok(zone)
    }
}

#[derive(Clone)]
struct Instrument {
    // [u8;21]
    name: String,
    global_zone: Option<InstrumentZone>,
    zones: Vec<InstrumentZone>,
}

impl Instrument {
    fn import_sfont(
        sf2: &sf2::SoundFont2,
        new_inst: &sf2::Instrument,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<Self, ()> {
        let mut inst = Self {
            name: String::new(),
            global_zone: None,
            zones: Vec::new(),
        };

        if new_inst.header.name.len() > 0 {
            inst.name = new_inst.header.name.clone();
        } else {
            inst.name = "<untitled>".into();
        }
        for (id, new_zone) in new_inst.zones.iter().enumerate() {
            let name = format!("{}/{}", new_inst.header.name, id);
            let zone = InstrumentZone::import_sfont(name, sf2, new_zone, samples)?;
            if id == 0 && zone.sample.is_none() {
                inst.global_zone = Some(zone);
            } else {
                inst.zones.push(zone);
            }
        }

        Ok(inst)
    }
}

#[derive(Clone)]
#[repr(C)]
struct InstrumentZone {
    name: String,
    sample: Option<Rc<Sample>>,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mods: Vec<Mod>,
}

impl InstrumentZone {
    fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        new_zone: &sf2::Zone,
        samples: &Vec<Rc<Sample>>,
    ) -> Result<InstrumentZone, ()> {
        let mut keylo = 0;
        let mut keyhi = 128;
        let mut gen = generator::get_default_values();

        for new_gen in new_zone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::GeneratorType::SampleID)
        {
            match new_gen.ty {
                sf2::data::GeneratorType::KeyRange | sf2::data::GeneratorType::VelRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    keylo = amount.low;
                    keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[new_gen.ty as usize].val = *new_gen.amount.as_i16().unwrap() as f64;
                    gen[new_gen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }

        let sample = if let Some(sample_id) = new_zone.sample() {
            let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();
            let name = &sample.name;

            // Find Sample by name:
            let sample = samples
                .iter()
                .find(|sample| &sample.name == name)
                .map(|s| s.clone());

            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }
            sample
        } else {
            None
        };

        let mut mods = Vec::new();

        for new_mod in new_zone.mod_list.iter() {
            let mod_dest = Mod::from(new_mod);
            /* Store the new modulator in the zone
             * The order of modulators will make a difference, at least in an instrument context:
             * The second modulator overwrites the first one, if they only differ in amount. */
            mods.push(mod_dest);
        }

        Ok(InstrumentZone {
            name: name,
            sample,
            keylo,
            keyhi,
            vello: 0,
            velhi: 128,
            gen,
            mods,
        })
    }
}

impl Synth {
    /// noteon
    pub(crate) fn sf_noteon(&mut self, chan: u8, key: u8, vel: u8) {
        fn preset_zone_inside_range(zone: &PresetZone, key: u8, vel: u8) -> bool {
            zone.keylo <= key
                && zone.keyhi >= key
                && zone.vello <= vel as i32
                && zone.velhi >= vel as i32
        }

        fn inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: u8) -> bool {
            zone.keylo <= key
                && zone.keyhi >= key
                && zone.vello <= vel as i32
                && zone.velhi >= vel as i32
        }

        fn sample_in_rom(sample: &Sample) -> i32 {
            // sampletype & FLUID_SAMPLETYPE_ROM
            sample.sampletype & 0x8000
        }

        let preset = &self.channels[chan as usize].preset.as_ref().unwrap().data;

        // list for 'sorting' preset modulators
        let mod_list_new: Vec<Option<&Mod>> = (0..64).into_iter().map(|_| None).collect();
        use std::convert::TryInto;
        let mut mod_list: [Option<&Mod>; 64] = mod_list_new.try_into().unwrap();

        let mut global_preset_zone = &preset.global_zone;

        // run thru all the zones of this preset
        for preset_zone in preset.zones.iter() {
            // check if the note falls into the key and velocity range of this preset
            if preset_zone_inside_range(preset_zone, key, vel) {
                let inst = preset_zone.inst.as_ref().unwrap();

                let mut global_inst_zone = &inst.global_zone;

                // run thru all the zones of this instrument
                for inst_zone in inst.zones.iter() {
                    // make sure this instrument zone has a valid sample
                    let sample = &inst_zone.sample;
                    if !(sample.is_none() || sample_in_rom(&sample.as_ref().unwrap()) != 0) {
                        // check if the note falls into the key and velocity range of this instrument
                        if inst_zone_inside_range(inst_zone, key, vel) && !sample.is_none() {
                            // this is a good zone. allocate a new synthesis process and initialize it

                            // Initialize Voice
                            let init = |voice: &mut Voice| {
                                voice.add_default_mods();

                                // Instrument level, generators
                                for i in 0..GEN_LAST {
                                    use num_traits::FromPrimitive;
                                    /* SF 2.01 section 9.4 'bullet' 4:
                                     *
                                     * A generator in a local instrument zone supersedes a
                                     * global instrument zone generator.  Both cases supersede
                                     * the default generator -> voice_gen_set */
                                    if inst_zone.gen[i as usize].flags != 0 {
                                        voice.gen_set(
                                            FromPrimitive::from_u8(i as u8).unwrap(),
                                            inst_zone.gen[i as usize].val,
                                        );
                                    } else if let Some(global_inst_zone) = &global_inst_zone {
                                        if global_inst_zone.gen[i as usize].flags as i32 != 0 {
                                            voice.gen_set(
                                                FromPrimitive::from_u8(i as u8).unwrap(),
                                                global_inst_zone.gen[i as usize].val,
                                            );
                                        }
                                    } else {
                                        /* The generator has not been defined in this instrument.
                                         * Do nothing, leave it at the default.
                                         */
                                    }
                                }

                                /* global instrument zone, modulators: Put them all into a
                                 * list. */
                                let mut mod_list_count = 0;
                                if let Some(global_inst_zone) = &mut global_inst_zone {
                                    for m in global_inst_zone.mods.iter() {
                                        mod_list[mod_list_count] = Some(m);
                                        mod_list_count += 1;
                                    }
                                }

                                /* local instrument zone, modulators.
                                 * Replace modulators with the same definition in the list:
                                 * SF 2.01 page 69, 'bullet' 8
                                 */
                                for m in inst_zone.mods.iter() {
                                    /* 'Identical' modulators will be deleted by setting their
                                     *  list entry to NULL.  The list length is known, NULL
                                     *  entries will be ignored later.  SF2.01 section 9.5.1
                                     *  page 69, 'bullet' 3 defines 'identical'.  */
                                    for i in 0..mod_list_count {
                                        if !mod_list[i].is_none()
                                            && m.test_identity(
                                                mod_list[i as usize].as_ref().unwrap(),
                                            )
                                        {
                                            mod_list[i] = None;
                                        }
                                    }

                                    /* Finally add the new modulator to to the list. */
                                    mod_list[mod_list_count] = Some(m);

                                    mod_list_count += 1;
                                }

                                // Add instrument modulators (global / local) to the voice.
                                for i in 0..mod_list_count {
                                    let mod_0 = mod_list[i as usize];
                                    if !mod_0.is_none() {
                                        // disabled modulators CANNOT be skipped.

                                        /* Instrument modulators -supersede- existing (default)
                                         * modulators.  SF 2.01 page 69, 'bullet' 6 */
                                        voice.add_mod(
                                            mod_0.as_ref().unwrap(),
                                            VoiceAddMode::Overwrite,
                                        );
                                    }
                                }

                                const GEN_STARTADDROFS: u32 = 0;
                                const GEN_ENDADDROFS: u32 = 1;
                                const GEN_STARTLOOPADDROFS: u32 = 2;
                                const GEN_ENDLOOPADDROFS: u32 = 3;
                                const GEN_STARTADDRCOARSEOFS: u32 = 4;

                                const GEN_ENDADDRCOARSEOFS: u32 = 12;

                                const GEN_STARTLOOPADDRCOARSEOFS: u32 = 45;
                                const GEN_KEYNUM: u32 = 46;
                                const GEN_VELOCITY: u32 = 47;

                                const GEN_ENDLOOPADDRCOARSEOFS: u32 = 50;
                                const GEN_SAMPLEMODE: u32 = 54;
                                const GEN_EXCLUSIVECLASS: u32 = 57;
                                const GEN_OVERRIDEROOTKEY: u32 = 58;
                                const GEN_LAST: u32 = 60;

                                /* Preset level, generators */
                                for i in 0..GEN_LAST {
                                    /* SF 2.01 section 8.5 page 58: If some generators are
                                     * encountered at preset level, they should be ignored */
                                    if i != GEN_STARTADDROFS
                                        && i != GEN_ENDADDROFS
                                        && i != GEN_STARTLOOPADDROFS
                                        && i != GEN_ENDLOOPADDROFS
                                        && i != GEN_STARTADDRCOARSEOFS
                                        && i != GEN_ENDADDRCOARSEOFS
                                        && i != GEN_STARTLOOPADDRCOARSEOFS
                                        && i != GEN_KEYNUM
                                        && i != GEN_VELOCITY
                                        && i != GEN_ENDLOOPADDRCOARSEOFS
                                        && i != GEN_SAMPLEMODE
                                        && i != GEN_EXCLUSIVECLASS
                                        && i != GEN_OVERRIDEROOTKEY
                                    {
                                        /* SF 2.01 section 9.4 'bullet' 9: A generator in a
                                         * local preset zone supersedes a global preset zone
                                         * generator.  The effect is -added- to the destination
                                         * summing node -> voice_gen_incr */
                                        if preset_zone.gen[i as usize].flags != 0 {
                                            voice.gen_incr(i, preset_zone.gen[i as usize].val);
                                        } else if let Some(global_preset_zone) = &global_preset_zone
                                        {
                                            if global_preset_zone.gen[i as usize].flags != 0 {
                                                voice.gen_incr(
                                                    i,
                                                    global_preset_zone.gen[i as usize].val,
                                                );
                                            }
                                        } else {
                                            /* The generator has not been defined in this preset
                                             * Do nothing, leave it unchanged.
                                             */
                                        }
                                    } /* if available at preset level */
                                } /* for all generators */

                                /* Global preset zone, modulators: put them all into a
                                 * list. */
                                let mut mod_list_count = 0;
                                if let Some(global_preset_zone) = &mut global_preset_zone {
                                    for m in global_preset_zone.mods.iter() {
                                        mod_list[mod_list_count] = Some(m);
                                        mod_list_count += 1;
                                    }
                                }

                                /* Process the modulators of the local preset zone.  Kick
                                 * out all identical modulators from the global preset zone
                                 * (SF 2.01 page 69, second-last bullet) */
                                for m in preset_zone.mods.iter() {
                                    for i in 0..mod_list_count {
                                        if !mod_list[i].is_none()
                                            && m.test_identity(
                                                mod_list[i as usize].as_ref().unwrap(),
                                            )
                                        {
                                            mod_list[i] = None;
                                        }
                                    }

                                    /* Finally add the new modulator to the list. */
                                    mod_list[mod_list_count] = Some(m);

                                    mod_list_count += 1;
                                }

                                // Add preset modulators (global / local) to the voice.
                                for i in 0..mod_list_count {
                                    if let Some(m) = mod_list[i] {
                                        if m.amount != 0.0 {
                                            // disabled modulators can be skipped.

                                            /* Preset modulators -add- to existing instrument /
                                             * default modulators.  SF2.01 page 70 first bullet on
                                             * page */
                                            voice.add_mod(m, VoiceAddMode::Add);
                                        }
                                    }
                                }

                                /* Store the ID of the first voice that was created by this noteon event.
                                 * Exclusive class may only terminate older voices.
                                 * That avoids killing voices, which have just been created.
                                 * (a noteon event can create several voice processes with the same exclusive
                                 * class - for example when using stereo samples)
                                 */
                            };

                            let desc = VoiceDescriptor {
                                sample: sample.as_ref().unwrap().clone(),
                                channel: &self.channels[chan as usize],
                                channel_id: ChannelId(chan as usize),
                                key,
                                vel,
                                id: self.storeid,
                                start_time: self.ticks,
                                gain: self.settings.gain,
                            };

                            let voice_id = self.voices.request_new_voice(self.noteid, desc, init);

                            if let Ok(voice_id) = voice_id {
                                log::trace!(
                                    "noteon\t{}\t{}\t{}\t{}\t{}",
                                    chan,
                                    key,
                                    vel,
                                    self.storeid,
                                    self.ticks as f32 / 44100.0,
                                );

                                // add the synthesis process to the synthesis loop.
                                self.voices.start_voice(&self.channels, voice_id);
                            } else {
                                log::warn!(
                                    "Failed to allocate a synthesis process. (chan={},key={})",
                                    chan,
                                    key
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
