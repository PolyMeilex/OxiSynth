pub mod dsp_float;

use crate::gen::GenParam;

use super::channel::{Channel, ChannelId, InterpMethod};
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

use super::gen::{self, Gen};
use super::modulator::Mod;
use super::soundfont::Sample;
use std::rc::Rc;

type Phase = u64;
type ModFlags = u32;
const FLUID_MOD_CC: ModFlags = 16;
const FLUID_MOD_BIPOLAR: ModFlags = 2;
type ModSrc = u32;
const FLUID_MOD_PITCHWHEEL: ModSrc = 14;
type GenType = u32;
const GEN_PITCH: GenType = 59;
const GEN_EXCLUSIVECLASS: GenType = 57;
const GEN_SCALETUNE: GenType = 56;
const GEN_SAMPLEMODE: GenType = 54;

const GEN_ATTENUATION: GenType = 48;
const GEN_KEYTOVOLENVDECAY: GenType = 40;
const GEN_KEYTOVOLENVHOLD: GenType = 39;
const GEN_VOLENVRELEASE: GenType = 38;
const GEN_VOLENVDECAY: GenType = 36;
const GEN_VOLENVHOLD: GenType = 35;
const GEN_KEYTOMODENVDECAY: GenType = 32;
const GEN_KEYTOMODENVHOLD: GenType = 31;
const GEN_MODENVRELEASE: GenType = 30;
const GEN_MODENVDECAY: GenType = 28;
const GEN_MODENVHOLD: GenType = 27;
type GenFlags = u32;
const GEN_ABS_NRPN: GenFlags = 2;
const GEN_SET: GenFlags = 1;

const FLUID_VOICE_ENVRELEASE: VoiceEnvelopeIndex = 5;
const FLUID_VOICE_ENVDECAY: VoiceEnvelopeIndex = 3;
const FLUID_VOICE_ENVHOLD: VoiceEnvelopeIndex = 2;
pub(crate) const FLUID_VOICE_ENVATTACK: VoiceEnvelopeIndex = 1;
const FLUID_VOICE_ENVDELAY: VoiceEnvelopeIndex = 0;

pub type FluidVoiceAddMod = u32;
const FLUID_VOICE_ADD: FluidVoiceAddMod = 1;
const FLUID_VOICE_OVERWRITE: FluidVoiceAddMod = 0;

const FLUID_OK: i32 = 0;

#[derive(PartialEq)]
pub enum VoiceStatus {
    Clean,
    On,
    Sustained,
    Off,
}

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
pub struct VoiceId(pub(crate) usize);

pub struct Voice {
    pub(crate) id: u32,
    pub(crate) status: VoiceStatus,
    pub(crate) chan: u8,
    pub(crate) key: u8,
    pub(crate) vel: u8,
    channel_id: Option<ChannelId>,
    pub(crate) gen: [Gen; 60],
    mod_0: [Mod; 64],
    mod_count: usize,
    pub(crate) has_looped: i32,
    pub(crate) sample: Option<Rc<Sample>>,
    check_sample_sanity_flag: i32,
    output_rate: f32,
    pub(crate) start_time: u32,
    pub(crate) ticks: u32,
    noteoff_ticks: u32,
    pub(crate) amp: f32,
    pub(crate) phase: Phase,
    pub(crate) phase_incr: f32,
    pub(crate) amp_incr: f32,
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
            status: VoiceStatus::Clean,
            chan: 0xff,
            key: 0,
            vel: 0,
            channel_id: None,
            gen: [Gen::default(); 60],
            mod_0: [Mod::default(); 64],
            mod_count: 0,
            has_looped: 0,
            sample: None,
            check_sample_sanity_flag: 0,
            output_rate,
            start_time: 0,
            ticks: 0,
            noteoff_ticks: 0,
            amp: 0.0,
            phase: 0 as Phase,
            phase_incr: 0.0,
            amp_incr: 0.0,
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

    pub(crate) fn init(
        &mut self,
        sample: Rc<Sample>,
        channel: &Channel,
        channel_id: ChannelId,
        key: u8,
        vel: i32,
        id: u32,
        start_time: u32,
        gain: f32,
    ) {
        self.id = id;
        self.chan = channel.get_num();
        self.key = key as u8;
        self.vel = vel as u8;
        self.interp_method = channel.get_interp_method();
        self.channel_id = Some(channel_id);
        self.mod_count = 0;
        self.sample = Some(sample);
        self.start_time = start_time;
        self.ticks = 0 as i32 as u32;
        self.noteoff_ticks = 0 as i32 as u32;
        self.debug = 0 as i32;
        self.has_looped = 0 as i32;
        self.last_fres = -(1 as i32) as f32;
        self.filter_startup = 1 as i32;
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
    }

    pub(crate) fn is_available(&self) -> bool {
        self.status == VoiceStatus::Clean || self.status == VoiceStatus::Off
    }

    pub(crate) fn is_on(&self) -> bool {
        self.status == VoiceStatus::On && self.volenv_section < FLUID_VOICE_ENVRELEASE as i32
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.status == VoiceStatus::On || self.status == VoiceStatus::Sustained
    }

    pub(crate) fn add_mod(&mut self, mod_0: &Mod, mode: u32) {
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
        if mode == FLUID_VOICE_ADD {
            for m in self.mod_0.iter_mut().take(self.mod_count) {
                if m.test_identity(mod_0) {
                    m.amount += mod_0.amount;
                    return;
                }
            }
        } else if mode == FLUID_VOICE_OVERWRITE {
            for m in self.mod_0.iter_mut().take(self.mod_count) {
                if m.test_identity(mod_0) {
                    m.amount = mod_0.amount;
                    return;
                }
            }
        }
        if self.mod_count < 64 {
            let fresh7 = self.mod_count;
            self.mod_count = self.mod_count + 1;
            self.mod_0[fresh7 as usize] = mod_0.clone();
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

    pub(crate) fn kill_excl(&mut self) {
        if !self.is_playing() {
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
        self.update_param(GenParam::VolEnvRelease);
        self.gen_set(GEN_MODENVRELEASE as i32, -200.0);
        self.update_param(GenParam::ModEnvRelease);
    }

    pub(crate) fn start(&mut self, channels: &[Channel]) {
        self.calculate_runtime_synthesis_parameters(channels);
        self.check_sample_sanity_flag = (1 as i32) << 1 as i32;
        self.status = VoiceStatus::On;
    }

    pub(crate) fn noteoff(&mut self, channels: &[Channel], min_note_length_ticks: u32) -> i32 {
        let at_tick;
        at_tick = min_note_length_ticks;
        if at_tick > self.ticks {
            self.noteoff_ticks = at_tick;
            return FLUID_OK as i32;
        }

        if let Some(channel_id) = &self.channel_id {
            if channels[channel_id.0].cc[SUSTAIN_SWITCH as i32 as usize] >= 64 {
                self.status = VoiceStatus::Sustained;
            } else {
                if self.volenv_section == FLUID_VOICE_ENVATTACK as i32 {
                    if self.volenv_val > 0 as i32 as f32 {
                        let lfo: f32 = self.modlfo_val * -self.modlfo_to_vol;
                        let amp: f32 = (self.volenv_val as f64
                            * f64::powf(10.0f64, (lfo / -(200 as i32) as f32) as f64))
                            as f32;
                        let mut env_value: f32 =
                            -((-(200 as i32) as f64 * f64::ln(amp as f64) / f64::ln(10.0f64)
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
        }
        return FLUID_OK as i32;
    }

    pub(crate) fn modulate(&mut self, channels: &[Channel], cc: i32, ctrl: u16) -> i32 {
        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &mut self.mod_0[i];
            if mod_0.src1 == ctrl as u8 && mod_0.flags1 as i32 & FLUID_MOD_CC as i32 != 0 && cc != 0
                || mod_0.src1 == ctrl as u8
                    && mod_0.flags1 as i32 & FLUID_MOD_CC as i32 == 0
                    && cc == 0
                || (mod_0.src2 == ctrl as u8
                    && mod_0.flags2 as i32 & FLUID_MOD_CC as i32 != 0
                    && cc != 0
                    || mod_0.src2 == ctrl as u8
                        && mod_0.flags2 as i32 & FLUID_MOD_CC as i32 == 0
                        && cc == 0)
            {
                let gen = mod_0.get_dest();
                let mut modval = 0.0;

                let mut k = 0;
                while k < self.mod_count {
                    if self.mod_0[k].dest == gen {
                        modval += self.mod_0[k]
                            .get_value(&channels[self.channel_id.as_ref().unwrap().0], self);
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

    pub(crate) fn modulate_all(&mut self, channels: &[Channel]) -> i32 {
        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &mut self.mod_0[i];
            let gen = mod_0.get_dest();
            let mut modval = 0.0f32;

            let mut k = 0;
            while k < self.mod_count {
                if self.mod_0[k].dest == gen {
                    modval += self.mod_0[k]
                        .get_value(&channels[self.channel_id.as_ref().unwrap().0], self)
                }
                k += 1
            }
            self.gen[gen as usize].mod_0 = modval as f64;
            self.update_param(gen);
            i += 1
        }

        FLUID_OK
    }

    pub(crate) fn off(&mut self) {
        self.chan = 0xff as i32 as u8;
        self.volenv_section = FLUID_VOICE_ENVFINISHED as i32;
        self.volenv_count = 0 as i32 as u32;
        self.modenv_section = FLUID_VOICE_ENVFINISHED as i32;
        self.modenv_count = 0 as i32 as u32;
        self.status = VoiceStatus::Off;
        if self.sample.is_some() {
            self.sample = None;
        }
    }

    pub(crate) fn get_channel(&self) -> Option<&ChannelId> {
        self.channel_id.as_ref()
    }

    fn get_lower_boundary_for_attenuation(&mut self, channels: &[Channel]) -> f32 {
        let mut possible_att_reduction_c_b: f32 = 0.0;
        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &self.mod_0[i as usize];
            if mod_0.dest as i32 == GEN_ATTENUATION as i32
                && (mod_0.flags1 as i32 & FLUID_MOD_CC as i32 != 0
                    || mod_0.flags2 as i32 & FLUID_MOD_CC as i32 != 0)
            {
                let current_val: f32 =
                    mod_0.get_value(&channels[self.channel_id.as_ref().unwrap().0], self);
                let mut v: f32 = f64::abs(mod_0.amount) as f32;
                if mod_0.src1 as i32 == FLUID_MOD_PITCHWHEEL as i32
                    || mod_0.flags1 as i32 & FLUID_MOD_BIPOLAR as i32 != 0
                    || mod_0.flags2 as i32 & FLUID_MOD_BIPOLAR as i32 != 0
                    || mod_0.amount < 0 as i32 as f64
                {
                    v = (v as f64 * -1.0f64) as f32
                } else {
                    v = 0.0;
                }
                if current_val > v {
                    possible_att_reduction_c_b += current_val - v
                }
            }
            i += 1
        }
        let mut lower_bound = self.attenuation - possible_att_reduction_c_b;
        if lower_bound < 0.0 {
            lower_bound = 0.0;
        }

        lower_bound
    }

    fn calculate_runtime_synthesis_parameters(&mut self, channels: &[Channel]) -> i32 {
        let list_of_generators_to_initialize: [GenParam; 34] = [
            GenParam::StartAddrOfs,
            GenParam::EndAddrOfs,
            GenParam::StartLoopAddrOfs,
            GenParam::EndLoopAddrOfs,
            GenParam::ModLfoToPitch,
            GenParam::VibLfoToPitch,
            GenParam::ModEnvToPitch,
            GenParam::FilterFc,
            GenParam::FilterQ,
            GenParam::ModLfoToFilterFc,
            GenParam::ModEnvToFilterFc,
            GenParam::ModLfoToVol,
            GenParam::ChorusSend,
            GenParam::ReverbSend,
            GenParam::Pan,
            GenParam::ModLfoDelay,
            GenParam::ModLfoFreq,
            GenParam::VibLfoDelay,
            GenParam::VibLfoFreq,
            GenParam::ModEnvDelay,
            GenParam::ModEnvAttack,
            GenParam::ModEnvHold,
            GenParam::ModEnvDecay,
            GenParam::ModEnvRelease,
            GenParam::VolEnvDelay,
            GenParam::VolEnvAttack,
            GenParam::VolEnvHold,
            GenParam::VolEnvDecay,
            GenParam::VolEnvRelease,
            GenParam::KeyNum,
            GenParam::Velocity,
            GenParam::Attenuation,
            GenParam::OverrideRootKey,
            GenParam::Pitch,
        ];

        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &self.mod_0[i];
            let modval: f32 = mod_0.get_value(&channels[self.channel_id.as_ref().unwrap().0], self);
            let dest_gen_index = mod_0.dest as usize;
            let mut dest_gen = &mut self.gen[dest_gen_index];
            dest_gen.mod_0 += modval as f64;
            i += 1
        }
        let tuning = &channels[self.channel_id.as_ref().unwrap().0].tuning;
        if let Some(tuning) = tuning {
            self.gen[GEN_PITCH as usize].val = tuning.pitch[60]
                + self.gen[GEN_SCALETUNE as usize].val / 100.0f32 as f64
                    * (tuning.pitch[self.key as usize] - tuning.pitch[60])
        } else {
            self.gen[GEN_PITCH as usize].val = self.gen[GEN_SCALETUNE as usize].val
                * (self.key as i32 as f32 - 60.0f32) as f64
                + (100.0f32 * 60.0f32) as f64
        }

        for gen in list_of_generators_to_initialize.iter() {
            self.update_param(*gen);
        }

        self.min_attenuation_c_b = self.get_lower_boundary_for_attenuation(channels);
        return FLUID_OK as i32;
    }

    pub fn check_sample_sanity(&mut self) {
        let min_index_nonloop: i32 = (self.sample.as_ref().unwrap()).start as i32;
        let max_index_nonloop: i32 = (self.sample.as_ref().unwrap()).end as i32;
        let min_index_loop: i32 = (self.sample.as_ref().unwrap()).start as i32 + 0 as i32;
        let max_index_loop: i32 = (self.sample.as_ref().unwrap()).end as i32 - 0 as i32 + 1 as i32;
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
            if self.loopstart >= self.sample.as_ref().unwrap().loopstart as i32
                && self.loopend <= self.sample.as_ref().unwrap().loopend as i32
            {
                if self
                    .sample
                    .as_ref()
                    .unwrap()
                    .amplitude_that_reaches_noise_floor_is_valid
                    != 0
                {
                    self.amplitude_that_reaches_noise_floor_loop = (self
                        .sample
                        .as_ref()
                        .unwrap()
                        .amplitude_that_reaches_noise_floor
                        / self.synth_gain as f64)
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

    pub fn set_param(&mut self, gen: GenParam, nrpn_value: f32, abs: i32) {
        self.gen[gen as usize].nrpn = nrpn_value as f64;
        self.gen[gen as usize].flags = if abs != 0 {
            GEN_ABS_NRPN as i32
        } else {
            GEN_SET as i32
        } as u8;
        self.update_param(gen);
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

    pub fn write(
        &mut self,
        channels: &[Channel],
        min_note_length_ticks: u32,
        dsp_left_buf: &mut [f32],
        dsp_right_buf: &mut [f32],
        fx_left_buf: &mut Vec<[f32; 64]>,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        let current_block: u64;
        let mut fres;
        let target_amp; /* target amplitude */

        let mut dsp_buf: [f32; 64] = [0.; 64];

        /* make sure we're playing and that we have sample data */
        if !self.is_playing() {
            return;
        }

        /******************* sample **********************/
        if self.sample.is_none() {
            self.off();
            return;
        }
        if self.noteoff_ticks != 0 as i32 as u32 && self.ticks >= self.noteoff_ticks {
            self.noteoff(channels, min_note_length_ticks);
        }

        /* Range checking for sample- and loop-related parameters
         * Initial phase is calculated here*/
        self.check_sample_sanity();

        /* skip to the next section of the envelope if necessary */
        let mut env_data = &self.volenv_data[self.volenv_section as usize];
        while self.volenv_count >= env_data.count {
            // If we're switching envelope stages from decay to sustain, force the value to be the end value of the previous stage
            if self.volenv_section == FLUID_VOICE_ENVDECAY as i32 {
                self.volenv_val = env_data.min * env_data.coeff
            }

            self.volenv_section += 1;
            env_data = &self.volenv_data[self.volenv_section as usize];
            self.volenv_count = 0;
        }

        /* calculate the envelope value and check for valid range */
        let mut x = env_data.coeff * self.volenv_val + env_data.incr;
        if x < env_data.min {
            x = env_data.min;
            self.volenv_section += 1;
            self.volenv_count = 0 as i32 as u32
        } else if x > env_data.max {
            x = env_data.max;
            self.volenv_section += 1;
            self.volenv_count = 0 as i32 as u32
        }

        self.volenv_val = x;
        self.volenv_count = self.volenv_count.wrapping_add(1);

        if self.volenv_section == FLUID_VOICE_ENVFINISHED as i32 {
            self.off();
            return;
        }

        /******************* mod env **********************/

        let mut env_data = &self.modenv_data[self.modenv_section as usize];

        /* skip to the next section of the envelope if necessary */
        while self.modenv_count >= env_data.count {
            self.modenv_section += 1;
            env_data = &self.modenv_data[self.modenv_section as usize];
            self.modenv_count = 0;
        }

        /* calculate the envelope value and check for valid range */
        let mut x = env_data.coeff * self.modenv_val + env_data.incr;

        if x < env_data.min {
            x = env_data.min;
            self.modenv_section += 1;
            self.modenv_count = 0 as i32 as u32
        } else if x > env_data.max {
            x = env_data.max;
            self.modenv_section += 1;
            self.modenv_count = 0 as i32 as u32
        }

        self.modenv_val = x;
        self.modenv_count = self.modenv_count.wrapping_add(1);

        /******************* mod lfo **********************/

        if self.ticks >= self.modlfo_delay {
            self.modlfo_val += self.modlfo_incr;
            if self.modlfo_val as f64 > 1.0f64 {
                self.modlfo_incr = -self.modlfo_incr;
                self.modlfo_val = 2.0f32 - self.modlfo_val
            } else if ((self).modlfo_val as f64) < -1.0f64 {
                self.modlfo_incr = -self.modlfo_incr;
                self.modlfo_val = -2.0f32 - self.modlfo_val
            }
        }

        /******************* vib lfo **********************/
        if self.ticks >= self.viblfo_delay {
            self.viblfo_val += self.viblfo_incr;
            if self.viblfo_val > 1.0f32 {
                self.viblfo_incr = -self.viblfo_incr;
                self.viblfo_val = 2.0f32 - self.viblfo_val
            } else if (self.viblfo_val as f64) < -1.0f64 {
                self.viblfo_incr = -self.viblfo_incr;
                self.viblfo_val = -2.0f32 - self.viblfo_val
            }
        }

        /******************* amplitude **********************/

        /* calculate final amplitude
         * - initial gain
         * - amplitude envelope
         */

        if !(self.volenv_section == FLUID_VOICE_ENVDELAY as i32) {
            if self.volenv_section == FLUID_VOICE_ENVATTACK as i32 {
                /* the envelope is in the attack section: ramp linearly to max value.
                 * A positive modlfo_to_vol should increase volume (negative attenuation).
                 */
                target_amp = fluid_atten2amp(self.attenuation)
                    * fluid_cb2amp(self.modlfo_val * -self.modlfo_to_vol)
                    * self.volenv_val;
                current_block = 576355610076403033;
            } else {
                let amplitude_that_reaches_noise_floor;
                let amp_max;
                target_amp = fluid_atten2amp(self.attenuation)
                    * fluid_cb2amp(
                        960.0f32 * (1.0f32 - self.volenv_val)
                            + self.modlfo_val * -self.modlfo_to_vol,
                    );

                /* We turn off a voice, if the volume has dropped low enough. */

                /* A voice can be turned off, when an estimate for the volume
                 * (upper bound) falls below that volume, that will drop the
                 * sample below the noise floor.
                 */

                /* If the loop amplitude is known, we can use it if the voice loop is within
                 * the sample loop
                 */

                /* Is the playing pointer already in the loop? */
                if self.has_looped != 0 {
                    amplitude_that_reaches_noise_floor =
                        self.amplitude_that_reaches_noise_floor_loop
                } else {
                    amplitude_that_reaches_noise_floor =
                        self.amplitude_that_reaches_noise_floor_nonloop
                }

                /* voice->attenuation_min is a lower boundary for the attenuation
                 * now and in the future (possibly 0 in the worst case).  Now the
                 * amplitude of sample and volenv cannot exceed amp_max (since
                 * volenv_val can only drop):
                 */
                amp_max = fluid_atten2amp(self.min_attenuation_c_b) * self.volenv_val;

                /* And if amp_max is already smaller than the known amplitude,
                 * which will attenuate the sample below the noise floor, then we
                 * can safely turn off the voice. Duh. */
                if amp_max < amplitude_that_reaches_noise_floor {
                    self.off();
                    //  goto post_process;
                    current_block = 3632332525568699835;
                } else {
                    current_block = 576355610076403033;
                }
            }
            match current_block {
                3632332525568699835 => {}
                _ => {
                    /* Volume increment to go from voice->amp to target_amp in FLUID_BUFSIZE steps */
                    self.amp_incr = (target_amp - self.amp) / 64 as i32 as f32;
                    /* no volume and not changing? - No need to process */
                    if !(self.amp == 0.0f32 && self.amp_incr == 0.0f32) {
                        /* Calculate the number of samples, that the DSP loop advances
                         * through the original waveform with each step in the output
                         * buffer. It is the ratio between the frequencies of original
                         * waveform and output waveform.*/
                        self.phase_incr = fluid_ct2hz_real(
                            self.pitch
                                + self.modlfo_val * self.modlfo_to_pitch
                                + self.viblfo_val * self.viblfo_to_pitch
                                + self.modenv_val * self.modenv_to_pitch,
                        ) / self.root_pitch;

                        /* if phase_incr is not advancing, set it to the minimum fraction value (prevent stuckage) */
                        if self.phase_incr == 0 as i32 as f32 {
                            self.phase_incr = 1 as i32 as f32
                        }

                        /*************** resonant filter ******************/

                        /* calculate the frequency of the resonant filter in Hz */
                        fres = fluid_ct2hz(
                            self.fres
                                + self.modlfo_val * self.modlfo_to_fc
                                + self.modenv_val * self.modenv_to_fc,
                        );

                        /* FIXME - Still potential for a click during turn on, can we interpolate
                        between 20khz cutoff and 0 Q? */

                        /* I removed the optimization of turning the filter off when the
                         * resonance frequence is above the maximum frequency. Instead, the
                         * filter frequency is set to a maximum of 0.45 times the sampling
                         * rate. For a 44100 kHz sampling rate, this amounts to 19845
                         * Hz. The reason is that there were problems with anti-aliasing when the
                         * synthesizer was run at lower sampling rates. Thanks to Stephan
                         * Tassart for pointing me to this bug. By turning the filter on and
                         * clipping the maximum filter frequency at 0.45*srate, the filter
                         * is used as an anti-aliasing filter. */
                        if fres > 0.45f32 * self.output_rate {
                            fres = 0.45f32 * self.output_rate
                        } else if fres < 5 as i32 as f32 {
                            fres = 5 as i32 as f32
                        }

                        /* if filter enabled and there is a significant frequency change.. */
                        if f64::abs((fres - self.last_fres) as f64) > 0.01f64 {
                            /* The filter coefficients have to be recalculated (filter
                             * parameters have changed). Recalculation for various reasons is
                             * forced by setting last_fres to -1.  The flag filter_startup
                             * indicates, that the DSP loop runs for the first time, in this
                             * case, the filter is set directly, instead of smoothly fading
                             * between old and new settings.
                             *
                             * Those equations from Robert Bristow-Johnson's `Cookbook
                             * formulae for audio EQ biquad filter coefficients', obtained
                             * from Harmony-central.com / Computer / Programming. They are
                             * the result of the bilinear transform on an analogue filter
                             * prototype. To quote, `BLT frequency warping has been taken
                             * into account for both significant frequency relocation and for
                             * bandwidth readjustment'. */

                            let omega: f32 =
                                (2.0f64 * std::f64::consts::PI * (fres / self.output_rate) as f64)
                                    as f32;
                            let sin_coeff: f32 = f64::sin(omega.into()) as f32;
                            let cos_coeff: f32 = f64::cos(omega.into()) as f32;
                            let alpha_coeff: f32 = sin_coeff / (2.0f32 * (self).q_lin);
                            let a0_inv: f32 = 1.0f32 / (1.0f32 + alpha_coeff);

                            /* Calculate the filter coefficients. All coefficients are
                             * normalized by a0. Think of `a1' as `a1/a0'.
                             *
                             * Here a couple of multiplications are saved by reusing common expressions.
                             * The original equations should be:
                             *  voice->b0=(1.-cos_coeff)*a0_inv*0.5*voice->filter_gain;
                             *  voice->b1=(1.-cos_coeff)*a0_inv*voice->filter_gain;
                             *  voice->b2=(1.-cos_coeff)*a0_inv*0.5*voice->filter_gain; */
                            let a1_temp: f32 = -2.0f32 * cos_coeff * a0_inv;
                            let a2_temp: f32 = (1.0f32 - alpha_coeff) * a0_inv;
                            let b1_temp: f32 = (1.0f32 - cos_coeff) * a0_inv * (self).filter_gain;
                            /* both b0 -and- b2 */
                            let b02_temp: f32 = b1_temp * 0.5f32;

                            if self.filter_startup != 0 {
                                /* The filter is calculated, because the voice was started up.
                                 * In this case set the filter coefficients without delay.
                                 */
                                self.a1 = a1_temp;
                                self.a2 = a2_temp;
                                self.b02 = b02_temp;
                                self.b1 = b1_temp;
                                self.filter_coeff_incr_count = 0 as i32;
                                self.filter_startup = 0 as i32
                            } else {
                                /* The filter frequency is changed.  Calculate an increment
                                 * factor, so that the new setting is reached after one buffer
                                 * length. x_incr is added to the current value FLUID_BUFSIZE
                                 * times. The length is arbitrarily chosen. Longer than one
                                 * buffer will sacrifice some performance, though.  Note: If
                                 * the filter is still too 'grainy', then increase this number
                                 * at will.
                                 */
                                self.a1_incr = (a1_temp - (self).a1) / 64 as i32 as f32;
                                self.a2_incr = (a2_temp - (self).a2) / 64 as i32 as f32;
                                self.b02_incr = (b02_temp - (self).b02) / 64 as i32 as f32;
                                self.b1_incr = (b1_temp - (self).b1) / 64 as i32 as f32;
                                /* Have to add the increments filter_coeff_incr_count times. */
                                self.filter_coeff_incr_count = 64 as i32
                            }
                            self.last_fres = fres
                        }

                        let count = match self.interp_method {
                            InterpMethod::None => self.dsp_float_interpolate_none(&mut dsp_buf),
                            InterpMethod::Linear => self.dsp_float_interpolate_linear(&mut dsp_buf),
                            InterpMethod::FourthOrder => {
                                self.dsp_float_interpolate_4th_order(&mut dsp_buf)
                            }
                            InterpMethod::SeventhOrder => {
                                self.dsp_float_interpolate_7th_order(&mut dsp_buf)
                            }
                        };

                        if count > 0 {
                            self.effects(
                                &mut dsp_buf,
                                count,
                                dsp_left_buf,
                                dsp_right_buf,
                                fx_left_buf,
                                reverb_active,
                                chorus_active,
                            );
                        }
                        /* turn off voice if short count (sample ended and not looping) */
                        if count < 64 {
                            self.off();
                        }
                    }
                }
            }
        }
        self.ticks = self.ticks.wrapping_add(64 as u32);
    }
    //removed inline
    #[inline]
    fn effects(
        &mut self,
        dsp_buf: &mut [f32; 64],
        count: usize,
        dsp_left_buf: &mut [f32],
        dsp_right_buf: &mut [f32],
        fx_left_buf: &mut Vec<[f32; 64]>,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        /* IIR filter sample history */
        let mut dsp_hist1: f32 = self.hist1;
        let mut dsp_hist2: f32 = self.hist2;

        /* IIR filter coefficients */
        let mut dsp_a1: f32 = self.a1;
        let mut dsp_a2: f32 = self.a2;
        let mut dsp_b02: f32 = self.b02;
        let mut dsp_b1: f32 = self.b1;
        let dsp_a1_incr: f32 = self.a1_incr;
        let dsp_a2_incr: f32 = self.a2_incr;
        let dsp_b02_incr: f32 = self.b02_incr;
        let dsp_b1_incr: f32 = self.b1_incr;
        let mut dsp_filter_coeff_incr_count: i32 = self.filter_coeff_incr_count;

        let mut dsp_centernode;
        let mut v;

        /* filter (implement the voice filter according to SoundFont standard) */

        /* Check for denormal number (too close to zero). */
        if f64::abs(dsp_hist1 as f64) < 1e-20f64 {
            dsp_hist1 = 0.0f32; /* FIXME JMG - Is this even needed? */
        }

        /* Two versions of the filter loop. One, while the filter is
         * changing towards its new setting. The other, if the filter
         * doesn't change.
         */
        if dsp_filter_coeff_incr_count > 0 as i32 {
            /* Increment is added to each filter coefficient filter_coeff_incr_count times. */
            for dsp_i in 0..count {
                /* The filter is implemented in Direct-II form. */
                dsp_centernode = dsp_buf[dsp_i] - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                dsp_buf[dsp_i] = dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
                let fresh0 = dsp_filter_coeff_incr_count;
                dsp_filter_coeff_incr_count = dsp_filter_coeff_incr_count - 1;

                if fresh0 > 0 {
                    dsp_a1 += dsp_a1_incr;
                    dsp_a2 += dsp_a2_incr;
                    dsp_b02 += dsp_b02_incr;
                    dsp_b1 += dsp_b1_incr
                }
            }
        }
        /* The filter parameters are constant.  This is duplicated to save time. */
        else {
            /* The filter is implemented in Direct-II form. */
            for dsp_i in 0..count {
                dsp_centernode = dsp_buf[dsp_i] - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                dsp_buf[dsp_i] = dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
            }
        }

        /* pan (Copy the signal to the left and right output buffer) The voice
         * panning generator has a range of -500 .. 500.  If it is centered,
         * it's close to 0.  voice->amp_left and voice->amp_right are then the
         * same, and we can save one multiplication per voice and sample.
         */
        if -0.5f64 < (self).pan as f64 && ((self).pan as f64) < 0.5f64 {
            /* The voice is centered. Use voice->amp_left twice. */
            for dsp_i in 0..count {
                v = self.amp_left * dsp_buf[dsp_i];
                dsp_left_buf[dsp_i as usize] += v;
                dsp_right_buf[dsp_i as usize] += v;
            }
        }
        /* The voice is not centered. Stereo samples have one side zero. */
        else {
            if self.amp_left as f64 != 0.0f64 {
                for dsp_i in 0..count {
                    dsp_left_buf[dsp_i] += (self).amp_left * dsp_buf[dsp_i];
                }
            }
            if self.amp_right as f64 != 0.0f64 {
                for dsp_i in 0..count {
                    dsp_right_buf[dsp_i] += self.amp_right * dsp_buf[dsp_i];
                }
            }
        }

        if reverb_active {
            if self.amp_reverb != 0.0 {
                for dsp_i in 0..count {
                    // dsp_reverb_buf
                    fx_left_buf[0][dsp_i] += self.amp_reverb * dsp_buf[dsp_i];
                }
            }
        }

        if chorus_active {
            if self.amp_chorus != 0.0 {
                for dsp_i in 0..count {
                    // dsp_chorus_buf
                    fx_left_buf[1][dsp_i] += self.amp_chorus * dsp_buf[dsp_i];
                }
            }
        }

        self.hist1 = dsp_hist1;
        self.hist2 = dsp_hist2;
        self.a1 = dsp_a1;
        self.a2 = dsp_a2;
        self.b02 = dsp_b02;
        self.b1 = dsp_b1;
        self.filter_coeff_incr_count = dsp_filter_coeff_incr_count;
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

    /// Purpose:
    ///
    /// The value of a generator (gen) has changed.  (The different
    /// generators are listed in fluidlite.h, or in SF2.01 page 48-49)
    /// Now the dependent 'voice' parameters are calculated.
    ///
    /// fluid_voice_update_param can be called during the setup of the
    /// voice (to calculate the initial value for a voice parameter), or
    /// during its operation (a generator has been changed due to
    /// real-time parameter modifications like pitch-bend).
    ///
    /// Note: The generator holds three values: The base value .val, an
    /// offset caused by modulators .mod, and an offset caused by the
    /// NRPN system. _GEN(voice, generator_enumerator) returns the sum
    /// of all three.
    pub fn update_param(&mut self, gen: GenParam) {
        macro_rules! gen_sum {
            ($id: expr) => {{
                let Gen {
                    val, mod_0, nrpn, ..
                } = &self.gen[$id as usize];

                (val + mod_0 + nrpn) as f32
            }};
        }

        match gen {
            GenParam::Pan => {
                // range checking is done in the fluid_pan function
                self.pan = gen_sum!(GenParam::Pan);

                self.amp_left = fluid_pan(self.pan, 1) * self.synth_gain / 32768.0;
                self.amp_right = fluid_pan(self.pan, 0) * self.synth_gain / 32768.0;
            }

            GenParam::Attenuation => {
                // Alternate attenuation scale used by EMU10K1 cards when setting the attenuation at the preset or instrument level within the SoundFont bank.
                static ALT_ATTENUATION_SCALE: f64 = 0.4;

                self.attenuation =
                    (self.gen[GenParam::Attenuation as usize].val * ALT_ATTENUATION_SCALE
                        + self.gen[GenParam::Attenuation as usize].mod_0
                        + self.gen[GenParam::Attenuation as usize].nrpn) as f32;

                /* Range: SF2.01 section 8.1.3 # 48
                 * Motivation for range checking:
                 * OHPiano.SF2 sets initial attenuation to a whooping -96 dB */
                self.attenuation = if self.attenuation < 0.0 {
                    0.0
                } else if self.attenuation > 1440.0 {
                    1440.0
                } else {
                    self.attenuation
                };
            }
            /* The pitch is calculated from three different generators.
             * Read comment in fluidlite.h about GEN_PITCH.
             */
            GenParam::Pitch | GenParam::CoarseTune | GenParam::FineTune => {
                /* The testing for allowed range is done in 'fluid_ct2hz' */

                self.pitch = gen_sum!(GenParam::Pitch)
                    + 100.0 * gen_sum!(GenParam::CoarseTune)
                    + gen_sum!(GenParam::FineTune);
            }

            GenParam::ReverbSend => {
                /* The generator unit is 'tenths of a percent'. */
                self.reverb_send = gen_sum!(GenParam::ReverbSend) / 1000.0;

                self.reverb_send = if self.reverb_send < 0.0 {
                    0.0
                } else if self.reverb_send > 1.0 {
                    1.0
                } else {
                    self.reverb_send
                };
                self.amp_reverb = self.reverb_send * self.synth_gain / 32768.0;
            }

            GenParam::ChorusSend => {
                /* The generator unit is 'tenths of a percent'. */
                self.chorus_send = gen_sum!(GenParam::ChorusSend) / 1000.0;

                self.chorus_send = if self.chorus_send < 0.0 {
                    0.0
                } else if self.chorus_send > 1.0 {
                    1.0
                } else {
                    self.chorus_send
                };
                self.amp_chorus = self.chorus_send * self.synth_gain / 32768.0;
            }

            GenParam::OverrideRootKey => {
                /* This is a non-realtime parameter. Therefore the .mod part of the generator
                 * can be neglected.
                 * NOTE: origpitch sets MIDI root note while pitchadj is a fine tuning amount
                 * which offsets the original rate.  This means that the fine tuning is
                 * inverted with respect to the root note (so subtract it, not add).
                 */
                //FIXME: use flag instead of -1
                if self.gen[GenParam::OverrideRootKey as usize].val > -1.0 as f64 {
                    self.root_pitch = (self.gen[GenParam::OverrideRootKey as usize].val * 100.0
                        - self.sample.as_ref().unwrap().pitchadj as f64)
                        as f32
                } else {
                    self.root_pitch = self.sample.as_ref().unwrap().origpitch as f32 * 100.0
                        - self.sample.as_ref().unwrap().pitchadj as f32
                }
                self.root_pitch = fluid_ct2hz(self.root_pitch);

                if self.sample.is_some() {
                    self.root_pitch *=
                        self.output_rate / self.sample.as_ref().unwrap().samplerate as f32
                }
            }

            GenParam::FilterFc => {
                /* The resonance frequency is converted from absolute cents to
                 * midicents .val and .mod are both used, this permits real-time
                 * modulation.  The allowed range is tested in the 'fluid_ct2hz'
                 * function [PH,20021214]
                 */
                self.fres = gen_sum!(GenParam::FilterFc);
                /* The synthesis loop will have to recalculate the filter
                 * coefficients. */
                self.last_fres = -1.0;
            }

            GenParam::FilterQ => {
                /* The generator contains 'centibels' (1/10 dB) => divide by 10 to
                 * obtain dB */
                let q_db = gen_sum!(GenParam::FilterQ) / 10.0;
                /* Range: SF2.01 section 8.1.3 # 8 (convert from cB to dB => /10) */
                let mut q_db = if q_db < 0.0 {
                    0.0
                } else if q_db > 96.0 {
                    96.0
                } else {
                    q_db
                };
                /* Short version: Modify the Q definition in a way, that a Q of 0
                 * dB leads to no resonance hump in the freq. response.
                 *
                 * Long version: From SF2.01, page 39, item 9 (initialFilterQ):
                 * "The gain at the cutoff frequency may be less than zero when
                 * zero is specified".  Assume q_dB=0 / q_lin=1: If we would leave
                 * q as it is, then this results in a 3 dB hump slightly below
                 * fc. At fc, the gain is exactly the DC gain (0 dB).  What is
                 * (probably) meant here is that the filter does not show a
                 * resonance hump for q_dB=0. In this case, the corresponding
                 * q_lin is 1/sqrt(2)=0.707.  The filter should have 3 dB of
                 * attenuation at fc now.  In this case Q_dB is the height of the
                 * resonance peak not over the DC gain, but over the frequency
                 * response of a non-resonant filter.  This idea is implemented as
                 * follows: */
                q_db -= 3.01;

                /* The 'sound font' Q is defined in dB. The filter needs a linear
                q. Convert. */
                self.q_lin = f32::powf(10.0, q_db / 20.0);
                /* SF 2.01 page 59:
                 *
                 *  The SoundFont specs ask for a gain reduction equal to half the
                 *  height of the resonance peak (Q).  For example, for a 10 dB
                 *  resonance peak, the gain is reduced by 5 dB.  This is done by
                 *  multiplying the total gain with sqrt(1/Q).  `Sqrt' divides dB
                 *  by 2 (100 lin = 40 dB, 10 lin = 20 dB, 3.16 lin = 10 dB etc)
                 *  The gain is later factored into the 'b' coefficients
                 *  (numerator of the filter equation).  This gain factor depends
                 *  only on Q, so this is the right place to calculate it.
                 */
                self.filter_gain = 1.0 / f32::sqrt(self.q_lin);

                /* The synthesis loop will have to recalculate the filter coefficients. */
                self.last_fres = -1.0;
            }

            GenParam::ModLfoToPitch => {
                self.modlfo_to_pitch = gen_sum!(GenParam::ModLfoToPitch);

                self.modlfo_to_pitch = if self.modlfo_to_pitch < -12000.0 {
                    -12000.0
                } else if self.modlfo_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.modlfo_to_pitch
                };
            }

            GenParam::ModLfoToVol => {
                self.modlfo_to_vol = gen_sum!(GenParam::ModLfoToVol);

                self.modlfo_to_vol = if self.modlfo_to_vol < -960.0 {
                    -960.0
                } else if self.modlfo_to_vol > 960.0 {
                    960.0
                } else {
                    self.modlfo_to_vol
                };
            }

            GenParam::ModLfoToFilterFc => {
                self.modlfo_to_fc = gen_sum!(GenParam::ModLfoToFilterFc);

                self.modlfo_to_fc = if self.modlfo_to_fc < -12000.0 {
                    -12000.0
                } else if self.modlfo_to_fc > 12000.0 {
                    12000.0
                } else {
                    self.modlfo_to_fc
                };
            }

            GenParam::ModLfoDelay => {
                let val = gen_sum!(GenParam::ModLfoDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };
                self.modlfo_delay = (self.output_rate * fluid_tc2sec_delay(val)) as u32;
            }

            GenParam::ModLfoFreq => {
                /* - the frequency is converted into a delta value, per buffer of FLUID_BUFSIZE samples
                 * - the delay into a sample delay
                 */
                let val = gen_sum!(GenParam::ModLfoFreq);

                let val = if val < -16000.0 {
                    -16000.0
                } else if val > 4500.0 {
                    4500.0
                } else {
                    val
                };
                self.modlfo_incr = 4.0 * 64.0 * fluid_act2hz(val) / self.output_rate;
            }

            GenParam::VibLfoFreq => {
                /* vib lfo
                 *
                 * - the frequency is converted into a delta value, per buffer of FLUID_BUFSIZE samples
                 * - the delay into a sample delay
                 */
                let freq = gen_sum!(GenParam::VibLfoFreq);

                let freq = if freq < -16000.0 {
                    -16000.0
                } else if freq > 4500.0 {
                    4500.0
                } else {
                    freq
                };
                self.viblfo_incr = 4.0 * 64.0 * fluid_act2hz(freq) / self.output_rate;
            }

            GenParam::VibLfoDelay => {
                let val = gen_sum!(GenParam::VibLfoDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };
                self.viblfo_delay = (self.output_rate * fluid_tc2sec_delay(val)) as u32;
            }

            GenParam::VibLfoToPitch => {
                self.viblfo_to_pitch = gen_sum!(GenParam::VibLfoToPitch);

                self.viblfo_to_pitch = if self.viblfo_to_pitch < -12000.0 {
                    -12000.0
                } else if self.viblfo_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.viblfo_to_pitch
                };
            }

            GenParam::KeyNum => {
                /* GEN_KEYNUM: SF2.01 page 46, item 46
                 *
                 * If this generator is active, it forces the key number to its
                 * value.  Non-realtime controller.
                 *
                 * There is a flag, which should indicate, whether a generator is
                 * enabled or not.  But here we rely on the default value of -1.
                 * */
                let val = gen_sum!(GenParam::KeyNum);

                if val >= 0.0 {
                    self.key = val as u8;
                }
            }

            GenParam::Velocity => {
                /* GEN_VELOCITY: SF2.01 page 46, item 47
                 *
                 * If this generator is active, it forces the velocity to its
                 * value. Non-realtime controller.
                 *
                 * There is a flag, which should indicate, whether a generator is
                 * enabled or not. But here we rely on the default value of -1.  */
                let val = gen_sum!(GenParam::Velocity);
                if val > 0.0 {
                    self.vel = val as u8;
                }
            }

            GenParam::ModEnvToPitch => {
                self.modenv_to_pitch = gen_sum!(GenParam::ModEnvToPitch);

                self.modenv_to_pitch = if self.modenv_to_pitch < -12000.0 {
                    -12000.0
                } else if self.modenv_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.modenv_to_pitch
                };
            }

            GenParam::ModEnvToFilterFc => {
                self.modenv_to_fc = gen_sum!(GenParam::ModEnvToFilterFc);

                /* Range: SF2.01 section 8.1.3 # 1
                 * Motivation for range checking:
                 * Filter is reported to make funny noises now and then
                 */
                self.modenv_to_fc = if self.modenv_to_fc < -12000.0 {
                    -12000.0
                } else if self.modenv_to_fc > 12000.0 {
                    12000.0
                } else {
                    self.modenv_to_fc
                };
            }

            /* sample start and ends points
             *
             * Range checking is initiated via the
             * voice->check_sample_sanity flag,
             * because it is impossible to check here:
             * During the voice setup, all modulators are processed, while
             * the voice is inactive. Therefore, illegal settings may
             * occur during the setup (for example: First move the loop
             * end point ahead of the loop start point => invalid, then
             * move the loop start point forward => valid again.
             */
            GenParam::StartAddrOfs | GenParam::StartAddrCoarseOfs => {
                if self.sample.is_some() {
                    self.start = self
                        .sample
                        .as_ref()
                        .unwrap()
                        .start
                        .wrapping_add(gen_sum!(GenParam::StartAddrOfs) as u32)
                        .wrapping_add(32768 * gen_sum!(GenParam::StartAddrCoarseOfs) as u32)
                        as i32;
                    self.check_sample_sanity_flag = 1 << 0;
                }
            }

            GenParam::EndAddrOfs | GenParam::EndAddrCoarseOfs => {
                if self.sample.is_some() {
                    self.end = self
                        .sample
                        .as_ref()
                        .unwrap()
                        .end
                        .wrapping_add(gen_sum!(GenParam::EndAddrCoarseOfs) as u32)
                        .wrapping_add(32768 * gen_sum!(GenParam::EndAddrCoarseOfs) as u32)
                        as i32;
                    self.check_sample_sanity_flag = 1 << 0;
                }
            }

            GenParam::StartLoopAddrOfs | GenParam::StartLoopAddrCoarseOfs => {
                if self.sample.is_some() {
                    self.loopstart = self
                        .sample
                        .as_ref()
                        .unwrap()
                        .loopstart
                        .wrapping_add(gen_sum!(GenParam::StartLoopAddrOfs) as u32)
                        .wrapping_add(32768 * gen_sum!(GenParam::StartLoopAddrCoarseOfs) as u32)
                        as i32;
                    self.check_sample_sanity_flag = 1 << 0;
                }
            }

            GenParam::EndLoopAddrOfs | GenParam::EndLoopAddrCoarseOfs => {
                if self.sample.is_some() {
                    self.loopend = self
                        .sample
                        .as_ref()
                        .unwrap()
                        .loopend
                        .wrapping_add(gen_sum!(GenParam::EndLoopAddrOfs) as u32)
                        .wrapping_add(32768 * gen_sum!(GenParam::EndLoopAddrCoarseOfs) as u32)
                        as i32;
                    self.check_sample_sanity_flag = 1 << 0;
                }
            }

            /* volume envelope
             *
             * - delay and hold times are converted to absolute number of samples
             * - sustain is converted to its absolute value
             * - attack, decay and release are converted to their increment per sample
             */
            GenParam::VolEnvDelay => {
                let val = gen_sum!(GenParam::VolEnvDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };

                let count = (self.output_rate * fluid_tc2sec_delay(val) / 64.0) as u32;
                self.volenv_data[FLUID_VOICE_ENVDELAY as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVDELAY as usize].coeff = 0.0;
                self.volenv_data[FLUID_VOICE_ENVDELAY as usize].incr = 0.0;
                self.volenv_data[FLUID_VOICE_ENVDELAY as usize].min = -1.0;
                self.volenv_data[FLUID_VOICE_ENVDELAY as usize].max = 1.0;
            }

            GenParam::VolEnvAttack => {
                let val = gen_sum!(GenParam::VolEnvAttack);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count = (1 as u32)
                    .wrapping_add((self.output_rate * fluid_tc2sec_attack(val) / 64.0) as u32);
                self.volenv_data[FLUID_VOICE_ENVATTACK as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVATTACK as usize].coeff = 1.0;
                self.volenv_data[FLUID_VOICE_ENVATTACK as usize].incr =
                    if count != 0 { 1.0 / count as f32 } else { 0.0 };
                self.volenv_data[FLUID_VOICE_ENVATTACK as usize].min = -1.0;
                self.volenv_data[FLUID_VOICE_ENVATTACK as usize].max = 1.0;
            }

            GenParam::VolEnvHold | GenParam::KeyToVolEnvHold => {
                let count = self.calculate_hold_decay_buffers(
                    GEN_VOLENVHOLD as i32,
                    GEN_KEYTOVOLENVHOLD as i32,
                    0,
                ) as u32;
                self.volenv_data[FLUID_VOICE_ENVHOLD as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVHOLD as usize].coeff = 1.0;
                self.volenv_data[FLUID_VOICE_ENVHOLD as usize].incr = 0.0;
                self.volenv_data[FLUID_VOICE_ENVHOLD as usize].min = -1.0;
                self.volenv_data[FLUID_VOICE_ENVHOLD as usize].max = 2.0;
            }

            GenParam::VolEnvDecay | GenParam::VolEnvSustain | GenParam::KeyToVolEnvDecay => {
                let y = 1.0 - 0.001 * gen_sum!(GenParam::VolEnvSustain);

                let y = if y < 0.0 {
                    0.0
                } else if y > 1.0 {
                    1.0
                } else {
                    y
                };

                let count = self.calculate_hold_decay_buffers(
                    GEN_VOLENVDECAY as i32,
                    GEN_KEYTOVOLENVDECAY as i32,
                    1,
                ) as u32;

                self.volenv_data[FLUID_VOICE_ENVDECAY as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVDECAY as usize].coeff = 1.0;
                self.volenv_data[FLUID_VOICE_ENVDECAY as usize].incr =
                    if count != 0 { -1.0 / count as f32 } else { 0.0 };
                self.volenv_data[FLUID_VOICE_ENVDECAY as usize].min = y;
                self.volenv_data[FLUID_VOICE_ENVDECAY as usize].max = 2.0;
            }

            GenParam::VolEnvRelease => {
                let val = gen_sum!(GenParam::VolEnvRelease);

                let val = if val < -7200.0 {
                    -7200.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count = (1 as u32)
                    .wrapping_add((self.output_rate * fluid_tc2sec_release(val) / 64.0) as u32);
                self.volenv_data[FLUID_VOICE_ENVRELEASE as usize].count = count;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as usize].coeff = 1.0;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as usize].incr =
                    if count != 0 { -1.0 / count as f32 } else { 0.0 };
                self.volenv_data[FLUID_VOICE_ENVRELEASE as usize].min = 0.0;
                self.volenv_data[FLUID_VOICE_ENVRELEASE as usize].max = 1.0;
            }

            GenParam::ModEnvDelay => {
                let val = gen_sum!(GenParam::ModEnvDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };

                self.modenv_data[FLUID_VOICE_ENVDELAY as usize].count =
                    (self.output_rate * fluid_tc2sec_delay(val) / 64.0) as u32;
                self.modenv_data[FLUID_VOICE_ENVDELAY as usize].coeff = 0.0;
                self.modenv_data[FLUID_VOICE_ENVDELAY as usize].incr = 0.0;
                self.modenv_data[FLUID_VOICE_ENVDELAY as usize].min = -1.0;
                self.modenv_data[FLUID_VOICE_ENVDELAY as usize].max = 1.0;
            }

            GenParam::ModEnvAttack => {
                let val = gen_sum!(GenParam::ModEnvAttack);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count = (1 as u32)
                    .wrapping_add((self.output_rate * fluid_tc2sec_attack(val) / 64.0) as u32);
                self.modenv_data[FLUID_VOICE_ENVATTACK as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVATTACK as usize].coeff = 1.0;
                self.modenv_data[FLUID_VOICE_ENVATTACK as usize].incr =
                    if count != 0 { 1.0 / count as f32 } else { 0.0 };
                self.modenv_data[FLUID_VOICE_ENVATTACK as usize].min = -1.0;
                self.modenv_data[FLUID_VOICE_ENVATTACK as usize].max = 1.0;
            }

            GenParam::ModEnvHold | GenParam::KeyToModEnvHold => {
                let count = self.calculate_hold_decay_buffers(
                    GEN_MODENVHOLD as i32,
                    GEN_KEYTOMODENVHOLD as i32,
                    0,
                ) as u32;
                self.modenv_data[FLUID_VOICE_ENVHOLD as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVHOLD as usize].coeff = 1.0;
                self.modenv_data[FLUID_VOICE_ENVHOLD as usize].incr = 0.0;
                self.modenv_data[FLUID_VOICE_ENVHOLD as usize].min = -1.0;
                self.modenv_data[FLUID_VOICE_ENVHOLD as usize].max = 2.0;
            }

            GenParam::ModEnvDecay | GenParam::ModEnvSustain | GenParam::KeyToModEnvDecay => {
                let count = self.calculate_hold_decay_buffers(
                    GEN_MODENVDECAY as i32,
                    GEN_KEYTOMODENVDECAY as i32,
                    1,
                ) as u32;

                let y = 1.0 - 0.001 * gen_sum!(GenParam::ModEnvSustain);

                let y = if y < 0.0 {
                    0.0
                } else if y > 1.0 {
                    1.0
                } else {
                    y
                };

                self.modenv_data[FLUID_VOICE_ENVDECAY as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVDECAY as usize].coeff = 1.0;
                self.modenv_data[FLUID_VOICE_ENVDECAY as usize].incr =
                    if count != 0 { -1.0 / count as f32 } else { 0.0 };
                self.modenv_data[FLUID_VOICE_ENVDECAY as usize].min = y;
                self.modenv_data[FLUID_VOICE_ENVDECAY as usize].max = 2.0;
            }

            GenParam::ModEnvRelease => {
                let val = gen_sum!(GenParam::ModEnvRelease);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count = (1 as u32)
                    .wrapping_add((self.output_rate * fluid_tc2sec_release(val) / 64.0) as u32);
                self.modenv_data[FLUID_VOICE_ENVRELEASE as usize].count = count;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as usize].coeff = 1.0;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as usize].incr =
                    if count != 0 { -1.0 / count as f32 } else { 0.0 };
                self.modenv_data[FLUID_VOICE_ENVRELEASE as usize].min = 0.0;
                self.modenv_data[FLUID_VOICE_ENVRELEASE as usize].max = 2.0;
            }
            _ => {}
        }
    }
}
