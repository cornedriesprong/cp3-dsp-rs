use rusty_link::{AblLink, SessionState};
use serde::Deserialize;
use std::{collections::HashMap, usize};

pub trait LinkTrait {
    fn start(&mut self);
    fn get_beat_time(&mut self) -> f32;
    fn get_tempo(&mut self) -> f32;
}

pub struct Link {
    abl_link: AblLink,
    state: SessionState,
    quantum: f64,
}

impl Link {
    pub fn new() -> Self {
        Self {
            abl_link: AblLink::new(40.),
            state: SessionState::new(),
            quantum: 4.,
        }
    }
}

impl LinkTrait for Link {
    fn start(&mut self) {
        self.abl_link.enable(true);
        self.abl_link.capture_audio_session_state(&mut self.state);
    }

    fn get_beat_time(&mut self) -> f32 {
        self.state
            .beat_at_time(self.abl_link.clock_micros(), self.quantum) as f32
    }

    fn get_tempo(&mut self) -> f32 {
        self.state.tempo() as f32
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Event {
    pub step: f32,
    pub pitch: u8,
    pub duration: f32,
}

#[derive(Debug, PartialEq)]
pub enum Note {
    On { pitch: u8, velocity: u8 },
    Off { pitch: u8 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sequence {
    events: Vec<Event>,
    length: i32,
}

pub struct Sequencer<T: LinkTrait> {
    link: T,
    sequence: Sequence,
    playing_notes: Vec<Event>,
    sample_rate: f32,
}

impl<T: LinkTrait> Sequencer<T> {
    pub fn new(sample_rate: f32, link: T) -> Self {
        Self {
            link,
            sequence: Sequence {
                events: Vec::new(),
                length: 1,
            },
            playing_notes: vec![],
            sample_rate,
        }
    }

    pub fn init(&mut self) {
        self.link.start();
    }

    #[inline]
    pub fn process(&mut self, events: &mut HashMap<usize, Vec<Note>>, frame_count: f32) {
        let tempo = self.link.get_tempo();
        let seq_length_in_samples =
            Self::beat_time_to_samples(self.sequence.length as f32, tempo, self.sample_rate);
        let beat_pos_in_samples = Self::beat_time_to_samples(
            self.link.get_beat_time() % self.sequence.length as f32,
            tempo,
            self.sample_rate,
        );
        let buf_start_time = beat_pos_in_samples % seq_length_in_samples;
        let buf_end_time = buf_start_time + frame_count;

        // stop playing notes
        let mut stopped_notes = vec![];
        for ev in &self.playing_notes {
            let stop_time =
                Self::beat_time_to_samples(ev.step + ev.duration, tempo, self.sample_rate);
            let is_in_buffer = Self::is_in_buffer(stop_time, buf_start_time, buf_end_time);
            let loops_around = Self::loops_around(stop_time, buf_end_time, seq_length_in_samples);
            if is_in_buffer || loops_around {
                let offset = if loops_around {
                    stop_time + (seq_length_in_samples - buf_start_time)
                } else {
                    stop_time - buf_start_time
                };

                if !events.contains_key(&(offset as usize)) {
                    events.insert(offset as usize, vec![Note::Off { pitch: ev.pitch }]);
                } else {
                    events
                        .get_mut(&(offset as usize))
                        .unwrap()
                        .push(Note::Off { pitch: ev.pitch });
                }
                stopped_notes.push(ev.clone());
            }
        }
        for note in stopped_notes {
            self.playing_notes.retain(|n| n != &note);
        }

        for ev in &self.sequence.events {
            let ev_time = Self::beat_time_to_samples(ev.step, tempo, self.sample_rate);
            let is_in_buffer = Self::is_in_buffer(ev_time, buf_start_time, buf_end_time);
            let loops_around = Self::loops_around(ev_time, buf_end_time, seq_length_in_samples);
            if is_in_buffer || loops_around {
                let offset = if loops_around {
                    ev_time + (seq_length_in_samples - buf_start_time)
                } else {
                    ev_time - buf_start_time
                };

                if !events.contains_key(&(offset as usize)) {
                    events.insert(
                        offset as usize,
                        vec![Note::On {
                            pitch: ev.pitch,
                            velocity: 100,
                        }],
                    );
                } else {
                    events.get_mut(&(offset as usize)).unwrap().push(Note::On {
                        pitch: ev.pitch,
                        velocity: 100,
                    });
                }
                self.playing_notes.push(ev.clone());
            }
        }
    }

    pub fn add_event(&mut self, ev: Event) {
        self.sequence.events.push(ev);
    }

    pub fn load_sequence(&mut self, sequence: Sequence) {
        println!("received new sequence...");
        self.sequence = sequence;
    }

    fn beat_time_to_samples(beat_time: f32, tempo: f32, sample_rate: f32) -> f32 {
        beat_time / tempo * 60. * sample_rate
    }

    fn is_in_buffer(ev_time: f32, buf_start_time: f32, buffer_end_time: f32) -> bool {
        ev_time >= buf_start_time && ev_time < buffer_end_time
    }

    fn loops_around(ev_time: f32, buffer_end_time: f32, seq_length_in_samples: f32) -> bool {
        buffer_end_time > seq_length_in_samples
            && ev_time <= (buffer_end_time % seq_length_in_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLink {
        beat_time: f32,
        tempo: f32,
    }

    impl LinkTrait for MockLink {
        fn start(&mut self) {
            self.beat_time = -1.;
        }

        fn get_beat_time(&mut self) -> f32 {
            self.beat_time
        }

        fn get_tempo(&mut self) -> f32 {
            self.tempo += 1.;
            self.tempo
        }
    }

    #[test]
    fn test_beat_time_to_samples() {
        let tempo = 120.;
        let sample_rate = 48000.;
        let beat_time = 1.;
        let samples = Sequencer::<MockLink>::beat_time_to_samples(beat_time, tempo, sample_rate);
        assert_eq!(samples, 24000.);
    }

    #[test]
    fn test_is_in_buffer() {
        let buf_start_time = 0.;
        let buf_end_time = 10.;
        let ev_time = 5.;
        assert_eq!(
            Sequencer::<MockLink>::is_in_buffer(ev_time, buf_start_time, buf_end_time),
            true
        );
    }
}
