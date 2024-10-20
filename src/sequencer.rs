use crate::PROGRESS_CALLBACK;
use std::{collections::HashMap, usize};

struct Sequence {
    events: Vec<Event>,
    length: f32,
}

#[derive(Clone)]
pub struct Event {
    pub beat_time: f32,
    pub pitch: u8,
    pub velocity: u8,
    pub param1: f32,
    pub param2: f32,
    pub track: u8,
    pub duration: f32,
}

pub enum Message {
    Schedule(Event),
    ParameterChange(i8, f32, u8),
    NoteOn { track: u8, velocity: u8 },
    Clear,
}

#[derive(Clone, Debug)]
pub enum ScheduledEvent {
    NoteOn {
        time: i32,
        pitch: u8,
        velocity: u8,
        track: u8,
    },
    NoteOff {
        time: i32,
        pitch: u8,
        track: u8,
    },
}

pub struct Sequencer {
    sequence: Sequence,
    scheduled_events: Vec<ScheduledEvent>,
    sample_rate: f32,
}

impl Sequencer {
    pub fn new(length: f32, sample_rate: f32) -> Self {
        Sequencer {
            sequence: Sequence {
                events: Vec::new(),
                length,
            },
            scheduled_events: Vec::new(),
            sample_rate,
        }
    }

    pub fn process(
        &mut self,
        events: &mut HashMap<usize, Vec<ScheduledEvent>>,
        sample_time: i64,
        tempo: f32,
        num_frames: i32,
    ) {
        let length = self.beat_to_sample(self.sequence.length, tempo);
        let buffer_start = (sample_time % length as i64) as i32;
        let buffer_end = buffer_start as i32 + num_frames;

        let beat_time = self.sample_to_beat(sample_time % length as i64, tempo);
        Self::update_playback_progress(beat_time);

        for ev in &self.sequence.events {
            let mut event_time = self.beat_to_sample(ev.beat_time, tempo);
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
                    track: ev.track,
                };
                // TODO: stop already playing notes at same pitch
                self.scheduled_events.push(note_on);

                let duration = self.beat_to_sample(ev.duration, tempo);
                let note_off = ScheduledEvent::NoteOff {
                    time: (event_time + duration) % length,
                    pitch: ev.pitch,
                    track: ev.track,
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

    fn update_playback_progress(progress: f32) {
        if let Some(callback) = *PROGRESS_CALLBACK.lock().unwrap() {
            callback(progress);
        }
    }

    pub fn beat_to_sample(&self, beat_time: f32, tempo: f32) -> i32 {
        (beat_time / tempo * 60.0 * self.sample_rate as f32) as i32
    }

    pub fn sample_to_beat(&self, sample_time: i64, tempo: f32) -> f32 {
        sample_time as f32 / self.sample_rate as f32 * tempo / 60.0
    }

    fn is_in_buffer(time: i32, buffer_start: i32, buffer_end: i32) -> bool {
        time >= buffer_start && time < buffer_end
    }

    fn loops_around(time: i32, buffer_end: i32, length: i32) -> bool {
        buffer_end > length && time <= (buffer_end % length)
    }

    pub(crate) fn add_event(&mut self, event: Event) {
        self.sequence.events.push(event);
    }

    pub(crate) fn clear(&mut self) {
        self.sequence.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_sequencer() {
        // let (_, rx) = channel::unbounded();
        let sample_rate = 48000.0;
        let sequencer = Sequencer::new(4., sample_rate);
        assert_eq!(sequencer.sequence.events.len(), 0);
        assert_eq!(sequencer.sequence.length, 4.);
    }

    #[test]
    fn add_event() {
        let length = 4.;
        let sample_rate = 48000.0;
        let mut sequencer = Sequencer::new(length, sample_rate);
        let tempo = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            track: 0,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        sequencer.add_event(event);

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
        let length = 4.;
        let sample_rate = 48000.0;
        let mut sequencer = Sequencer::new(length, sample_rate);
        let tempo = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;

        let ev1 = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            track: 0,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        sequencer.add_event(ev1);

        let ev2 = Event {
            beat_time,
            pitch: 67,
            velocity: 100,
            track: 0,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        sequencer.add_event(ev2);

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
        let length = 4.;
        let sample_rate = 48000.0;
        let mut sequencer = Sequencer::new(length, sample_rate);
        let tempo: f32 = 120.0;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            track: 0,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        sequencer.add_event(event);

        // process one block to move event to scheduled events
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);

        assert_eq!(sequencer.sequence.events.len(), 1);

        // clear events
        sequencer.clear();
        sequencer.process(&mut HashMap::new(), 0, tempo, 1);
        assert_eq!(sequencer.sequence.events.len(), 0);
    }

    #[test]
    fn schedule_event() {
        let length = 4.;
        let sample_rate = 48000.0;
        let mut sequencer = Sequencer::new(length, sample_rate);

        let tempo: f32 = 120.0;
        let frame_count = 60.0 / tempo * length * sample_rate as f32;
        let beat_time = 1.0;
        let duration = 1.0;
        let event = Event {
            beat_time,
            pitch: 60,
            velocity: 100,
            track: 0,
            param1: 0.0,
            param2: 0.0,
            duration,
        };
        sequencer.add_event(event);

        for i in 0..frame_count as usize {
            let mut events = HashMap::new();
            sequencer.process(&mut events, i as i64, tempo, 1);
            let sample_time = sequencer.beat_to_sample(beat_time, tempo);
            let duration_in_samples = sequencer.beat_to_sample(duration, tempo);
            if i == sample_time as usize {
                match events.get(&0).unwrap()[0] {
                    ScheduledEvent::NoteOn {
                        time: _,
                        pitch,
                        velocity,
                        track,
                    } => {
                        assert_eq!(pitch, 60);
                        assert_eq!(velocity, 100);
                        assert_eq!(track, 0);
                    }
                    _ => panic!("expected note on"),
                }
            } else if i == (sample_time + duration_in_samples) as usize {
                match events.get(&0).unwrap()[0] {
                    ScheduledEvent::NoteOff {
                        time: _,
                        pitch,
                        track: _,
                    } => {
                        assert_eq!(pitch, 60)
                    }
                    _ => panic!("expected note on"),
                }
            } else {
                assert!(events.get(&0).is_none());
            }
        }
    }

    #[test]
    fn check_timing() {
        let length = 4.;
        let sample_rate = 48000.0;
        let mut sequencer = Sequencer::new(length, sample_rate);
        let tempo: f32 = 120.0;
        let frame_count = 60.0 / tempo * length * sample_rate as f32;

        // schedule 4 events at equidistant intervals
        for i in 0..4 {
            let event = Event {
                beat_time: i as f32,
                pitch: 60,
                velocity: 100,
                track: 0,
                param1: 0.0,
                param2: 0.0,
                duration: 1.0,
            };
            sequencer.add_event(event);
        }

        for i in 0..frame_count as usize {
            let mut events = HashMap::new();
            sequencer.process(&mut events, i as i64, tempo, 1);
            // check if we have a note on
            if let Some(ev) = events.get(&0) {
                for ev in ev.iter() {
                    match ev {
                        ScheduledEvent::NoteOn {
                            time,
                            pitch: _,
                            velocity: _,
                            track: _,
                        } => {
                            println!("time: {}", time);
                            assert_eq!(*time, i as i32);
                        }
                        _ => (), // ignore note offs,
                    }
                }
            }
        }
    }
}
