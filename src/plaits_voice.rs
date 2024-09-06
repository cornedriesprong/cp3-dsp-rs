use crate::envelopes::{CurveType, AR};
use crate::filters::SVF;
use crate::osc::{BlitSawOsc, FmOsc};
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;
use mi_plaits_dsp::dsp::drums::{analog_bass_drum, analog_snare_drum, hihat};

const BLOCK_SIZE: usize = 1;

#[derive(Debug, Clone, Copy)]
pub struct FmVoice {
    osc: FmOsc,
    sample_rate: f32,
}

impl SynthVoice for FmVoice {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: FmOsc::new(sample_rate),
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.reset();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        self.osc.process()
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        // self.osc.reset();
        // let freq = pitch_to_freq(pitch);
        // self.osc.set_carrier_freq(freq);
        self.osc.trigger(velocity);
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.osc.carrier.freq_hz = value,
            1 => self.osc.modulator.freq_hz = value,
            2 => self.osc.filter.update_freq(value, self.sample_rate),
            3 => self.osc.filter.update_q(value),
            4 => self.osc.fm_amt = value,
            5 => self.osc.mod_index = value,
            6 => self.osc.carrier.fb_amt = value,
            7 => self.osc.modulator.fb_amt = value,
            8 => self.osc.carrier_env.attack_ms = value,
            9 => self.osc.carrier_env.decay_ms = value,
            10 => self.osc.mod_env.attack_ms = value,
            11 => self.osc.mod_env.decay_ms = value,
            12 => self.osc.filter_carrier_env_amt = value,
            13 => self.osc.filter_mod_env_amt = value,
            14 => self.osc.pitch_carrier_env_amt = value,
            15 => self.osc.pitch_mod_env_amt = value,
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
        self.filter.process(y) * env * 0.5
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.osc.set_freq(pitch_to_freq(pitch));
        self.env.trigger(velocity);
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.filter.update_freq(value * 10000.0, self.sample_rate),
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
        // self.modulations.level > 0.0
        true
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
