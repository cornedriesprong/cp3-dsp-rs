use crate::reverb::Reverb;

pub const VOICE_COUNT: usize = 1;

pub trait SynthVoice {
    fn new() -> Self;
    fn init(&mut self);
    fn get_pitch(&self) -> u8;
    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32);
    fn stop(&mut self);
    fn set_parameter(&mut self, parameter: i8, value: f32);
    fn reset(&mut self);
    fn is_active(&self) -> bool;
    fn process(&mut self, buf: &mut [f32]);
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
            let mut voice = V::new();
            voice.init();
            voices.push(voice);
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

    pub fn process(&mut self, buf_l: &mut [f32], buf_r: &mut [f32]) {
        assert_eq!(
            buf_l.len(),
            buf_r.len(),
            "Left and right buffers must be the same length"
        );

        for voice in self.voices.iter_mut().filter(|v| v.is_active()) {
            voice.process(buf_l);
        }

        buf_r.copy_from_slice(buf_l);

        // TODO: mix in reverb
        // *l = mix + (self.rev_l.process(mix) * self.rev_level);
        // *y2 = mix + (self.rev_r.process(mix) * self.rev_level);
    }

    pub fn set_sound(&mut self, sound: i8) {
        // TODO: change voice type
        todo!()
    }

    pub(crate) fn set_parameter(&mut self, parameter: i8, value: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_parameter(parameter, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subtractive::SubtractiveVoice;

    #[test]
    fn new_creates_synth() {
        let synth = Synth::<SubtractiveVoice>::new();
        assert_eq!(synth.voices.len(), VOICE_COUNT);
        assert_eq!(synth.current_voice_index, 0);
    }
}
