use crate::plaits_voice::PlaitsKick;
use crate::sequencer::{ScheduledEvent, Sequencer};
use crate::synth::Synth;
use crate::{Message, NOTE_CALLBACK};
use crossbeam::channel::Receiver;
use std::collections::HashMap;

pub struct Engine {
    sequencer: Sequencer,
    synth: Synth<PlaitsKick>,
    rx: Receiver<Message>,
}

impl Engine {
    pub fn new(rx: Receiver<Message>) -> Self {
        Engine {
            sequencer: Sequencer::new(4.),
            synth: Synth::<PlaitsKick>::new(),
            rx,
        }
    }

    pub fn process(
        &mut self,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
        sample_time: i64,
        tempo: f32,
        num_frames: i32,
    ) {
        self.get_msgs();

        let mut events = HashMap::new();
        self.sequencer
            .process(&mut events, sample_time, tempo, num_frames);

        for frame in 0..num_frames {
            // play scheduled events
            if let Some(ev) = events.get(&(frame as usize)) {
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
                            Self::note_played(true, *pitch);
                        }
                        ScheduledEvent::NoteOff { time: _, pitch } => {
                            self.synth.stop(*pitch as u8);
                            Self::note_played(false, *pitch);
                        }
                    }
                }
            }
        }

        // process audio
        self.synth.process(buf_l, buf_r);
    }

    pub fn note_on(&mut self, pitch: u8, velocity: u8, param1: f32, param2: f32) {
        self.synth.play(pitch, velocity, param1, param2);
    }

    pub fn note_off(&mut self, pitch: u8) {
        self.synth.stop(pitch);
    }

    pub fn set_sound(&mut self, sound: i8) {
        self.synth.set_sound(sound);
    }

    pub fn get_msgs(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Schedule(event) => {
                    self.sequencer.add_event(event);
                }
                Message::Clear => {
                    self.sequencer.clear();
                }
                Message::ParameterChange(parameter, value) => {
                    self.synth.set_parameter(parameter, value);
                }
            }
        }
    }

    fn note_played(note_on: bool, pitch: i8) {
        if let Some(callback) = *NOTE_CALLBACK.lock().unwrap() {
            callback(note_on, pitch);
        }
    }
}

#[cfg(test)]
mod tests {}
