use crate::envelopes::{CurveType, AR};
use crate::osc::{Osc, Waveform};

pub struct Kick {
    pitch_hz: f32,
    pitch_env_amt: f32,
    osc: Osc,
    amp_env: AR,
    pitch_env: AR,
    click_amt: f32,
    click_env: AR,
    noise: Osc,
}

impl Kick {
    pub fn new(
        pitch_hz: f32,
        pitch_env_amt: f32,
        click_amt: f32,
        release_ms: f32,
        sample_rate: f32,
    ) -> Self {
        Self {
            pitch_hz,
            pitch_env_amt,
            osc: Osc::new(Waveform::Sine, sample_rate),
            amp_env: AR::new(
                1.0,
                release_ms,
                CurveType::Exponential { pow: 3 },
                sample_rate,
            ),
            pitch_env: AR::new(
                0.0,
                release_ms,
                CurveType::Exponential { pow: 3 },
                sample_rate,
            ),
            click_amt,
            click_env: AR::new(0.0, 10.0, CurveType::Exponential { pow: 3 }, sample_rate),
            noise: Osc::new(Waveform::Noise, sample_rate),
        }
    }

    pub fn trigger(&mut self, velocity: u8) {
        self.amp_env.trigger(velocity);
        self.pitch_env.trigger(velocity);
        self.click_env.trigger(velocity);
    }

    pub fn process(&mut self) -> f32 {
        let pitch_env_freq = self.pitch_env_amt * 2000.0; // max pitch env (1.0) is 2000 Hz
        let freq = (self.pitch_env.process() * pitch_env_freq) + self.pitch_hz;
        self.osc.set_freq(freq);
        let click = self.noise.process() * self.click_env.process() * self.click_amt;
        self.amp_env.process() * self.osc.process() + click
    }
}

pub struct Burst {
    env: AR,
    noise: Osc,
}

impl Burst {
    pub fn new(release: f32, sample_rate: f32) -> Self {
        Self {
            env: AR::new(0.0, release, CurveType::Exponential { pow: 2 }, sample_rate),
            noise: Osc::new(Waveform::Noise, sample_rate),
        }
    }

    pub fn trigger(&mut self, velocity: u8) {
        self.env.trigger(velocity);
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        self.env.process() * self.noise.process()
    }
}
