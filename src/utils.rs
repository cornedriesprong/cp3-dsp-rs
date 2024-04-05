// TODO: put constants in a separate file or module
pub const A4_FREQ: f32 = 440.0;
pub const A4_MIDI: u8 = 69;

/// test signals
pub const DC_SIGNAL: [f32; 7] = [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
pub const NYQUIST_SIGNAL: [f32; 7] = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0];
pub const HALF_NYQUIST_SIGNAL: [f32; 7] = [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0];
pub const QUARTER_NYQUIST_SIGNAL: [f32; 9] =
    [0.0, 0.707, 1.0, 0.707, 0.0, -0.707, -1.0, -0.707, 0.0];
pub const IMPULSE_SIGNAL: [f32; 7] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

/// utility functions
pub fn pitch_to_freq(pitch: u8) -> f32 {
    A4_FREQ * (2f32).powf((pitch as f32 - A4_MIDI as f32) as f32 / 12.0)
}

pub fn freq_to_pitch(freq: f32) -> u8 {
    ((freq / A4_FREQ).log2() * 12.0 + A4_MIDI as f32).round() as u8
}

pub fn freq_to_period(sample_rate: f32, freq: f32) -> f32 {
    sample_rate / freq
}

pub fn scale_log(value: f32, min: f32, max: f32) -> f32 {
    min * (max / min).powf(value)
}

pub fn lerp(x: f32, length: f32) -> f32 {
    x / length
}

pub fn xerp(x: f32, length: f32, pow: i8) -> f32 {
    (x / length).powf(pow as f32)
}

pub fn lin_to_log(lin: f32, lin_min: f32, lin_max: f32, min_log: f32, max_log: f32) -> f32 {
    let lin_norm = (lin - lin_min) / (lin_max - lin_min);
    return min_log * (max_log / min_log).powf(lin_norm);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        assert_eq!(pitch_to_freq(0), 8.175798);
        assert_eq!(pitch_to_freq(69), 440.0);
        assert_eq!(pitch_to_freq(127), 12543.855);
    }

    #[test]
    fn test_freq_to_midi() {
        assert_eq!(freq_to_pitch(8.17), 0);
        assert_eq!(freq_to_pitch(440.0), 69);
        assert_eq!(freq_to_pitch(12543.855), 127);
    }
}
