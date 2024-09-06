use crate::envelopes::{CurveType, EnvelopeState, AR};
use crate::filters::SVF;
use crate::osc::{BlitSawOsc, FmOp};
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;
use mi_plaits_dsp::dsp::drums::{analog_bass_drum, analog_snare_drum, hihat};
use std::f32::consts::PI;

const BLOCK_SIZE: usize = 1;

#[derive(Debug, Clone, Copy)]
pub struct FmVoice {
    pub carrier: FmOp,
    pub carrier_env: AR,
    pub modulator: FmOp,
    pub mod_env: AR,
    pub fm_amt: f32,
    pub mod_index: f32,
    pub filter_mod_env_amt: f32,
    pub pitch_carrier_env_amt: f32,
    pub pitch_mod_env_amt: f32,
    pub filter: SVF,
}

impl FmVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: FmOp::new(sample_rate),
            carrier_env: AR::new(1.0, 500.0, CurveType::Exponential { pow: 3 }, sample_rate),
            fm_amt: 0.0,
            modulator: FmOp::new(sample_rate),
            mod_env: AR::new(1.0, 100.0, CurveType::Exponential { pow: 3 }, sample_rate),
            mod_index: 0.0,
            filter_mod_env_amt: 0.0,
            pitch_carrier_env_amt: 0.0,
            pitch_mod_env_amt: 0.0,
            filter: SVF::new(1000.0, 0.717, sample_rate),
        }
    }

    pub fn trigger(&mut self, velocity: u8) {
        self.carrier_env.trigger(velocity);
        self.mod_env.trigger(velocity);
    }

    pub fn reset(&mut self) {
        // start carrier phase at 90 degrees to increase percussiveness/attack
        self.carrier.phase = PI / 2.0;
        self.modulator.phase = 0.0;
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let mod_env_signal = self.mod_env.process();

        let mod_out = self
            .modulator
            .process(0.0, mod_env_signal * self.pitch_mod_env_amt);
        let mod_signal = self.fm_amt * self.mod_index * mod_out;
        let carrier_env_signal = self.carrier_env.process();

        let carrier_out = self.carrier.process(
            mod_signal * mod_env_signal,
            carrier_env_signal * self.pitch_carrier_env_amt,
        );
        let mut y = carrier_out + (mod_out * (1.0 - self.fm_amt));
        y = y * carrier_env_signal;

        self.filter
            .process(y, mod_env_signal * self.filter_mod_env_amt)
            * 0.5
    }

    pub fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.carrier.freq_hz = value,
            1 => self.modulator.freq_hz = value,
            2 => self.filter.update_freq(value),
            3 => self.filter.update_q(value),
            4 => self.fm_amt = value,
            5 => self.mod_index = value,
            6 => self.carrier.fb_amt = value,
            7 => self.modulator.fb_amt = value,
            8 => self.carrier_env.attack_ms = value,
            9 => self.carrier_env.decay_ms = value,
            10 => self.mod_env.attack_ms = value,
            11 => self.mod_env.decay_ms = value,
            12 => self.filter_mod_env_amt = value,
            13 => self.pitch_carrier_env_amt = value,
            14 => self.pitch_mod_env_amt = value,
            _ => (),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.carrier_env.state, EnvelopeState::Off)
    }
}
pub struct BLITVoice {
    osc: BlitSawOsc,
    env: AR,
    filter: SVF,
    sample_rate: f32,
}

impl SynthVoice for BLITVoice {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: BlitSawOsc::new(sample_rate),
            env: AR::new(10.0, 500.0, CurveType::Exponential { pow: 3 }, sample_rate),
            filter: SVF::new(500.0, 1.717, sample_rate),
            sample_rate,
        }
    }

    fn init(&mut self) {}

    #[inline]
    fn process(&mut self) -> f32 {
        let y = self.osc.process();
        let env = self.env.process();
        self.filter.process(y, 0.0) * env * 0.5
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.osc.set_freq(pitch_to_freq(pitch));
        self.env.trigger(velocity);
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.filter.update_freq(value * 10000.0),
            1 => self.filter.update_q(value * 10.0),
            2 => self.env.attack_ms = value,
            3 => self.env.decay_ms = value,
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        !matches!(self.env.state, EnvelopeState::Off)
    }
}

pub struct PlaitsKick {
    osc: analog_bass_drum::AnalogBassDrum,
    frequency: f32,
    accent: f32,
    tone: f32,
    decay: f32,
    attack_fm_amount: f32,
    self_fm_amount: f32,
    trigger: bool,
    sample_rate: f32,
}

impl SynthVoice for PlaitsKick {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: analog_bass_drum::AnalogBassDrum::new(),
            frequency: 50.0,
            accent: 1.0,
            tone: 1.0,
            decay: 0.5,
            attack_fm_amount: 0.0,
            self_fm_amount: 0.0,
            trigger: false,
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / self.sample_rate;

        let mut buf = [0.0; BLOCK_SIZE];
        self.osc.render(
            false,
            self.trigger,
            self.accent,
            f0,
            self.tone,
            self.decay,
            self.attack_fm_amount,
            self.self_fm_amount,
            &mut buf,
        );
        self.trigger = false;

        buf[0]
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.accent = velocity as f32 / 127.0;
        self.frequency = pitch_to_freq(pitch);
        self.trigger = true;
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.tone = value,
            1 => self.decay = value,
            2 => self.attack_fm_amount = value,
            3 => self.self_fm_amount = value,
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        // self.modulations.level > 0.0
        true
    }
}

pub struct PlaitsSnare {
    osc: analog_snare_drum::AnalogSnareDrum,
    frequency: f32,
    sustain: bool,
    accent: f32,
    tone: f32,
    decay: f32,
    snappy: f32,
    trigger: bool,
    sample_rate: f32,
}

impl SynthVoice for PlaitsSnare {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: analog_snare_drum::AnalogSnareDrum::new(),
            frequency: 50.0,
            sustain: false,
            accent: 1.0,
            tone: 1.0,
            decay: 0.5,
            snappy: 0.5,
            trigger: false,
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / self.sample_rate;
        let mut buf = [0.0; BLOCK_SIZE];

        self.osc.render(
            self.sustain,
            self.trigger,
            self.accent,
            f0,
            self.tone,
            self.decay,
            self.snappy,
            &mut buf,
        );
        self.trigger = false;

        buf[0]
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.accent = velocity as f32 / 127.0;
        self.frequency = pitch_to_freq(pitch);
        self.trigger = true;
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.tone = value,
            1 => self.decay = value,
            2 => self.snappy = value,
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        // self.modulations.level > 0.0
        true
    }
}

pub struct PlaitsHihat {
    osc: hihat::Hihat,
    frequency: f32,
    sustain: bool,
    accent: f32,
    tone: f32,
    decay: f32,
    noisiness: f32,
    trigger: bool,
    sample_rate: f32,
}

impl SynthVoice for PlaitsHihat {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: hihat::Hihat::new(),
            frequency: 50.0,
            sustain: false,
            accent: 1.0,
            tone: 1.0,
            decay: 0.2,
            noisiness: 0.5,
            trigger: false,
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / self.sample_rate;
        let mut buf = [0.0; BLOCK_SIZE];
        let mut temp_1 = [0.0; BLOCK_SIZE];
        let mut temp_2 = [0.0; BLOCK_SIZE];

        self.osc.render(
            self.sustain,
            self.trigger,
            self.accent,
            f0,
            self.tone,
            self.decay,
            self.noisiness,
            &mut temp_1,
            &mut temp_2,
            &mut buf,
            hihat::NoiseType::RingMod,
            hihat::VcaType::Swing,
            false,
            false,
        );

        self.trigger = false;

        buf[0]
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.accent = velocity as f32 / 127.0;
        self.frequency = pitch_to_freq(pitch);
        self.trigger = true;
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.tone = value,
            1 => self.decay = value,
            2 => self.noisiness = value,
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        // self.modulations.level > 0.0
        true
    }
}

pub struct PlaitsDrums {
    kick: PlaitsKick,
    snare: PlaitsSnare,
    hihat: PlaitsHihat,
}

impl SynthVoice for PlaitsDrums {
    fn new(sample_rate: f32) -> Self {
        Self {
            kick: PlaitsKick::new(sample_rate),
            snare: PlaitsSnare::new(sample_rate),
            hihat: PlaitsHihat::new(sample_rate),
        }
    }

    fn init(&mut self) {
        self.kick.init();
        self.snare.init();
        self.hihat.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let mut mix = 0.0;
        mix += self.kick.process();
        mix += self.snare.process();
        mix += self.hihat.process();

        mix /= 3.0;

        mix
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        match pitch {
            36 => self.kick.play(40, velocity, 0.0, 0.0),
            38 => self.snare.play(40, velocity, 0.0, 0.0),
            42 => self.hihat.play(40, velocity, 0.0, 0.0),
            _ => (),
        }
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => {
                self.kick.tone = value;
                self.snare.tone = value;
                self.hihat.tone = value;
            }
            1 => {
                self.kick.decay = value;
                self.snare.decay = value;
                self.hihat.decay = value;
            }
            2 => {
                self.kick.attack_fm_amount = value;
                self.snare.tone = value;
                self.hihat.tone = value;
            }
            3 => {
                self.kick.self_fm_amount = value;
                self.snare.snappy = value;
                self.hihat.noisiness = value;
            }
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        true
    }
}
