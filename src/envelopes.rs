use crate::utils::{lerp, xerp};

pub enum EnvelopeState {
    Attack,
    Release,
    Off,
}

pub enum CurveType {
    Linear,
    Exponential { pow: i8 },
    // Logarithmic,
}

/*
    Attack/Release envelope
*/
pub struct AR {
    attack_ms: f32,
    release_ms: f32,
    delta: f32,
    time: f32,
    velocity: f32,
    state: EnvelopeState,
    curve_type: CurveType,
    sample_rate: f32,
}

impl AR {
    pub fn new(attack_ms: f32, release_ms: f32, curve_type: CurveType, sample_rate: f32) -> Self {
        let ar = AR {
            attack_ms,
            release_ms,
            delta: 0.0,
            time: 0.0,
            velocity: 1.0,
            state: EnvelopeState::Off,
            curve_type,
            sample_rate,
        };

        ar
    }

    pub fn trigger(&mut self, velocity: u8) {
        self.velocity = velocity as f32 / 127.0;
        self.state = EnvelopeState::Attack;
    }

    pub fn release(&mut self) {
        // TODO: trigger release
        self.state = EnvelopeState::Off;
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        use EnvelopeState as E;
        match self.state {
            E::Attack => {
                let length = self.attack_ms * (self.sample_rate / 1000.0);
                if length == 0.0 {
                    self.delta = 1.0;
                } else {
                    self.delta = self.get_curve(length) * self.velocity;
                }

                if self.delta >= 1.0 {
                    self.delta = 1.0;
                    self.time = 0.0;
                    self.state = E::Release;
                }
            }
            E::Release => {
                let length = self.release_ms * (self.sample_rate / 1000.0);
                self.delta = self.get_curve_rev(length) * self.velocity;
                if self.delta <= 0.0 {
                    self.delta = 0.0;
                    self.time = 0.0;
                    self.state = E::Off;
                }
            }
            E::Off => {
                self.time = 0.0;
            }
        }
        self.time += 1.0;

        self.delta
    }

    fn get_curve(&self, length: f32) -> f32 {
        match self.curve_type {
            CurveType::Linear => lerp(self.time, length),
            CurveType::Exponential { pow } => xerp(self.time, length, pow),
        }
    }

    fn get_curve_rev(&self, length: f32) -> f32 {
        match self.curve_type {
            CurveType::Linear => 1.0 - lerp(self.time, length),
            CurveType::Exponential { pow } => xerp(length - self.time, length, pow),
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            EnvelopeState::Attack => true,
            EnvelopeState::Release => true,
            _ => false,
        }
    }

    pub fn set_attack(&mut self, attack_ms: f32) {
        self.attack_ms = attack_ms;
    }

    pub fn set_release(&mut self, release_ms: f32) {
        self.release_ms = release_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_new_ar_envelope() {
        let attack = 10.0;
        let release = 500.0;
        let sample_rate = 48000.0;
        let ar = AR::new(
            attack,
            release,
            CurveType::Exponential { pow: 2 },
            sample_rate,
        );

        assert_eq!(ar.attack_ms, attack);
        assert_eq!(ar.release_ms, release);
    }

    #[test]
    fn test_lin_attack() {
        let attack_ms = 10.0;
        let release_ms = 500.0;
        let sample_rate = 48000.0;
        let mut ar = AR::new(attack_ms, release_ms, CurveType::Linear, sample_rate);

        ar.trigger(127);
        matches!(ar.state, EnvelopeState::Attack);
        matches!(ar.curve_type, CurveType::Linear);
        assert_eq!(ar.process(), 0.0);
        assert_eq!(ar.process(), 0.0020833334);
    }

    #[test]
    fn test_exp_attack() {
        let attack = 10.0;
        let release = 500.0;
        let sample_rate = 48000.0;
        let mut ar = AR::new(
            attack,
            release,
            CurveType::Exponential { pow: 2 },
            sample_rate,
        );

        ar.trigger(127);
        matches!(ar.state, EnvelopeState::Attack);
        matches!(ar.curve_type, CurveType::Exponential { pow: 2 });
        assert_eq!(ar.process(), 0.0);
        assert_eq!(ar.process(), 4.3402783e-6);
    }

    #[test]
    fn test_lin_release() {
        let attack = 0.0;
        let release = 5.0;
        let sample_rate = 48000.0;
        let mut ar = AR::new(attack, release, CurveType::Linear, sample_rate);

        ar.trigger(127);
        matches!(ar.state, EnvelopeState::Release);
        assert_eq!(ar.process(), 1.0);
        assert_eq!(ar.process(), 0.99583334);
    }

    #[test]
    fn test_exp_release() {
        let attack = 0.0;
        let release = 5.0;
        let sample_rate = 48000.0;
        let mut ar = AR::new(
            attack,
            release,
            CurveType::Exponential { pow: 2 },
            sample_rate,
        );

        ar.trigger(127);
        matches!(ar.state, EnvelopeState::Release);
        assert_eq!(ar.process(), 1.0);
        assert_eq!(ar.process(), 0.991684);
    }

    #[test]
    fn test_is_active() {
        let attack = 1.0;
        let release = 1.0;
        let sample_rate = 48000.0;
        let mut ar = AR::new(
            attack,
            release,
            CurveType::Exponential { pow: 2 },
            sample_rate,
        );
        assert_eq!(ar.is_active(), false);

        ar.trigger(127);
        assert_eq!(ar.is_active(), true);

        for _ in 0..100 {
            // play envelope
            ar.process();
        }
        assert_eq!(ar.is_active(), false);
    }
}
