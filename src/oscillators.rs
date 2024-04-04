use crate::utils::A4_FREQ;
use crate::SAMPLE_RATE;
use std::f32::consts::{FRAC_PI_4, PI};
extern crate rand;
use rand::prelude::*;

/*
    Bandlimited Impulse Train (BLIT) Sawtooth Oscillator
    Implementation based on the example in the book "Creating Synthesizer Plug-Ins with C++ and JUCE" by Matthijs Hollemans
*/
pub struct BlitSawOsc {
    period: f32,
    amplitude: f32,
    phase: f32,
    phase_max: f32,
    inc: f32,
    sin0: f32,
    sin1: f32,
    dsin: f32,
    dc: f32,
    saw: f32,
}

impl BlitSawOsc {
    pub fn new() -> Self {
        Self {
            period: 0.0,
            amplitude: 1.0,
            phase: 0.0,
            phase_max: 0.0,
            inc: 0.0,
            sin0: 0.0,
            sin1: 0.0,
            dsin: 0.0,
            dc: 0.0,
            saw: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.inc = 0.0;
        self.phase = 0.0;
        self.sin0 = 0.0;
        self.sin1 = 0.0;
        self.dsin = 0.0;
        self.dc = 0.0;
        self.saw = 0.0;
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let sample = self.next_sample();
        self.saw = self.saw * 0.997 + sample;
        self.saw
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.period = SAMPLE_RATE / freq;
    }

    fn next_sample(&mut self) -> f32 {
        let mut y = 0.0;
        self.phase += self.inc;

        if self.phase <= FRAC_PI_4 {
            let half_period = self.period / 2.0;
            self.phase_max = (0.5 + half_period).floor() - 0.5;
            self.dc = 0.5 * self.amplitude / self.phase_max; // calculate DC offset
            self.phase_max *= std::f32::consts::PI;

            self.inc = self.phase_max / half_period;
            self.phase = -self.phase;

            // digital resonator approximation of a sine function
            self.sin0 = self.amplitude * self.phase.sin();
            self.sin1 = self.amplitude * (self.phase - self.inc).sin();
            self.dsin = 2.0 * self.inc.cos();

            if self.phase * self.phase > 1e-9 {
                y = self.sin0 / self.phase;
            } else {
                y = self.amplitude;
            }
        } else {
            if self.phase > self.phase_max {
                self.phase = self.phase_max + self.phase_max - self.phase;
                self.inc = -self.inc;
            }

            let sinp = self.dsin * self.sin0 - self.sin1;
            self.sin1 = self.sin0;
            self.sin0 = sinp;
            y = sinp / self.phase;
        }

        y - self.dc
    }
}

pub enum Waveform {
    Sine,
    Saw,
    Square,
    Noise,
}

/*
    Naive, non-bandlimited oscillator with multiple waveforms
*/
pub struct Osc {
    waveform: Waveform,
    phase: f32,
    increment: f32,
    rng: rand::rngs::ThreadRng,
}

impl Osc {
    pub fn new(waveform: Waveform) -> Self {
        Self {
            waveform,
            phase: 0.0,
            increment: A4_FREQ / SAMPLE_RATE, // default to 440 Hz
            rng: rand::thread_rng(),
        }
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let output = self.generate_waveform(&self.waveform, self.phase);
        self.phase += self.increment;

        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        output
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.increment = freq / SAMPLE_RATE;
    }

    fn generate_waveform(&self, waveform: &Waveform, phase: f32) -> f32 {
        // TODO: bandlimit waveforms
        // TODO: implement noise
        match waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Saw => phase * 2.0 - 1.0,
            Waveform::Square => {
                if phase < 0.5 {
                    -1.0
                } else {
                    1.0
                }
            }
            Waveform::Noise => self.rng.gen::<f32>() * 2.0 - 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_oscillator() {
        let rate = 440.0;
        let osc = Osc::new(Waveform::Sine);

        assert_eq!(osc.phase, 0.0);
        assert_eq!(osc.increment, rate / SAMPLE_RATE);
    }
}
