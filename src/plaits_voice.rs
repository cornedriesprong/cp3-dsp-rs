use crate::filters::SVF;
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;
use mi_plaits_dsp::dsp::drums::{analog_bass_drum, analog_snare_drum, hihat};
use mi_plaits_dsp::dsp::envelope::DecayEnvelope;
use mi_plaits_dsp::dsp::oscillator;
use mi_plaits_dsp::dsp::voice::{Modulations, Patch, Voice};

const BLOCK_SIZE: usize = 1;

pub struct PlaitsVoice<'a> {
    osc: Voice<'a>,
    patch: Patch,
    modulations: Modulations,
    sample_rate: f32,
}

impl PlaitsVoice<'_> {
    fn reset_params(&mut self) {
        // reset params to saved settings (after changing them in sequence)
        self.patch.engine = 0;
        self.patch.harmonics = 0.5;
        self.patch.timbre = 0.5;
        self.patch.morph = 0.5;
    }
}

impl SynthVoice for PlaitsVoice<'_> {
    fn new(sample_rate: f32) -> Self {
        Self {
            // TODO: don't hardcode block size
            osc: Voice::new(&std::alloc::System, BLOCK_SIZE),
            patch: Patch {
                note: 48.0,
                harmonics: 0.5,
                timbre: 0.5,
                morph: 0.5,
                frequency_modulation_amount: 0.0,
                timbre_modulation_amount: 0.5,
                morph_modulation_amount: 0.5,
                engine: 12,
                decay: 0.5,
                lpg_colour: 0.5,
            },
            modulations: Modulations {
                engine: 0.0,
                note: 0.0,
                frequency: 0.0,
                harmonics: 0.0,
                timbre: 0.0,
                morph: 0.0,
                trigger: 0.0,
                level: 0.0,
                frequency_patched: false,
                timbre_patched: false,
                morph_patched: false,
                trigger_patched: true,
                level_patched: false,
            },
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.init();
        self.reset_params();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let mut buf = vec![0.0; BLOCK_SIZE];
        let mut aux = vec![0.0; BLOCK_SIZE];
        self.osc
            .render(&self.patch, &self.modulations, &mut buf, &mut aux);
        self.modulations.trigger = 0.0;

        buf[0]
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        println!("playing note at pitch: {}", pitch);
        self.patch.note = pitch as f32;
        self.modulations.trigger = velocity as f32 / 127.0;
    }

    fn reset(&mut self) {
        self.reset_params();
    }

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.patch.harmonics = value,
            1 => self.patch.timbre = value,
            2 => self.patch.morph = value,
            3 => self.patch.frequency_modulation_amount = value,
            4 => self.patch.timbre_modulation_amount = value,
            5 => self.patch.morph_modulation_amount = value,
            6 => self.patch.decay = value,
            7 => self.patch.lpg_colour = value,
            _ => (),
        }
    }

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        // TODO: figure out how to determine if voice is active
        true
    }
}

pub struct PlaitsOscillator {
    osc: oscillator::variable_saw_oscillator::VariableSawOscillator,
    env: DecayEnvelope,
    filter: SVF,
    frequency: f32,
    pulse_width: f32,
    waveshape: f32,
    trigger: bool,
    sample_rate: f32,
}

impl SynthVoice for PlaitsOscillator {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: oscillator::variable_saw_oscillator::VariableSawOscillator::new(),
            env: DecayEnvelope::new(),
            // env: LpgEnvelope::new(),
            filter: SVF::new(1000.0, 1.717, sample_rate),
            frequency: 50.0 / sample_rate,
            pulse_width: 0.5,
            waveshape: 0.5,
            trigger: false,
            sample_rate,
        }
    }

    fn init(&mut self) {
        self.osc.init();
        self.env.init();
        // self.lpg.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let mut buf = [0.0; BLOCK_SIZE];
        self.osc
            .render(self.frequency, self.pulse_width, self.waveshape, &mut buf);
        self.trigger = false;

        self.env.process(0.0002);
        self.filter.process(buf[0]) * self.env.value() * 0.5
    }

    fn play(&mut self, pitch: u8, velocity: u8, _: f32, _: f32) {
        self.frequency = pitch_to_freq(pitch) / self.sample_rate;
        self.env.trigger();
        self.trigger = true;
    }

    fn reset(&mut self) {}

    fn stop(&mut self) {}

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        match parameter {
            0 => self.pulse_width = value,
            1 => self.waveshape = value,
            2 => self.filter.set_frequency(value * 10000.0, self.sample_rate),
            3 => self.filter.set_q(value * 10.0),
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
        println!("playing drum at pitch: {}", pitch);
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
