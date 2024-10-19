use crate::delay::Delay;
use crate::limiter::Limiter;
use crate::plaits_voice::FmVoice;
use crate::reverb::Reverb;
use crate::sequencer::{ScheduledEvent, Sequencer};
use crate::{Message, NOTE_CALLBACK};
use crossbeam::channel::Receiver;
use std::collections::HashMap;

pub struct Engine {
    pub is_playing: bool,
    sequencer: Sequencer,
    voices: [FmVoice; 16],
    reverb: Reverb,
    delay: Delay,
    limiter: Limiter,
    rx: Receiver<Message>,
}

impl Engine {
    pub fn new(rx: Receiver<Message>, sample_rate: f32) -> Self {
        Engine {
            is_playing: false,
            sequencer: Sequencer::new(4., sample_rate),
            voices: [FmVoice::new(sample_rate); 16],
            reverb: Reverb::new(sample_rate),
            delay: Delay::new(sample_rate * 0.5, 0.5),
            limiter: Limiter::new(0.1, 0.5, 0.5, sample_rate),
            rx,
        }
    }

    pub fn init(&mut self) {
        println!("Engine init");
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
        self.get_msgs();

        if self.is_playing {
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
                        } => {
                            Self::note_played(true, *pitch, *track);
                            self.voices[*track as usize].trigger(*velocity as u8);
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
            let mut reverb_bus = 0.0;
            let mut delay_bus = 0.0;
            let mut active_voice_count = 1.0;

            for voice in self.voices.iter_mut() {
                if voice.is_active() {
                    let y = voice.process();
                    mix += y;

                    reverb_bus += y * voice.reverb_amt;
                    delay_bus += y * voice.delay_amt;

                    active_voice_count += 1.0;
                }
            }

            mix /= active_voice_count;
            reverb_bus /= active_voice_count;
            delay_bus /= active_voice_count;

            mix += self.reverb.process(reverb_bus);
            mix += self.delay.process(delay_bus);

            // mix = self.limiter.process(mix);

            buf_l[frame as usize] = mix;
            buf_r[frame as usize] = mix;
        }
    }

    pub fn get_msgs(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Schedule(event) => {
                    self.sequencer.add_event(event);
                }
                Message::NoteOn { track, velocity } => {
                    Self::note_played(true, 0, track);
                    self.voices[track as usize].trigger(velocity as u8);
                }
                Message::Clear => {
                    self.sequencer.clear();
                }
                Message::ParameterChange(parameter, value, track) => {
                    self.voices[track as usize].set_parameter(parameter, value);
                }
            }
        }
    }

    fn note_played(note_on: bool, pitch: u8, track: u8) {
        if let Some(callback) = *NOTE_CALLBACK.lock().unwrap() {
            callback(note_on, pitch, track);
        }
    }
}

#[cfg(test)]
mod tests {}
