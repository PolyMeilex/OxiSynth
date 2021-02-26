use std::io::{Read, Seek, SeekFrom};

use super::gen::fluid_gen_set_default_values;
use super::gen::Gen;
use super::modulator::Mod;
use super::soundfont::Preset;
use super::soundfont::Sample;
use super::soundfont::SoundFont;
use super::soundfont::SoundFontLoader;
use super::synth::Synth;
use super::voice::fluid_voice_add_mod;
use super::voice::fluid_voice_gen_incr;
use super::voice::fluid_voice_gen_set;
use super::voice::fluid_voice_optimize_sample;
use super::voice::FluidVoiceAddMod;
use std::slice::from_raw_parts_mut;
use std::{
    ffi::{CStr, CString},
    path::Path,
};
const FLUID_OK: i32 = 0;
const FLUID_FAILED: i32 = -1;
#[derive(Clone)]
#[repr(C)]
pub struct DefaultSoundFont {
    pub filename: String,
    pub samplepos: u32,
    pub samplesize: u32,
    pub sampledata: *mut i16,
    pub sample: Vec<Sample>,
    pub preset: *mut DefaultPreset,
}

impl DefaultSoundFont {
    fn get_sample(&mut self, s: &[u8]) -> Option<&mut Sample> {
        for sample in self.sample.iter_mut() {
            let name_a = CString::new(sample.name.clone()).unwrap();
            let name_b = unsafe { CStr::from_ptr(s.as_ptr() as _) };

            if name_a.as_c_str() == name_b {
                return Some(sample);
            }
        }
        return None;
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct DefaultPreset {
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

        let mut count = 0 as i32;
        for sfzone in sfpreset.zones.iter() {
            let mut zone_name: [u8; 256] = [0; 256];

            libc::strcpy(
                zone_name.as_mut_ptr() as _,
                CString::new(format!("{}/{}", sfpreset.header.name, count))
                    .unwrap()
                    .as_c_str()
                    .as_ptr(),
            );
            let zone = new_fluid_preset_zone(&zone_name);
            if zone.is_null() {
                return Err(());
            }
            if fluid_preset_zone_import_sfont(sf2, zone, sfzone, sfont) != FLUID_OK as i32 {
                return Err(());
            }
            if count == 0 as i32 && fluid_preset_zone_get_inst(zone).is_null() {
                fluid_defpreset_set_global_zone(&mut preset, zone);
            } else if fluid_defpreset_add_zone(&mut preset, zone) != FLUID_OK as i32 {
                return Err(());
            }
            count += 1
        }
        return Ok(preset);
    }
}

#[derive(Clone)]
#[repr(C)]
struct PresetZone {
    next: *mut PresetZone,
    name: Vec<u8>,
    inst: *mut Instrument,
    keylo: i32,
    keyhi: i32,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mod_0: *mut Mod,
}
#[derive(Copy, Clone)]
#[repr(C)]
struct Instrument {
    name: [u8; 21],
    global_zone: *mut InstrumentZone,
    zone: *mut InstrumentZone,
}
#[derive(Clone)]
#[repr(C)]
struct InstrumentZone {
    next: *mut InstrumentZone,
    name: Vec<u8>,
    sample: *mut Sample,
    keylo: i32,
    keyhi: i32,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mod_0: *mut Mod,
}

const FLUID_MOD_SWITCH: ModFlags = 12;
const FLUID_MOD_CONVEX: ModFlags = 8;
const FLUID_MOD_CONCAVE: ModFlags = 4;
const FLUID_MOD_LINEAR: ModFlags = 0;
const FLUID_MOD_UNIPOLAR: ModFlags = 0;
const FLUID_MOD_BIPOLAR: ModFlags = 2;
const FLUID_MOD_POSITIVE: ModFlags = 0;
const FLUID_MOD_NEGATIVE: ModFlags = 1;
const FLUID_MOD_GC: ModFlags = 0;
const FLUID_MOD_CC: ModFlags = 16;
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
type ModFlags = u32;
type GenType = u32;
type GenFlags = u32;

impl SoundFontLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub unsafe fn load(&mut self, filename: String) -> Option<SoundFont> {
        let mut defsfont = DefaultSoundFont {
            filename: String::new(),
            samplepos: 0 as _,
            samplesize: 0 as _,
            sample: Vec::new(),
            sampledata: 0 as _,
            preset: 0 as _,
        };

        defsfont.load(filename).ok().map(|_| SoundFont {
            data: defsfont,
            id: 0 as _,
        })
    }
}

impl SoundFont {
    pub fn get_name(&self) -> String {
        self.data.filename.clone()
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
                    sfont: self,
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
        unsafe fn delete_fluid_defsfont(sfont: &mut DefaultSoundFont) -> i32 {
            let mut preset: *mut DefaultPreset;
            for sample in (*sfont).sample.iter() {
                if sample.refcount != 0 as i32 as u32 {
                    return -(1 as i32);
                }
            }
            if !sfont.sampledata.is_null() {
                libc::free(sfont.sampledata as *mut libc::c_void);
            }
            preset = (*sfont).preset;
            while !preset.is_null() {
                (*sfont).preset = (*preset).next;
                delete_fluid_defpreset(preset);
                preset = sfont.preset
            }
            return FLUID_OK as i32;
        }
        unsafe {
            delete_fluid_defsfont(&mut self.data);
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

    pub fn noteon(&mut self, synth: &mut Synth, chan: i32, key: i32, vel: i32) -> i32 {
        unsafe fn fluid_defpreset_noteon(
            preset: *mut DefaultPreset,
            synth: &mut Synth,
            chan: i32,
            key: i32,
            vel: i32,
        ) -> i32 {
            let mut preset_zone: *mut PresetZone;
            let global_preset_zone: *mut PresetZone;
            let mut inst: *mut Instrument;
            let mut inst_zone: *mut InstrumentZone;
            let mut global_inst_zone: *mut InstrumentZone;
            let mut sample: *mut Sample;
            // let mut voice: *mut Voice;
            let mut mod_0: *mut Mod;
            let mut mod_list: [*mut Mod; 64] = [0 as *mut Mod; 64];
            let mut mod_list_count: i32;
            let mut i: i32;
            global_preset_zone = fluid_defpreset_get_global_zone(preset);
            preset_zone = fluid_defpreset_get_zone(preset);
            while !preset_zone.is_null() {
                if fluid_preset_zone_inside_range(preset_zone, key, vel) != 0 {
                    inst = fluid_preset_zone_get_inst(preset_zone);
                    global_inst_zone = fluid_inst_get_global_zone(inst);
                    inst_zone = fluid_inst_get_zone(inst);
                    while !inst_zone.is_null() {
                        sample = fluid_inst_zone_get_sample(inst_zone);
                        if fluid_sample_in_rom(sample) != 0 || sample.is_null() {
                            inst_zone = fluid_inst_zone_next(inst_zone)
                        } else {
                            if fluid_inst_zone_inside_range(inst_zone, key, vel) != 0
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
                            inst_zone = fluid_inst_zone_next(inst_zone)
                        }
                    }
                }
                preset_zone = fluid_preset_zone_next(preset_zone)
            }
            return FLUID_OK as i32;
        }

        unsafe { fluid_defpreset_noteon(self.data, synth, chan, key, vel) }
    }
}

static mut PRESET_CALLBACK: Option<unsafe fn(_: u32, _: u32, _: &str) -> ()> = None;

impl DefaultSoundFont {
    unsafe fn load(&mut self, file: String) -> Result<(), ()> {
        unsafe fn fluid_defsfont_add_preset(
            mut sfont: &mut DefaultSoundFont,
            mut preset: &mut DefaultPreset,
        ) -> i32 {
            let mut cur: *mut DefaultPreset;
            let mut prev: *mut DefaultPreset;
            if sfont.preset.is_null() {
                preset.next = 0 as *mut DefaultPreset;
                sfont.preset = preset
            } else {
                cur = sfont.preset;
                prev = 0 as *mut DefaultPreset;
                while !cur.is_null() {
                    if preset.bank < (*cur).bank
                        || preset.bank == (*cur).bank && preset.num < (*cur).num
                    {
                        if prev.is_null() {
                            preset.next = cur;
                            sfont.preset = preset
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

        unsafe fn fluid_defsfont_load_sampledata(sfont: &mut DefaultSoundFont) -> i32 {
            let mut endian: u16;

            let file = std::fs::File::open(Path::new(&sfont.filename));

            let mut file = match file {
                Err(err) => {
                    log::error!("Can't open soundfont file: {:?}", err);
                    return FLUID_FAILED as i32;
                }
                Ok(file) => file,
            };
            if file.seek(SeekFrom::Start(sfont.samplepos as _)).is_err() {
                libc::perror(b"error\x00" as *const u8 as *const i8);
                log::error!("Failed to seek position in data file",);
                return FLUID_FAILED as i32;
            }
            sfont.sampledata = libc::malloc(sfont.samplesize as libc::size_t) as *mut i16;
            if sfont.sampledata.is_null() {
                log::error!("Out of memory",);
                return FLUID_FAILED as i32;
            }

            if file
                .read(from_raw_parts_mut(
                    sfont.sampledata as _,
                    sfont.samplesize as _,
                ))
                .is_err()
            {
                log::error!("Failed to read sample data",);
                return FLUID_FAILED as i32;
            }
            endian = 0x100 as i32 as u16;
            if *(&mut endian as *mut u16 as *mut i8).offset(0 as i32 as isize) != 0 {
                let cbuf: *mut u8;
                let mut hi: u8;
                let mut lo: u8;
                let mut i: u32;
                let mut j: u32;
                let mut s: i16;
                cbuf = sfont.sampledata as *mut u8;
                i = 0 as i32 as u32;
                j = 0 as i32 as u32;
                while j < sfont.samplesize {
                    let fresh0 = j;
                    j = j.wrapping_add(1);
                    lo = *cbuf.offset(fresh0 as isize);
                    let fresh1 = j;
                    j = j.wrapping_add(1);
                    hi = *cbuf.offset(fresh1 as isize);
                    s = ((hi as i32) << 8 as i32 | lo as i32) as i16;
                    *sfont.sampledata.offset(i as isize) = s;
                    i = i.wrapping_add(1)
                }
            }
            return FLUID_OK as i32;
        }

        self.filename = file.clone();

        let mut file = std::fs::File::open("./testdata/test.sf2").unwrap();

        let data = sf2::data::SFData::load(&mut file);
        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        self.samplepos = smpl.offset() as u32 + 8;
        self.samplesize = smpl.len();
        if fluid_defsfont_load_sampledata(self) != FLUID_OK {
            return Err(());
        }

        for sfsample in sf2.sample_headers.iter() {
            if let Ok(sample) = Sample::import_sfont(sfsample, self) {
                let mut sample = sample;
                fluid_voice_optimize_sample(&mut sample);

                self.sample.push(sample);
            } else {
                return Err(());
            }
        }

        for sfpreset in sf2.presets.iter() {
            if let Ok(preset) = DefaultPreset::import_sfont(&sf2, sfpreset, self) {
                let preset = Box::into_raw(Box::new(preset));

                fluid_defsfont_add_preset(self, &mut *preset);
                if PRESET_CALLBACK.is_some() {
                    PRESET_CALLBACK.expect("non-null function pointer")(
                        (*preset).bank,
                        (*preset).num,
                        &(*preset).name,
                    );
                }
            } else {
                return Err(());
            }
        }
        return Ok(());
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

unsafe fn fluid_defpreset_set_global_zone(
    mut preset: *mut DefaultPreset,
    zone: *mut PresetZone,
) -> i32 {
    (*preset).global_zone = zone;
    return FLUID_OK as i32;
}

unsafe fn fluid_defpreset_add_zone(
    mut preset: *mut DefaultPreset,
    mut zone: *mut PresetZone,
) -> i32 {
    if (*preset).zone.is_null() {
        (*zone).next = 0 as *mut PresetZone;
        (*preset).zone = zone
    } else {
        (*zone).next = (*preset).zone;
        (*preset).zone = zone
    }
    return FLUID_OK as i32;
}

unsafe fn fluid_defpreset_get_zone(preset: *mut DefaultPreset) -> *mut PresetZone {
    return (*preset).zone;
}

unsafe fn fluid_defpreset_get_global_zone(preset: *mut DefaultPreset) -> *mut PresetZone {
    return (*preset).global_zone;
}

unsafe fn fluid_preset_zone_next(preset: *mut PresetZone) -> *mut PresetZone {
    return (*preset).next;
}

unsafe fn new_fluid_preset_zone(name: &[u8]) -> *mut PresetZone {
    let mut zone: *mut PresetZone;
    zone = libc::malloc(::std::mem::size_of::<PresetZone>() as libc::size_t) as *mut PresetZone;
    if zone.is_null() {
        log::error!("Out of memory",);
        return 0 as *mut PresetZone;
    }
    libc::memset(zone as _, 0, std::mem::size_of::<PresetZone>() as _);
    (*zone).next = 0 as *mut PresetZone;
    (*zone).name = name.to_vec();
    (*zone).inst = 0 as *mut Instrument;
    (*zone).keylo = 0 as i32;
    (*zone).keyhi = 128 as i32;
    (*zone).vello = 0 as i32;
    (*zone).velhi = 128 as i32;
    fluid_gen_set_default_values(&mut *(*zone).gen.as_mut_ptr().offset(0 as i32 as isize));
    (*zone).mod_0 = 0 as *mut Mod;
    return zone;
}

unsafe fn delete_fluid_preset_zone(zone: *mut PresetZone) -> i32 {
    let mut mod_0: *mut Mod;
    let mut tmp: *mut Mod;
    mod_0 = (*zone).mod_0;
    while !mod_0.is_null() {
        tmp = mod_0;
        mod_0 = (*mod_0).next;
        tmp.as_mut().unwrap().delete();
    }
    if !(*zone).inst.is_null() {
        delete_fluid_inst((*zone).inst);
    }
    libc::free(zone as *mut libc::c_void);
    return FLUID_OK as i32;
}

unsafe fn fluid_preset_zone_import_sfont(
    sf2: &sf2::SoundFont2,
    mut zone: *mut PresetZone,
    sfzone: &sf2::Zone,
    sfont: &mut DefaultSoundFont,
) -> i32 {
    let mut count: i32;
    count = 0 as i32;
    for sfgen in sfzone
        .gen_list
        .iter()
        .filter(|g| g.ty != sf2::data::SFGeneratorType::Instrument)
    {
        match sfgen.ty {
            sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
                let amount = sfgen.amount.as_range().unwrap();
                (*zone).keylo = amount.low as i32;
                (*zone).keyhi = amount.high as i32
            }
            _ => {
                (*zone).gen[sfgen.ty as usize].val = *sfgen.amount.as_i16().unwrap() as f64;
                (*zone).gen[sfgen.ty as usize].flags = GEN_SET as u8;
            }
        }

        count += 1
    }

    if let Some(inst) = sfzone.instrument() {
        (*zone).inst = new_fluid_inst();
        if (*zone).inst.is_null() {
            log::error!("Out of memory");
            return FLUID_FAILED as i32;
        }
        if fluid_inst_import_sfont(sf2, (*zone).inst, &sf2.instruments[*inst as usize], sfont)
            != FLUID_OK as i32
        {
            return FLUID_FAILED as i32;
        }
    }

    count = 0 as i32;
    for mod_src in sfzone.mod_list.iter() {
        let mut mod_dest: *mut Mod = Mod::new();
        let mut type_0: i32;
        if mod_dest.is_null() {
            return FLUID_FAILED as i32;
        }
        (*mod_dest).next = 0 as *mut Mod;
        (*mod_dest).amount = mod_src.amount as f64;
        (*mod_dest).src1 = (mod_src.src as i32 & 127 as i32) as u8;
        (*mod_dest).flags1 = 0 as i32 as u8;
        if mod_src.src as i32 & (1 as i32) << 7 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CC as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_GC as i32) as u8
        }
        if mod_src.src as i32 & (1 as i32) << 8 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_NEGATIVE as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_POSITIVE as i32) as u8
        }
        if mod_src.src as i32 & (1 as i32) << 9 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_BIPOLAR as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_UNIPOLAR as i32) as u8
        }
        type_0 = mod_src.src as i32 >> 10 as i32;
        type_0 &= 63 as i32;
        if type_0 == 0 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_LINEAR as i32) as u8
        } else if type_0 == 1 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CONCAVE as i32) as u8
        } else if type_0 == 2 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CONVEX as i32) as u8
        } else if type_0 == 3 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_SWITCH as i32) as u8
        } else {
            (*mod_dest).amount = 0 as i32 as f64
        }
        (*mod_dest).dest = mod_src.dest as u8;
        (*mod_dest).src2 = (mod_src.amt_src as i32 & 127 as i32) as u8;
        (*mod_dest).flags2 = 0 as i32 as u8;
        if mod_src.amt_src as i32 & (1 as i32) << 7 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CC as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_GC as i32) as u8
        }
        if mod_src.amt_src as i32 & (1 as i32) << 8 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_NEGATIVE as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_POSITIVE as i32) as u8
        }
        if mod_src.amt_src as i32 & (1 as i32) << 9 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_BIPOLAR as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_UNIPOLAR as i32) as u8
        }
        type_0 = mod_src.amt_src as i32 >> 10 as i32;
        type_0 &= 63 as i32;
        if type_0 == 0 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_LINEAR as i32) as u8
        } else if type_0 == 1 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CONCAVE as i32) as u8
        } else if type_0 == 2 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CONVEX as i32) as u8
        } else if type_0 == 3 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_SWITCH as i32) as u8
        } else {
            (*mod_dest).amount = 0 as i32 as f64
        }
        if mod_src.transform as i32 != 0 as i32 {
            (*mod_dest).amount = 0 as i32 as f64
        }
        if count == 0 as i32 {
            (*zone).mod_0 = mod_dest
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

unsafe fn fluid_preset_zone_get_inst(zone: *mut PresetZone) -> *mut Instrument {
    return (*zone).inst;
}

unsafe fn fluid_preset_zone_inside_range(zone: *mut PresetZone, key: i32, vel: i32) -> i32 {
    return ((*zone).keylo <= key
        && (*zone).keyhi >= key
        && (*zone).vello <= vel
        && (*zone).velhi >= vel) as i32;
}

unsafe fn new_fluid_inst() -> *mut Instrument {
    let mut inst: *mut Instrument =
        libc::malloc(::std::mem::size_of::<Instrument>() as libc::size_t) as *mut Instrument;
    if inst.is_null() {
        log::error!("Out of memory",);
        return 0 as *mut Instrument;
    }
    (*inst).name = [0; 21];
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

unsafe fn fluid_inst_set_global_zone(mut inst: *mut Instrument, zone: *mut InstrumentZone) -> i32 {
    (*inst).global_zone = zone;
    return FLUID_OK as i32;
}

unsafe fn fluid_inst_import_sfont(
    sf2: &sf2::SoundFont2,
    inst: *mut Instrument,
    new_inst: &sf2::Instrument,
    sfont: *mut DefaultSoundFont,
) -> i32 {
    let mut zone: *mut InstrumentZone;
    let mut zone_name: [u8; 256] = [0; 256];
    let mut count: i32;

    if new_inst.header.name.len() > 0 {
        let cstr = CString::new(new_inst.header.name.clone()).unwrap();
        libc::strcpy(
            (*inst).name.as_mut_ptr() as _,
            cstr.as_c_str().as_ptr() as _,
        );
    } else {
        libc::strcpy(
            (*inst).name.as_mut_ptr() as _,
            b"<untitled>\x00" as *const u8 as *const i8,
        );
    }

    count = 0 as i32;
    for new_zone in new_inst.zones.iter() {
        libc::strcpy(
            zone_name.as_mut_ptr() as _,
            CString::new(format!("{}/{}", new_inst.header.name, count))
                .unwrap()
                .as_c_str()
                .as_ptr(),
        );
        zone = new_fluid_inst_zone(&zone_name);
        if zone.is_null() {
            return FLUID_FAILED as i32;
        }
        if fluid_inst_zone_import_sfont(sf2, zone, new_zone, &mut *sfont) != FLUID_OK as i32 {
            return FLUID_FAILED as i32;
        }
        if count == 0 as i32 && fluid_inst_zone_get_sample(zone).is_null() {
            fluid_inst_set_global_zone(inst, zone);
        } else if fluid_inst_add_zone(inst, zone) != FLUID_OK as i32 {
            return FLUID_FAILED as i32;
        }
        count += 1
    }
    return FLUID_OK as i32;
}

unsafe fn fluid_inst_add_zone(mut inst: *mut Instrument, mut zone: *mut InstrumentZone) -> i32 {
    if (*inst).zone.is_null() {
        (*zone).next = 0 as *mut InstrumentZone;
        (*inst).zone = zone
    } else {
        (*zone).next = (*inst).zone;
        (*inst).zone = zone
    }
    return FLUID_OK as i32;
}

unsafe fn fluid_inst_get_zone(inst: *mut Instrument) -> *mut InstrumentZone {
    return (*inst).zone;
}

unsafe fn fluid_inst_get_global_zone(inst: *mut Instrument) -> *mut InstrumentZone {
    return (*inst).global_zone;
}

unsafe fn new_fluid_inst_zone(name: &[u8]) -> *mut InstrumentZone {
    let mut zone: *mut InstrumentZone;
    zone = libc::malloc(::std::mem::size_of::<InstrumentZone>() as libc::size_t)
        as *mut InstrumentZone;
    libc::memset(zone as _, 0, std::mem::size_of::<InstrumentZone>() as _);
    if zone.is_null() {
        log::error!("Out of memory",);
        return 0 as *mut InstrumentZone;
    }
    (*zone).next = 0 as *mut InstrumentZone;
    (*zone).name = name.to_vec();
    (*zone).sample = 0 as *mut Sample;
    (*zone).keylo = 0 as i32;
    (*zone).keyhi = 128 as i32;
    (*zone).vello = 0 as i32;
    (*zone).velhi = 128 as i32;
    fluid_gen_set_default_values(&mut *(*zone).gen.as_mut_ptr().offset(0 as i32 as isize));
    (*zone).mod_0 = 0 as *mut Mod;
    return zone;
}

unsafe fn delete_fluid_inst_zone(zone: *mut InstrumentZone) -> i32 {
    let mut mod_0: *mut Mod;
    let mut tmp: *mut Mod;
    mod_0 = (*zone).mod_0;
    while !mod_0.is_null() {
        tmp = mod_0;
        mod_0 = (*mod_0).next;
        tmp.as_mut().unwrap().delete();
    }
    libc::free(zone as *mut libc::c_void);
    return FLUID_OK as i32;
}

unsafe fn fluid_inst_zone_next(zone: *mut InstrumentZone) -> *mut InstrumentZone {
    return (*zone).next;
}

unsafe fn fluid_inst_zone_import_sfont(
    sf2: &sf2::SoundFont2,
    mut zone: *mut InstrumentZone,
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
                (*zone).keylo = amount.low as i32;
                (*zone).keyhi = amount.high as i32
            }
            _ => {
                (*zone).gen[new_gen.ty as usize].val = *new_gen.amount.as_i16().unwrap() as f64;
                (*zone).gen[new_gen.ty as usize].flags = GEN_SET as u8;
            }
        }

        count += 1
    }
    if let Some(sample_id) = new_zone.sample() {
        let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();

        let name = CString::new(sample.name.clone()).unwrap();

        (*zone).sample = sfont.get_sample(name.as_c_str().to_bytes()).unwrap() as *mut _;

        if (*zone).sample.is_null() {
            log::error!("Couldn't find sample name",);
            return FLUID_FAILED as i32;
        }
    }
    count = 0 as i32;
    for new_mod in new_zone.mod_list.iter() {
        let mut type_0: i32;
        let mut mod_dest: *mut Mod;
        mod_dest = Mod::new();
        if mod_dest.is_null() {
            return FLUID_FAILED as i32;
        }
        (*mod_dest).next = 0 as *mut Mod;
        (*mod_dest).amount = new_mod.amount as f64;
        (*mod_dest).src1 = (new_mod.src as i32 & 127 as i32) as u8;
        (*mod_dest).flags1 = 0 as i32 as u8;
        if new_mod.src as i32 & (1 as i32) << 7 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CC as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_GC as i32) as u8
        }
        if new_mod.src as i32 & (1 as i32) << 8 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_NEGATIVE as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_POSITIVE as i32) as u8
        }
        if new_mod.src as i32 & (1 as i32) << 9 as i32 != 0 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_BIPOLAR as i32) as u8
        } else {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_UNIPOLAR as i32) as u8
        }
        type_0 = new_mod.src as i32 >> 10 as i32;
        type_0 &= 63 as i32;
        if type_0 == 0 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_LINEAR as i32) as u8
        } else if type_0 == 1 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CONCAVE as i32) as u8
        } else if type_0 == 2 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_CONVEX as i32) as u8
        } else if type_0 == 3 as i32 {
            (*mod_dest).flags1 = ((*mod_dest).flags1 as i32 | FLUID_MOD_SWITCH as i32) as u8
        } else {
            (*mod_dest).amount = 0 as i32 as f64
        }
        (*mod_dest).dest = new_mod.dest as u8;
        (*mod_dest).src2 = (new_mod.amt_src as i32 & 127 as i32) as u8;
        (*mod_dest).flags2 = 0 as i32 as u8;
        if new_mod.amt_src as i32 & (1 as i32) << 7 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CC as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_GC as i32) as u8
        }
        if new_mod.amt_src as i32 & (1 as i32) << 8 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_NEGATIVE as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_POSITIVE as i32) as u8
        }
        if new_mod.amt_src as i32 & (1 as i32) << 9 as i32 != 0 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_BIPOLAR as i32) as u8
        } else {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_UNIPOLAR as i32) as u8
        }
        type_0 = new_mod.amt_src as i32 >> 10 as i32;
        type_0 &= 63 as i32;
        if type_0 == 0 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_LINEAR as i32) as u8
        } else if type_0 == 1 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CONCAVE as i32) as u8
        } else if type_0 == 2 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_CONVEX as i32) as u8
        } else if type_0 == 3 as i32 {
            (*mod_dest).flags2 = ((*mod_dest).flags2 as i32 | FLUID_MOD_SWITCH as i32) as u8
        } else {
            (*mod_dest).amount = 0 as i32 as f64
        }
        if new_mod.transform as i32 != 0 as i32 {
            (*mod_dest).amount = 0 as i32 as f64
        }
        if count == 0 as i32 {
            (*zone).mod_0 = mod_dest
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

unsafe fn fluid_inst_zone_get_sample(zone: *mut InstrumentZone) -> *mut Sample {
    return (*zone).sample;
}

unsafe fn fluid_inst_zone_inside_range(zone: *mut InstrumentZone, key: i32, vel: i32) -> i32 {
    return ((*zone).keylo <= key
        && (*zone).keyhi >= key
        && (*zone).vello <= vel
        && (*zone).velhi >= vel) as i32;
}

unsafe fn fluid_sample_in_rom(sample: *mut Sample) -> i32 {
    return (*sample).sampletype & 0x8000 as i32;
}
