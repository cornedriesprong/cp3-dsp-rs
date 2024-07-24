use crate::karplus::KarplusVoice;
use crate::sequencer::{Message, ScheduledEvent, Sequencer};
use crate::synth::Synth;
use crossbeam::channel::Receiver;
use std::collections::HashMap;

pub struct Engine {
    sequencer: Sequencer,
    synth: Synth<KarplusVoice>,
}

impl Engine {
    pub fn new(rx: Receiver<Message>) -> Self {
        Engine {
            sequencer: Sequencer::new(rx, 4.),
            synth: Synth::<KarplusVoice>::new(),
        }
    }

    pub fn process(
        &mut self,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
        sample_time: i32,
        tempo: f32,
        num_frames: i32,
    ) {
        let mut events = HashMap::new();
        self.sequencer
            .process(&mut events, sample_time, tempo, num_frames);

        let mut frame = 0;
        for (l, r) in buf_l.iter_mut().zip(buf_r.iter_mut()) {
            // play scheduled events
            if let Some(ev) = events.get(&frame) {
                for event in ev.iter() {
                    match event {
                        ScheduledEvent::NoteOn {
                            time: _,
                            pitch,
                            velocity,
                            param1,
                            param2,
                        } => {
                            self.synth
                                .play(*pitch as u8, *velocity as u8, *param1, *param2);
                        }
                        ScheduledEvent::NoteOff { time: _, pitch } => {
                            self.synth.stop(*pitch as u8);
                        }
                    }
                }
            }

            // process audio
            let mut left = 0.0;
            let mut right = 0.0;
            self.synth.process(&mut left, &mut right);
            *l += left;
            *r += right;
            frame += 1;
        }
    }
}

#[cfg(test)]
mod tests {}
