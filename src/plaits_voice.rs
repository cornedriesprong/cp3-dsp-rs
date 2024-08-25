use crate::consts::SAMPLE_RATE;
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;
use mi_plaits_dsp::dsp::drums::{analog_bass_drum, analog_snare_drum, hihat};
use mi_plaits_dsp::dsp::voice::{Modulations, Patch, Voice};

const BLOCK_SIZE: usize = 1;

pub struct PlaitsVoice<'a> {
    osc: Voice<'a>,
    patch: Patch,
    modulations: Modulations,
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
    fn new() -> Self {
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
                engine: 0,
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

pub struct PlaitsKick {
    osc: analog_bass_drum::AnalogBassDrum,
    frequency: f32,
    accent: f32,
    tone: f32,
    decay: f32,
    attack_fm_amount: f32,
    self_fm_amount: f32,
    trigger: bool,
}

impl SynthVoice for PlaitsKick {
    fn new() -> Self {
        Self {
            osc: analog_bass_drum::AnalogBassDrum::new(),
            frequency: 50.0,
            accent: 1.0,
            tone: 1.0,
            decay: 0.5,
            attack_fm_amount: 0.0,
            self_fm_amount: 0.0,
            trigger: false,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / SAMPLE_RATE;

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
}

impl SynthVoice for PlaitsSnare {
    fn new() -> Self {
        Self {
            osc: analog_snare_drum::AnalogSnareDrum::new(),
            frequency: 50.0,
            sustain: false,
            accent: 1.0,
            tone: 1.0,
            decay: 0.5,
            snappy: 0.5,
            trigger: false,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / SAMPLE_RATE;
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
}

impl SynthVoice for PlaitsHihat {
    fn new() -> Self {
        Self {
            osc: hihat::Hihat::new(),
            frequency: 50.0,
            sustain: false,
            accent: 1.0,
            tone: 1.0,
            decay: 0.2,
            noisiness: 0.5,
            trigger: false,
        }
    }

    fn init(&mut self) {
        self.osc.init();
    }

    #[inline]
    fn process(&mut self) -> f32 {
        let f0 = self.frequency / SAMPLE_RATE;
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
    fn new() -> Self {
        Self {
            kick: PlaitsKick::new(),
            snare: PlaitsSnare::new(),
            hihat: PlaitsHihat::new(),
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

        mix / 3.0
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

    fn set_parameter(&mut self, _: i8, _: f32) {}

    fn get_pitch(&self) -> u8 {
        0
    }

    fn is_active(&self) -> bool {
        true
    }
}
