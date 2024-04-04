use crate::envelopes::{CurveType, AR};
use crate::filters::SVF;
use crate::karplus::KarplusVoice;
use crate::oscillators::{BlitSawOsc, Osc, Waveform};
use crate::utils::{lin_to_log, pitch_to_freq, xerp};
use crate::SAMPLE_RATE;

pub const VOICE_COUNT: usize = 8;

pub struct Voice {
    osc: BlitSawOsc,
    env: AR,
    velocity: f32,
    filter: SVF,
    lfo: Osc,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            osc: BlitSawOsc::new(),
            env: AR::new(0.0, 30000.0, CurveType::Exponential { pow: 8 }),
            velocity: 1.0,
            filter: SVF::new(),
            lfo: Osc::new(Waveform::Sine),
        }
    }

    pub fn init(&mut self) {
        self.set_filter_freq(1000.0);
        self.set_filter_q(0.707);
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let y = self.osc.process() * self.env.process() * self.velocity;
        self.filter.process(y)
    }

    pub fn play(&mut self, pitch: u8, velocity: u8) {
        self.velocity = velocity as f32 / 128.0;
        self.lfo.set_freq(0.5);
        let freq = pitch_to_freq(pitch);
        self.osc.reset(); // resetting the phase is optional!
        self.osc.set_freq(freq);
        self.env.trigger(velocity);
    }

    pub fn set_attack(&mut self, attack: f32) {
        // self.env.set_attack(attack);
    }

    pub fn set_release(&mut self, release: f32) {
        // self.env.set_release(release);
    }

    pub fn set_filter_freq(&mut self, freq: f32) {
        self.filter.set_frequency(freq);
    }

    pub fn set_filter_q(&mut self, q: f32) {
        self.filter.set_q(q);
    }

    pub fn is_active(&self) -> bool {
        // self.env.is_active()
        true
    }
}

pub struct Synth {
    voices: Vec<KarplusVoice>,
    current_voice_index: usize,
}

impl Synth {
    pub fn new() -> Self {
        let mut voices = Vec::new();
        for _ in 0..VOICE_COUNT {
            let mut voice = KarplusVoice::new(SAMPLE_RATE as f32);
            // voice.init();
            voices.push(voice);
        }

        Self {
            voices,
            current_voice_index: 0,
        }
    }

    pub fn note_on(&mut self, pitch: u8, velocity: u8, freq: Option<f32>, q: Option<f32>) {
        let voice = &mut self.voices[self.current_voice_index];
        voice.play(pitch, velocity);
        if let Some(freq) = freq {
            let freq = xerp(freq, 1.0, 2);
            let freq = freq * (SAMPLE_RATE / 2.0 as f32);
            // voice.set_filter_freq(freq);
        }
        if let Some(q) = q {
            let q = lin_to_log(q, 0.0, 1.0, 0.5, 25.0);
            // voice.set_filter_q(q);
        }
        self.current_voice_index = (self.current_voice_index + 1) % VOICE_COUNT;
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let mut mix = 0.0;
        for voice in self.voices.iter_mut() {
            if voice.is_active() {
                mix += voice.process();
            }
        }

        mix / (self.voices.len() as f32).sqrt()
        // limit the output to -1.0 to 1.0
        // out.max(-1.0).min(1.0)
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
        let synth = Synth::new();

        assert_eq!(synth.voices.len(), VOICE_COUNT);
        assert_eq!(synth.current_voice_index, 0);
    }
}
