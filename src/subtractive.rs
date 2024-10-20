use crate::envelopes::{CurveType, AR};
use crate::filters::SVF;
use crate::osc::BlitSawOsc;
use crate::synth::SynthVoice;
use crate::utils::pitch_to_freq;

pub struct SubtractiveVoice {
    osc: BlitSawOsc,
    env: AR,
    velocity: f32,
    filter: SVF,
    pitch: Option<u8>,
    sample_rate: f32,
}

impl SynthVoice for SubtractiveVoice {
    fn new(sample_rate: f32) -> Self {
        Self {
            osc: BlitSawOsc::new(sample_rate),
            env: AR::new(0.0, 30000.0, CurveType::Exponential { pow: 8 }, sample_rate),
            velocity: 1.0,
            filter: SVF::new(5000.0, 0.707, sample_rate),
            pitch: None,
            sample_rate,
        }
    }

    fn init(&mut self) {
        // no-op
    }

    #[inline]
    fn process(&mut self) -> f32 {
        todo!()
        // if !self.env.is_active() {
        //     return 0.0;
        // }
        // let y = self.osc.process();
        // self.filter.process(y)
    }

    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.velocity = velocity as f32 / 128.0;
        self.pitch = Some(pitch);
        self.filter.update_freq(param1 * 10000.0);
        self.filter.update_q(param2 * 20.0);
        let freq = pitch_to_freq(pitch);
        self.osc.reset(); // resetting the phase is optional!
        self.osc.set_freq(freq);
        self.env.trigger(velocity);
    }

    fn reset(&mut self) {
        self.env.decay();
        self.osc.reset();
    }

    fn stop(&mut self) {
        self.env.decay();
        self.pitch = None;
    }

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        todo!()
    }

    fn get_pitch(&self) -> u8 {
        self.pitch.unwrap_or(0)
    }

    fn is_active(&self) -> bool {
        self.env.is_active()
    }
}
