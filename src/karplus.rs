use crate::synth::SynthVoice;
use crate::utils::{freq_to_period, pitch_to_freq};
use rand::Rng;

// this is the number of samples we need to represent a full period
// of the lowest possible MIDI pitch's frequency (A0 / 27.50 Hz)
pub const MAX_BUFFER_SIZE: u16 = 8192;

enum Mode {
    String,
    Drum,
}

pub struct KarplusVoice {
    // mode: Mode,
    tone: f32,
    damping: f32,
    buffer: [f32; MAX_BUFFER_SIZE as usize],
    period: f32,
    read_pos: usize,
    pitch_track: f32,
    is_stopped: bool,
    sample_rate: f32,
}

impl KarplusVoice {
    fn generate_triangle_wave(sample: i32, period: f32) -> f32 {
        let phase = sample as f32 / period;
        if phase < 0.25 {
            4.0 * phase
        } else if phase < 0.75 {
            2.0 - 4.0 * phase
        } else {
            -4.0 + 4.0 * phase
        }
    }
}

impl SynthVoice for KarplusVoice {
    fn new(sample_rate: f32) -> Self {
        Self {
            // mode: Mode::String,
            tone: 0.5,
            damping: 0.5,
            buffer: [0.0; MAX_BUFFER_SIZE as usize],
            period: 0.0,
            read_pos: 0,
            pitch_track: 0.0,
            is_stopped: true,
            sample_rate,
        }
    }

    fn init(&mut self) {
        // no-op
    }

    fn reset(&mut self) {
        self.period = 0.0;
        self.read_pos = 0;
        self.pitch_track = 0.0;
    }

    #[inline]
    fn process(&mut self) -> f32 {
        todo!()
        // if !self.is_active() {
        //     return 0.0;
        // }
        // // increment read position
        // // TODO: is it a problem that we're rounding here?
        // // should we interpolate between buffer values?
        // self.read_pos = (self.read_pos + 1) % self.period as usize;

        // // smooth signal using simple averaging
        // // try more advanced filter
        // let mut sum = 0.0;
        // // let window = 10.0;
        // let mut window = self.damping.powf(2.0);
        // window = (2.0 as f32).max(window * self.pitch_track);
        // for i in 0..window as usize {
        //     let idx = (self.read_pos + i) % self.period as usize;
        //     sum += self.buffer[idx];
        // }
        // self.buffer[self.read_pos] = sum * (1.0 / window);

        // if self.is_stopped {
        //     // fade out note
        //     self.buffer[self.read_pos] *= 0.9;
        // }

        // self.buffer[self.read_pos]
    }

    fn play(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.tone = param1;
        self.damping = param2;

        self.is_stopped = false;
        let freq = pitch_to_freq(pitch);
        self.period = freq_to_period(self.sample_rate, freq);
        self.read_pos = 0;

        self.pitch_track = (5.0 as f32).max(self.period / 7.0);
        assert!(self.period < MAX_BUFFER_SIZE as f32);

        for i in 0..self.period as usize {
            if i > self.period as usize {
                self.buffer[i] = 0.0;
            }
            // generate one period of a sine wave
            // let sine = ((i as f32 / self.period) * (PI * 2.0)).sin();
            let tri = Self::generate_triangle_wave(i as i32, self.period);

            let noise = if rand::thread_rng().gen::<bool>() {
                1.0
            } else {
                -1.0
            };
            let y = (tri * self.tone) + (noise * (1.0 - self.tone));
            self.buffer[i] = y;
        }
    }

    fn stop(&mut self) {
        self.is_stopped = true;
    }

    fn set_parameter(&mut self, parameter: i8, value: f32) {
        todo!()
    }

    fn get_pitch(&self) -> u8 {
        (self.period * 27.5) as u8
    }

    fn is_active(&self) -> bool {
        self.period > 0.0
    }
}
