use crate::SAMPLE_RATE;
use std::f32::consts::PI;

/// a 1st order linear feed-forward FIR lowpass filter
pub struct FIRLowpassFilter {
    a0: f32, // a0 coefficient
    a1: f32, // a1 coefficient
    z: f32,  // 1 sample delay register: z^-1
}

impl FIRLowpassFilter {
    pub fn new(a0: f32, a1: f32) -> Self {
        return Self { a0, a1, z: 0.0 };
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = (x * self.a0) + (self.z * self.a1);
        self.z = x;

        y
    }
}

/*
    One-pole 1st order lowpass filter,
    useful for smoothing control signals
*/
pub struct OnePoleLPF {
    prev: f32,
    alpha: f32,
}

impl OnePoleLPF {
    pub fn new(alpha: f32) -> OnePoleLPF {
        OnePoleLPF { prev: 0.0, alpha }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.prev = ((1.0 - self.alpha) * x) + (self.alpha * self.prev);
        self.prev
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.alpha = Self::calculate_alpha(freq);
    }

    fn calculate_alpha(freq: f32) -> f32 {
        1.0 / (1.0 + PI * freq / SAMPLE_RATE)
    }
}

/*
    Cytomic (Andrew Simper) state-variable filter
*/
pub struct SVF {
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl SVF {
    pub fn new() -> SVF {
        SVF {
            g: 0.0,
            k: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let v3 = x - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        v2 // return lowpass
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.g = (std::f32::consts::PI * freq / SAMPLE_RATE).tan();
        self.update_coefficients();
    }

    pub fn set_q(&mut self, q: f32) {
        self.k = 1.0 / q;
        self.update_coefficients();
    }

    pub fn reset(&mut self) {
        self.g = 0.0;
        self.k = 0.0;
        self.a1 = 0.0;
        self.a2 = 0.0;
        self.a3 = 0.0;
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    fn update_coefficients(&mut self) {
        self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_pole_lpf_dc() {
        let dc = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let mut lpf = OnePoleLPF::new(0.5);
    }

    #[test]
    fn test_1st_fff_dc() {
        let dc = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    }

    #[test]
    fn test_svf_dc() {
        let dc = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let mut svf = SVF::new();
        svf.set_frequency(0.5);
        svf.set_q(0.5);
        // assert_eq!(lpf.process(dc[0]), 0.0);
        // assert_eq!(lpf.process(dc[1]), 0.5);
        // assert_eq!(lpf.process(dc[2]), 1.0);
    }

    #[test]
    fn test_svf_ir() {
        // let mut impulse = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut impulse = vec![1.0];
        for i in 0..700 {
            impulse.push(0.0);
            println!("{}", impulse[i]);
        }
        let mut svf = SVF::new();
        svf.set_frequency(0.5);
        svf.set_q(0.5);
        println!("svf ir");
        for i in 0..700 {
            println!("{}", svf.process(impulse[i]));
        }
        // assert_eq!(lpf.process(dc[0]), 0.0);
        // assert_eq!(lpf.process(dc[1]), 0.5);
        // assert_eq!(lpf.process(dc[2]), 1.0);
    }

    // #[test]
    // fn test_one_pole_lpf_nyquist() {
    //     let nyquist = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0];
    //     let mut lpf = OnePoleLPF::new(0.5);
    //     assert_eq!(lpf.process(nyquist[0]), 0.5);
    //     assert_eq!(lpf.process(nyquist[1]), 0.0);
    //     assert_eq!(lpf.process(nyquist[2]), 0.0);
    // }
    //
    // #[test]
    // fn test_one_pole_lpf_half_nyquist() {
    //     let half_nyquist = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
    //     let mut lpf = OnePoleLPF::new(0.5);
    //     assert_eq!(lpf.process(half_nyquist[0]), 0.0);
    //     assert_eq!(lpf.process(half_nyquist[1]), 0.5);
    //     assert_eq!(lpf.process(half_nyquist[2]), 1.0);
    // }
    //
    // #[test]
    // fn test_one_pole_lpf_impulse_response() {
    // let impulse = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    //     let mut lpf = OnePoleLPF::new(0.5);
    //     assert_eq!(lpf.process(impulse[0]), 0.0);
    //     assert_eq!(lpf.process(impulse[1]), 0.5);
    //     assert_eq!(lpf.process(impulse[2]), 1.0);
    // }

    #[test]
    fn test_svf() {
        let mut svf = SVF::new();
    }
}
