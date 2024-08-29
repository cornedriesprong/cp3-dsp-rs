/*
  adapted from https://www.musicdsp.org/en/latest/Filters/265-output-limiter-using-envelope-follower-in-c.html
  not actually sure if it works as it should
*/

pub struct Limiter {
    threshold: f32,
    env_follower: EnvelopeFollower,
    sample_rate: f32,
}

impl Limiter {
    pub fn new(attack: f32, release: f32, threshold: f32, sample_rate: f32) -> Self {
        Self {
            threshold,
            env_follower: EnvelopeFollower::new(attack, release, sample_rate),
            sample_rate,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        self.env_follower.process(input);
        if self.env_follower.env > self.threshold {
            input / self.env_follower.env
        } else {
            input
        }
    }
}

struct EnvelopeFollower {
    attack: f32,
    release: f32,
    env: f32,
}

impl EnvelopeFollower {
    pub fn new(attack: f32, release: f32, sample_rate: f32) -> Self {
        Self {
            // makes attack and release curves exponential?
            attack: (0.01 as f32).powf(1.0 / (attack * sample_rate * 0.001)),
            release: (0.01 as f32).powf(1.0 / (release * sample_rate * 0.001)),
            env: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) {
        let v = input.abs();
        if v > self.env {
            self.env = self.attack * (self.env - v) + v
        } else {
            self.env = self.release * (self.env - v) + v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_new_limiter() {
        let attack = 0.5;
        let release = 0.5;
        let threshold = 0.5;
        let sample_rate = 48000.0;
        let limiter = Limiter::new(attack, release, threshold, sample_rate);

        assert_eq!(limiter.threshold, 0.5);
    }

    #[test]
    fn test_limiter() {
        let attack = 0.0;
        let release = 0.0;
        let threshold = 0.1;
        let sample_rate = 48000.0;
        let mut limiter = Limiter::new(attack, release, threshold, sample_rate);

        // should limit value
        assert_eq!(limiter.process(1.0), 1.0);
        assert_eq!(limiter.process(1.0), 1.0);
        assert_eq!(limiter.process(1.0), 1.0);
        assert_eq!(limiter.process(1.0), 1.0);
        assert_eq!(limiter.process(1.0), 1.0);
        assert_eq!(limiter.process(1.0), 1.0);
    }

    #[test]
    fn creates_new_envelope_follower() {
        let attack = 0.5;
        let release = 0.5;
        let sample_rate = 48000.0;
        let limiter = EnvelopeFollower::new(attack, release, sample_rate);

        assert_eq!(limiter.attack, 0.82540417);
        assert_eq!(limiter.release, 0.82540417);
    }
}
