use crate::delay::Delay;
use crate::karplus::KarplusVoice;
use crate::limiter::Limiter;
use crate::plaits_voice::{FmVoice, PlaitsDrums};
use crate::reverb::Reverb;
use crate::sequencer::{ScheduledEvent, Sequencer};
use crate::synth::SynthVoice;
use crate::{Message, NOTE_CALLBACK};
use crossbeam::channel::Receiver;
use std::collections::HashMap;

pub struct Engine {
    pub is_playing: bool,
    sequencer: Sequencer,
    // synth: BLITVoice,
    synth: FmVoice,
    string: KarplusVoice,
    drums: PlaitsDrums,
    reverb: Reverb,
    delay: Delay,
    limiter: Limiter,
    rx: Receiver<Message>,
}

impl Engine {
    pub fn new(rx: Receiver<Message>, sample_rate: f32) -> Self {
        Engine {
            is_playing: true,
            sequencer: Sequencer::new(4., sample_rate),
            synth: FmVoice::new(sample_rate),
            string: KarplusVoice::new(sample_rate),
            drums: PlaitsDrums::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            delay: Delay::new(sample_rate * 0.5, 0.5),
            limiter: Limiter::new(0.1, 0.5, 0.5, sample_rate),
            rx,
        }
    }

    pub fn init(&mut self) {
        println!("Engine init");
        self.synth.init();
    }

    pub fn process(
        &mut self,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
        sample_time: i64,
        tempo: f32,
        num_frames: i32,
    ) {
        let mut events = HashMap::new();

        if self.is_playing {
            self.get_msgs();

            self.sequencer
                .process(&mut events, sample_time, tempo, num_frames);
        }

        for frame in 0..num_frames {
            // play scheduled events
            if let Some(ev) = events.get(&(frame as usize)) {
                for event in ev.iter() {
                    match event {
                        ScheduledEvent::NoteOn {
                            time: _,
                            pitch,
                            velocity,
                            track,
                            param1,
                            param2,
                        } => {
                            Self::note_played(true, *pitch, *track);
                            match track {
                                0 => {
                                    self.drums
                                        .play(*pitch as u8, *velocity as u8, *param1, *param2)
                                }
                                1 => {
                                    self.synth
                                        .play(*pitch as u8, *velocity as u8, *param1, *param2)
                                }
                                2 => self.string.play(
                                    *pitch as u8,
                                    *velocity as u8,
                                    *param1,
                                    *param2,
                                ),
                                _ => (),
                            }
                        }
                        ScheduledEvent::NoteOff {
                            time: _,
                            pitch,
                            track,
                        } => {
                            // self.synth.stop();
                            Self::note_played(false, *pitch, *track);
                        }
                    }
                }
            }

            let mut mix = 0.0;
            mix += self.drums.process();
            mix += self.synth.process();
            mix += self.string.process();

            mix /= 3.0;

            let reverb = self.reverb.process(mix);
            let delay = self.delay.tick(mix);
            mix += (reverb * 0.1) + (delay * 0.5);

            // mix = self.limiter.process(mix);

            buf_l[frame as usize] = mix;
            buf_r[frame as usize] = mix;
        }
    }

    pub fn note_on(&mut self, pitch: u8, velocity: u8, track: i8, param1: f32, param2: f32) {
        match track {
            0 => self.drums.play(pitch, velocity, param1, param2),
            1 => self.synth.play(pitch, velocity, param1, param2),
            2 => self.string.play(pitch, velocity, param1, param2),
            _ => (),
        }
    }

    pub fn note_off(&mut self, pitch: u8, track: i8) {}

    pub fn set_sound(&mut self, sound: i8, track: i8) {}

    pub fn get_msgs(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Schedule(event) => {
                    self.sequencer.add_event(event);
                }
                Message::Clear => {
                    self.sequencer.clear();
                }
                Message::ParameterChange(parameter, value, track) => match track {
                    0 => self.drums.set_parameter(parameter, value),
                    1 => self.synth.set_parameter(parameter, value),
                    2 => self.string.set_parameter(parameter, value),
                    _ => (),
                },
            }
        }
    }

    fn note_played(note_on: bool, pitch: i8, track: i8) {
        if let Some(callback) = *NOTE_CALLBACK.lock().unwrap() {
            callback(note_on, pitch, track);
        }
    }
}

#[cfg(test)]
mod tests {}
