use super::channel::{Channel, InterpMethod};
use super::conv::fluid_act2hz;
use super::conv::fluid_atten2amp;
use super::conv::fluid_cb2amp;
use super::conv::fluid_ct2hz;
use super::conv::fluid_ct2hz_real;
use super::conv::fluid_pan;
use super::conv::fluid_tc2sec;
use super::conv::fluid_tc2sec_attack;
use super::conv::fluid_tc2sec_delay;
use super::conv::fluid_tc2sec_release;
use super::dsp_float::fluid_dsp_float_interpolate_4th_order;
use super::dsp_float::fluid_dsp_float_interpolate_7th_order;
use super::dsp_float::fluid_dsp_float_interpolate_linear;
use super::dsp_float::fluid_dsp_float_interpolate_none;
use super::gen::{self, Gen};
use super::modulator::Mod;
use super::soundfont::Sample;

pub type Phase = u64;
pub type ModFlags = u32;
pub const FLUID_MOD_CC: ModFlags = 16;
pub const FLUID_MOD_BIPOLAR: ModFlags = 2;
pub type ModSrc = u32;
pub const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
pub type GenType = u32;
pub const GEN_PITCH: GenType = 59;
pub const GEN_OVERRIDEROOTKEY: GenType = 58;
pub const GEN_EXCLUSIVECLASS: GenType = 57;
pub const GEN_SCALETUNE: GenType = 56;
pub const GEN_SAMPLEMODE: GenType = 54;
pub const GEN_FINETUNE: GenType = 52;
pub const GEN_COARSETUNE: GenType = 51;
pub const GEN_ENDLOOPADDRCOARSEOFS: GenType = 50;
pub const GEN_ATTENUATION: GenType = 48;
pub const GEN_VELOCITY: GenType = 47;
pub const GEN_KEYNUM: GenType = 46;
pub const GEN_STARTLOOPADDRCOARSEOFS: GenType = 45;
pub const GEN_KEYTOVOLENVDECAY: GenType = 40;
pub const GEN_KEYTOVOLENVHOLD: GenType = 39;
pub const GEN_VOLENVRELEASE: GenType = 38;
pub const GEN_VOLENVSUSTAIN: GenType = 37;
pub const GEN_VOLENVDECAY: GenType = 36;
pub const GEN_VOLENVHOLD: GenType = 35;
pub const GEN_VOLENVATTACK: GenType = 34;
pub const GEN_VOLENVDELAY: GenType = 33;
pub const GEN_KEYTOMODENVDECAY: GenType = 32;
pub const GEN_KEYTOMODENVHOLD: GenType = 31;
pub const GEN_MODENVRELEASE: GenType = 30;
pub const GEN_MODENVSUSTAIN: GenType = 29;
pub const GEN_MODENVDECAY: GenType = 28;
pub const GEN_MODENVHOLD: GenType = 27;
pub const GEN_MODENVATTACK: GenType = 26;
pub const GEN_MODENVDELAY: GenType = 25;
pub const GEN_VIBLFOFREQ: GenType = 24;
pub const GEN_VIBLFODELAY: GenType = 23;
pub const GEN_MODLFOFREQ: GenType = 22;
pub const GEN_MODLFODELAY: GenType = 21;
pub const GEN_PAN: GenType = 17;
pub const GEN_REVERBSEND: GenType = 16;
pub const GEN_CHORUSSEND: GenType = 15;
pub const GEN_MODLFOTOVOL: GenType = 13;
pub const GEN_ENDADDRCOARSEOFS: GenType = 12;
pub const GEN_MODENVTOFILTERFC: GenType = 11;
pub const GEN_MODLFOTOFILTERFC: GenType = 10;
pub const GEN_FILTERQ: GenType = 9;
pub const GEN_FILTERFC: GenType = 8;
pub const GEN_MODENVTOPITCH: GenType = 7;
pub const GEN_VIBLFOTOPITCH: GenType = 6;
pub const GEN_MODLFOTOPITCH: GenType = 5;
pub const GEN_STARTADDRCOARSEOFS: GenType = 4;
pub const GEN_ENDLOOPADDROFS: GenType = 3;
pub const GEN_STARTLOOPADDROFS: GenType = 2;
pub const GEN_ENDADDROFS: GenType = 1;
pub const GEN_STARTADDROFS: GenType = 0;
pub type GenFlags = u32;
pub const GEN_ABS_NRPN: GenFlags = 2;
pub const GEN_SET: GenFlags = 1;
pub const FLUID_VOICE_ENVRELEASE: VoiceEnvelopeIndex = 5;
pub const FLUID_VOICE_ENVDECAY: VoiceEnvelopeIndex = 3;
pub const FLUID_VOICE_ENVHOLD: VoiceEnvelopeIndex = 2;
pub const FLUID_VOICE_ENVATTACK: VoiceEnvelopeIndex = 1;
pub const FLUID_VOICE_ENVDELAY: VoiceEnvelopeIndex = 0;
pub type FluidVoiceAddMod = u32;
pub const FLUID_VOICE_ADD: FluidVoiceAddMod = 1;
pub const FLUID_VOICE_OVERWRITE: FluidVoiceAddMod = 0;
pub const FLUID_VOICE_SUSTAINED: VoiceStatus = 2;
pub const FLUID_VOICE_ON: VoiceStatus = 1;
pub const FLUID_OK: i32 = 0;
pub type VoiceStatus = u32;
pub const FLUID_VOICE_OFF: VoiceStatus = 3;
pub const FLUID_VOICE_CLEAN: VoiceStatus = 0;
pub type VoiceEnvelopeIndex = u32;
pub const FLUID_VOICE_ENVFINISHED: VoiceEnvelopeIndex = 6;
pub const FLUID_VOICE_ENVSUSTAIN: VoiceEnvelopeIndex = 4;
pub const FLUID_LOOP_DURING_RELEASE: LoopMode = 1;
pub const FLUID_LOOP_UNTIL_RELEASE: LoopMode = 3;
pub const FLUID_UNLOOPED: LoopMode = 0;
pub const SUSTAIN_SWITCH: MidiControlChange = 64;
pub type MidiControlChange = u32;
pub type LoopMode = u32;

#[derive(Copy, Clone)]
pub struct VoiceId(pub usize);

impl From<usize> for VoiceId {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

pub struct Voice {
    pub(crate) id: u32,
    pub(crate) status: u8,
    pub(crate) chan: u8,
    pub(crate) key: u8,
    pub(crate) vel: u8,
    channel: *mut Channel,
    pub(crate) gen: [Gen; 60],
    mod_0: [Mod; 64],
    mod_count: i32,
    pub(crate) has_looped: i32,
    pub(crate) sample: *mut Sample,
    check_sample_sanity_flag: i32,
    output_rate: f32,
    pub(crate) start_time: u32,
    pub(crate) ticks: u32,
    noteoff_ticks: u32,
    pub(crate) amp: f32,
    pub(crate) phase: Phase,
    pub(crate) phase_incr: f32,
    pub(crate) amp_incr: f32,
    pub(crate) dsp_buf: *mut f32,
    pitch: f32,
    attenuation: f32,
    min_attenuation_c_b: f32,
    root_pitch: f32,
    pub(crate) start: i32,
    pub(crate) end: i32,
    pub(crate) loopstart: i32,
    pub(crate) loopend: i32,
    synth_gain: f32,
    volenv_data: [EnvData; 7],
    volenv_count: u32,
    pub(crate) volenv_section: i32,
    pub(crate) volenv_val: f32,
    amplitude_that_reaches_noise_floor_nonloop: f32,
    amplitude_that_reaches_noise_floor_loop: f32,
    modenv_data: [EnvData; 7],
    modenv_count: u32,
    modenv_section: i32,
    modenv_val: f32,
    modenv_to_fc: f32,
    modenv_to_pitch: f32,
    modlfo_val: f32,
    modlfo_delay: u32,
    modlfo_incr: f32,
    modlfo_to_fc: f32,
    modlfo_to_pitch: f32,
    modlfo_to_vol: f32,
    viblfo_val: f32,
    viblfo_delay: u32,
    viblfo_incr: f32,
    viblfo_to_pitch: f32,
    fres: f32,
    last_fres: f32,
    q_lin: f32,
    filter_gain: f32,
    hist1: f32,
    hist2: f32,
    filter_startup: i32,
    b02: f32,
    b1: f32,
    a1: f32,
    a2: f32,
    b02_incr: f32,
    b1_incr: f32,
    a1_incr: f32,
    a2_incr: f32,
    filter_coeff_incr_count: i32,
    pan: f32,
    amp_left: f32,
    amp_right: f32,
    reverb_send: f32,
    amp_reverb: f32,
    chorus_send: f32,
    amp_chorus: f32,
    interp_method: InterpMethod,
    debug: i32,
}
#[derive(Copy, Default, Clone)]
pub struct EnvData {
    pub count: u32,
    pub coeff: f32,
    pub incr: f32,
    pub min: f32,
    pub max: f32,
}

impl Voice {
    pub(crate) fn new(output_rate: f32) -> Voice {
        let mut volenv_data = [EnvData::default(); 7];
        {
            let sustain = &mut volenv_data[FLUID_VOICE_ENVSUSTAIN as usize];

            sustain.count = 0xffffffff as u32;
            sustain.coeff = 1.0f32;
            sustain.incr = 0.0f32;
            sustain.min = -1.0f32;
            sustain.max = 2.0f32;

            let finished = &mut volenv_data[FLUID_VOICE_ENVFINISHED as usize];
            finished.count = 0xffffffff as u32;
            finished.coeff = 0.0f32;
            finished.incr = 0.0f32;
            finished.min = -1.0f32;
            finished.max = 1.0f32;
        }
        let mut modenv_data = [EnvData::default(); 7];
        {
            let sustain = &mut modenv_data[FLUID_VOICE_ENVSUSTAIN as usize];
            sustain.count = 0xffffffff as u32;
            sustain.coeff = 1.0f32;
            sustain.incr = 0.0f32;
            sustain.min = -1.0f32;
            sustain.max = 2.0f32;

            let finished = &mut modenv_data[FLUID_VOICE_ENVFINISHED as usize];
            finished.count = 0xffffffff as u32;
            finished.coeff = 0.0f32;
            finished.incr = 0.0f32;
            finished.min = -1.0f32;
            finished.max = 1.0f32;
        }

        Voice {
            id: 0,
            status: FLUID_VOICE_CLEAN as i32 as u8,
            chan: 0xff,
            key: 0,
            vel: 0,
            channel: 0 as *mut Channel,
            gen: [Gen::default(); 60],
            mod_0: [Mod::default(); 64],
            mod_count: 0,
            has_looped: 0,
            sample: std::ptr::null_mut(),
            check_sample_sanity_flag: 0,
            output_rate,
            start_time: 0,
            ticks: 0,
            noteoff_ticks: 0,
            amp: 0.0,
            phase: 0 as Phase,
            phase_incr: 0.0,
            amp_incr: 0.0,
            dsp_buf: std::ptr::null_mut(),
            pitch: 0.0,
            attenuation: 0.0,
            min_attenuation_c_b: 0.0,
            root_pitch: 0.0,
            start: 0,
            end: 0,
            loopstart: 0,
            loopend: 0,
            synth_gain: 0.0,
            volenv_data: volenv_data,
            volenv_count: 0,
            volenv_section: 0,
            volenv_val: 0.0,
            amplitude_that_reaches_noise_floor_nonloop: 0.0,
            amplitude_that_reaches_noise_floor_loop: 0.0,
            modenv_data: modenv_data,
            modenv_count: 0,
            modenv_section: 0,
            modenv_val: 0.0,
            modenv_to_fc: 0.0,
            modenv_to_pitch: 0.0,
            modlfo_val: 0.0,
            modlfo_delay: 0,
            modlfo_incr: 0.0,
            modlfo_to_fc: 0.0,
            modlfo_to_pitch: 0.0,
            modlfo_to_vol: 0.0,
            viblfo_val: 0.0,
            viblfo_delay: 0,
            viblfo_incr: 0.0,
            viblfo_to_pitch: 0.0,
            fres: 0.0,
            last_fres: 0.0,
            q_lin: 0.0,
            filter_gain: 0.0,
            hist1: 0.0,
            hist2: 0.0,
            filter_startup: 0,
            b02: 0.0,
            b1: 0.0,
            a1: 0.0,
            a2: 0.0,
            b02_incr: 0.0,
            b1_incr: 0.0,
            a1_incr: 0.0,
            a2_incr: 0.0,
            filter_coeff_incr_count: 0,
            pan: 0.0,
            amp_left: 0.0,
            amp_right: 0.0,
            reverb_send: 0.0,
            amp_reverb: 0.0,
            chorus_send: 0.0,
            amp_chorus: 0.0,
            interp_method: InterpMethod::None,
            debug: 0,
        }
    }

    pub(crate) unsafe fn init(
        &mut self,
        sample: *mut Sample,
        channel: *mut Channel,
        key: u8,
        vel: i32,
        id: u32,
        start_time: u32,
        gain: f32,
    ) {
        self.id = id;
        self.chan = channel.as_ref().unwrap().get_num() as u8;
        self.key = key as u8;
        self.vel = vel as u8;
        self.channel = channel;
        self.mod_count = 0 as i32;
        self.sample = sample;
        self.start_time = start_time;
        self.ticks = 0 as i32 as u32;
        self.noteoff_ticks = 0 as i32 as u32;
        self.debug = 0 as i32;
        self.has_looped = 0 as i32;
        self.last_fres = -(1 as i32) as f32;
        self.filter_startup = 1 as i32;
        self.interp_method = self.channel.as_ref().unwrap().get_interp_method();
        self.volenv_count = 0 as i32 as u32;
        self.volenv_section = 0 as i32;
        self.volenv_val = 0.0f32;
        self.amp = 0.0f32;
        self.modenv_count = 0 as i32 as u32;
        self.modenv_section = 0 as i32;
        self.modenv_val = 0.0f32;
        self.modlfo_val = 0.0f32;
        self.viblfo_val = 0.0f32;
        self.hist1 = 0 as i32 as f32;
        self.hist2 = 0 as i32 as f32;
        self.gen = gen::gen_init(&*channel);
        self.synth_gain = gain;
        if (self.synth_gain as f64) < 0.0000001f64 {
            self.synth_gain = 0.0000001f32
        }
        self.amplitude_that_reaches_noise_floor_nonloop =
            (0.00003f64 / self.synth_gain as f64) as f32;
        self.amplitude_that_reaches_noise_floor_loop = (0.00003f64 / self.synth_gain as f64) as f32;
        (*self.sample).refcount = (*self.sample).refcount.wrapping_add(1);
    }

    pub(crate) unsafe fn add_mod(&mut self, mod_0: &Mod, mode: i32) {
        let mut i;
        if mod_0.flags1 as i32 & FLUID_MOD_CC as i32 == 0 as i32
            && (mod_0.src1 as i32 != 0 as i32
                && mod_0.src1 as i32 != 2 as i32
                && mod_0.src1 as i32 != 3 as i32
                && mod_0.src1 as i32 != 10 as i32
                && mod_0.src1 as i32 != 13 as i32
                && mod_0.src1 as i32 != 14 as i32
                && mod_0.src1 as i32 != 16 as i32)
        {
            log::warn!(
                "Ignoring invalid controller, using non-CC source {}.",
                mod_0.src1 as i32
            );
            return;
        }
        if mode == FLUID_VOICE_ADD as i32 {
            i = 0 as i32;
            while i < self.mod_count {
                if self.mod_0[i as usize].test_identity(mod_0) != 0 {
                    //		printf("Adding modulator...\n");
                    self.mod_0[i as usize].amount += (*mod_0).amount;
                    return;
                }
                i += 1
            }
        } else if mode == FLUID_VOICE_OVERWRITE as i32 {
            i = 0 as i32;
            while i < self.mod_count {
                if self
                    .mod_0
                    .as_mut_ptr()
                    .offset(i as isize)
                    .as_ref()
                    .unwrap()
                    .test_identity(mod_0)
                    != 0
                {
                    //		printf("Replacing modulator...amount is %f\n",mod->amount);
                    self.mod_0[i as usize].amount = (*mod_0).amount;
                    return;
                }
                i += 1
            }
        }
        if self.mod_count < 64 as i32 {
            let fresh7 = self.mod_count;
            self.mod_count = self.mod_count + 1;
            *self.mod_0.as_mut_ptr().offset(fresh7 as isize) = mod_0.clone();
        };
    }

    pub(crate) fn gen_incr(&mut self, i: u32, val: f64) {
        self.gen[i as usize].val += val;
        self.gen[i as usize].flags = GEN_SET as u8;
    }

    pub(crate) fn gen_set(&mut self, i: i32, val: f64) {
        self.gen[i as usize].val = val;
        self.gen[i as usize].flags = GEN_SET as u8;
    }

    pub(crate) unsafe fn kill_excl(&mut self) {
        if !(self.status as i32 == FLUID_VOICE_ON as i32
            || self.status as i32 == FLUID_VOICE_SUSTAINED as i32)
        {
            return;
        }
        self.gen_set(GEN_EXCLUSIVECLASS as i32, 0.0);
        if self.volenv_section != FLUID_VOICE_ENVRELEASE as i32 {
            self.volenv_section = FLUID_VOICE_ENVRELEASE as i32;
            self.volenv_count = 0 as i32 as u32;
            self.modenv_section = FLUID_VOICE_ENVRELEASE as i32;
            self.modenv_count = 0 as i32 as u32
        }
        self.gen_set(GEN_VOLENVRELEASE as i32, -200.0);
        self.update_param(GEN_VOLENVRELEASE as i32);
        self.gen_set(GEN_MODENVRELEASE as i32, -200.0);
        self.update_param(GEN_MODENVRELEASE as i32);
    }

    pub(crate) unsafe fn start(&mut self) {
        self.calculate_runtime_synthesis_parameters();
        self.check_sample_sanity_flag = (1 as i32) << 1 as i32;
        self.status = FLUID_VOICE_ON as i32 as u8;
    }

    pub(crate) fn noteoff(&mut self, min_note_length_ticks: u32) -> i32 {
        let at_tick;
        at_tick = min_note_length_ticks;
        if at_tick > self.ticks {
            self.noteoff_ticks = at_tick;
            return FLUID_OK as i32;
        }
        if !self.channel.is_null()
            && unsafe { &mut *self.channel }.cc[SUSTAIN_SWITCH as i32 as usize] as i32 >= 64 as i32
        {
            self.status = FLUID_VOICE_SUSTAINED as i32 as u8
        } else {
            if self.volenv_section == FLUID_VOICE_ENVATTACK as i32 {
                if self.volenv_val > 0 as i32 as f32 {
                    let lfo: f32 = self.modlfo_val * -self.modlfo_to_vol;
                    let amp: f32 = (self.volenv_val as f64
                        * f64::powf(10.0f64, (lfo / -(200 as i32) as f32) as f64))
                        as f32;
                    let mut env_value: f32 = -((-(200 as i32) as f64 * f64::ln(amp as f64)
                        / f64::ln(10.0f64)
                        - lfo as f64)
                        / 960.0f64
                        - 1 as i32 as f64) as f32;
                    env_value = if (env_value as f64) < 0.0f64 {
                        0.0f64
                    } else if env_value as f64 > 1.0f64 {
                        1.0f64
                    } else {
                        env_value as f64
                    } as f32;
                    self.volenv_val = env_value
                }
            }
            self.volenv_section = FLUID_VOICE_ENVRELEASE as i32;
            self.volenv_count = 0 as i32 as u32;
            self.modenv_section = FLUID_VOICE_ENVRELEASE as i32;
            self.modenv_count = 0 as i32 as u32
        }
        return FLUID_OK as i32;
    }

    pub(crate) unsafe fn modulate(&mut self, cc: i32, ctrl: i32) -> i32 {
        let mut i;
        let mut k;
        let mut mod_0;
        let mut gen;
        let mut modval;
        i = 0 as i32;
        while i < self.mod_count {
            mod_0 = &mut *self.mod_0.as_mut_ptr().offset(i as isize) as *mut Mod;
            if (*mod_0).src1 as i32 == ctrl
                && (*mod_0).flags1 as i32 & FLUID_MOD_CC as i32 != 0 as i32
                && cc != 0 as i32
                || (*mod_0).src1 as i32 == ctrl
                    && (*mod_0).flags1 as i32 & FLUID_MOD_CC as i32 == 0 as i32
                    && cc == 0 as i32
                || ((*mod_0).src2 as i32 == ctrl
                    && (*mod_0).flags2 as i32 & FLUID_MOD_CC as i32 != 0 as i32
                    && cc != 0 as i32
                    || (*mod_0).src2 as i32 == ctrl
                        && (*mod_0).flags2 as i32 & FLUID_MOD_CC as i32 == 0 as i32
                        && cc == 0 as i32)
            {
                gen = mod_0.as_ref().unwrap().get_dest();
                modval = 0.0f32;
                k = 0 as i32;
                while k < self.mod_count {
                    if (*self).mod_0[k as usize].dest as i32 == gen {
                        modval += self
                            .mod_0
                            .as_mut_ptr()
                            .offset(k as isize)
                            .as_mut()
                            .unwrap()
                            .get_value(self.channel.as_mut().unwrap(), self)
                    }
                    k += 1
                }
                self.gen[gen as usize].mod_0 = modval as f64;
                self.update_param(gen);
            }
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub(crate) unsafe fn modulate_all(&mut self) -> i32 {
        let mut mod_0;
        let mut i;
        let mut k;
        let mut gen;
        let mut modval;
        i = 0 as i32;
        while i < self.mod_count {
            mod_0 = &mut *self.mod_0.as_mut_ptr().offset(i as isize) as *mut Mod;
            gen = mod_0.as_ref().unwrap().get_dest();
            modval = 0.0f32;
            k = 0 as i32;
            while k < self.mod_count {
                if self.mod_0[k as usize].dest as i32 == gen {
                    modval += self
                        .mod_0
                        .as_mut_ptr()
                        .offset(k as isize)
                        .as_mut()
                        .unwrap()
                        .get_value(self.channel.as_mut().unwrap(), self)
                }
                k += 1
            }
            self.gen[gen as usize].mod_0 = modval as f64;
            self.update_param(gen);
            i += 1
        }
        return FLUID_OK as i32;
    }

    pub(crate) fn off(&mut self) {
        self.chan = 0xff as i32 as u8;
        self.volenv_section = FLUID_VOICE_ENVFINISHED as i32;
        self.volenv_count = 0 as i32 as u32;
        self.modenv_section = FLUID_VOICE_ENVFINISHED as i32;
        self.modenv_count = 0 as i32 as u32;
        self.status = FLUID_VOICE_OFF as i32 as u8;
        if !self.sample.is_null() {
            unsafe {
                (*self.sample).refcount = (*self.sample).refcount.wrapping_sub(1);
            }
            self.sample = 0 as *mut Sample
        }
    }

    pub(crate) fn get_channel(&self) -> *mut Channel {
        self.channel
    }

    unsafe fn get_lower_boundary_for_attenuation(&mut self) -> f32 {
        let mut i;
        let mut mod_0;
        let mut possible_att_reduction_c_b: f32 = 0 as i32 as f32;
        let mut lower_bound;
        i = 0 as i32;
        while i < self.mod_count {
            mod_0 = &mut *self.mod_0.as_mut_ptr().offset(i as isize) as *mut Mod;
            if (*mod_0).dest as i32 == GEN_ATTENUATION as i32
                && ((*mod_0).flags1 as i32 & FLUID_MOD_CC as i32 != 0
                    || (*mod_0).flags2 as i32 & FLUID_MOD_CC as i32 != 0)
            {
                let current_val: f32 = mod_0
                    .as_mut()
                    .unwrap()
                    .get_value(self.channel.as_mut().unwrap(), self);
                let mut v: f32 = f64::abs((*mod_0).amount) as f32;
                if (*mod_0).src1 as i32 == FLUID_MOD_PITCHWHEEL as i32
                    || (*mod_0).flags1 as i32 & FLUID_MOD_BIPOLAR as i32 != 0
                    || (*mod_0).flags2 as i32 & FLUID_MOD_BIPOLAR as i32 != 0
                    || (*mod_0).amount < 0 as i32 as f64
                {
                    v = (v as f64 * -1.0f64) as f32
                } else {
                    v = 0 as i32 as f32
                }
                if current_val > v {
                    possible_att_reduction_c_b += current_val - v
                }
            }
            i += 1
        }
        lower_bound = self.attenuation - possible_att_reduction_c_b;
        if lower_bound < 0 as i32 as f32 {
            lower_bound = 0 as i32 as f32
        }
        return lower_bound;
    }

    unsafe fn calculate_runtime_synthesis_parameters(&mut self) -> i32 {
        let mut i;
        let list_of_generators_to_initialize: [i32; 35] = [
            GEN_STARTADDROFS as i32,
            GEN_ENDADDROFS as i32,
            GEN_STARTLOOPADDROFS as i32,
            GEN_ENDLOOPADDROFS as i32,
            GEN_MODLFOTOPITCH as i32,
            GEN_VIBLFOTOPITCH as i32,
            GEN_MODENVTOPITCH as i32,
            GEN_FILTERFC as i32,
            GEN_FILTERQ as i32,
            GEN_MODLFOTOFILTERFC as i32,
            GEN_MODENVTOFILTERFC as i32,
            GEN_MODLFOTOVOL as i32,
            GEN_CHORUSSEND as i32,
            GEN_REVERBSEND as i32,
            GEN_PAN as i32,
            GEN_MODLFODELAY as i32,
            GEN_MODLFOFREQ as i32,
            GEN_VIBLFODELAY as i32,
            GEN_VIBLFOFREQ as i32,
            GEN_MODENVDELAY as i32,
            GEN_MODENVATTACK as i32,
            GEN_MODENVHOLD as i32,
            GEN_MODENVDECAY as i32,
            GEN_MODENVRELEASE as i32,
            GEN_VOLENVDELAY as i32,
            GEN_VOLENVATTACK as i32,
            GEN_VOLENVHOLD as i32,
            GEN_VOLENVDECAY as i32,
            GEN_VOLENVRELEASE as i32,
            GEN_KEYNUM as i32,
            GEN_VELOCITY as i32,
            GEN_ATTENUATION as i32,
            GEN_OVERRIDEROOTKEY as i32,
            GEN_PITCH as i32,
            -(1 as i32),
        ];
        i = 0 as i32;
        while i < self.mod_count {
            let mod_0: *mut Mod = &mut *self.mod_0.as_mut_ptr().offset(i as isize) as *mut Mod;
            let modval: f32 = mod_0
                .as_mut()
                .unwrap()
                .get_value(self.channel.as_mut().unwrap(), self);
            let dest_gen_index: i32 = (*mod_0).dest as i32;
            let mut dest_gen: *mut Gen =
                &mut *self.gen.as_mut_ptr().offset(dest_gen_index as isize) as *mut Gen;
            (*dest_gen).mod_0 += modval as f64;
            i += 1
        }
        if !(*self.channel).tuning.is_none() {
            let tuning = (*self.channel).tuning.as_ref().unwrap();
            self.gen[GEN_PITCH as i32 as usize].val = tuning.pitch[60 as i32 as usize]
                + self.gen[GEN_SCALETUNE as i32 as usize].val / 100.0f32 as f64
                    * (tuning.pitch[self.key as usize] - tuning.pitch[60 as i32 as usize])
        } else {
            self.gen[GEN_PITCH as i32 as usize].val = self.gen[GEN_SCALETUNE as i32 as usize].val
                * (self.key as i32 as f32 - 60.0f32) as f64
                + (100.0f32 * 60.0f32) as f64
        }
        i = 0 as i32;
        while list_of_generators_to_initialize[i as usize] != -(1 as i32) {
            self.update_param(list_of_generators_to_initialize[i as usize]);
            i += 1
        }
        self.min_attenuation_c_b = self.get_lower_boundary_for_attenuation();
        return FLUID_OK as i32;
    }

    pub unsafe fn update_param(&mut self, gen: i32) {
        let mut q_d_b;
        let mut x;
        let mut y;
        let mut count;
        // Alternate attenuation scale used by EMU10K1 cards when setting the attenuation at the preset or instrument level within the SoundFont bank.
        static mut ALT_ATTENUATION_SCALE: f32 = 0.4f32;
        let current_block_195: u64;
        match gen as u32 {
            17 => {
                self.pan = self.gen[GEN_PAN as i32 as usize].val as f32
                    + self.gen[GEN_PAN as i32 as usize].mod_0 as f32
                    + self.gen[GEN_PAN as i32 as usize].nrpn as f32;
                self.amp_left = fluid_pan(self.pan, 1 as i32) * self.synth_gain / 32768.0f32;
                self.amp_right = fluid_pan(self.pan, 0 as i32) * self.synth_gain / 32768.0f32;
                current_block_195 = 5267916556966421873;
            }
            48 => {
                self.attenuation = self.gen[GEN_ATTENUATION as i32 as usize].val as f32
                    * ALT_ATTENUATION_SCALE
                    + self.gen[GEN_ATTENUATION as i32 as usize].mod_0 as f32
                    + self.gen[GEN_ATTENUATION as i32 as usize].nrpn as f32;
                self.attenuation = if (self.attenuation as f64) < 0.0f64 {
                    0.0f64
                } else if self.attenuation as f64 > 1440.0f64 {
                    1440.0f64
                } else {
                    self.attenuation as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            59 | 51 | 52 => {
                self.pitch = self.gen[GEN_PITCH as i32 as usize].val as f32
                    + self.gen[GEN_PITCH as i32 as usize].mod_0 as f32
                    + self.gen[GEN_PITCH as i32 as usize].nrpn as f32
                    + 100.0f32
                        * (self.gen[GEN_COARSETUNE as i32 as usize].val as f32
                            + self.gen[GEN_COARSETUNE as i32 as usize].mod_0 as f32
                            + self.gen[GEN_COARSETUNE as i32 as usize].nrpn as f32)
                    + (self.gen[GEN_FINETUNE as i32 as usize].val as f32
                        + self.gen[GEN_FINETUNE as i32 as usize].mod_0 as f32
                        + self.gen[GEN_FINETUNE as i32 as usize].nrpn as f32);
                current_block_195 = 5267916556966421873;
            }
            16 => {
                self.reverb_send = (self.gen[GEN_REVERBSEND as i32 as usize].val as f32
                    + self.gen[GEN_REVERBSEND as i32 as usize].mod_0 as f32
                    + self.gen[GEN_REVERBSEND as i32 as usize].nrpn as f32)
                    / 1000.0f32;
                self.reverb_send = if (self.reverb_send as f64) < 0.0f64 {
                    0.0f64
                } else if self.reverb_send as f64 > 1.0f64 {
                    1.0f64
                } else {
                    self.reverb_send as f64
                } as f32;
                self.amp_reverb = self.reverb_send * self.synth_gain / 32768.0f32;
                current_block_195 = 5267916556966421873;
            }
            15 => {
                self.chorus_send = (self.gen[GEN_CHORUSSEND as i32 as usize].val as f32
                    + self.gen[GEN_CHORUSSEND as i32 as usize].mod_0 as f32
                    + self.gen[GEN_CHORUSSEND as i32 as usize].nrpn as f32)
                    / 1000.0f32;
                self.chorus_send = if (self.chorus_send as f64) < 0.0f64 {
                    0.0f64
                } else if self.chorus_send as f64 > 1.0f64 {
                    1.0f64
                } else {
                    self.chorus_send as f64
                } as f32;
                self.amp_chorus = self.chorus_send * self.synth_gain / 32768.0f32;
                current_block_195 = 5267916556966421873;
            }
            58 => {
                if self.gen[GEN_OVERRIDEROOTKEY as i32 as usize].val > -(1 as i32) as f64 {
                    //FIXME: use flag instead of -1
                    self.root_pitch =
                        (self.gen[GEN_OVERRIDEROOTKEY as i32 as usize].val * 100.0f32 as f64
                            - (*self.sample).pitchadj as f64) as f32
                } else {
                    self.root_pitch =
                        (*self.sample).origpitch as f32 * 100.0f32 - (*self.sample).pitchadj as f32
                }
                self.root_pitch = fluid_ct2hz(self.root_pitch);
                if !self.sample.is_null() {
                    self.root_pitch *= self.output_rate / (*self.sample).samplerate as f32
                }
                current_block_195 = 5267916556966421873;
            }
            8 => {
                self.fres = (*self).gen[GEN_FILTERFC as i32 as usize].val as f32
                    + self.gen[GEN_FILTERFC as i32 as usize].mod_0 as f32
                    + self.gen[GEN_FILTERFC as i32 as usize].nrpn as f32;
                self.last_fres = -1.0f32;
                current_block_195 = 5267916556966421873;
            }
            9 => {
                q_d_b = ((self.gen[GEN_FILTERQ as i32 as usize].val as f32
                    + self.gen[GEN_FILTERQ as i32 as usize].mod_0 as f32
                    + self.gen[GEN_FILTERQ as i32 as usize].nrpn as f32)
                    / 10.0f32) as f64;
                q_d_b = if q_d_b < 0.0f32 as f64 {
                    0.0f32 as f64
                } else if q_d_b > 96.0f32 as f64 {
                    96.0f32 as f64
                } else {
                    q_d_b
                };
                q_d_b -= 3.01f32 as f64;
                self.q_lin = f64::powf(10.0f32 as f64, q_d_b / 20.0f32 as f64) as f32;
                self.filter_gain = (1.0f64 / f64::sqrt(self.q_lin as f64)) as f32;
                self.last_fres = -1.0f32;
                current_block_195 = 5267916556966421873;
            }
            5 => {
                self.modlfo_to_pitch = self.gen[GEN_MODLFOTOPITCH as i32 as usize].val as f32
                    + self.gen[GEN_MODLFOTOPITCH as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODLFOTOPITCH as i32 as usize].nrpn as f32;
                self.modlfo_to_pitch = if (self.modlfo_to_pitch as f64) < -12000.0f64 {
                    -12000.0f64
                } else if self.modlfo_to_pitch as f64 > 12000.0f64 {
                    12000.0f64
                } else {
                    self.modlfo_to_pitch as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            13 => {
                self.modlfo_to_vol = self.gen[GEN_MODLFOTOVOL as i32 as usize].val as f32
                    + self.gen[GEN_MODLFOTOVOL as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODLFOTOVOL as i32 as usize].nrpn as f32;
                self.modlfo_to_vol = if (self.modlfo_to_vol as f64) < -960.0f64 {
                    -960.0f64
                } else if self.modlfo_to_vol as f64 > 960.0f64 {
                    960.0f64
                } else {
                    self.modlfo_to_vol as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            10 => {
                self.modlfo_to_fc = self.gen[GEN_MODLFOTOFILTERFC as i32 as usize].val as f32
                    + self.gen[GEN_MODLFOTOFILTERFC as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODLFOTOFILTERFC as i32 as usize].nrpn as f32;
                self.modlfo_to_fc = if self.modlfo_to_fc < -(12000 as i32) as f32 {
                    -(12000 as i32) as f32
                } else if self.modlfo_to_fc > 12000 as i32 as f32 {
                    12000 as i32 as f32
                } else {
                    self.modlfo_to_fc
                };
                current_block_195 = 5267916556966421873;
            }
            21 => {
                x = self.gen[GEN_MODLFODELAY as i32 as usize].val as f32
                    + self.gen[GEN_MODLFODELAY as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODLFODELAY as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 5000.0f32 {
                    5000.0f32
                } else {
                    x
                };
                self.modlfo_delay = (self.output_rate * fluid_tc2sec_delay(x)) as u32;
                current_block_195 = 5267916556966421873;
            }
            22 => {
                x = self.gen[GEN_MODLFOFREQ as i32 as usize].val as f32
                    + self.gen[GEN_MODLFOFREQ as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODLFOFREQ as i32 as usize].nrpn as f32;
                x = if x < -16000.0f32 {
                    -16000.0f32
                } else if x > 4500.0f32 {
                    4500.0f32
                } else {
                    x
                };
                self.modlfo_incr = 4.0f32 * 64 as i32 as f32 * fluid_act2hz(x) / self.output_rate;
                current_block_195 = 5267916556966421873;
            }
            24 => {
                x = self.gen[GEN_VIBLFOFREQ as i32 as usize].val as f32
                    + self.gen[GEN_VIBLFOFREQ as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VIBLFOFREQ as i32 as usize].nrpn as f32;
                x = if x < -16000.0f32 {
                    -16000.0f32
                } else if x > 4500.0f32 {
                    4500.0f32
                } else {
                    x
                };
                self.viblfo_incr = 4.0f32 * 64 as i32 as f32 * fluid_act2hz(x) / self.output_rate;
                current_block_195 = 5267916556966421873;
            }
            23 => {
                x = self.gen[GEN_VIBLFODELAY as i32 as usize].val as f32
                    + self.gen[GEN_VIBLFODELAY as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VIBLFODELAY as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 5000.0f32 {
                    5000.0f32
                } else {
                    x
                };
                self.viblfo_delay = (self.output_rate * fluid_tc2sec_delay(x)) as u32;
                current_block_195 = 5267916556966421873;
            }
            6 => {
                self.viblfo_to_pitch = self.gen[GEN_VIBLFOTOPITCH as i32 as usize].val as f32
                    + self.gen[GEN_VIBLFOTOPITCH as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VIBLFOTOPITCH as i32 as usize].nrpn as f32;
                self.viblfo_to_pitch = if (self.viblfo_to_pitch as f64) < -12000.0f64 {
                    -12000.0f64
                } else if self.viblfo_to_pitch as f64 > 12000.0f64 {
                    12000.0f64
                } else {
                    self.viblfo_to_pitch as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            46 => {
                x = self.gen[GEN_KEYNUM as i32 as usize].val as f32
                    + self.gen[GEN_KEYNUM as i32 as usize].mod_0 as f32
                    + self.gen[GEN_KEYNUM as i32 as usize].nrpn as f32;
                if x >= 0 as i32 as f32 {
                    self.key = x as u8
                }
                current_block_195 = 5267916556966421873;
            }
            47 => {
                x = self.gen[GEN_VELOCITY as i32 as usize].val as f32
                    + self.gen[GEN_VELOCITY as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VELOCITY as i32 as usize].nrpn as f32;
                if x > 0 as i32 as f32 {
                    self.vel = x as u8
                }
                current_block_195 = 5267916556966421873;
            }
            7 => {
                self.modenv_to_pitch = self.gen[GEN_MODENVTOPITCH as i32 as usize].val as f32
                    + self.gen[GEN_MODENVTOPITCH as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODENVTOPITCH as i32 as usize].nrpn as f32;
                self.modenv_to_pitch = if (self.modenv_to_pitch as f64) < -12000.0f64 {
                    -12000.0f64
                } else if self.modenv_to_pitch as f64 > 12000.0f64 {
                    12000.0f64
                } else {
                    self.modenv_to_pitch as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            11 => {
                self.modenv_to_fc = self.gen[GEN_MODENVTOFILTERFC as i32 as usize].val as f32
                    + self.gen[GEN_MODENVTOFILTERFC as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODENVTOFILTERFC as i32 as usize].nrpn as f32;
                self.modenv_to_fc = if (self.modenv_to_fc as f64) < -12000.0f64 {
                    -12000.0f64
                } else if self.modenv_to_fc as f64 > 12000.0f64 {
                    12000.0f64
                } else {
                    self.modenv_to_fc as f64
                } as f32;
                current_block_195 = 5267916556966421873;
            }
            0 | 4 => {
                if !self.sample.is_null() {
                    self.start = (*self.sample)
                        .start
                        .wrapping_add(
                            (self.gen[GEN_STARTADDROFS as i32 as usize].val as f32
                                + self.gen[GEN_STARTADDROFS as i32 as usize].mod_0 as f32
                                + self.gen[GEN_STARTADDROFS as i32 as usize].nrpn as f32)
                                as i32 as u32,
                        )
                        .wrapping_add(
                            (32768 as i32
                                * (self.gen[GEN_STARTADDRCOARSEOFS as i32 as usize].val as f32
                                    + self.gen[GEN_STARTADDRCOARSEOFS as i32 as usize].mod_0 as f32
                                    + self.gen[GEN_STARTADDRCOARSEOFS as i32 as usize].nrpn as f32)
                                    as i32) as u32,
                        ) as i32;
                    self.check_sample_sanity_flag = (1 as i32) << 0 as i32
                }
                current_block_195 = 5267916556966421873;
            }
            1 | 12 => {
                if !self.sample.is_null() {
                    self.end = (*self.sample)
                        .end
                        .wrapping_add(
                            (self.gen[GEN_ENDADDROFS as i32 as usize].val as f32
                                + self.gen[GEN_ENDADDROFS as i32 as usize].mod_0 as f32
                                + self.gen[GEN_ENDADDROFS as i32 as usize].nrpn as f32)
                                as i32 as u32,
                        )
                        .wrapping_add(
                            (32768 as i32
                                * (self.gen[GEN_ENDADDRCOARSEOFS as i32 as usize].val as f32
                                    + self.gen[GEN_ENDADDRCOARSEOFS as i32 as usize].mod_0 as f32
                                    + self.gen[GEN_ENDADDRCOARSEOFS as i32 as usize].nrpn as f32)
                                    as i32) as u32,
                        ) as i32;
                    self.check_sample_sanity_flag = (1 as i32) << 0 as i32
                }
                current_block_195 = 5267916556966421873;
            }
            2 | 45 => {
                if !self.sample.is_null() {
                    self.loopstart = (*self.sample)
                        .loopstart
                        .wrapping_add(
                            (self.gen[GEN_STARTLOOPADDROFS as i32 as usize].val as f32
                                + self.gen[GEN_STARTLOOPADDROFS as i32 as usize].mod_0 as f32
                                + self.gen[GEN_STARTLOOPADDROFS as i32 as usize].nrpn as f32)
                                as i32 as u32,
                        )
                        .wrapping_add(
                            (32768 as i32
                                * (self.gen[GEN_STARTLOOPADDRCOARSEOFS as i32 as usize].val as f32
                                    + self.gen[GEN_STARTLOOPADDRCOARSEOFS as i32 as usize].mod_0
                                        as f32
                                    + self.gen[GEN_STARTLOOPADDRCOARSEOFS as i32 as usize].nrpn
                                        as f32) as i32) as u32,
                        ) as i32;
                    self.check_sample_sanity_flag = (1 as i32) << 0 as i32
                }
                current_block_195 = 5267916556966421873;
            }
            3 | 50 => {
                if !self.sample.is_null() {
                    self.loopend = (*self.sample)
                        .loopend
                        .wrapping_add(
                            (self.gen[GEN_ENDLOOPADDROFS as i32 as usize].val as f32
                                + self.gen[GEN_ENDLOOPADDROFS as i32 as usize].mod_0 as f32
                                + self.gen[GEN_ENDLOOPADDROFS as i32 as usize].nrpn as f32)
                                as i32 as u32,
                        )
                        .wrapping_add(
                            (32768 as i32
                                * (self.gen[GEN_ENDLOOPADDRCOARSEOFS as i32 as usize].val as f32
                                    + self.gen[GEN_ENDLOOPADDRCOARSEOFS as i32 as usize].mod_0
                                        as f32
                                    + self.gen[GEN_ENDLOOPADDRCOARSEOFS as i32 as usize].nrpn
                                        as f32) as i32) as u32,
                        ) as i32;
                    self.check_sample_sanity_flag = (1 as i32) << 0 as i32
                }
                current_block_195 = 5267916556966421873;
            }
            33 => {
                x = self.gen[GEN_VOLENVDELAY as i32 as usize].val as f32
                    + self.gen[GEN_VOLENVDELAY as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VOLENVDELAY as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 5000.0f32 {
                    5000.0f32
                } else {
                    x
                };
                count = (self.output_rate * fluid_tc2sec_delay(x) / 64 as i32 as f32) as u32;
                self.volenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].coeff = 0.0f32;
                self.volenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].incr = 0.0f32;
                self.volenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].min = -1.0f32;
                self.volenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].max = 1.0f32;
                current_block_195 = 5267916556966421873;
            }
            34 => {
                x = self.gen[GEN_VOLENVATTACK as i32 as usize].val as f32
                    + self.gen[GEN_VOLENVATTACK as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VOLENVATTACK as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 8000.0f32 {
                    8000.0f32
                } else {
                    x
                };
                count = (1 as i32 as u32).wrapping_add(
                    (self.output_rate * fluid_tc2sec_attack(x) / 64 as i32 as f32) as u32,
                );
                self.volenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].coeff = 1.0f32;
                self.volenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].incr = if count != 0 {
                    (1.0f32) / count as f32
                } else {
                    0.0f32
                };
                self.volenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].min = -1.0f32;
                self.volenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].max = 1.0f32;
                current_block_195 = 5267916556966421873;
            }
            35 | 39 => {
                count = self.calculate_hold_decay_buffers(
                    GEN_VOLENVHOLD as i32,
                    GEN_KEYTOVOLENVHOLD as i32,
                    0 as i32,
                ) as u32;
                self.volenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].coeff = 1.0f32;
                self.volenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].incr = 0.0f32;
                self.volenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].min = -1.0f32;
                self.volenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].max = 2.0f32;
                current_block_195 = 5267916556966421873;
            }
            36 => {
                current_block_195 = 16592787104725195690;
            }
            37 | 40 => {
                current_block_195 = 16592787104725195690;
            }
            38 => {
                x = self.gen[GEN_VOLENVRELEASE as i32 as usize].val as f32
                    + self.gen[GEN_VOLENVRELEASE as i32 as usize].mod_0 as f32
                    + self.gen[GEN_VOLENVRELEASE as i32 as usize].nrpn as f32;
                x = if x < -7200.0f32 {
                    -7200.0f32
                } else if x > 8000.0f32 {
                    8000.0f32
                } else {
                    x
                };
                count = (1 as i32 as u32).wrapping_add(
                    (self.output_rate * fluid_tc2sec_release(x) / 64 as i32 as f32) as u32,
                );
                self.volenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].coeff = 1.0f32;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].incr = if count != 0 {
                    (-1.0f32) / count as f32
                } else {
                    0.0f32
                };
                self.volenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].min = 0.0f32;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].max = 1.0f32;
                current_block_195 = 5267916556966421873;
            }
            25 => {
                x = self.gen[GEN_MODENVDELAY as i32 as usize].val as f32
                    + self.gen[GEN_MODENVDELAY as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODENVDELAY as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 5000.0f32 {
                    5000.0f32
                } else {
                    x
                };
                self.modenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].count =
                    (self.output_rate * fluid_tc2sec_delay(x) / 64 as i32 as f32) as u32;
                self.modenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].coeff = 0.0f32;
                self.modenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].incr = 0.0f32;
                self.modenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].min = -1.0f32;
                self.modenv_data[FLUID_VOICE_ENVDELAY as i32 as usize].max = 1.0f32;
                current_block_195 = 5267916556966421873;
            }
            26 => {
                x = self.gen[GEN_MODENVATTACK as i32 as usize].val as f32
                    + self.gen[GEN_MODENVATTACK as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODENVATTACK as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 8000.0f32 {
                    8000.0f32
                } else {
                    x
                };
                count = (1 as i32 as u32).wrapping_add(
                    (self.output_rate * fluid_tc2sec_attack(x) / 64 as i32 as f32) as u32,
                );
                self.modenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].coeff = 1.0f32;
                self.modenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].incr = if count != 0 {
                    (1.0f32) / count as f32
                } else {
                    0.0f32
                };
                self.modenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].min = -1.0f32;
                self.modenv_data[FLUID_VOICE_ENVATTACK as i32 as usize].max = 1.0f32;
                current_block_195 = 5267916556966421873;
            }
            27 | 31 => {
                count = self.calculate_hold_decay_buffers(
                    GEN_MODENVHOLD as i32,
                    GEN_KEYTOMODENVHOLD as i32,
                    0 as i32,
                ) as u32;
                self.modenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].coeff = 1.0f32;
                self.modenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].incr = 0.0f32;
                self.modenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].min = -1.0f32;
                self.modenv_data[FLUID_VOICE_ENVHOLD as i32 as usize].max = 2.0f32;
                current_block_195 = 5267916556966421873;
            }
            28 => {
                current_block_195 = 9635119298622998056;
            }
            29 | 32 => {
                current_block_195 = 9635119298622998056;
            }
            30 => {
                x = self.gen[GEN_MODENVRELEASE as i32 as usize].val as f32
                    + self.gen[GEN_MODENVRELEASE as i32 as usize].mod_0 as f32
                    + self.gen[GEN_MODENVRELEASE as i32 as usize].nrpn as f32;
                x = if x < -12000.0f32 {
                    -12000.0f32
                } else if x > 8000.0f32 {
                    8000.0f32
                } else {
                    x
                };
                count = (1 as i32 as u32).wrapping_add(
                    (self.output_rate * fluid_tc2sec_release(x) / 64 as i32 as f32) as u32,
                );
                self.modenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].coeff = 1.0f32;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].incr = if count != 0 {
                    (-1.0f32 / count as f32) as f64
                } else {
                    0.0f64
                }
                    as f32;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].min = 0.0f32;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as i32 as usize].max = 2.0f32;
                current_block_195 = 5267916556966421873;
            }
            _ => {
                current_block_195 = 5267916556966421873;
            }
        }
        match current_block_195 {
            9635119298622998056 => {
                count = self.calculate_hold_decay_buffers(
                    GEN_MODENVDECAY as i32,
                    GEN_KEYTOMODENVDECAY as i32,
                    1 as i32,
                ) as u32;
                y = 1.0f32
                    - 0.001f32
                        * (self.gen[GEN_MODENVSUSTAIN as i32 as usize].val as f32
                            + self.gen[GEN_MODENVSUSTAIN as i32 as usize].mod_0 as f32
                            + self.gen[GEN_MODENVSUSTAIN as i32 as usize].nrpn as f32);
                y = if y < 0.0f32 {
                    0.0f32
                } else if y > 1.0f32 {
                    1.0f32
                } else {
                    y
                };
                self.modenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].coeff = 1.0f32;
                self.modenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].incr = if count != 0 {
                    (-1.0f32) / count as f32
                } else {
                    0.0f32
                };
                self.modenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].min = y;
                self.modenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].max = 2.0f32
            }
            16592787104725195690 => {
                y = 1.0f32
                    - 0.001f32
                        * (self.gen[GEN_VOLENVSUSTAIN as i32 as usize].val as f32
                            + self.gen[GEN_VOLENVSUSTAIN as i32 as usize].mod_0 as f32
                            + self.gen[GEN_VOLENVSUSTAIN as i32 as usize].nrpn as f32);
                y = if y < 0.0f32 {
                    0.0f32
                } else if y > 1.0f32 {
                    1.0f32
                } else {
                    y
                };
                count = self.calculate_hold_decay_buffers(
                    GEN_VOLENVDECAY as i32,
                    GEN_KEYTOVOLENVDECAY as i32,
                    1 as i32,
                ) as u32;
                self.volenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].coeff = 1.0f32;
                self.volenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].incr = if count != 0 {
                    (-1.0f32) / count as f32
                } else {
                    0.0f32
                };
                self.volenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].min = y;
                self.volenv_data[FLUID_VOICE_ENVDECAY as i32 as usize].max = 2.0f32
            }
            _ => {}
        };
    }

    pub unsafe fn check_sample_sanity(&mut self) {
        let min_index_nonloop: i32 = (*self.sample).start as i32;
        let max_index_nonloop: i32 = (*self.sample).end as i32;
        let min_index_loop: i32 = (*self.sample).start as i32 + 0 as i32;
        let max_index_loop: i32 = (*self.sample).end as i32 - 0 as i32 + 1 as i32;
        if self.check_sample_sanity_flag == 0 {
            return;
        }
        if self.start < min_index_nonloop {
            self.start = min_index_nonloop
        } else if self.start > max_index_nonloop {
            self.start = max_index_nonloop
        }
        if self.end < min_index_nonloop {
            self.end = min_index_nonloop
        } else if self.end > max_index_nonloop {
            self.end = max_index_nonloop
        }
        if self.start > self.end {
            let temp: i32 = self.start;
            self.start = self.end;
            self.end = temp
        }
        if self.start == self.end {
            self.off();
            return;
        }
        if self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
            || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_DURING_RELEASE as i32
        {
            if self.loopstart < min_index_loop {
                self.loopstart = min_index_loop
            } else if self.loopstart > max_index_loop {
                self.loopstart = max_index_loop
            }
            if self.loopend < min_index_loop {
                self.loopend = min_index_loop
            } else if self.loopend > max_index_loop {
                self.loopend = max_index_loop
            }
            if self.loopstart > self.loopend {
                let temp_0: i32 = self.loopstart;
                self.loopstart = self.loopend;
                self.loopend = temp_0
            }
            if self.loopend < self.loopstart + 2 as i32 {
                self.gen[GEN_SAMPLEMODE as i32 as usize].val = FLUID_UNLOOPED as i32 as f64
            }
            if self.loopstart >= (*self.sample).loopstart as i32
                && self.loopend <= (*self.sample).loopend as i32
            {
                if (*self.sample).amplitude_that_reaches_noise_floor_is_valid != 0 {
                    self.amplitude_that_reaches_noise_floor_loop =
                        ((*self.sample).amplitude_that_reaches_noise_floor / self.synth_gain as f64)
                            as f32
                } else {
                    self.amplitude_that_reaches_noise_floor_loop =
                        self.amplitude_that_reaches_noise_floor_nonloop
                }
            }
        }
        if self.check_sample_sanity_flag & (1 as i32) << 1 as i32 != 0 {
            if max_index_loop - min_index_loop < 2 as i32 {
                if self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                    == FLUID_LOOP_UNTIL_RELEASE as i32
                    || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                        == FLUID_LOOP_DURING_RELEASE as i32
                {
                    self.gen[GEN_SAMPLEMODE as i32 as usize].val = FLUID_UNLOOPED as i32 as f64
                }
            }
            self.phase = (self.start as u64) << 32 as i32
        }
        if self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32 == FLUID_LOOP_UNTIL_RELEASE as i32
            && self.volenv_section < FLUID_VOICE_ENVRELEASE as i32
            || self.gen[GEN_SAMPLEMODE as i32 as usize].val as i32
                == FLUID_LOOP_DURING_RELEASE as i32
        {
            let index_in_sample: i32 = (self.phase >> 32 as i32) as u32 as i32;
            if index_in_sample >= self.loopend {
                self.phase = (self.loopstart as u64) << 32 as i32
            }
        }
        self.check_sample_sanity_flag = 0 as i32;
    }

    pub fn set_param(&mut self, gen: u16, nrpn_value: f32, abs: i32) {
        self.gen[gen as usize].nrpn = nrpn_value as f64;
        self.gen[gen as usize].flags = if abs != 0 {
            GEN_ABS_NRPN as i32
        } else {
            GEN_SET as i32
        } as u8;
        unsafe {
            self.update_param(gen as _);
        }
    }

    pub fn set_gain(&mut self, mut gain: f64) {
        if gain < 0.0000001 {
            gain = 0.0000001;
        }
        let gain = gain as f32;
        self.synth_gain = gain;
        self.amp_left = fluid_pan(self.pan, 1 as i32) * gain / 32768.0f32;
        self.amp_right = fluid_pan(self.pan, 0 as i32) * gain / 32768.0f32;
        self.amp_reverb = self.reverb_send * gain / 32768.0f32;
        self.amp_chorus = self.chorus_send * gain / 32768.0f32;
    }

    pub unsafe fn write(
        voice: &mut Voice,
        min_note_length_ticks: u32,
        dsp_left_buf: *mut f32,
        dsp_right_buf: *mut f32,
        dsp_reverb_buf: *mut f32,
        dsp_chorus_buf: *mut f32,
    ) -> i32 {
        let current_block: u64;
        let mut fres;
        let target_amp;
        let count;
        let mut dsp_buf: [f32; 64] = [0.; 64];
        let mut x;
        if !(voice.status as i32 == FLUID_VOICE_ON as i32
            || voice.status as i32 == FLUID_VOICE_SUSTAINED as i32)
        {
            return FLUID_OK as i32;
        }
        if voice.sample.is_null() {
            voice.off();
            return FLUID_OK as i32;
        }
        if voice.noteoff_ticks != 0 as i32 as u32 && voice.ticks >= voice.noteoff_ticks {
            voice.noteoff(min_note_length_ticks);
        }
        voice.check_sample_sanity();
        let mut env_data = &mut *voice
            .volenv_data
            .as_mut_ptr()
            .offset((*voice).volenv_section as isize) as *mut EnvData;
        while (*voice).volenv_count >= (*env_data).count {
            // If we're switching envelope stages from decay to sustain, force the value to be the end value of the previous stage
            if !env_data.is_null() && (*voice).volenv_section == FLUID_VOICE_ENVDECAY as i32 {
                (*voice).volenv_val = (*env_data).min * (*env_data).coeff
            }
            (*voice).volenv_section += 1;
            env_data = &mut *(*voice)
                .volenv_data
                .as_mut_ptr()
                .offset((*voice).volenv_section as isize) as *mut EnvData;
            (*voice).volenv_count = 0 as i32 as u32
        }
        x = (*env_data).coeff * (*voice).volenv_val + (*env_data).incr;
        if x < (*env_data).min {
            x = (*env_data).min;
            (*voice).volenv_section += 1;
            (*voice).volenv_count = 0 as i32 as u32
        } else if x > (*env_data).max {
            x = (*env_data).max;
            (*voice).volenv_section += 1;
            (*voice).volenv_count = 0 as i32 as u32
        }
        (*voice).volenv_val = x;
        (*voice).volenv_count = (*voice).volenv_count.wrapping_add(1);
        if (*voice).volenv_section == FLUID_VOICE_ENVFINISHED as i32 {
            voice.off();
            return FLUID_OK as i32;
        }
        env_data = &mut *(*voice)
            .modenv_data
            .as_mut_ptr()
            .offset((*voice).modenv_section as isize) as *mut EnvData;
        while (*voice).modenv_count >= (*env_data).count {
            (*voice).modenv_section += 1;
            env_data = &mut *(*voice)
                .modenv_data
                .as_mut_ptr()
                .offset((*voice).modenv_section as isize) as *mut EnvData;
            (*voice).modenv_count = 0 as i32 as u32
        }
        x = (*env_data).coeff * (*voice).modenv_val + (*env_data).incr;
        if x < (*env_data).min {
            x = (*env_data).min;
            (*voice).modenv_section += 1;
            (*voice).modenv_count = 0 as i32 as u32
        } else if x > (*env_data).max {
            x = (*env_data).max;
            (*voice).modenv_section += 1;
            (*voice).modenv_count = 0 as i32 as u32
        }
        (*voice).modenv_val = x;
        (*voice).modenv_count = (*voice).modenv_count.wrapping_add(1);
        if (*voice).ticks >= (*voice).modlfo_delay {
            (*voice).modlfo_val += (*voice).modlfo_incr;
            if (*voice).modlfo_val as f64 > 1.0f64 {
                (*voice).modlfo_incr = -(*voice).modlfo_incr;
                (*voice).modlfo_val = 2.0f32 - (*voice).modlfo_val
            } else if ((*voice).modlfo_val as f64) < -1.0f64 {
                (*voice).modlfo_incr = -(*voice).modlfo_incr;
                (*voice).modlfo_val = -2.0f32 - (*voice).modlfo_val
            }
        }
        if (*voice).ticks >= (*voice).viblfo_delay {
            (*voice).viblfo_val += (*voice).viblfo_incr;
            if (*voice).viblfo_val > 1.0f32 {
                (*voice).viblfo_incr = -(*voice).viblfo_incr;
                (*voice).viblfo_val = 2.0f32 - (*voice).viblfo_val
            } else if ((*voice).viblfo_val as f64) < -1.0f64 {
                (*voice).viblfo_incr = -(*voice).viblfo_incr;
                (*voice).viblfo_val = -2.0f32 - (*voice).viblfo_val
            }
        }
        if !((*voice).volenv_section == FLUID_VOICE_ENVDELAY as i32) {
            if (*voice).volenv_section == FLUID_VOICE_ENVATTACK as i32 {
                target_amp = fluid_atten2amp((*voice).attenuation)
                    * fluid_cb2amp((*voice).modlfo_val * -(*voice).modlfo_to_vol)
                    * (*voice).volenv_val;
                current_block = 576355610076403033;
            } else {
                let amplitude_that_reaches_noise_floor;
                let amp_max;
                target_amp = fluid_atten2amp((*voice).attenuation)
                    * fluid_cb2amp(
                        960.0f32 * (1.0f32 - (*voice).volenv_val)
                            + (*voice).modlfo_val * -(*voice).modlfo_to_vol,
                    );
                if (*voice).has_looped != 0 {
                    amplitude_that_reaches_noise_floor =
                        (*voice).amplitude_that_reaches_noise_floor_loop
                } else {
                    amplitude_that_reaches_noise_floor =
                        (*voice).amplitude_that_reaches_noise_floor_nonloop
                }
                amp_max = fluid_atten2amp((*voice).min_attenuation_c_b) * (*voice).volenv_val;
                if amp_max < amplitude_that_reaches_noise_floor {
                    voice.off();
                    current_block = 3632332525568699835;
                } else {
                    current_block = 576355610076403033;
                }
            }
            match current_block {
                3632332525568699835 => {}
                _ => {
                    (*voice).amp_incr = (target_amp - (*voice).amp) / 64 as i32 as f32;
                    if !((*voice).amp == 0.0f32 && (*voice).amp_incr == 0.0f32) {
                        (*voice).phase_incr = fluid_ct2hz_real(
                            (*voice).pitch
                                + (*voice).modlfo_val * (*voice).modlfo_to_pitch
                                + (*voice).viblfo_val * (*voice).viblfo_to_pitch
                                + (*voice).modenv_val * (*voice).modenv_to_pitch,
                        ) / (*voice).root_pitch;
                        if (*voice).phase_incr == 0 as i32 as f32 {
                            (*voice).phase_incr = 1 as i32 as f32
                        }
                        fres = fluid_ct2hz(
                            (*voice).fres
                                + (*voice).modlfo_val * (*voice).modlfo_to_fc
                                + (*voice).modenv_val * (*voice).modenv_to_fc,
                        );
                        if fres > 0.45f32 * (*voice).output_rate {
                            fres = 0.45f32 * (*voice).output_rate
                        } else if fres < 5 as i32 as f32 {
                            fres = 5 as i32 as f32
                        }
                        if f64::abs((fres - (*voice).last_fres) as f64) > 0.01f64 {
                            let omega: f32 = (2.0f64
                                * std::f64::consts::PI
                                * (fres / (*voice).output_rate) as f64)
                                as f32;
                            let sin_coeff: f32 = f64::sin(omega.into()) as f32;
                            let cos_coeff: f32 = f64::cos(omega.into()) as f32;
                            let alpha_coeff: f32 = sin_coeff / (2.0f32 * (*voice).q_lin);
                            let a0_inv: f32 = 1.0f32 / (1.0f32 + alpha_coeff);
                            let a1_temp: f32 = -2.0f32 * cos_coeff * a0_inv;
                            let a2_temp: f32 = (1.0f32 - alpha_coeff) * a0_inv;
                            let b1_temp: f32 = (1.0f32 - cos_coeff) * a0_inv * (*voice).filter_gain;
                            let b02_temp: f32 = b1_temp * 0.5f32;
                            if (*voice).filter_startup != 0 {
                                (*voice).a1 = a1_temp;
                                (*voice).a2 = a2_temp;
                                (*voice).b02 = b02_temp;
                                (*voice).b1 = b1_temp;
                                (*voice).filter_coeff_incr_count = 0 as i32;
                                (*voice).filter_startup = 0 as i32
                            //       printf("Setting initial filter coefficients.\n");
                            } else {
                                (*voice).a1_incr = (a1_temp - (*voice).a1) / 64 as i32 as f32;
                                (*voice).a2_incr = (a2_temp - (*voice).a2) / 64 as i32 as f32;
                                (*voice).b02_incr = (b02_temp - (*voice).b02) / 64 as i32 as f32;
                                (*voice).b1_incr = (b1_temp - (*voice).b1) / 64 as i32 as f32;
                                (*voice).filter_coeff_incr_count = 64 as i32
                            }
                            (*voice).last_fres = fres
                        }
                        (*voice).dsp_buf = dsp_buf.as_mut_ptr();
                        match (*voice).interp_method {
                            InterpMethod::None => count = fluid_dsp_float_interpolate_none(voice),
                            InterpMethod::Linear => {
                                count = fluid_dsp_float_interpolate_linear(voice)
                            }
                            InterpMethod::FourthOrder => {
                                count = fluid_dsp_float_interpolate_4th_order(voice)
                            }
                            InterpMethod::SeventhOrder => {
                                count = fluid_dsp_float_interpolate_7th_order(voice)
                            }
                        }
                        if count > 0 as i32 {
                            Voice::effects(
                                voice,
                                count,
                                dsp_left_buf,
                                dsp_right_buf,
                                dsp_reverb_buf,
                                dsp_chorus_buf,
                            );
                        }
                        if count < 64 as i32 {
                            voice.off();
                        }
                    }
                }
            }
        }
        (*voice).ticks = (*voice).ticks.wrapping_add(64 as i32 as u32);
        return FLUID_OK as i32;
    }
    //removed inline
    #[inline]
    unsafe fn effects(
        mut voice: *mut Voice,
        count: i32,
        dsp_left_buf: *mut f32,
        dsp_right_buf: *mut f32,
        dsp_reverb_buf: *mut f32,
        dsp_chorus_buf: *mut f32,
    ) {
        let mut dsp_hist1: f32 = (*voice).hist1;
        let mut dsp_hist2: f32 = (*voice).hist2;
        let mut dsp_a1: f32 = (*voice).a1;
        let mut dsp_a2: f32 = (*voice).a2;
        let mut dsp_b02: f32 = (*voice).b02;
        let mut dsp_b1: f32 = (*voice).b1;
        let dsp_a1_incr: f32 = (*voice).a1_incr;
        let dsp_a2_incr: f32 = (*voice).a2_incr;
        let dsp_b02_incr: f32 = (*voice).b02_incr;
        let dsp_b1_incr: f32 = (*voice).b1_incr;
        let mut dsp_filter_coeff_incr_count: i32 = (*voice).filter_coeff_incr_count;
        let dsp_buf: *mut f32 = (*voice).dsp_buf;
        let mut dsp_centernode;
        let mut dsp_i;
        let mut v;
        if f64::abs(dsp_hist1 as f64) < 1e-20f64 {
            dsp_hist1 = 0.0f32
        }
        if dsp_filter_coeff_incr_count > 0 as i32 {
            dsp_i = 0 as i32;
            while dsp_i < count {
                dsp_centernode =
                    *dsp_buf.offset(dsp_i as isize) - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                *dsp_buf.offset(dsp_i as isize) =
                    dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
                let fresh0 = dsp_filter_coeff_incr_count;
                dsp_filter_coeff_incr_count = dsp_filter_coeff_incr_count - 1;
                if fresh0 > 0 as i32 {
                    dsp_a1 += dsp_a1_incr;
                    dsp_a2 += dsp_a2_incr;
                    dsp_b02 += dsp_b02_incr;
                    dsp_b1 += dsp_b1_incr
                }
                dsp_i += 1
            }
        } else {
            dsp_i = 0 as i32;
            while dsp_i < count {
                dsp_centernode =
                    *dsp_buf.offset(dsp_i as isize) - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                *dsp_buf.offset(dsp_i as isize) =
                    dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
                dsp_i += 1
            }
        }
        if -0.5f64 < (*voice).pan as f64 && ((*voice).pan as f64) < 0.5f64 {
            dsp_i = 0 as i32;
            while dsp_i < count {
                v = (*voice).amp_left * *dsp_buf.offset(dsp_i as isize);
                let ref mut fresh1 = *dsp_left_buf.offset(dsp_i as isize);
                *fresh1 += v;
                let ref mut fresh2 = *dsp_right_buf.offset(dsp_i as isize);
                *fresh2 += v;
                dsp_i += 1
            }
        } else {
            if (*voice).amp_left as f64 != 0.0f64 {
                dsp_i = 0 as i32;
                while dsp_i < count {
                    let ref mut fresh3 = *dsp_left_buf.offset(dsp_i as isize);
                    *fresh3 += (*voice).amp_left * *dsp_buf.offset(dsp_i as isize);
                    dsp_i += 1
                }
            }
            if (*voice).amp_right as f64 != 0.0f64 {
                dsp_i = 0 as i32;
                while dsp_i < count {
                    let ref mut fresh4 = *dsp_right_buf.offset(dsp_i as isize);
                    *fresh4 += (*voice).amp_right * *dsp_buf.offset(dsp_i as isize);
                    dsp_i += 1
                }
            }
        }
        if !dsp_reverb_buf.is_null() && (*voice).amp_reverb as f64 != 0.0f64 {
            dsp_i = 0 as i32;
            while dsp_i < count {
                let ref mut fresh5 = *dsp_reverb_buf.offset(dsp_i as isize);
                *fresh5 += (*voice).amp_reverb * *dsp_buf.offset(dsp_i as isize);
                dsp_i += 1
            }
        }
        if !dsp_chorus_buf.is_null() && (*voice).amp_chorus != 0 as i32 as f32 {
            dsp_i = 0 as i32;
            while dsp_i < count {
                let ref mut fresh6 = *dsp_chorus_buf.offset(dsp_i as isize);
                *fresh6 += (*voice).amp_chorus * *dsp_buf.offset(dsp_i as isize);
                dsp_i += 1
            }
        }
        (*voice).hist1 = dsp_hist1;
        (*voice).hist2 = dsp_hist2;
        (*voice).a1 = dsp_a1;
        (*voice).a2 = dsp_a2;
        (*voice).b02 = dsp_b02;
        (*voice).b1 = dsp_b1;
        (*voice).filter_coeff_incr_count = dsp_filter_coeff_incr_count;
    }

    pub fn calculate_hold_decay_buffers(
        &mut self,
        gen_base: i32,
        gen_key2base: i32,
        is_decay: i32,
    ) -> i32 {
        let mut timecents = (self.gen[gen_base as usize].val
            + self.gen[gen_base as usize].mod_0
            + self.gen[gen_base as usize].nrpn)
            + (self.gen[gen_key2base as usize].val
                + self.gen[gen_key2base as usize].mod_0
                + self.gen[gen_key2base as usize].nrpn)
                * (60.0 - self.key as f64);
        if is_decay != 0 {
            if timecents > 8000.0 {
                timecents = 8000.0;
            }
        } else {
            if timecents > 5000.0 {
                timecents = 5000.0;
            }
            if timecents <= -32768.0 {
                return 0;
            }
        }
        if (timecents as f64) < -12000.0 {
            timecents = -12000.0;
        }
        let seconds = fluid_tc2sec(timecents);
        let buffers = ((self.output_rate as f64 * seconds / 64.0) + 0.5) as i32;
        return buffers;
    }
}
