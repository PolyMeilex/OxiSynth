mod dsp_float;
mod envelope;

pub use envelope::EnvelopeStep;
use envelope::{Envelope, EnvelopePortion};

use super::super::{
    channel_pool::{Channel, InterpolationMethod},
    FxBuf,
};

use super::super::soundfont::{
    generator::{self, Generator, GeneratorType},
    modulator::Mod,
    Sample,
};

use super::super::conv::{
    act2hz, atten2amp, cb2amp, ct2hz, ct2hz_real, pan, tc2sec, tc2sec_attack, tc2sec_delay,
    tc2sec_release,
};

use soundfont::data::modulator::{ControllerPalette, GeneralPalette};

use std::sync::Arc;

type Phase = u64;

const GEN_ABS_NRPN: u32 = 2;
const GEN_SET: u32 = 1;

bitflags::bitflags! {
    /// Flags for marking samples for sanity checks
    struct SampleSanity: u32 {
        const CHECK = 1 << 0;
        const STARTUP = 1 << 1;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceAddMode {
    Overwrite = 0,
    Add = 1,
    Default = 2,
}

const FLUID_OK: i32 = 0;

#[derive(PartialEq, Clone)]
pub enum VoiceStatus {
    Clean,
    On,
    Sustained,
    Off,
}

pub enum LoopMode {
    UnLooped = 0,
    DuringRelease = 1,
    UntilRelease = 3,
}

pub struct VoiceDescriptor<'a> {
    pub sample: Arc<Sample>,
    pub channel: &'a Channel,
    pub key: u8,
    pub vel: u8,
    pub start_time: usize,
    pub gain: f32,
}

#[derive(Clone)]
pub struct Voice {
    pub note_id: usize,

    channel_id: usize,

    pub key: u8,
    pub vel: u8,

    interp_method: InterpolationMethod,
    mod_count: usize,

    pub sample: Arc<Sample>,
    pub start_time: usize,

    pub ticks: usize,
    noteoff_ticks: usize,

    pub has_looped: bool,

    filter_startup: bool,

    volenv_count: u32,
    pub volenv_section: EnvelopeStep,
    pub volenv_val: f32,

    pub amp: f32,
    modenv_count: u32,
    modenv_section: EnvelopeStep,
    modenv_val: f32,

    modlfo_val: f32,
    viblfo_val: f32,

    hist1: f32,
    hist2: f32,

    pub(crate) gen: [Generator; 60],
    synth_gain: f32,

    amplitude_that_reaches_noise_floor_nonloop: f32,
    amplitude_that_reaches_noise_floor_loop: f32,

    pub status: VoiceStatus,
    check_sample_sanity_flag: SampleSanity,
    min_attenuation_c_b: f32,

    last_fres: f32,

    // GenParams
    pan: f32,
    amp_left: f32,
    amp_right: f32,

    attenuation: f32,
    pitch: f32,

    reverb_send: f32,
    amp_reverb: f32,
    chorus_send: f32,
    amp_chorus: f32,

    root_pitch: f32,
    fres: f32,

    q_lin: f32,
    filter_gain: f32,

    modlfo_to_pitch: f32,
    modlfo_to_vol: f32,
    modlfo_to_fc: f32,
    modlfo_delay: usize,
    modlfo_incr: f32,

    viblfo_incr: f32,
    viblfo_delay: usize,
    viblfo_to_pitch: f32,

    modenv_to_pitch: f32,
    modenv_to_fc: f32,

    pub start: i32,
    pub end: i32,
    pub loopstart: i32,
    pub loopend: i32,

    volenv_data: Envelope,
    modenv_data: Envelope,
    mod_0: [Mod; 64],

    output_rate: f32,

    pub phase: Phase,

    filter_coeff_incr_count: i32,

    a1: f32,
    a2: f32,
    b02: f32,
    b1: f32,

    a1_incr: f32,
    a2_incr: f32,
    b02_incr: f32,
    b1_incr: f32,
}

impl Voice {
    pub fn new(output_rate: f32, desc: VoiceDescriptor, note_id: usize) -> Voice {
        let mut volenv_data = Envelope::default();

        volenv_data[EnvelopeStep::Sustain] = EnvelopePortion {
            count: 0xffffffff,
            coeff: 1.0,
            incr: 0.0,
            min: -1.0,
            max: 2.0,
        };

        volenv_data[EnvelopeStep::Finished] = EnvelopePortion {
            count: 0xffffffff,
            coeff: 0.0,
            incr: 0.0,
            min: -1.0,
            max: 1.0,
        };

        let mut modenv_data = Envelope::default();

        modenv_data[EnvelopeStep::Sustain] = EnvelopePortion {
            count: 0xffffffff,
            coeff: 1.0,
            incr: 0.0,
            min: -1.0,
            max: 2.0,
        };

        modenv_data[EnvelopeStep::Finished] = EnvelopePortion {
            count: 0xffffffff,
            coeff: 0.0,
            incr: 0.0,
            min: -1.0,
            max: 1.0,
        };

        let synth_gain = if desc.gain < 0.0000001 {
            0.0000001
        } else {
            desc.gain
        };

        Voice {
            note_id,
            channel_id: desc.channel.id(),

            key: desc.key,
            vel: desc.vel,

            interp_method: desc.channel.interp_method(),
            mod_count: 0,

            sample: desc.sample,
            start_time: desc.start_time,

            ticks: 0,
            noteoff_ticks: 0,

            has_looped: false,

            last_fres: -1.0,
            filter_startup: true,

            volenv_count: 0,
            volenv_section: EnvelopeStep::Delay,
            volenv_val: 0.0,

            amp: 0.0,
            modenv_count: 0,
            modenv_section: EnvelopeStep::Decay,
            modenv_val: 0.0,

            modlfo_val: 0.0,
            viblfo_val: 0.0,

            hist1: 0.0,
            hist2: 0.0,

            gen: generator::gen_init(desc.channel),
            synth_gain,

            amplitude_that_reaches_noise_floor_nonloop: 0.00003 / synth_gain,
            amplitude_that_reaches_noise_floor_loop: 0.00003 / synth_gain,

            status: VoiceStatus::Clean,
            mod_0: [Mod::default(); 64],
            check_sample_sanity_flag: SampleSanity::empty(),
            output_rate,
            phase: 0,
            pitch: 0.0,
            attenuation: 0.0,
            min_attenuation_c_b: 0.0,
            root_pitch: 0.0,
            start: 0,
            end: 0,
            loopstart: 0,
            loopend: 0,
            volenv_data,
            modenv_data,
            modenv_to_fc: 0.0,
            modenv_to_pitch: 0.0,
            modlfo_delay: 0,
            modlfo_incr: 0.0,
            modlfo_to_fc: 0.0,
            modlfo_to_pitch: 0.0,
            modlfo_to_vol: 0.0,
            viblfo_delay: 0,
            viblfo_incr: 0.0,
            viblfo_to_pitch: 0.0,
            fres: 0.0,
            q_lin: 0.0,
            filter_gain: 0.0,
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
        }
    }

    pub fn is_available(&self) -> bool {
        self.status == VoiceStatus::Clean || self.status == VoiceStatus::Off
    }

    pub fn is_on(&self) -> bool {
        self.status == VoiceStatus::On && self.volenv_section < EnvelopeStep::Release
    }

    pub fn is_playing(&self) -> bool {
        self.status == VoiceStatus::On || self.status == VoiceStatus::Sustained
    }

    /// Adds a modulator to the voice.  "mode" indicates, what to do, if
    /// an identical modulator exists already.
    ///
    /// mode == FLUID_VOICE_ADD: Identical modulators on preset level are added
    /// mode == FLUID_VOICE_OVERWRITE: Identical modulators on instrument level are overwritten
    /// mode == FLUID_VOICE_DEFAULT: This is a default modulator, there can be no identical modulator.
    ///                             Don't check.
    pub fn add_mod(&mut self, mod_0: &Mod, mode: VoiceAddMode) {
        /*
         * Some soundfonts come with a huge number of non-standard
         * controllers, because they have been designed for one particular
         * sound card.  Discard them, maybe print a warning.
         */
        if let ControllerPalette::General(g) = &mod_0.src.controller_palette {
            match g {
                GeneralPalette::Unknown(_) | GeneralPalette::Link => {
                    log::warn!("Ignoring invalid controller, using non-CC source {:?}.", g);
                    return;
                }
                _ => {}
            }
        }

        if mode == VoiceAddMode::Add {
            /* if identical modulator exists, add them */
            for m in self.mod_0.iter_mut().take(self.mod_count) {
                if m.test_identity(mod_0) {
                    m.amount += mod_0.amount;
                    return;
                }
            }
        } else if mode == VoiceAddMode::Overwrite {
            /* if identical modulator exists, replace it (only the amount has to be changed) */
            for m in self.mod_0.iter_mut().take(self.mod_count) {
                if m.test_identity(mod_0) {
                    m.amount = mod_0.amount;
                    return;
                }
            }
        }

        /* Add a new modulator (No existing modulator to add / overwrite).
        Also, default modulators (VOICE_DEFAULT) are added without
        checking, if the same modulator already exists. */
        if self.mod_count < 64 {
            self.mod_0[self.mod_count] = *mod_0;
            self.mod_count += 1;
        };
    }

    pub fn add_default_mods(&mut self) {
        use crate::core::soundfont::modulator::default::*;
        self.add_mod(&DEFAULT_VEL2ATT_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_VEL2FILTER_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_AT2VIBLFO_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_MOD2VIBLFO_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_ATT_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_PAN_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_EXPR_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_REVERB_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_CHORUS_MOD, VoiceAddMode::Default);
        self.add_mod(&DEFAULT_PITCH_BEND_MOD, VoiceAddMode::Default);
    }

    pub fn gen_incr(&mut self, i: u32, val: f64) {
        self.gen[i as usize].val += val;
        self.gen[i as usize].flags = GEN_SET as u8;
    }

    pub fn gen_set(&mut self, i: GeneratorType, val: f64) {
        self.gen[i as usize].val = val;
        self.gen[i as usize].flags = GEN_SET as u8;
    }

    /*
     * Percussion sounds can be mutually exclusive: for example, a 'closed
     * hihat' sound will terminate an 'open hihat' sound ringing at the
     * same time. This behaviour is modeled using 'exclusive classes',
     * turning on a voice with an exclusive class other than 0 will kill
     * all other voices having that exclusive class within the same preset
     * or channel.  fluid_voice_kill_excl gets called, when 'voice' is to
     * be killed for that reason.
     */
    pub fn kill_excl(&mut self) {
        if !self.is_playing() {
            return;
        }

        /* Turn off the exclusive class information for this voice,
           so that it doesn't get killed twice
        */
        self.gen_set(GeneratorType::ExclusiveClass, 0.0);

        /* If the voice is not yet in release state, put it into release state */
        if self.volenv_section != EnvelopeStep::Release {
            self.volenv_section = EnvelopeStep::Release;
            self.volenv_count = 0;
            self.modenv_section = EnvelopeStep::Release;
            self.modenv_count = 0;
        }

        /* Speed up the volume envelope */
        /* The value was found through listening tests with hi-hat samples. */
        self.gen_set(GeneratorType::VolEnvRelease, -200.0);
        self.update_param(GeneratorType::VolEnvRelease);

        /* Speed up the modulation envelope */
        self.gen_set(GeneratorType::ModEnvRelease, -200.0);
        self.update_param(GeneratorType::ModEnvRelease);
    }

    pub fn start(&mut self, channel: &Channel) {
        // The maximum volume of the loop is calculated and cached once for each
        // sample with its nominal loop settings. This happens, when the sample is used
        // for the first time.
        self.calculate_runtime_synthesis_parameters(channel);
        // Force setting of the phase at the first DSP loop run
        // This cannot be done earlier, because it depends on modulators.
        self.check_sample_sanity_flag = SampleSanity::STARTUP;
        self.status = VoiceStatus::On;
    }

    pub fn noteoff(&mut self, channel: &Channel, min_note_length_ticks: usize) {
        if min_note_length_ticks > self.ticks {
            /* Delay noteoff */
            self.noteoff_ticks = min_note_length_ticks;
            return;
        }

        let sustained = {
            const SUSTAIN_SWITCH: usize = 64;
            // check is channel is sustained
            channel.cc(SUSTAIN_SWITCH) >= 64
        };

        if sustained {
            self.status = VoiceStatus::Sustained;
        } else {
            if self.volenv_section == EnvelopeStep::Attack {
                /* A voice is turned off during the attack section of the volume
                 * envelope.  The attack section ramps up linearly with
                 * amplitude. The other sections use logarithmic scaling. Calculate new
                 * volenv_val to achieve equievalent amplitude during the release phase
                 * for seamless volume transition.
                 */
                if self.volenv_val > 0.0 {
                    let lfo: f32 = self.modlfo_val * -self.modlfo_to_vol;
                    let amp: f32 = self.volenv_val * f32::powf(10.0, lfo / -200.0);
                    let mut env_value = -((-200.0 * amp.ln() / f32::ln(10.0) - lfo) / 960.0 - 1.0);
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
            self.volenv_section = EnvelopeStep::Release;
            self.volenv_count = 0;
            self.modenv_section = EnvelopeStep::Release;
            self.modenv_count = 0;
        }
    }

    pub fn modulate(&mut self, channel: &Channel, is_cc: bool, ctrl: u8) {
        #[inline(always)]
        fn mod_has_source(m: &Mod, is_cc: bool, ctrl: u8) -> bool {
            let a1 = (m.src.index == ctrl) && m.src.is_cc() && is_cc;
            let a2 = (m.src.index == ctrl) && !m.src.is_cc() && !is_cc;
            let a3 = a1 || a2;

            let b1 = (m.src2.index == ctrl) && m.src2.is_cc() && is_cc;
            let b2 = (m.src2.index == ctrl) && !m.src2.is_cc() && !is_cc;
            let b3 = b1 || b2;

            a3 || b3
        }

        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &mut self.mod_0[i];
            if mod_has_source(mod_0, is_cc, ctrl) {
                let gen = mod_0.get_dest();
                let mut modval = 0.0;

                let mut k = 0;
                while k < self.mod_count {
                    if self.mod_0[k].dest == gen {
                        modval += self.mod_0[k].get_value(channel, self);
                    }
                    k += 1
                }
                self.gen[gen as usize].mod_0 = modval as f64;
                self.update_param(gen);
            }
            i += 1
        }
    }

    pub fn modulate_all(&mut self, channel: &Channel) -> i32 {
        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &mut self.mod_0[i];
            let gen = mod_0.get_dest();
            let mut modval = 0.0f32;

            let mut k = 0;
            while k < self.mod_count {
                if self.mod_0[k].dest == gen {
                    modval += self.mod_0[k].get_value(channel, self)
                }
                k += 1
            }
            self.gen[gen as usize].mod_0 = modval as f64;
            self.update_param(gen);
            i += 1
        }

        FLUID_OK
    }

    /// Turns off a voice, meaning that it is not processed
    /// anymore by the DSP loop.
    pub fn off(&mut self) {
        self.channel_id = 0xff;
        self.volenv_section = EnvelopeStep::Finished;
        self.volenv_count = 0;
        self.modenv_section = EnvelopeStep::Finished;
        self.modenv_count = 0;
        self.status = VoiceStatus::Off;
    }

    pub fn get_channel_id(&self) -> usize {
        self.channel_id
    }

    pub fn get_note_id(&self) -> usize {
        self.note_id
    }

    /// A lower boundary for the attenuation (as in 'the minimum
    /// attenuation of this voice, with volume pedals, modulators
    /// etc. resulting in minimum attenuation, cannot fall below x cB) is
    /// calculated.  This has to be called during fluid_voice_init, after
    /// all modulators have been run on the voice once.  Also,
    /// voice.attenuation has to be initialized.
    fn get_lower_boundary_for_attenuation(&mut self, channel: &Channel) -> f32 {
        const MOD_PITCHWHEEL: i32 = 14;

        let mut possible_att_reduction_c_b = 0.0;
        for i in 0..self.mod_count {
            let mod_0 = &self.mod_0[i];

            /* Modulator has attenuation as target and can change over time? */
            if mod_0.dest == GeneratorType::Attenuation && (mod_0.src.is_cc() || mod_0.src2.is_cc())
            {
                let current_val: f32 = mod_0.get_value(channel, self);
                let mut v = mod_0.amount.abs() as f32;

                if mod_0.src.index as i32 == MOD_PITCHWHEEL
                    || mod_0.src.is_bipolar()
                    || mod_0.src2.is_bipolar()
                    || mod_0.amount < 0.0
                {
                    /* Can this modulator produce a negative contribution? */
                    v *= -1.0;
                } else {
                    v = 0.0;
                }

                /* For example:
                 * - current_val=100
                 * - min_val=-4000
                 * - possible_att_reduction_cB += 4100
                 */
                if current_val > v {
                    possible_att_reduction_c_b += current_val - v
                }
            }
        }

        let mut lower_bound = self.attenuation - possible_att_reduction_c_b;

        /* SF2.01 specs do not allow negative attenuation */
        if lower_bound < 0.0 {
            lower_bound = 0.0;
        }

        lower_bound
    }

    fn calculate_runtime_synthesis_parameters(&mut self, channel: &Channel) {
        let list_of_generators_to_initialize: [GeneratorType; 34] = [
            GeneratorType::StartAddrOfs,
            GeneratorType::EndAddrOfs,
            GeneratorType::StartLoopAddrOfs,
            GeneratorType::EndLoopAddrOfs,
            GeneratorType::ModLfoToPitch,
            GeneratorType::VibLfoToPitch,
            GeneratorType::ModEnvToPitch,
            GeneratorType::FilterFc,
            GeneratorType::FilterQ,
            GeneratorType::ModLfoToFilterFc,
            GeneratorType::ModEnvToFilterFc,
            GeneratorType::ModLfoToVol,
            GeneratorType::ChorusSend,
            GeneratorType::ReverbSend,
            GeneratorType::Pan,
            GeneratorType::ModLfoDelay,
            GeneratorType::ModLfoFreq,
            GeneratorType::VibLfoDelay,
            GeneratorType::VibLfoFreq,
            GeneratorType::ModEnvDelay,
            GeneratorType::ModEnvAttack,
            GeneratorType::ModEnvHold,
            GeneratorType::ModEnvDecay,
            GeneratorType::ModEnvRelease,
            GeneratorType::VolEnvDelay,
            GeneratorType::VolEnvAttack,
            GeneratorType::VolEnvHold,
            GeneratorType::VolEnvDecay,
            GeneratorType::VolEnvRelease,
            GeneratorType::KeyNum,
            GeneratorType::Velocity,
            GeneratorType::Attenuation,
            GeneratorType::OverrideRootKey,
            GeneratorType::Pitch,
        ];

        let mut i = 0;
        while i < self.mod_count {
            let mod_0 = &self.mod_0[i];
            let modval: f32 = mod_0.get_value(channel, self);
            let dest_gen_index = mod_0.dest as usize;
            let dest_gen = &mut self.gen[dest_gen_index];
            dest_gen.mod_0 += modval as f64;
            i += 1
        }
        let tuning = channel.tuning();
        if let Some(tuning) = tuning {
            self.gen[GeneratorType::Pitch as usize].val = tuning.pitch[60]
                + self.gen[GeneratorType::ScaleTune as usize].val / 100.0f32 as f64
                    * (tuning.pitch[self.key as usize] - tuning.pitch[60])
        } else {
            self.gen[GeneratorType::Pitch as usize].val =
                self.gen[GeneratorType::ScaleTune as usize].val
                    * (self.key as i32 as f32 - 60.0f32) as f64
                    + (100.0f32 * 60.0f32) as f64
        }

        for gen in list_of_generators_to_initialize.iter() {
            self.update_param(*gen);
        }

        self.min_attenuation_c_b = self.get_lower_boundary_for_attenuation(channel);
    }

    /// Make sure, that sample start / end point and loop points are in
    /// proper order. When starting up, calculate the initial phase.
    pub fn check_sample_sanity(&mut self) {
        let min_index_nonloop = self.sample.start as i32;
        let max_index_nonloop = self.sample.end as i32;

        /* make sure we have enough samples surrounding the loop */
        let min_index_loop = self.sample.start as i32;
        /* 'end' is last valid sample, loopend can be + 1 */
        let max_index_loop = self.sample.end as i32 + 1;

        if self.check_sample_sanity_flag.is_empty() {
            return;
        }

        /* Keep the start point within the sample data */
        if self.start < min_index_nonloop {
            self.start = min_index_nonloop
        } else if self.start > max_index_nonloop {
            self.start = max_index_nonloop
        }
        /* Keep the end point within the sample data */
        if self.end < min_index_nonloop {
            self.end = min_index_nonloop
        } else if self.end > max_index_nonloop {
            self.end = max_index_nonloop
        }

        /* Keep start and end point in the right order */
        if self.start > self.end {
            let temp: i32 = self.start;
            self.start = self.end;
            self.end = temp
        }

        /* Zero length? */
        if self.start == self.end {
            self.off();
            return;
        }

        if self.gen[GeneratorType::SampleMode as usize].val as i32 == LoopMode::UntilRelease as i32
            || self.gen[GeneratorType::SampleMode as usize].val as i32
                == LoopMode::DuringRelease as i32
        {
            /* Keep the loop start point within the sample data */
            if self.loopstart < min_index_loop {
                self.loopstart = min_index_loop
            } else if self.loopstart > max_index_loop {
                self.loopstart = max_index_loop
            }

            /* Keep the loop end point within the sample data */
            if self.loopend < min_index_loop {
                self.loopend = min_index_loop
            } else if self.loopend > max_index_loop {
                self.loopend = max_index_loop
            }

            /* Keep loop start and end point in the right order */
            if self.loopstart > self.loopend {
                let temp_0: i32 = self.loopstart;
                self.loopstart = self.loopend;
                self.loopend = temp_0
            }

            /* Loop too short? Then don't loop. */
            if self.loopend < self.loopstart + 2 {
                self.gen[GeneratorType::SampleMode as i32 as usize].val =
                    LoopMode::UnLooped as i32 as f64
            }

            /* The loop points may have changed. Obtain a new estimate for the loop volume. */
            /* Is the voice loop within the sample loop? */
            if self.loopstart >= self.sample.loop_start as i32
                && self.loopend <= self.sample.loop_end as i32
            {
                /* Is there a valid peak amplitude available for the loop? */
                if self.sample.amplitude_that_reaches_noise_floor_is_valid != 0 {
                    self.amplitude_that_reaches_noise_floor_loop =
                        (self.sample.amplitude_that_reaches_noise_floor / self.synth_gain as f64)
                            as f32
                } else {
                    /* Worst case */
                    self.amplitude_that_reaches_noise_floor_loop =
                        self.amplitude_that_reaches_noise_floor_nonloop
                }
            }
        }

        /* Run startup specific code (only once, when the voice is started) */
        if self
            .check_sample_sanity_flag
            .contains(SampleSanity::STARTUP)
        {
            if max_index_loop - min_index_loop < 2 {
                if self.gen[GeneratorType::SampleMode as i32 as usize].val as i32
                    == LoopMode::UntilRelease as i32
                    || self.gen[GeneratorType::SampleMode as i32 as usize].val as i32
                        == LoopMode::DuringRelease as i32
                {
                    self.gen[GeneratorType::SampleMode as i32 as usize].val =
                        LoopMode::UnLooped as i32 as f64
                }
            }

            /* Set the initial phase of the voice (using the result from the
            start offset modulators). */
            self.phase = (self.start as u64) << 32i32
        }

        /* Is this voice run in loop mode, or does it run straight to the
        end of the waveform data? */
        if self.gen[GeneratorType::SampleMode as i32 as usize].val as i32
            == LoopMode::UntilRelease as i32
            && self.volenv_section < EnvelopeStep::Release
            || self.gen[GeneratorType::SampleMode as usize].val as i32
                == LoopMode::DuringRelease as i32
        {
            /* Yes, it will loop as soon as it reaches the loop point.  In
             * this case we must prevent, that the playback pointer (phase)
             * happens to end up beyond the 2nd loop point, because the
             * point has moved.  The DSP algorithm is unable to cope with
             * that situation.  So if the phase is beyond the 2nd loop
             * point, set it to the start of the loop. No way to avoid some
             * noise here.  Note: If the sample pointer ends up -before the
             * first loop point- instead, then the DSP loop will just play
             * the sample, enter the loop and proceed as expected => no
             * actions required.
             */
            let index_in_sample: i32 = (self.phase >> 32i32) as u32 as i32;
            if index_in_sample >= self.loopend {
                self.phase = (self.loopstart as u64) << 32i32
            }
        }

        /* Sample sanity has been assured. Don't check again, until some
        sample parameter is changed by modulation. */
        self.check_sample_sanity_flag = SampleSanity::empty();
    }

    pub fn set_param(&mut self, gen: GeneratorType, nrpn_value: f32, abs: i32) {
        self.gen[gen as usize].nrpn = nrpn_value as f64;
        self.gen[gen as usize].flags = if abs != 0 {
            GEN_ABS_NRPN as i32
        } else {
            GEN_SET as i32
        } as u8;
        self.update_param(gen);
    }

    pub fn set_gain(&mut self, mut gain: f32) {
        /* avoid division by zero*/
        if gain < 0.0000001 {
            gain = 0.0000001;
        }

        self.synth_gain = gain;
        self.amp_left = pan(self.pan, 1) * gain / 32768.0;
        self.amp_right = pan(self.pan, 0) * gain / 32768.0;
        self.amp_reverb = self.reverb_send * gain / 32768.0;
        self.amp_chorus = self.chorus_send * gain / 32768.0;
    }

    pub(super) fn write(
        &mut self,
        channel: &Channel,
        min_note_length_ticks: usize,
        (dsp_left_buf, dsp_right_buf): (&mut [f32; 64], &mut [f32; 64]),
        fx_left_buf: &mut FxBuf,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        let mut dsp_buf: [f32; 64] = [0.; 64];

        /* make sure we're playing and that we have sample data */
        if !self.is_playing() {
            return;
        }

        /******************* sample **********************/
        if self.noteoff_ticks != 0 && self.ticks >= self.noteoff_ticks {
            self.noteoff(channel, min_note_length_ticks);
        }

        /* Range checking for sample- and loop-related parameters
         * Initial phase is calculated here*/
        self.check_sample_sanity();

        /* skip to the next section of the envelope if necessary */
        let mut env_data = &self.volenv_data[self.volenv_section];
        while self.volenv_count >= env_data.count {
            // If we're switching envelope stages from decay to sustain, force the value to be the end value of the previous stage
            if self.volenv_section == EnvelopeStep::Decay {
                self.volenv_val = env_data.min * env_data.coeff
            }

            self.volenv_section.next();
            env_data = &self.volenv_data[self.volenv_section];
            self.volenv_count = 0;
        }

        /* calculate the envelope value and check for valid range */
        let mut x = env_data.coeff * self.volenv_val + env_data.incr;
        if x < env_data.min {
            x = env_data.min;
            self.volenv_section.next();
            self.volenv_count = 0;
        } else if x > env_data.max {
            x = env_data.max;
            self.volenv_section.next();
            self.volenv_count = 0;
        }

        self.volenv_val = x;
        self.volenv_count = self.volenv_count.wrapping_add(1);

        if self.volenv_section == EnvelopeStep::Finished {
            self.off();
            return;
        }

        /******************* mod env **********************/

        let mut env_data = &self.modenv_data[self.modenv_section];

        /* skip to the next section of the envelope if necessary */
        while self.modenv_count >= env_data.count {
            self.modenv_section.next();
            env_data = &self.modenv_data[self.modenv_section];
            self.modenv_count = 0;
        }

        /* calculate the envelope value and check for valid range */
        let mut x = env_data.coeff * self.modenv_val + env_data.incr;

        if x < env_data.min {
            x = env_data.min;
            self.modenv_section.next();
            self.modenv_count = 0;
        } else if x > env_data.max {
            x = env_data.max;
            self.modenv_section.next();
            self.modenv_count = 0;
        }

        self.modenv_val = x;
        self.modenv_count = self.modenv_count.wrapping_add(1);

        /******************* mod lfo **********************/

        if self.ticks >= self.modlfo_delay {
            self.modlfo_val += self.modlfo_incr;
            if self.modlfo_val > 1.0 {
                self.modlfo_incr = -self.modlfo_incr;
                self.modlfo_val = 2.0 - self.modlfo_val
            } else if self.modlfo_val < -1.0 {
                self.modlfo_incr = -self.modlfo_incr;
                self.modlfo_val = -2.0 - self.modlfo_val
            }
        }

        /******************* vib lfo **********************/
        if self.ticks >= self.viblfo_delay {
            self.viblfo_val += self.viblfo_incr;
            if self.viblfo_val > 1.0 {
                self.viblfo_incr = -self.viblfo_incr;
                self.viblfo_val = 2.0 - self.viblfo_val
            } else if self.viblfo_val < -1.0 {
                self.viblfo_incr = -self.viblfo_incr;
                self.viblfo_val = -2.0 - self.viblfo_val
            }
        }

        /******************* amplitude **********************/

        /* calculate final amplitude
         * - initial gain
         * - amplitude envelope
         */

        let target_amplitude = if self.volenv_section != EnvelopeStep::Delay {
            if self.volenv_section == EnvelopeStep::Attack {
                /* the envelope is in the attack section: ramp linearly to max value.
                 * A positive modlfo_to_vol should increase volume (negative attenuation).
                 */
                let target_amp = atten2amp(self.attenuation)
                    * cb2amp(self.modlfo_val * -self.modlfo_to_vol)
                    * self.volenv_val;

                Some(target_amp)
            } else {
                let target_amp = atten2amp(self.attenuation)
                    * cb2amp(
                        960.0 * (1.0 - self.volenv_val) + self.modlfo_val * -self.modlfo_to_vol,
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
                let amplitude_that_reaches_noise_floor = if self.has_looped {
                    self.amplitude_that_reaches_noise_floor_loop
                } else {
                    self.amplitude_that_reaches_noise_floor_nonloop
                };

                /* voice->attenuation_min is a lower boundary for the attenuation
                 * now and in the future (possibly 0 in the worst case).  Now the
                 * amplitude of sample and volenv cannot exceed amp_max (since
                 * volenv_val can only drop):
                 */
                let amp_max = atten2amp(self.min_attenuation_c_b) * self.volenv_val;

                /* And if amp_max is already smaller than the known amplitude,
                 * which will attenuate the sample below the noise floor, then we
                 * can safely turn off the voice. Duh. */
                if amp_max < amplitude_that_reaches_noise_floor {
                    self.off();
                    None
                } else {
                    Some(target_amp)
                }
            }
        } else {
            None
        };

        if let Some(target_amp) = target_amplitude {
            /* Volume increment to go from voice->amp to target_amp in FLUID_BUFSIZE steps */
            let amp_incr = (target_amp - self.amp) / 64.0;
            /* no volume and not changing? - No need to process */
            if !(self.amp == 0.0 && amp_incr == 0.0) {
                /* Calculate the number of samples, that the DSP loop advances
                 * through the original waveform with each step in the output
                 * buffer. It is the ratio between the frequencies of original
                 * waveform and output waveform.*/
                let mut phase_incr = ct2hz_real(
                    self.pitch
                        + self.modlfo_val * self.modlfo_to_pitch
                        + self.viblfo_val * self.viblfo_to_pitch
                        + self.modenv_val * self.modenv_to_pitch,
                ) / self.root_pitch;

                /* if phase_incr is not advancing, set it to the minimum fraction value (prevent stuckage) */
                if phase_incr == 0.0 {
                    phase_incr = 1.0;
                }

                /*************** resonant filter ******************/

                /* calculate the frequency of the resonant filter in Hz */
                let fres = ct2hz(
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
                let fres = if fres > 0.45 * self.output_rate {
                    0.45 * self.output_rate
                } else if fres < 5.0 {
                    5.0
                } else {
                    fres
                };

                /* if filter enabled and there is a significant frequency change.. */
                if (fres - self.last_fres).abs() > 0.01 {
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

                    let omega = 2.0 * std::f32::consts::PI * (fres / self.output_rate);
                    let sin_coeff = omega.sin();
                    let cos_coeff = omega.cos();
                    let alpha_coeff = sin_coeff / (2.0 * self.q_lin);
                    let a0_inv = 1.0 / (1.0 + alpha_coeff);

                    /* Calculate the filter coefficients. All coefficients are
                     * normalized by a0. Think of `a1' as `a1/a0'.
                     *
                     * Here a couple of multiplications are saved by reusing common expressions.
                     * The original equations should be:
                     *  voice->b0=(1.-cos_coeff)*a0_inv*0.5*voice->filter_gain;
                     *  voice->b1=(1.-cos_coeff)*a0_inv*voice->filter_gain;
                     *  voice->b2=(1.-cos_coeff)*a0_inv*0.5*voice->filter_gain; */
                    let a1_temp = -2.0 * cos_coeff * a0_inv;
                    let a2_temp = (1.0 - alpha_coeff) * a0_inv;
                    let b1_temp = (1.0 - cos_coeff) * a0_inv * (self).filter_gain;
                    /* both b0 -and- b2 */
                    let b02_temp = b1_temp * 0.5;

                    if self.filter_startup {
                        /* The filter is calculated, because the voice was started up.
                         * In this case set the filter coefficients without delay.
                         */
                        self.a1 = a1_temp;
                        self.a2 = a2_temp;
                        self.b02 = b02_temp;
                        self.b1 = b1_temp;
                        self.filter_coeff_incr_count = 0;
                        self.filter_startup = false;
                    } else {
                        /* The filter frequency is changed.  Calculate an increment
                         * factor, so that the new setting is reached after one buffer
                         * length. x_incr is added to the current value FLUID_BUFSIZE
                         * times. The length is arbitrarily chosen. Longer than one
                         * buffer will sacrifice some performance, though.  Note: If
                         * the filter is still too 'grainy', then increase this number
                         * at will.
                         */
                        self.a1_incr = (a1_temp - self.a1) / 64.0;
                        self.a2_incr = (a2_temp - self.a2) / 64.0;
                        self.b02_incr = (b02_temp - self.b02) / 64.0;
                        self.b1_incr = (b1_temp - self.b1) / 64.0;
                        /* Have to add the increments filter_coeff_incr_count times. */
                        self.filter_coeff_incr_count = 64;
                    }
                    self.last_fres = fres
                }

                let count = match self.interp_method {
                    InterpolationMethod::None => {
                        self.dsp_float_interpolate_none(&mut dsp_buf, amp_incr, phase_incr)
                    }
                    InterpolationMethod::Linear => {
                        self.dsp_float_interpolate_linear(&mut dsp_buf, amp_incr, phase_incr)
                    }
                    InterpolationMethod::FourthOrder => {
                        self.dsp_float_interpolate_4th_order(&mut dsp_buf, amp_incr, phase_incr)
                    }
                    InterpolationMethod::SeventhOrder => {
                        self.dsp_float_interpolate_7th_order(&mut dsp_buf, amp_incr, phase_incr)
                    }
                };

                if count > 0 {
                    self.effects(
                        &mut dsp_buf,
                        count,
                        (dsp_left_buf, dsp_right_buf),
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

        self.ticks += self.ticks.wrapping_add(64);
    }

    /// Purpose:
    ///
    /// - filters (applies a lowpass filter with variable cutoff frequency and quality factor)
    /// - mixes the processed sample to left and right output using the pan setting
    /// - sends the processed sample to chorus and reverb
    ///
    /// Variable description:
    /// - dsp_data: Pointer to the original waveform data
    /// - dsp_left_buf: The generated signal goes here, left channel
    /// - dsp_right_buf: right channel
    /// - dsp_reverb_buf: Send to reverb unit
    /// - dsp_chorus_buf: Send to chorus unit
    /// - dsp_a1: Coefficient for the filter
    /// - dsp_a2: same
    /// - dsp_b0: same
    /// - dsp_b1: same
    /// - dsp_b2: same
    /// - voice holds the voice structure
    ///
    /// A couple of variables are used internally, their results are discarded:
    /// - dsp_i: Index through the output buffer
    /// - dsp_phase_fractional: The fractional part of dsp_phase
    /// - dsp_coeff: A table of four coefficients, depending on the fractional phase.
    ///              Used to interpolate between samples.
    /// - dsp_process_buffer: Holds the processed signal between stages
    /// - dsp_centernode: delay line for the IIR filter
    /// - dsp_hist1: same
    /// - dsp_hist2: same
    #[inline]
    fn effects(
        &mut self,
        dsp_buf: &mut [f32; 64],
        count: usize,
        (dsp_left_buf, dsp_right_buf): (&mut [f32], &mut [f32]),
        fx_buf: &mut FxBuf,
        reverb_active: bool,
        chorus_active: bool,
    ) {
        // IIR filter sample history
        let mut dsp_hist1 = self.hist1;
        let mut dsp_hist2 = self.hist2;

        // IIR filter coefficients
        let mut dsp_a1 = self.a1;
        let mut dsp_a2 = self.a2;
        let mut dsp_b02 = self.b02;
        let mut dsp_b1 = self.b1;
        let mut dsp_filter_coeff_incr_count = self.filter_coeff_incr_count;

        let mut dsp_centernode;

        // filter (implement the voice filter according to SoundFont standard)

        // Check for denormal number (too close to zero).
        if dsp_hist1.abs() < 1e-20 {
            dsp_hist1 = 0.0; // FIXME JMG - Is this even needed?
        }

        // Two versions of the filter loop. One, while the filter is
        // changing towards its new setting. The other, if the filter
        // doesn't change.

        if dsp_filter_coeff_incr_count > 0 {
            // Increment is added to each filter coefficient filter_coeff_incr_count times.
            for dsp in dsp_buf.iter_mut().take(count) {
                // The filter is implemented in Direct-II form.
                dsp_centernode = *dsp - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                *dsp = dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
                let fresh0 = dsp_filter_coeff_incr_count;
                dsp_filter_coeff_incr_count -= 1;

                if fresh0 > 0 {
                    dsp_a1 += self.a1_incr;
                    dsp_a2 += self.a2_incr;
                    dsp_b02 += self.b02_incr;
                    dsp_b1 += self.b1_incr;
                }
            }
        }
        // The filter parameters are constant.  This is duplicated to save time.
        else {
            // The filter is implemented in Direct-II form.
            for dsp in dsp_buf.iter_mut().take(count) {
                dsp_centernode = *dsp - dsp_a1 * dsp_hist1 - dsp_a2 * dsp_hist2;
                *dsp = dsp_b02 * (dsp_centernode + dsp_hist2) + dsp_b1 * dsp_hist1;
                dsp_hist2 = dsp_hist1;
                dsp_hist1 = dsp_centernode;
            }
        }

        // pan (Copy the signal to the left and right output buffer) The voice
        // panning generator has a range of -500 .. 500.  If it is centered,
        // it's close to 0.  voice->amp_left and voice->amp_right are then the
        // same, and we can save one multiplication per voice and sample.

        if self.pan > -0.5 && self.pan < 0.5 {
            for ((left, right), dsp) in dsp_left_buf
                .iter_mut()
                .zip(dsp_right_buf.iter_mut())
                .zip(dsp_buf.iter().copied())
                .take(count)
            {
                // The voice is centered. Use voice->amp_left twice.
                let v = self.amp_left * dsp;
                *left += v;
                *right += v;
            }
        }
        // The voice is not centered. Stereo samples have one side zero.
        else {
            if self.amp_left != 0.0 {
                for (left, dsp) in dsp_left_buf
                    .iter_mut()
                    .zip(dsp_buf.iter().copied())
                    .take(count)
                {
                    *left += self.amp_left * dsp;
                }
            }
            if self.amp_right != 0.0 {
                for (right, dsp) in dsp_right_buf
                    .iter_mut()
                    .zip(dsp_buf.iter().copied())
                    .take(count)
                {
                    *right += self.amp_right * dsp;
                }
            }
        }

        if reverb_active && self.amp_reverb != 0.0 {
            for (fx, dsp) in fx_buf
                .reverb
                .iter_mut()
                .zip(dsp_buf.iter().copied())
                .take(count)
            {
                *fx += self.amp_reverb * dsp;
            }
        }

        if chorus_active && self.amp_chorus != 0.0 {
            for (fx, dsp) in fx_buf
                .chorus
                .iter_mut()
                .zip(dsp_buf.iter().copied())
                .take(count)
            {
                *fx += self.amp_chorus * dsp;
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
        gen_base: GeneratorType,
        gen_key2base: GeneratorType,
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
        if timecents < -12000.0 {
            timecents = -12000.0;
        }
        let seconds = tc2sec(timecents);
        // buffers
        ((self.output_rate as f64 * seconds / 64.0) + 0.5) as i32
    }

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
    pub fn update_param(&mut self, gen: GeneratorType) {
        macro_rules! gen_sum {
            ($id: expr) => {{
                let Generator {
                    val, mod_0, nrpn, ..
                } = &self.gen[$id as usize];

                (val + mod_0 + nrpn) as f32
            }};
        }

        match gen {
            GeneratorType::Pan => {
                // range checking is done in the fluid_pan function
                self.pan = gen_sum!(GeneratorType::Pan);

                self.amp_left = pan(self.pan, 1) * self.synth_gain / 32768.0;
                self.amp_right = pan(self.pan, 0) * self.synth_gain / 32768.0;
            }

            GeneratorType::Attenuation => {
                // Alternate attenuation scale used by EMU10K1 cards when setting the attenuation at the preset or instrument level within the SoundFont bank.
                static ALT_ATTENUATION_SCALE: f64 = 0.4;

                self.attenuation = (self.gen[GeneratorType::Attenuation as usize].val
                    * ALT_ATTENUATION_SCALE
                    + self.gen[GeneratorType::Attenuation as usize].mod_0
                    + self.gen[GeneratorType::Attenuation as usize].nrpn)
                    as f32;

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
            GeneratorType::Pitch | GeneratorType::CoarseTune | GeneratorType::FineTune => {
                /* The testing for allowed range is done in 'fluid_ct2hz' */

                self.pitch = gen_sum!(GeneratorType::Pitch)
                    + 100.0 * gen_sum!(GeneratorType::CoarseTune)
                    + gen_sum!(GeneratorType::FineTune);
            }

            GeneratorType::ReverbSend => {
                /* The generator unit is 'tenths of a percent'. */
                self.reverb_send = gen_sum!(GeneratorType::ReverbSend) / 1000.0;

                self.reverb_send = if self.reverb_send < 0.0 {
                    0.0
                } else if self.reverb_send > 1.0 {
                    1.0
                } else {
                    self.reverb_send
                };
                self.amp_reverb = self.reverb_send * self.synth_gain / 32768.0;
            }

            GeneratorType::ChorusSend => {
                /* The generator unit is 'tenths of a percent'. */
                self.chorus_send = gen_sum!(GeneratorType::ChorusSend) / 1000.0;

                self.chorus_send = if self.chorus_send < 0.0 {
                    0.0
                } else if self.chorus_send > 1.0 {
                    1.0
                } else {
                    self.chorus_send
                };
                self.amp_chorus = self.chorus_send * self.synth_gain / 32768.0;
            }

            GeneratorType::OverrideRootKey => {
                /* This is a non-realtime parameter. Therefore the .mod part of the generator
                 * can be neglected.
                 * NOTE: origpitch sets MIDI root note while pitchadj is a fine tuning amount
                 * which offsets the original rate.  This means that the fine tuning is
                 * inverted with respect to the root note (so subtract it, not add).
                 */
                //FIXME: use flag instead of -1
                if self.gen[GeneratorType::OverrideRootKey as usize].val > -1.0 {
                    self.root_pitch = (self.gen[GeneratorType::OverrideRootKey as usize].val
                        * 100.0
                        - self.sample.pitchadj as f64) as f32
                } else {
                    self.root_pitch =
                        self.sample.origpitch as f32 * 100.0 - self.sample.pitchadj as f32
                }
                self.root_pitch = ct2hz(self.root_pitch);

                self.root_pitch *= self.output_rate / self.sample.sample_rate as f32
            }

            GeneratorType::FilterFc => {
                /* The resonance frequency is converted from absolute cents to
                 * midicents .val and .mod are both used, this permits real-time
                 * modulation.  The allowed range is tested in the 'fluid_ct2hz'
                 * function [PH,20021214]
                 */
                self.fres = gen_sum!(GeneratorType::FilterFc);
                /* The synthesis loop will have to recalculate the filter
                 * coefficients. */
                self.last_fres = -1.0;
            }

            GeneratorType::FilterQ => {
                /* The generator contains 'centibels' (1/10 dB) => divide by 10 to
                 * obtain dB */
                let q_db = gen_sum!(GeneratorType::FilterQ) / 10.0;
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

            GeneratorType::ModLfoToPitch => {
                self.modlfo_to_pitch = gen_sum!(GeneratorType::ModLfoToPitch);

                self.modlfo_to_pitch = if self.modlfo_to_pitch < -12000.0 {
                    -12000.0
                } else if self.modlfo_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.modlfo_to_pitch
                };
            }

            GeneratorType::ModLfoToVol => {
                self.modlfo_to_vol = gen_sum!(GeneratorType::ModLfoToVol);

                self.modlfo_to_vol = if self.modlfo_to_vol < -960.0 {
                    -960.0
                } else if self.modlfo_to_vol > 960.0 {
                    960.0
                } else {
                    self.modlfo_to_vol
                };
            }

            GeneratorType::ModLfoToFilterFc => {
                self.modlfo_to_fc = gen_sum!(GeneratorType::ModLfoToFilterFc);

                self.modlfo_to_fc = if self.modlfo_to_fc < -12000.0 {
                    -12000.0
                } else if self.modlfo_to_fc > 12000.0 {
                    12000.0
                } else {
                    self.modlfo_to_fc
                };
            }

            GeneratorType::ModLfoDelay => {
                let val = gen_sum!(GeneratorType::ModLfoDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };
                self.modlfo_delay = (self.output_rate * tc2sec_delay(val)) as usize;
            }

            GeneratorType::ModLfoFreq => {
                /* - the frequency is converted into a delta value, per buffer of FLUID_BUFSIZE samples
                 * - the delay into a sample delay
                 */
                let val = gen_sum!(GeneratorType::ModLfoFreq);

                let val = if val < -16000.0 {
                    -16000.0
                } else if val > 4500.0 {
                    4500.0
                } else {
                    val
                };
                self.modlfo_incr = 4.0 * 64.0 * act2hz(val) / self.output_rate;
            }

            GeneratorType::VibLfoFreq => {
                /* vib lfo
                 *
                 * - the frequency is converted into a delta value, per buffer of FLUID_BUFSIZE samples
                 * - the delay into a sample delay
                 */
                let freq = gen_sum!(GeneratorType::VibLfoFreq);

                let freq = if freq < -16000.0 {
                    -16000.0
                } else if freq > 4500.0 {
                    4500.0
                } else {
                    freq
                };
                self.viblfo_incr = 4.0 * 64.0 * act2hz(freq) / self.output_rate;
            }

            GeneratorType::VibLfoDelay => {
                let val = gen_sum!(GeneratorType::VibLfoDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };
                self.viblfo_delay = (self.output_rate * tc2sec_delay(val)) as usize;
            }

            GeneratorType::VibLfoToPitch => {
                self.viblfo_to_pitch = gen_sum!(GeneratorType::VibLfoToPitch);

                self.viblfo_to_pitch = if self.viblfo_to_pitch < -12000.0 {
                    -12000.0
                } else if self.viblfo_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.viblfo_to_pitch
                };
            }

            GeneratorType::KeyNum => {
                /* GEN_KEYNUM: SF2.01 page 46, item 46
                 *
                 * If this generator is active, it forces the key number to its
                 * value.  Non-realtime controller.
                 *
                 * There is a flag, which should indicate, whether a generator is
                 * enabled or not.  But here we rely on the default value of -1.
                 * */
                let val = gen_sum!(GeneratorType::KeyNum);

                if val >= 0.0 {
                    self.key = val as u8;
                }
            }

            GeneratorType::Velocity => {
                /* GEN_VELOCITY: SF2.01 page 46, item 47
                 *
                 * If this generator is active, it forces the velocity to its
                 * value. Non-realtime controller.
                 *
                 * There is a flag, which should indicate, whether a generator is
                 * enabled or not. But here we rely on the default value of -1.  */
                let val = gen_sum!(GeneratorType::Velocity);
                if val > 0.0 {
                    self.vel = val as u8;
                }
            }

            GeneratorType::ModEnvToPitch => {
                self.modenv_to_pitch = gen_sum!(GeneratorType::ModEnvToPitch);

                self.modenv_to_pitch = if self.modenv_to_pitch < -12000.0 {
                    -12000.0
                } else if self.modenv_to_pitch > 12000.0 {
                    12000.0
                } else {
                    self.modenv_to_pitch
                };
            }

            GeneratorType::ModEnvToFilterFc => {
                self.modenv_to_fc = gen_sum!(GeneratorType::ModEnvToFilterFc);

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
            GeneratorType::StartAddrOfs | GeneratorType::StartAddrCoarseOfs => {
                self.start = self
                    .sample
                    .start
                    .wrapping_add(gen_sum!(GeneratorType::StartAddrOfs) as u32)
                    .wrapping_add(32768 * gen_sum!(GeneratorType::StartAddrCoarseOfs) as u32)
                    as i32;
                self.check_sample_sanity_flag = SampleSanity::CHECK;
            }

            GeneratorType::EndAddrOfs | GeneratorType::EndAddrCoarseOfs => {
                self.end = self
                    .sample
                    .end
                    .wrapping_add(gen_sum!(GeneratorType::EndAddrCoarseOfs) as u32)
                    .wrapping_add(32768 * gen_sum!(GeneratorType::EndAddrCoarseOfs) as u32)
                    as i32;
                self.check_sample_sanity_flag = SampleSanity::CHECK;
            }

            GeneratorType::StartLoopAddrOfs | GeneratorType::StartLoopAddrCoarseOfs => {
                self.loopstart = self
                    .sample
                    .loop_start
                    .wrapping_add(gen_sum!(GeneratorType::StartLoopAddrOfs) as u32)
                    .wrapping_add(32768 * gen_sum!(GeneratorType::StartLoopAddrCoarseOfs) as u32)
                    as i32;
                self.check_sample_sanity_flag = SampleSanity::CHECK;
            }

            GeneratorType::EndLoopAddrOfs | GeneratorType::EndLoopAddrCoarseOfs => {
                self.loopend = self
                    .sample
                    .loop_end
                    .wrapping_add(gen_sum!(GeneratorType::EndLoopAddrOfs) as u32)
                    .wrapping_add(32768 * gen_sum!(GeneratorType::EndLoopAddrCoarseOfs) as u32)
                    as i32;
                self.check_sample_sanity_flag = SampleSanity::CHECK;
            }

            /* volume envelope
             *
             * - delay and hold times are converted to absolute number of samples
             * - sustain is converted to its absolute value
             * - attack, decay and release are converted to their increment per sample
             */
            GeneratorType::VolEnvDelay => {
                let val = gen_sum!(GeneratorType::VolEnvDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };

                let count = (self.output_rate * tc2sec_delay(val) / 64.0) as u32;

                self.volenv_data[EnvelopeStep::Delay] = EnvelopePortion {
                    count,
                    coeff: 0.0,
                    incr: 0.0,
                    min: -1.0,
                    max: 1.0,
                };
            }

            GeneratorType::VolEnvAttack => {
                let val = gen_sum!(GeneratorType::VolEnvAttack);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count =
                    1u32.wrapping_add((self.output_rate * tc2sec_attack(val) / 64.0) as u32);

                self.volenv_data[EnvelopeStep::Attack] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { 1.0 / count as f32 } else { 0.0 },
                    min: -1.0,
                    max: 1.0,
                };
            }

            GeneratorType::VolEnvHold | GeneratorType::KeyToVolEnvHold => {
                let count = self.calculate_hold_decay_buffers(
                    GeneratorType::VolEnvHold,
                    GeneratorType::KeyToVolEnvHold,
                    0,
                ) as u32;

                self.volenv_data[EnvelopeStep::Hold] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: 0.0,
                    min: -1.0,
                    max: 2.0,
                };
            }

            GeneratorType::VolEnvDecay
            | GeneratorType::VolEnvSustain
            | GeneratorType::KeyToVolEnvDecay => {
                let y = 1.0 - 0.001 * gen_sum!(GeneratorType::VolEnvSustain);

                let y = if y < 0.0 {
                    0.0
                } else if y > 1.0 {
                    1.0
                } else {
                    y
                };

                let count = self.calculate_hold_decay_buffers(
                    GeneratorType::VolEnvDecay,
                    GeneratorType::KeyToVolEnvDecay,
                    1,
                ) as u32;

                self.volenv_data[EnvelopeStep::Decay] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { -1.0 / count as f32 } else { 0.0 },
                    min: y,
                    max: 2.0,
                };
            }

            GeneratorType::VolEnvRelease => {
                let val = gen_sum!(GeneratorType::VolEnvRelease);

                let val = if val < -7200.0 {
                    -7200.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count =
                    1u32.wrapping_add((self.output_rate * tc2sec_release(val) / 64.0) as u32);

                self.volenv_data[EnvelopeStep::Release] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { -1.0 / count as f32 } else { 0.0 },
                    min: 0.0,
                    max: 1.0,
                };
            }

            GeneratorType::ModEnvDelay => {
                let val = gen_sum!(GeneratorType::ModEnvDelay);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 5000.0 {
                    5000.0
                } else {
                    val
                };

                self.modenv_data[EnvelopeStep::Delay] = EnvelopePortion {
                    count: (self.output_rate * tc2sec_delay(val) / 64.0) as u32,
                    coeff: 0.0,
                    incr: 0.0,
                    min: -1.0,
                    max: 1.0,
                };
            }

            GeneratorType::ModEnvAttack => {
                let val = gen_sum!(GeneratorType::ModEnvAttack);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count =
                    1u32.wrapping_add((self.output_rate * tc2sec_attack(val) / 64.0) as u32);

                self.modenv_data[EnvelopeStep::Attack] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { 1.0 / count as f32 } else { 0.0 },
                    min: -1.0,
                    max: 1.0,
                };
            }

            GeneratorType::ModEnvHold | GeneratorType::KeyToModEnvHold => {
                let count = self.calculate_hold_decay_buffers(
                    GeneratorType::ModEnvHold,
                    GeneratorType::KeyToModEnvHold,
                    0,
                ) as u32;
                self.modenv_data[EnvelopeStep::Hold] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: 0.0,
                    min: -1.0,
                    max: 2.0,
                };
            }

            GeneratorType::ModEnvDecay
            | GeneratorType::ModEnvSustain
            | GeneratorType::KeyToModEnvDecay => {
                let count = self.calculate_hold_decay_buffers(
                    GeneratorType::ModEnvDecay,
                    GeneratorType::KeyToModEnvDecay,
                    1,
                ) as u32;

                let y = 1.0 - 0.001 * gen_sum!(GeneratorType::ModEnvSustain);

                let y = if y < 0.0 {
                    0.0
                } else if y > 1.0 {
                    1.0
                } else {
                    y
                };

                self.modenv_data[EnvelopeStep::Decay] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { -1.0 / count as f32 } else { 0.0 },
                    min: y,
                    max: 2.0,
                };
            }

            GeneratorType::ModEnvRelease => {
                let val = gen_sum!(GeneratorType::ModEnvRelease);

                let val = if val < -12000.0 {
                    -12000.0
                } else if val > 8000.0 {
                    8000.0
                } else {
                    val
                };

                let count =
                    1u32.wrapping_add((self.output_rate * tc2sec_release(val) / 64.0) as u32);

                self.modenv_data[EnvelopeStep::Release] = EnvelopePortion {
                    count,
                    coeff: 1.0,
                    incr: if count != 0 { -1.0 / count as f32 } else { 0.0 },
                    min: 0.0,
                    max: 2.0,
                };
            }
            _ => {}
        }
    }
}
