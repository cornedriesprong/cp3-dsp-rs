use crate::consts::SAMPLE_RATE;
use crate::envelopes::{CurveType, AR};
use crate::filters::SVF;
use crate::osc::{BlitSawOsc, Osc, Waveform};
use crate::reverb::Reverb;
use crate::utils::{lin_to_log, pitch_to_freq, xerp};

pub const VOICE_COUNT: usize = 8;

pub struct Voice {
    osc: BlitSawOsc,
    env: AR,
    velocity: f32,
    filter: SVF,
    lfo: Osc,
    pitch: Option<u8>,
}

impl SynthVoice for Voice {
    fn new() -> Self {
        Self {
            osc: BlitSawOsc::new(),
            env: AR::new(0.0, 30000.0, CurveType::Exponential { pow: 8 }),
            velocity: 1.0,
            filter: SVF::new(5000.0, 0.707),
            lfo: Osc::new(Waveform::Sine),
            pitch: None,
        }
    }

    #[inline]
    fn process(&mut self) -> f32 {
        if !self.env.is_active() {
            return 0.0;
        }
        // let y = self.osc.process() * self.env.process() * self.velocity;
        let y = self.osc.process();
        self.filter.process(y)
    }

    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.velocity = velocity as f32 / 128.0;
        self.pitch = Some(pitch);
        self.filter.set_frequency(param1 * 10000.0);
        self.filter.set_q(param2 * 20.0);
        let freq = pitch_to_freq(pitch);
        self.osc.reset(); // resetting the phase is optional!
        self.osc.set_freq(freq);
        self.env.trigger(velocity);
    }

    fn reset(&mut self) {
        self.env.release();
        self.osc.reset();
    }

    fn stop(&mut self) {
        self.env.release();
        self.pitch = None;
    }

    fn get_pitch(&self) -> u8 {
        self.pitch.unwrap_or(0)
    }

    fn is_active(&self) -> bool {
        self.env.is_active()
    }
}

pub trait SynthVoice {
    fn new() -> Self;
    fn get_pitch(&self) -> u8;
    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32);
    fn stop(&mut self);
    fn reset(&mut self);
    fn is_active(&self) -> bool;
    fn process(&mut self) -> f32;
}

pub struct Synth<V: SynthVoice> {
    // voices: Vec<SynthVoice>,
    voices: Vec<V>,
    current_voice_index: usize,
    rev_l: Reverb,
    rev_r: Reverb,
    rev_level: f32,
}

impl<V: SynthVoice> Synth<V> {
    pub fn new() -> Self {
        let mut voices = Vec::new();
        for _ in 0..VOICE_COUNT {
            voices.push(V::new());
        }

        Self {
            voices,
            current_voice_index: 0,
            rev_l: Reverb::new(),
            rev_r: Reverb::new(),
            rev_level: 0.5,
        }
    }

    pub fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        // println!("playing note at pitch: {}", pitch);
        let voice = &mut self.voices[self.current_voice_index];
        voice.play(pitch, velocity, param1, param2);
        self.current_voice_index = (self.current_voice_index + 1) % VOICE_COUNT;
    }

    pub fn stop(&mut self, pitch: u8) {
        for voice in self.voices.iter_mut() {
            if voice.get_pitch() == pitch {
                // println!("stopping note at pitch: {}", pitch);
                voice.stop();
            }
        }
    }

    #[inline]
    pub fn process(&mut self, y1: &mut f32, y2: &mut f32) {
        // mix down voices
        let mix = self
            .voices
            .iter_mut()
            .filter(|voice| voice.is_active())
            .fold(0.0, |acc, voice| acc + voice.process())
            / VOICE_COUNT as f32;

        // mix in reverb
        *y1 = mix + (self.rev_l.process(mix) * self.rev_level);
        *y2 = mix + (self.rev_r.process(mix) * self.rev_level);
    }

    pub fn set_filter_freq(&mut self, value: f32) {
        let freq = xerp(value as f32, 1.0, 2);
        let freq = freq * (SAMPLE_RATE / 2.0 as f32);
        for voice in self.voices.iter_mut() {
            // voice.set_filter_freq(freq);
        }
    }

    pub fn set_filter_q(&mut self, value: f32) {
        let q = lin_to_log(value as f32, 0.0, 1.0, 0.5, 50.0);
        for voice in self.voices.iter_mut() {
            // voice.set_filter_q(q);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_synth() {
        let synth = Synth::<Voice>::new();
        assert_eq!(synth.voices.len(), VOICE_COUNT);
        assert_eq!(synth.current_voice_index, 0);
    }
}
