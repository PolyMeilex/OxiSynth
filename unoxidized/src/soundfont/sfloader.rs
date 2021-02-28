use soundfont_rs as sf2;
use std::rc::Rc;

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::gen::{self, Gen};
use crate::modulator::Mod;
use crate::soundfont::Preset;
use crate::soundfont::Sample;
use crate::soundfont::SoundFont;
use crate::synth::Synth;
use crate::voice::FluidVoiceAddMod;
use std::path::Path;
use std::slice::from_raw_parts_mut;

const GEN_SET: GenFlags = 1;
const GEN_VELOCITY: u32 = 47;
const GEN_KEYNUM: u32 = 46;
const FLUID_VOICE_ADD: FluidVoiceAddMod = 1;
const GEN_OVERRIDEROOTKEY: GenType = 58;
const GEN_EXCLUSIVECLASS: GenType = 57;
const GEN_SAMPLEMODE: GenType = 54;
const GEN_ENDLOOPADDRCOARSEOFS: GenType = 50;
const GEN_STARTLOOPADDRCOARSEOFS: GenType = 45;
const GEN_ENDADDRCOARSEOFS: GenType = 12;
const GEN_STARTADDRCOARSEOFS: GenType = 4;
const GEN_ENDLOOPADDROFS: GenType = 3;
const GEN_STARTLOOPADDROFS: GenType = 2;
const GEN_ENDADDROFS: GenType = 1;
const GEN_STARTADDROFS: GenType = 0;
const GEN_LAST: GenType = 60;
const FLUID_VOICE_OVERWRITE: FluidVoiceAddMod = 0;
type GenType = u32;
type GenFlags = u32;

const FLUID_OK: i32 = 0;
const FLUID_FAILED: i32 = -1;

#[derive(Clone)]
#[repr(C)]
pub(super) struct DefaultSoundFont {
    filename: PathBuf,
    samplepos: u64,
    samplesize: u32,
    pub(super) sampledata: *mut i16,
    sample: Vec<Rc<Sample>>,
    preset: *mut DefaultPreset,
}

impl DefaultSoundFont {
    fn get_sample(&mut self, name: &str) -> Option<Rc<Sample>> {
        self.sample
            .iter()
            .find(|sample| name == &sample.name)
            .map(|s| s.clone())
    }
}

#[derive(Clone)]
#[repr(C)]
pub(super) struct DefaultPreset {
    next: *mut DefaultPreset,
    sfont: *mut DefaultSoundFont,
    // [u8;21]
    name: String,
    bank: u32,
    num: u32,
    global_zone: *mut PresetZone,
    zone: *mut PresetZone,
}

impl DefaultPreset {
    unsafe fn import_sfont(
        sf2: &sf2::SoundFont2,
        sfpreset: &sf2::Preset,
        sfont: &mut DefaultSoundFont,
    ) -> Result<Self, ()> {
        let mut preset = DefaultPreset {
            next: 0 as *mut DefaultPreset,
            sfont: sfont,
            name: String::new(),
            bank: 0 as i32 as u32,
            num: 0 as i32 as u32,
            global_zone: 0 as *mut PresetZone,
            zone: 0 as *mut PresetZone,
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
            let zone = PresetZone::import_sfont(name, sf2, sfzone, sfont)?;

            let zone = Box::into_raw(Box::new(zone));

            if id == 0 && (*zone).inst.is_null() {
                preset.global_zone = zone;
            } else {
                // fluid_defpreset_add_zone
                if preset.zone.is_null() {
                    (*zone).next = 0 as *mut PresetZone;
                    preset.zone = zone
                } else {
                    (*zone).next = preset.zone;
                    preset.zone = zone
                }
            }
        }

        Ok(preset)
    }
}

#[derive(Clone)]
#[repr(C)]
struct PresetZone {
    next: *mut PresetZone,
    name: String,
    inst: *mut Instrument,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mod_0: *mut Mod,
}

impl PresetZone {
    unsafe fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        sfzone: &sf2::Zone,
        sfont: &mut DefaultSoundFont,
    ) -> Result<Self, ()> {
        let mut zone = Self {
            next: 0 as *mut PresetZone,
            name,
            inst: 0 as *mut Instrument,
            keylo: 0,
            keyhi: 128,
            vello: 0 as i32,
            velhi: 128 as i32,
            gen: gen::get_default_values(),
            mod_0: 0 as *mut Mod,
        };

        for sfgen in sfzone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::SFGeneratorType::Instrument)
        {
            match sfgen.ty {
                sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
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
        if let Some(inst) = sfzone.instrument() {
            zone.inst = new_fluid_inst();
            if zone.inst.is_null() {
                log::error!("Out of memory");
                return Err(());
            }
            if fluid_inst_import_sfont(
                sf2,
                &mut *zone.inst,
                &sf2.instruments[*inst as usize],
                sfont,
            ) != FLUID_OK as i32
            {
                return Err(());
            }
        }
        // Import the modulators (only SF2.1 and higher)
        for (id, mod_src) in sfzone.mod_list.iter().enumerate() {
            let mod_dest = Mod::from(mod_src);
            let mod_dest: *mut Mod = Box::into_raw(Box::new(mod_dest));
            /* Store the new modulator in the zone The order of modulators
             * will make a difference, at least in an instrument context: The
             * second modulator overwrites the first one, if they only differ
             * in amount. */
            if id == 0 {
                zone.mod_0 = mod_dest
            } else {
                let mut last_mod: *mut Mod = zone.mod_0;
                // Find the end of the list
                while !(*last_mod).next.is_null() {
                    last_mod = (*last_mod).next
                }
                (*last_mod).next = mod_dest
            }
        }

        Ok(zone)
    }
}

#[derive(Clone)]
#[repr(C)]
struct Instrument {
    // [u8;21]
    name: String,
    global_zone: *mut InstrumentZone,
    zone: *mut InstrumentZone,
}
#[derive(Clone)]
#[repr(C)]
struct InstrumentZone {
    next: *mut InstrumentZone,
    name: String,
    sample: Option<Rc<Sample>>,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mod_0: *mut Mod,
}

impl InstrumentZone {
    unsafe fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        new_zone: &sf2::Zone,
        sfont: &mut DefaultSoundFont,
    ) -> Result<InstrumentZone, ()> {
        let mut keylo = 0;
        let mut keyhi = 128;
        let mut gen = gen::get_default_values();

        for new_gen in new_zone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::SFGeneratorType::SampleID)
        {
            match new_gen.ty {
                sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
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
            let sample = sfont.get_sample(&sample.name);
            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }
            sample
        } else {
            None
        };

        let mut mod_0 = 0 as *mut Mod;

        for (id, new_mod) in new_zone.mod_list.iter().enumerate() {
            let mod_dest = Mod::from(new_mod);
            let mod_dest = Box::into_raw(Box::new(mod_dest));
            /* Store the new modulator in the zone
             * The order of modulators will make a difference, at least in an instrument context:
             * The second modulator overwrites the first one, if they only differ in amount. */
            if id == 0 {
                mod_0 = mod_dest
            } else {
                let mut last_mod: *mut Mod = mod_0;
                while !(*last_mod).next.is_null() {
                    last_mod = (*last_mod).next
                }
                (*last_mod).next = mod_dest
            }
        }

        Ok(InstrumentZone {
            next: 0 as *mut InstrumentZone,
            name: name,
            sample,
            keylo,
            keyhi,
            vello: 0,
            velhi: 128,
            gen,
            mod_0,
        })
    }
}

impl SoundFont {
    pub(crate) fn load(filename: &Path) -> Result<Self, ()> {
        DefaultSoundFont::load(filename).map(|defsfont| Self {
            data: defsfont,
            id: 0,
        })
    }

    pub fn get_name(&self) -> &Path {
        &self.data.filename
    }

    pub fn get_preset(&self, bank: u32, prenum: u32) -> Option<Preset> {
        unsafe {
            let defpreset = (|| {
                let mut preset: *mut DefaultPreset = self.data.preset;
                while !preset.is_null() {
                    if (*preset).bank == bank && (*preset).num == prenum {
                        return Some(preset);
                    }
                    preset = (*preset).next
                }
                None
            })();

            if let Some(defpreset) = defpreset {
                let preset = Preset {
                    sfont_id: self.id,
                    data: defpreset as *mut _,
                };

                Some(preset)
            } else {
                None
            }
        }
    }
}

impl Drop for SoundFont {
    fn drop(&mut self) {
        let sfont = &mut self.data;
        if !sfont.sampledata.is_null() {
            unsafe {
                libc::free(sfont.sampledata as *mut libc::c_void);
            }
        }

        let mut preset = sfont.preset;
        while !preset.is_null() {
            unsafe {
                sfont.preset = (*preset).next;
                delete_fluid_defpreset(preset);
                preset = sfont.preset
            }
        }
    }
}

impl Preset {
    pub fn get_name(&self) -> String {
        unsafe { (*self.data).name.clone() }
    }

    pub fn get_banknum(&self) -> u32 {
        unsafe { (*self.data).bank }
    }

    pub fn get_num(&self) -> u32 {
        unsafe { (*self.data).num }
    }

    pub fn noteon(&mut self, synth: &mut Synth, chan: u8, key: u8, vel: i32) -> i32 {
        unsafe fn fluid_defpreset_noteon(
            preset: *mut DefaultPreset,
            synth: &mut Synth,
            chan: u8,
            key: u8,
            vel: i32,
        ) -> i32 {
            fn fluid_preset_zone_inside_range(zone: &PresetZone, key: u8, vel: i32) -> bool {
                zone.keylo <= key && zone.keyhi >= key && zone.vello <= vel && zone.velhi >= vel
            }

            fn fluid_inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: i32) -> bool {
                zone.keylo <= key && zone.keyhi >= key && zone.vello <= vel && zone.velhi >= vel
            }

            fn fluid_sample_in_rom(sample: &Sample) -> i32 {
                // sampletype & FLUID_SAMPLETYPE_ROM
                sample.sampletype & 0x8000
            }

            let mut mod_list: [*mut Mod; 64] = [0 as *mut Mod; 64]; // list for 'sorting' preset modulators

            let global_preset_zone = if (*preset).global_zone.is_null() {
                None
            } else {
                Some(&mut *(*preset).global_zone)
            };

            // run thru all the zones of this preset
            let mut preset_zone = (*preset).zone;
            while !preset_zone.is_null() {
                // check if the note falls into the key and velocity range of this preset
                if fluid_preset_zone_inside_range(&*preset_zone, key, vel) {
                    let inst = &*(*preset_zone).inst;

                    let global_inst_zone = if inst.global_zone.is_null() {
                        None
                    } else {
                        Some(&mut *inst.global_zone)
                    };

                    // run thru all the zones of this instrument
                    let mut inst_zone_raw = inst.zone;
                    while !inst_zone_raw.is_null() {
                        let inst_zone = &mut (*inst_zone_raw);

                        // make sure this instrument zone has a valid sample
                        let sample = &inst_zone.sample;
                        if sample.is_none() || fluid_sample_in_rom(&sample.as_ref().unwrap()) != 0 {
                            inst_zone_raw = inst_zone.next;
                        } else {
                            // check if the note falls into the key and velocity range of this instrument
                            if fluid_inst_zone_inside_range(inst_zone, key, vel)
                                && !sample.is_none()
                            {
                                // this is a good zone. allocate a new synthesis process and initialize it
                                let voice_id = synth.alloc_voice(
                                    sample.as_ref().unwrap().clone(),
                                    chan,
                                    key,
                                    vel,
                                );

                                if let Some(voice_id) = voice_id {
                                    // Instrument level, generators
                                    let mut i = 0;
                                    while i < GEN_LAST as i32 {
                                        /* SF 2.01 section 9.4 'bullet' 4:
                                         *
                                         * A generator in a local instrument zone supersedes a
                                         * global instrument zone generator.  Both cases supersede
                                         * the default generator -> voice_gen_set */
                                        if inst_zone.gen[i as usize].flags != 0 {
                                            synth.voices[voice_id.0]
                                                .gen_set(i, inst_zone.gen[i as usize].val);
                                        } else if let Some(global_inst_zone) = &global_inst_zone {
                                            if global_inst_zone.gen[i as usize].flags as i32 != 0 {
                                                synth.voices[voice_id.0].gen_set(
                                                    i,
                                                    global_inst_zone.gen[i as usize].val,
                                                );
                                            }
                                        } else {
                                            /* The generator has not been defined in this instrument.
                                             * Do nothing, leave it at the default.
                                             */
                                        }
                                        i += 1
                                    }

                                    /* global instrument zone, modulators: Put them all into a
                                     * list. */
                                    let mut mod_list_count = 0;
                                    if let Some(global_inst_zone) = &global_inst_zone {
                                        let mut mod_0 = global_inst_zone.mod_0;
                                        while !mod_0.is_null() {
                                            mod_list[mod_list_count] = mod_0;
                                            mod_0 = (*mod_0).next;

                                            mod_list_count += 1;
                                        }
                                    }

                                    /* local instrument zone, modulators.
                                     * Replace modulators with the same definition in the list:
                                     * SF 2.01 page 69, 'bullet' 8
                                     */
                                    let mut mod_0 = inst_zone.mod_0;
                                    while !mod_0.is_null() {
                                        /* 'Identical' modulators will be deleted by setting their
                                         *  list entry to NULL.  The list length is known, NULL
                                         *  entries will be ignored later.  SF2.01 section 9.5.1
                                         *  page 69, 'bullet' 3 defines 'identical'.  */
                                        let mut i = 0;
                                        while i < mod_list_count {
                                            if !mod_list[i].is_null()
                                                && mod_0.as_ref().unwrap().test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                ) != 0
                                            {
                                                mod_list[i] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }

                                        /* Finally add the new modulator to to the list. */
                                        mod_list[mod_list_count] = mod_0;
                                        mod_0 = (*mod_0).next;

                                        mod_list_count += 1;
                                    }

                                    // Add instrument modulators (global / local) to the voice.
                                    let mut i = 0;
                                    while i < mod_list_count {
                                        let mod_0 = mod_list[i as usize];
                                        if !mod_0.is_null() {
                                            // disabled modulators CANNOT be skipped.

                                            /* Instrument modulators -supersede- existing (default)
                                             * modulators.  SF 2.01 page 69, 'bullet' 6 */
                                            synth.voices[voice_id.0].add_mod(
                                                mod_0.as_ref().unwrap(),
                                                FLUID_VOICE_OVERWRITE as i32,
                                            );
                                        }
                                        i += 1
                                    }

                                    /* Preset level, generators */
                                    let mut i = 0;
                                    while i < GEN_LAST {
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
                                            if (*preset_zone).gen[i as usize].flags != 0 {
                                                synth.voices[voice_id.0].gen_incr(
                                                    i,
                                                    (*preset_zone).gen[i as usize].val,
                                                );
                                            } else if let Some(global_preset_zone) =
                                                &global_preset_zone
                                            {
                                                if global_preset_zone.gen[i as usize].flags != 0 {
                                                    synth.voices[voice_id.0].gen_incr(
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
                                        i += 1
                                    } /* for all generators */

                                    /* Global preset zone, modulators: put them all into a
                                     * list. */
                                    let mut mod_list_count = 0;
                                    if let Some(global_preset_zone) = &global_preset_zone {
                                        mod_0 = global_preset_zone.mod_0;
                                        while !mod_0.is_null() {
                                            mod_list[mod_list_count] = mod_0;
                                            mod_0 = (*mod_0).next;

                                            mod_list_count += 1;
                                        }
                                    }

                                    /* Process the modulators of the local preset zone.  Kick
                                     * out all identical modulators from the global preset zone
                                     * (SF 2.01 page 69, second-last bullet) */
                                    let mut mod_0 = (*preset_zone).mod_0;
                                    while !mod_0.is_null() {
                                        let mut i = 0;
                                        while i < mod_list_count {
                                            if !mod_list[i].is_null()
                                                && mod_0.as_ref().unwrap().test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                ) != 0
                                            {
                                                mod_list[i] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }

                                        /* Finally add the new modulator to the list. */
                                        mod_list[mod_list_count] = mod_0;
                                        mod_0 = (*mod_0).next;

                                        mod_list_count += 1;
                                    }

                                    // Add preset modulators (global / local) to the voice.
                                    let mut i = 0;
                                    while i < mod_list_count {
                                        mod_0 = mod_list[i];
                                        if !mod_0.is_null() && (*mod_0).amount != 0 as i32 as f64 {
                                            // disabled modulators can be skipped.

                                            /* Preset modulators -add- to existing instrument /
                                             * default modulators.  SF2.01 page 70 first bullet on
                                             * page */
                                            synth.voices[voice_id.0].add_mod(
                                                mod_0.as_ref().unwrap(),
                                                FLUID_VOICE_ADD as i32,
                                            );
                                        }
                                        i += 1
                                    }

                                    // add the synthesis process to the synthesis loop.
                                    synth.start_voice(voice_id);

                                    /* Store the ID of the first voice that was created by this noteon event.
                                     * Exclusive class may only terminate older voices.
                                     * That avoids killing voices, which have just been created.
                                     * (a noteon event can create several voice processes with the same exclusive
                                     * class - for example when using stereo samples)
                                     */
                                } else {
                                    return FLUID_FAILED as i32;
                                }
                            }
                            inst_zone_raw = inst_zone.next;
                        }
                    }
                }
                preset_zone = (*preset_zone).next
            }
            return FLUID_OK as i32;
        }

        unsafe { fluid_defpreset_noteon(self.data, synth, chan, key, vel) }
    }
}

impl DefaultSoundFont {
    fn load(path: &Path) -> Result<Self, ()> {
        let filename = path.to_owned();
        let mut file = std::fs::File::open(&filename).unwrap();

        let data = sf2::data::SFData::load(&mut file);
        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let samplepos = smpl.offset() + 8;
        let samplesize = smpl.len();

        let sampledata = unsafe { Self::load_sampledata(&mut file, samplepos, samplesize)? };

        let mut defsfont = DefaultSoundFont {
            filename,
            samplepos,
            samplesize,
            sample: Vec::new(),
            sampledata,
            preset: std::ptr::null_mut(),
        };

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import_sfont(sfsample, &mut defsfont)?;
            let mut sample = sample;

            unsafe {
                sample.optimize_sample();
            }

            defsfont.sample.push(Rc::new(sample));
        }

        for sfpreset in sf2.presets.iter() {
            let preset = unsafe { DefaultPreset::import_sfont(&sf2, sfpreset, &mut defsfont)? };
            let preset = Box::into_raw(Box::new(preset));

            unsafe {
                defsfont.add_preset(&mut *preset);
            }
        }

        Ok(defsfont)
    }

    unsafe fn load_sampledata(
        file: &mut std::fs::File,
        samplepos: u64,
        samplesize: u32,
    ) -> Result<*mut i16, ()> {
        if file.seek(SeekFrom::Start(samplepos)).is_err() {
            libc::perror(b"error\x00" as *const u8 as *const i8);
            log::error!("Failed to seek position in data file",);
            return Err(());
        }
        let sampledata = libc::malloc(samplesize as libc::size_t) as *mut i16;
        if sampledata.is_null() {
            log::error!("Out of memory",);
            return Err(());
        }

        if file
            .read(from_raw_parts_mut(sampledata as _, samplesize as _))
            .is_err()
        {
            log::error!("Failed to read sample data",);
            return Err(());
        }
        let mut endian = 0x100 as u16;
        if *(&mut endian as *mut u16 as *mut i8).offset(0 as i32 as isize) != 0 {
            let cbuf: *mut u8;
            let mut hi: u8;
            let mut lo: u8;
            let mut i: u32;
            let mut j: u32;
            let mut s: i16;
            cbuf = sampledata as *mut u8;
            i = 0 as i32 as u32;
            j = 0 as i32 as u32;
            while j < samplesize {
                let fresh0 = j;
                j = j.wrapping_add(1);
                lo = *cbuf.offset(fresh0 as isize);
                let fresh1 = j;
                j = j.wrapping_add(1);
                hi = *cbuf.offset(fresh1 as isize);
                s = ((hi as i32) << 8 as i32 | lo as i32) as i16;
                *sampledata.offset(i as isize) = s;
                i = i.wrapping_add(1)
            }
        }
        Ok(sampledata)
    }

    unsafe fn add_preset(&mut self, mut preset: &mut DefaultPreset) -> i32 {
        let mut cur: *mut DefaultPreset;
        let mut prev: *mut DefaultPreset;
        if self.preset.is_null() {
            preset.next = 0 as *mut DefaultPreset;
            self.preset = preset
        } else {
            cur = self.preset;
            prev = 0 as *mut DefaultPreset;
            while !cur.is_null() {
                if preset.bank < (*cur).bank
                    || preset.bank == (*cur).bank && preset.num < (*cur).num
                {
                    if prev.is_null() {
                        preset.next = cur;
                        self.preset = preset
                    } else {
                        preset.next = cur;
                        (*prev).next = preset
                    }
                    return FLUID_OK as i32;
                }
                prev = cur;
                cur = (*cur).next
            }
            preset.next = 0 as *mut DefaultPreset;
            (*prev).next = preset
        }
        return FLUID_OK as i32;
    }
}

unsafe fn delete_fluid_defpreset(mut preset: *mut DefaultPreset) -> i32 {
    let mut err: i32 = FLUID_OK as i32;
    let mut zone: *mut PresetZone;
    if !(*preset).global_zone.is_null() {
        if delete_fluid_preset_zone((*preset).global_zone) != FLUID_OK as i32 {
            err = FLUID_FAILED as i32
        }
        (*preset).global_zone = 0 as *mut PresetZone
    }
    zone = (*preset).zone;
    while !zone.is_null() {
        (*preset).zone = (*zone).next;
        if delete_fluid_preset_zone(zone) != FLUID_OK as i32 {
            err = FLUID_FAILED as i32
        }
        zone = (*preset).zone
    }
    libc::free(preset as *mut libc::c_void);
    return err;
}

unsafe fn delete_fluid_preset_zone(zone: *mut PresetZone) -> i32 {
    let mut mod_0: *mut Mod;
    let mut tmp: *mut Mod;
    mod_0 = (*zone).mod_0;
    while !mod_0.is_null() {
        tmp = mod_0;
        mod_0 = (*mod_0).next;

        Box::from_raw(tmp);
    }
    if !(*zone).inst.is_null() {
        delete_fluid_inst((*zone).inst);
    }
    libc::free(zone as *mut libc::c_void);
    return FLUID_OK as i32;
}

unsafe fn new_fluid_inst() -> *mut Instrument {
    let mut inst: *mut Instrument =
        libc::malloc(::std::mem::size_of::<Instrument>() as libc::size_t) as *mut Instrument;
    if inst.is_null() {
        log::error!("Out of memory",);
        return 0 as *mut Instrument;
    }
    (*inst).name = String::new();
    (*inst).global_zone = 0 as *mut InstrumentZone;
    (*inst).zone = 0 as *mut InstrumentZone;
    return inst;
}

unsafe fn delete_fluid_inst(mut inst: *mut Instrument) -> i32 {
    let mut zone: *mut InstrumentZone;
    let mut err: i32 = FLUID_OK as i32;
    if !(*inst).global_zone.is_null() {
        if delete_fluid_inst_zone((*inst).global_zone) != FLUID_OK as i32 {
            err = FLUID_FAILED as i32
        }
        (*inst).global_zone = 0 as *mut InstrumentZone
    }
    zone = (*inst).zone;
    while !zone.is_null() {
        (*inst).zone = (*zone).next;
        if delete_fluid_inst_zone(zone) != FLUID_OK as i32 {
            err = FLUID_FAILED as i32
        }
        zone = (*inst).zone
    }
    libc::free(inst as *mut libc::c_void);
    return err;
}

unsafe fn fluid_inst_import_sfont(
    sf2: &sf2::SoundFont2,
    inst: &mut Instrument,
    new_inst: &sf2::Instrument,
    sfont: &mut DefaultSoundFont,
) -> i32 {
    let mut count: i32;

    if new_inst.header.name.len() > 0 {
        inst.name = new_inst.header.name.clone();
    } else {
        inst.name = "<untitled>".into();
    }

    count = 0 as i32;
    for new_zone in new_inst.zones.iter() {
        let name = format!("{}/{}", new_inst.header.name, count);

        let mut zone = InstrumentZone::import_sfont(name, sf2, new_zone, &mut *sfont).unwrap();

        if count == 0 as i32 && zone.sample.is_none() {
            inst.global_zone = Box::into_raw(Box::new(zone));
        } else {
            // fluid_inst_add_zone
            if inst.zone.is_null() {
                zone.next = 0 as *mut InstrumentZone;
                inst.zone = Box::into_raw(Box::new(zone));
            } else {
                zone.next = (*inst).zone;
                inst.zone = Box::into_raw(Box::new(zone));
            }
        }
        count += 1
    }
    return FLUID_OK as i32;
}

unsafe fn delete_fluid_inst_zone(zone: *mut InstrumentZone) -> i32 {
    let mut mod_0: *mut Mod;
    let mut tmp: *mut Mod;
    mod_0 = (*zone).mod_0;
    while !mod_0.is_null() {
        tmp = mod_0;
        mod_0 = (*mod_0).next;

        Box::from_raw(tmp);
    }
    libc::free(zone as *mut libc::c_void);
    return FLUID_OK as i32;
}
