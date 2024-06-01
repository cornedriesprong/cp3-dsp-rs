use crate::consts::SAMPLE_RATE;
use crate::karplus::KarplusVoice;
use crate::synth::Synth;
use std::sync::mpsc::Receiver;

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

pub struct Sequencer {
    sequence: Sequence,
    scheduled_events: Vec<ScheduledEvent>,
    rx: Receiver<Message>,
    synth: Synth<KarplusVoice>,
}

impl Sequencer {
    pub fn new(rx: Receiver<Message>) -> Self {
        Sequencer {
            sequence: Sequence {
                events: Vec::new(),
                length: 4.,
            },
            scheduled_events: Vec::new(),
            rx,
            synth: Synth::<KarplusVoice>::new(),
        }
    }

    fn get_msgs(&mut self) {
        if let Ok(msg) = self.rx.try_recv() {
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

    fn beat_to_samples(beat_time: f32, tempo: f32) -> i32 {
        (beat_time / tempo * 60.0 * SAMPLE_RATE as f32) as i32
    }

    fn is_in_buffer(time: i32, buffer_start: i32, buffer_end: i32) -> bool {
        time >= buffer_start && time < buffer_end
    }

    fn loops_around(time: i32, buffer_end: i32, length: i32) -> bool {
        buffer_end > length && time <= (buffer_end % length)
    }

    pub fn process(
        &mut self,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
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

        for frame in buffer_start..buffer_end {
            let mut to_remove = Vec::new();

            for (index, ev) in self.scheduled_events.iter().enumerate() {
                // Function to extract the time field from an event
                let event_time = match *ev {
                    ScheduledEvent::NoteOn { time, .. } | ScheduledEvent::NoteOff { time, .. } => {
                        time
                    }
                };

                // Compare the extracted time with frame % length
                if event_time == frame % length {
                    match ev {
                        ScheduledEvent::NoteOn {
                            time: _,
                            pitch,
                            velocity,
                            param1,
                            param2,
                        } => self
                            .synth
                            .play(*pitch as u8, *velocity as u8, *param1, *param2),
                        ScheduledEvent::NoteOff { time: _, pitch } => self.synth.stop(*pitch as u8),
                    }
                    to_remove.push(index);
                }
            }

            for index in to_remove.iter().rev() {
                self.scheduled_events.swap_remove(*index);
            }
        }

        for (l, r) in buf_l.iter_mut().zip(buf_r.iter_mut()) {
            let mut left = 0.0;
            let mut right = 0.0;
            self.synth.process(&mut left, &mut right);
            *l += left;
            *r += right;
        }
    }
}

enum ScheduledEvent {
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

struct Sequence {
    events: Vec<Event>,
    length: f32,
}
