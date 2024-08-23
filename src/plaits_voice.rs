use crate::consts::SAMPLE_RATE;
use crate::envelopes::{CurveType, AR};
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;
use mi_plaits_dsp::dsp::drums::analog_bass_drum;
use mi_plaits_dsp::dsp::envelope::LpgEnvelope;
use mi_plaits_dsp::dsp::fm::dx_units::pitch_envelope_increment;
use mi_plaits_dsp::dsp::voice::{Modulations, Patch, Voice};

const BLOCK_SIZE: usize = 512;

pub struct PlaitsVoice<'a> {
    osc: Voice<'a>,
    patch: Patch,
    modulations: Modulations,
    env: LpgEnvelope,
    pitch: Option<u8>,
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
            env: LpgEnvelope::new(),
            pitch: None,
        }
    }

    fn init(&mut self) {
        self.osc.init();
        self.env.init();
        self.reset_params();
    }

    #[inline]
    fn process(&mut self, buf: &mut [f32]) {
        // if !self.env.is_active() {
        //     return 0.0;
        // }

        // let mut ys = vec![0.0; 1];
        let mut aux = vec![0.0; buf.len()];

        self.osc
            .render(&self.patch, &self.modulations, buf, &mut aux);
        // let mut env = 0.0;
        // self.env.process_lp(env, env, env, 1000.0);
        self.modulations.trigger = 0.0;
    }

    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.patch.note = pitch as f32;
        self.modulations.trigger = velocity as f32 / 127.0;
    }

    fn reset(&mut self) {
        // self.env.release();
        self.reset_params();
    }

    fn stop(&mut self) {
        // self.env.release();
        self.pitch = None;
    }

    fn set_sound(&mut self, sound: i8) {
        println!("setting sound to: {}", sound);
        self.patch.engine = sound as usize;
    }

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        println!("setting parameter {} to: {}", parameter, value);
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
        self.pitch.unwrap_or(0)
    }

    fn is_active(&self) -> bool {
        // self.modulations.level > 0.0
        true
    }
}

pub struct PlaitsKick {
    osc: analog_bass_drum::AnalogBassDrum,
    pitch: Option<u8>,
    frequency: f32,
    accent: f32,
    tone: f32,
    decay: f32,
    attack_fm_amount: f32,
    self_fm_amount: f32,
    trigger: bool,
}

impl PlaitsKick {
    fn reset_params(&mut self) {
        // no-op
    }
}

impl SynthVoice for PlaitsKick {
    fn new() -> Self {
        Self {
            osc: analog_bass_drum::AnalogBassDrum::new(),
            pitch: None,
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
    fn process(&mut self, buf: &mut [f32]) {
        let f0 = self.frequency / SAMPLE_RATE;

        self.osc.render(
            false,
            self.trigger,
            self.accent,
            f0,
            self.tone,
            self.decay,
            self.attack_fm_amount,
            self.self_fm_amount,
            buf,
        );
        self.trigger = false;
    }

    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.accent = velocity as f32 / 127.0;
        self.frequency = pitch_to_freq(pitch);
        self.trigger = true;
    }

    fn reset(&mut self) {
        // self.env.release();
        self.reset_params();
    }

    fn stop(&mut self) {
        // self.env.release();
        self.pitch = None;
    }

    fn set_sound(&mut self, sound: i8) {
        println!("setting sound to: {}", sound);
        // self.patch.engine = sound as usize;
    }

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
        self.pitch.unwrap_or(0)
    }

    fn is_active(&self) -> bool {
        // self.modulations.level > 0.0
        true
    }
}
