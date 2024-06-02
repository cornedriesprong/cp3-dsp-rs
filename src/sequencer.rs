use crate::consts::SAMPLE_RATE;
use std::sync::mpsc::Receiver;
use std::{collections::HashMap, usize};

#[derive(Clone)]
pub struct Event {
    pub beat_time: f32,
    pub pitch: i8,
    pub velocity: i8,
    pub param1: f32,
    pub param2: f32,
    pub duration: f32,
}

pub enum Message {
    Add(Event),
    Clear,
}

#[derive(Clone, Debug)]
pub enum ScheduledEvent {
    NoteOn {
        time: i32,
        pitch: i8,
        velocity: i8,
        param1: f32,
        param2: f32,
    },
    NoteOff {
        time: i32,
        pitch: i8,
    },
}

pub struct Sequencer {
    sequence: Sequence,
    scheduled_events: Vec<ScheduledEvent>,
    rx: Receiver<Message>,
}

impl Sequencer {
    pub fn new(rx: Receiver<Message>, length: f32) -> Self {
        Sequencer {
            sequence: Sequence {
                events: Vec::new(),
                length,
            },
            scheduled_events: Vec::new(),
            rx,
        }
    }

    pub fn process(
        &mut self,
        events: &mut HashMap<usize, Vec<ScheduledEvent>>,
        sample_time: i32,
        tempo: f32,
        num_frames: i32,
    ) {
        self.get_msgs();

        let length = Self::beat_to_samples(self.sequence.length, tempo);
        let buffer_start = sample_time % length;
        let buffer_end = buffer_start + num_frames;

        for ev in &self.sequence.events {
            let mut event_time = Self::beat_to_samples(ev.beat_time, tempo);
            let mut is_in_buffer = Self::is_in_buffer(event_time, buffer_start, buffer_end);

            // check if event loops around (ie, is in beginning of next buffer)
            if Self::loops_around(event_time, buffer_end, length) {
                is_in_buffer = true;
                event_time += length - buffer_start;
            }

            if is_in_buffer {
                let note_on = ScheduledEvent::NoteOn {
                    time: event_time,
                    pitch: ev.pitch,
                    velocity: ev.velocity,
                    param1: ev.param1,
                    param2: ev.param2,
                };
                // TODO: stop already playing notes at same pitch
                self.scheduled_events.push(note_on);

                let duration = Self::beat_to_samples(ev.duration, tempo);
                let note_off = ScheduledEvent::NoteOff {
                    time: (event_time + duration) % length,
                    pitch: ev.pitch,
                };
                self.scheduled_events.push(note_off);
            }
        }

        for frame_offset in 0..num_frames {
            let mut to_remove = Vec::new();

            for (index, ev) in self.scheduled_events.iter().enumerate() {
                let event_time = match *ev {
                    ScheduledEvent::NoteOn { time, .. } | ScheduledEvent::NoteOff { time, .. } => {
                        time
                    }
                };

                if event_time == (buffer_start + frame_offset) % length {
                    if !events.contains_key(&(frame_offset as usize)) {
                        events.insert(frame_offset as usize, vec![(*ev).clone()]);
                    } else {
                        events
                            .get_mut(&(frame_offset as usize))
                            .unwrap()
                            .push((*ev).clone());
                    }
                    to_remove.push(index);
                }
            }

            for index in to_remove.iter().rev() {
                self.scheduled_events.swap_remove(*index);
            }
        }
    }

    pub fn beat_to_samples(beat_time: f32, tempo: f32) -> i32 {
        (beat_time / tempo * 60.0 * SAMPLE_RATE as f32) as i32
    }

    fn get_msgs(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Add(event) => {
                    self.sequence.events.push(event);
                }
                Message::Clear => {
                    self.sequence.events.clear();
                }
            }
        }
    }

    fn is_in_buffer(time: i32, buffer_start: i32, buffer_end: i32) -> bool {
        time >= buffer_start && time < buffer_end
    }

    fn loops_around(time: i32, buffer_end: i32, length: i32) -> bool {
        buffer_end > length && time <= (buffer_end % length)
    }
}

struct Sequence {
    events: Vec<Event>,
    length: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequencer;

    #[test]
    fn new_creates_sequencer() {
        let (_, rx) = std::sync::mpsc::channel();
        let sequencer = Sequencer::new(rx, 4.);
        assert_eq!(sequencer.sequence.events.len(), 0);
        assert_eq!(sequencer.sequence.length, 4.);
    }

    #[test]
    fn add_event() {
        let (tx, rx) = std::sync::mpsc::channel();
        let length = 4.;
        let mut sequencer = Sequencer::new(rx, length);
        let tempo = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        _ = tx.send(sequencer::Message::Add(event)).is_ok();

        // process one block to move event to scheduled events
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);
        assert_eq!(sequencer.sequence.events.len(), 1);
        assert_eq!(sequencer.sequence.events[0].beat_time, beat_time);
        assert_eq!(sequencer.sequence.events[0].pitch, 60);
        assert_eq!(sequencer.sequence.events[0].velocity, 100);
        assert_eq!(sequencer.sequence.events[0].param1, 0.0);
        assert_eq!(sequencer.sequence.events[0].param2, 0.0);
        assert_eq!(sequencer.sequence.events[0].duration, duration);
    }

    #[test]
    fn polyphonic_event() {
        let (tx, rx) = std::sync::mpsc::channel();
        let length = 4.;
        let mut sequencer = Sequencer::new(rx, length);
        let tempo = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;

        let ev1 = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        _ = tx.send(sequencer::Message::Add(ev1)).is_ok();

        let ev2 = Event {
            beat_time,
            pitch: 67,
            velocity: 100,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        _ = tx.send(sequencer::Message::Add(ev2)).is_ok();

        // process one block to move event to scheduled events
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);
        assert_eq!(sequencer.sequence.events.len(), 2);
        assert_eq!(sequencer.sequence.events[0].beat_time, beat_time);
        assert_eq!(sequencer.sequence.events[0].pitch, 60);
        assert_eq!(sequencer.sequence.events[0].velocity, 100);
        assert_eq!(sequencer.sequence.events[0].param1, 0.0);
        assert_eq!(sequencer.sequence.events[0].param2, 0.0);
        assert_eq!(sequencer.sequence.events[0].duration, duration);

        assert_eq!(sequencer.sequence.events[1].beat_time, beat_time);
        assert_eq!(sequencer.sequence.events[1].pitch, 67);
        assert_eq!(sequencer.sequence.events[1].velocity, 100);
        assert_eq!(sequencer.sequence.events[1].param1, 0.0);
        assert_eq!(sequencer.sequence.events[1].param2, 0.0);
        assert_eq!(sequencer.sequence.events[1].duration, duration);
    }

    #[test]
    fn clear_events() {
        let (tx, rx) = std::sync::mpsc::channel();
        let length = 4.;
        let mut sequencer = Sequencer::new(rx, length);
        let tempo: f32 = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        _ = tx.send(sequencer::Message::Add(event)).is_ok();

        // process one block to move event to scheduled events
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);

        assert_eq!(sequencer.sequence.events.len(), 1);

        // clear events
        _ = tx.send(sequencer::Message::Clear).is_ok();
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);
        assert_eq!(sequencer.sequence.events.len(), 0);
    }

    #[test]
    fn schedule_event() {
        let (tx, rx) = std::sync::mpsc::channel();
        let length = 4.;
        let mut sequencer = Sequencer::new(rx, length);

        let tempo: f32 = 120.0;
        let frame_count = 60.0 / tempo * length * SAMPLE_RATE as f32;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        _ = tx.send(sequencer::Message::Add(event)).is_ok();

        for i in 0..frame_count as usize {
            let mut events = HashMap::new();
            sequencer.process(&mut events, i as i32, tempo, 1);
            let sample_time = Sequencer::beat_to_samples(beat_time, tempo);
            let duration_in_samples = Sequencer::beat_to_samples(duration, tempo);
            if i == sample_time as usize {
                match events.get(&0).unwrap()[0] {
                    ScheduledEvent::NoteOn {
                        time: _,
                        pitch,
                        velocity,
                        param1,
                        param2,
                    } => {
                        assert_eq!(pitch, 60);
                        assert_eq!(velocity, 100);
                        assert_eq!(param1, 0.0);
                        assert_eq!(param2, 0.0);
                    }
                    _ => panic!("expected note on"),
                }
            } else if i == (sample_time + duration_in_samples) as usize {
                match events.get(&0).unwrap()[0] {
                    ScheduledEvent::NoteOff { time: _, pitch } => {
                        assert_eq!(pitch, 60)
                    }
                    _ => panic!("expected note on"),
                }
            } else {
                assert!(events.get(&0).is_none());
            }
        }
    }
}
