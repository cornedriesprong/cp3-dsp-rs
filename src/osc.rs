use crate::consts::A4_FREQ;
use std::f32::consts::{FRAC_PI_4, PI};
extern crate rand;

/*
    Bandlimited Impulse Train (BLIT) Sawtooth Oscillator
    Implementation based on an example from the book "Creating Synthesizer Plug-Ins with C++ and JUCE" by Matthijs Hollemans
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
    sample_rate: f32,
}

impl BlitSawOsc {
    pub fn new(sample_rate: f32) -> Self {
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
            sample_rate,
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
        self.period = self.sample_rate / freq;
    }

    fn next_sample(&mut self) -> f32 {
        let y;
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
    frequency: f32,
    increment: f32,
    sample_rate: f32,
}

impl Osc {
    pub fn new(waveform: Waveform, sample_rate: f32) -> Self {
        Self {
            waveform,
            phase: 0.0,
            frequency: A4_FREQ,
            increment: 2.0 * PI * A4_FREQ / sample_rate, // default to 440 Hz
            sample_rate,
        }
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let output = self.generate_waveform();
        self.phase += self.increment;

        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        output
    }

    #[inline]
    pub fn process_phase_mod(&mut self, phase_mod: f32) -> f32 {
        let output = self.generate_waveform();
        self.phase += self.increment + phase_mod;

        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        output
    }

    pub fn set_freq(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.increment = 2.0 * PI * frequency / self.sample_rate;
    }

    fn generate_waveform(&self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.phase.sin(),
            Waveform::Saw => 2.0 * (self.phase / (2.0 * PI)) - 1.0,
            Waveform::Square => {
                if self.phase < PI {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Noise => rand::random::<f32>() * 2.0 - 1.0,
        }
    }
}

pub struct FmOsc {
    carrier: Osc,
    modulator: Osc,
    fm_amount: f32,
    mod_index: f32,
}

impl FmOsc {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier: Osc::new(Waveform::Sine, sample_rate),
            fm_amount: 0.5,
            modulator: Osc::new(Waveform::Sine, sample_rate),
            mod_index: 0.5,
        }
    }

    pub fn set_carrier_freq(&mut self, freq: f32) {
        self.carrier.set_freq(freq);
    }

    pub fn set_mod_freq(&mut self, freq: f32) {
        self.modulator.set_freq(freq);
    }

    pub fn set_fm_amount(&mut self, fm_amount: f32) {
        self.fm_amount = fm_amount;
    }

    pub fn set_mod_index(&mut self, mod_index: f32) {
        println!("setting mod index: {}", mod_index);
        self.mod_index = mod_index;
    }

    pub fn reset(&mut self) {
        self.carrier.phase = PI / 2.0;
        self.modulator.phase = 0.0;
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let mod_out = self.modulator.process();
        let mod_signal = self.fm_amount * self.mod_index * mod_out;
        let carrier_out = self.carrier.process_phase_mod(mod_signal);
        carrier_out + (mod_out * (1.0 - self.fm_amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::plot_graph;

    #[test]
    fn create_osc() {
        let rate = 440.0;
        let sample_rate = 48000.0;
        let osc = Osc::new(Waveform::Sine, sample_rate);

        assert_eq!(osc.phase, 0.0);
        assert_eq!(osc.increment, rate / sample_rate);
    }

    #[test]
    fn generate_waveform() {
        let sample_rate = 48000.0;
        let mut osc = Osc::new(Waveform::Sine, sample_rate);
        let output = osc.process();

        assert!(output >= -1.0 && output <= 1.0);
    }

    #[test]
    fn create_blit_osc() {
        let sample_rate = 48000.0;
        let osc = BlitSawOsc::new(sample_rate);
        assert_eq!(osc.period, 0.0);
        assert_eq!(osc.amplitude, 1.0);
        assert_eq!(osc.phase, 0.0);
        assert_eq!(osc.phase_max, 0.0);
        assert_eq!(osc.inc, 0.0);
        assert_eq!(osc.sin0, 0.0);
        assert_eq!(osc.sin1, 0.0);
        assert_eq!(osc.dsin, 0.0);
        assert_eq!(osc.dc, 0.0);
        assert_eq!(osc.saw, 0.0);
    }

    #[test]
    fn blit_generate_waveform() {
        let sample_rate = 48000.0;
        let mut osc = BlitSawOsc::new(sample_rate);
        osc.set_freq(440.0);
        // generate 1st 100 samples
        for _ in 0..100 {
            let output = osc.process();
            assert!(output >= -1.0 && output <= 1.0);
        }
    }

    #[test]
    fn blit_reset() {
        let sample_rate = 48000.0;
        let mut osc = BlitSawOsc::new(sample_rate);
        osc.set_freq(440.0);
        osc.process();
        osc.reset();
        assert_eq!(osc.inc, 0.0);
        assert_eq!(osc.phase, 0.0);
        assert_eq!(osc.sin0, 0.0);
        assert_eq!(osc.sin1, 0.0);
        assert_eq!(osc.dsin, 0.0);
        assert_eq!(osc.dc, 0.0);
        assert_eq!(osc.saw, 0.0);
    }

    #[test]
    fn blit_set_freq() {
        let sample_rate = 48000.0;
        let mut osc = BlitSawOsc::new(sample_rate);
        osc.set_freq(440.0);
        assert_eq!(osc.period, sample_rate / 440.0);
    }

    #[test]
    fn plot_blit_saw() {
        let sample_rate = 48000.0;
        let mut osc = BlitSawOsc::new(sample_rate);
        osc.set_freq(440.0);
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut i = 0.0;
        for _ in 0..4000 {
            xs.push(i);
            ys.push(osc.process());
            i += 1.0;
        }
        // ys.iter().for_each(|y| println!("{}", y));
        plot_graph(&xs, &ys, "blit_saw.png");
    }
}
