use soundfont_rs as sf2;

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::gen::{self, Gen};
use crate::modulator::Mod;
use crate::soundfont::Preset;
use crate::soundfont::Sample;
use crate::soundfont::SoundFont;
use crate::synth::Synth;
use crate::voice::fluid_voice_add_mod;
use crate::voice::fluid_voice_gen_incr;
use crate::voice::fluid_voice_gen_set;
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
    sample: Vec<Sample>,
    preset: *mut DefaultPreset,
}

impl DefaultSoundFont {
    fn get_sample(&mut self, name: &str) -> Option<&mut Sample> {
        self.sample.iter_mut().find(|sample| name == &sample.name)
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
            let mut zone = Box::into_raw(Box::new(PresetZone::new(format!(
                "{}/{}",
                sfpreset.header.name, id
            ))));

            fluid_preset_zone_import_sfont(sf2, &mut *zone, sfzone, sfont)?;

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
    fn new(name: String) -> PresetZone {
        PresetZone {
            next: 0 as *mut PresetZone,
            name,
            inst: 0 as *mut Instrument,
            keylo: 0,
            keyhi: 128,
            vello: 0 as i32,
            velhi: 128 as i32,
            gen: gen::get_default_values(),
            mod_0: 0 as *mut Mod,
        }
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
    sample: *mut Sample,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mod_0: *mut Mod,
}

impl InstrumentZone {
    fn new(name: String) -> InstrumentZone {
        InstrumentZone {
            next: 0 as *mut InstrumentZone,
            name: name,
            sample: 0 as *mut Sample,
            keylo: 0,
            keyhi: 128,
            vello: 0,
            velhi: 128,
            gen: gen::get_default_values(),
            mod_0: 0 as *mut Mod,
        }
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
        for sample in sfont.sample.iter() {
            if sample.refcount != 0 as i32 as u32 {
                return;
            }
        }
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

            unsafe fn fluid_inst_zone_inside_range(
                zone: *mut InstrumentZone,
                key: u8,
                vel: i32,
            ) -> bool {
                (*zone).keylo <= key
                    && (*zone).keyhi >= key
                    && (*zone).vello <= vel
                    && (*zone).velhi >= vel
            }

            unsafe fn fluid_sample_in_rom(sample: *mut Sample) -> i32 {
                return (*sample).sampletype & 0x8000 as i32;
            }

            let mut inst: *mut Instrument;
            let mut inst_zone: *mut InstrumentZone;
            let mut global_inst_zone: *mut InstrumentZone;
            let mut sample: *mut Sample;

            let mut mod_0: *mut Mod;
            let mut mod_list: [*mut Mod; 64] = [0 as *mut Mod; 64];
            let mut mod_list_count: i32;
            let mut i: i32;

            let global_preset_zone = (*preset).global_zone;
            let mut preset_zone = (*preset).zone;

            while !preset_zone.is_null() {
                if fluid_preset_zone_inside_range(&*preset_zone, key, vel) {
                    inst = (*preset_zone).inst;
                    global_inst_zone = (*inst).global_zone;
                    inst_zone = (*inst).zone;
                    while !inst_zone.is_null() {
                        sample = (*inst_zone).sample;
                        if fluid_sample_in_rom(sample) != 0 || sample.is_null() {
                            inst_zone = (*inst_zone).next;
                        } else {
                            if fluid_inst_zone_inside_range(inst_zone, key, vel)
                                && !sample.is_null()
                            {
                                let voice_id = synth.alloc_voice(sample, chan, key, vel);

                                if let Some(voice_id) = voice_id {
                                    i = 0 as i32;
                                    while i < GEN_LAST as i32 {
                                        if (*inst_zone).gen[i as usize].flags != 0 {
                                            fluid_voice_gen_set(
                                                &mut synth.voices[voice_id.0],
                                                i,
                                                (*inst_zone).gen[i as usize].val as f32,
                                            );
                                        } else if !global_inst_zone.is_null()
                                            && (*global_inst_zone).gen[i as usize].flags as i32 != 0
                                        {
                                            fluid_voice_gen_set(
                                                &mut synth.voices[voice_id.0],
                                                i,
                                                (*global_inst_zone).gen[i as usize].val as f32,
                                            );
                                        }
                                        i += 1
                                    }
                                    mod_list_count = 0 as i32;
                                    if !global_inst_zone.is_null() {
                                        mod_0 = (*global_inst_zone).mod_0;
                                        while !mod_0.is_null() {
                                            let fresh2 = mod_list_count;
                                            mod_list_count = mod_list_count + 1;
                                            mod_list[fresh2 as usize] = mod_0;
                                            mod_0 = (*mod_0).next
                                        }
                                    }
                                    mod_0 = (*inst_zone).mod_0;
                                    while !mod_0.is_null() {
                                        i = 0 as i32;
                                        while i < mod_list_count {
                                            if !mod_list[i as usize].is_null()
                                                && mod_0.as_ref().unwrap().test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                ) != 0
                                            {
                                                mod_list[i as usize] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }
                                        let fresh3 = mod_list_count;
                                        mod_list_count = mod_list_count + 1;
                                        mod_list[fresh3 as usize] = mod_0;
                                        mod_0 = (*mod_0).next
                                    }
                                    i = 0 as i32;
                                    while i < mod_list_count {
                                        mod_0 = mod_list[i as usize];
                                        if !mod_0.is_null() {
                                            fluid_voice_add_mod(
                                                &mut synth.voices[voice_id.0],
                                                mod_0.as_ref().unwrap(),
                                                FLUID_VOICE_OVERWRITE as i32,
                                            );
                                        }
                                        i += 1
                                    }
                                    i = 0 as i32;
                                    while i < GEN_LAST as i32 {
                                        if i != GEN_STARTADDROFS as i32
                                            && i != GEN_ENDADDROFS as i32
                                            && i != GEN_STARTLOOPADDROFS as i32
                                            && i != GEN_ENDLOOPADDROFS as i32
                                            && i != GEN_STARTADDRCOARSEOFS as i32
                                            && i != GEN_ENDADDRCOARSEOFS as i32
                                            && i != GEN_STARTLOOPADDRCOARSEOFS as i32
                                            && i != GEN_KEYNUM as i32
                                            && i != GEN_VELOCITY as i32
                                            && i != GEN_ENDLOOPADDRCOARSEOFS as i32
                                            && i != GEN_SAMPLEMODE as i32
                                            && i != GEN_EXCLUSIVECLASS as i32
                                            && i != GEN_OVERRIDEROOTKEY as i32
                                        {
                                            if (*preset_zone).gen[i as usize].flags != 0 {
                                                fluid_voice_gen_incr(
                                                    &mut synth.voices[voice_id.0],
                                                    i,
                                                    (*preset_zone).gen[i as usize].val as f32,
                                                );
                                            } else if !global_preset_zone.is_null()
                                                && (*global_preset_zone).gen[i as usize].flags
                                                    as i32
                                                    != 0
                                            {
                                                fluid_voice_gen_incr(
                                                    &mut synth.voices[voice_id.0],
                                                    i,
                                                    (*global_preset_zone).gen[i as usize].val
                                                        as f32,
                                                );
                                            }
                                        }
                                        i += 1
                                    }
                                    mod_list_count = 0 as i32;
                                    if !global_preset_zone.is_null() {
                                        mod_0 = (*global_preset_zone).mod_0;
                                        while !mod_0.is_null() {
                                            let fresh4 = mod_list_count;
                                            mod_list_count = mod_list_count + 1;
                                            mod_list[fresh4 as usize] = mod_0;
                                            mod_0 = (*mod_0).next
                                        }
                                    }
                                    mod_0 = (*preset_zone).mod_0;
                                    while !mod_0.is_null() {
                                        i = 0 as i32;
                                        while i < mod_list_count {
                                            if !mod_list[i as usize].is_null()
                                                && mod_0.as_ref().unwrap().test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                ) != 0
                                            {
                                                mod_list[i as usize] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }
                                        let fresh5 = mod_list_count;
                                        mod_list_count = mod_list_count + 1;
                                        mod_list[fresh5 as usize] = mod_0;
                                        mod_0 = (*mod_0).next
                                    }
                                    i = 0 as i32;
                                    while i < mod_list_count {
                                        mod_0 = mod_list[i as usize];
                                        if !mod_0.is_null() && (*mod_0).amount != 0 as i32 as f64 {
                                            fluid_voice_add_mod(
                                                &mut synth.voices[voice_id.0],
                                                mod_0.as_ref().unwrap(),
                                                FLUID_VOICE_ADD as i32,
                                            );
                                        }
                                        i += 1
                                    }
                                    synth.start_voice(voice_id);
                                } else {
                                    return FLUID_FAILED as i32;
                                }
                            }
                            inst_zone = (*inst_zone).next;
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

            defsfont.sample.push(sample);
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

unsafe fn fluid_preset_zone_import_sfont(
    sf2: &sf2::SoundFont2,
    zone: &mut PresetZone,
    sfzone: &sf2::Zone,
    sfont: &mut DefaultSoundFont,
) -> Result<(), ()> {
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
            let mut last_mod: *mut Mod = (*zone).mod_0;
            // Find the end of the list
            while !(*last_mod).next.is_null() {
                last_mod = (*last_mod).next
            }
            (*last_mod).next = mod_dest
        }
    }

    Ok(())
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
        let mut zone = InstrumentZone::new(format!("{}/{}", new_inst.header.name, count));

        if fluid_inst_zone_import_sfont(sf2, &mut zone, new_zone, &mut *sfont) != FLUID_OK as i32 {
            return FLUID_FAILED as i32;
        }
        if count == 0 as i32 && zone.sample.is_null() {
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

unsafe fn fluid_inst_zone_import_sfont(
    sf2: &sf2::SoundFont2,
    zone: &mut InstrumentZone,
    new_zone: &sf2::Zone,
    sfont: &mut DefaultSoundFont,
) -> i32 {
    let mut count: i32;
    count = 0 as i32;

    for new_gen in new_zone
        .gen_list
        .iter()
        .filter(|g| g.ty != sf2::data::SFGeneratorType::SampleID)
    {
        match new_gen.ty {
            sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
                let amount = new_gen.amount.as_range().unwrap();
                zone.keylo = amount.low;
                zone.keyhi = amount.high;
            }
            _ => {
                zone.gen[new_gen.ty as usize].val = *new_gen.amount.as_i16().unwrap() as f64;
                zone.gen[new_gen.ty as usize].flags = GEN_SET as u8;
            }
        }

        count += 1
    }
    if let Some(sample_id) = new_zone.sample() {
        let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();

        zone.sample = sfont.get_sample(&sample.name).unwrap() as *mut _;

        if zone.sample.is_null() {
            log::error!("Couldn't find sample name",);
            return FLUID_FAILED as i32;
        }
    }
    count = 0 as i32;
    for new_mod in new_zone.mod_list.iter() {
        let mod_dest = Mod::from(new_mod);
        let mod_dest = Box::into_raw(Box::new(mod_dest));

        /* Store the new modulator in the zone
         * The order of modulators will make a difference, at least in an instrument context:
         * The second modulator overwrites the first one, if they only differ in amount. */
        if count == 0 as i32 {
            zone.mod_0 = mod_dest
        } else {
            let mut last_mod: *mut Mod = (*zone).mod_0;
            while !(*last_mod).next.is_null() {
                last_mod = (*last_mod).next
            }
            (*last_mod).next = mod_dest
        }
        count += 1
    }
    return FLUID_OK as i32;
}
