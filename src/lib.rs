use crossbeam::channel;
use engine::Engine;
use lazy_static::lazy_static;
use sequencer::{Event, Message};
use std::os::raw::c_float;
use std::sync::Mutex;

pub mod consts;
pub mod delay;
pub mod drums;
pub mod engine;
pub mod envelopes;
pub mod filters;
pub mod karplus;
pub mod limiter;
pub mod osc;
pub mod plaits_voice;
pub mod plot;
pub mod reverb;
pub mod sequencer;
pub mod subtractive;
pub mod synth;
pub mod utils;

// Callback type definition
type PlaybackProgressCallback = extern "C" fn(f32);

type NotePlayedCallback = extern "C" fn(bool, i8, i8);

lazy_static! {
    static ref CHANNEL: Mutex<(channel::Sender<Message>, channel::Receiver<Message>)> =
        Mutex::new(channel::unbounded());
    static ref PROGRESS_CALLBACK: Mutex<Option<PlaybackProgressCallback>> = Mutex::new(None);
    static ref NOTE_CALLBACK: Mutex<Option<NotePlayedCallback>> = Mutex::new(None);
}

fn get_sender() -> channel::Sender<Message> {
    CHANNEL.lock().unwrap().0.clone()
}

fn get_receiver() -> channel::Receiver<Message> {
    CHANNEL.lock().unwrap().1.clone()
}

#[no_mangle]
pub extern "C" fn set_playback_progress_callback(callback: PlaybackProgressCallback) {
    let mut cb = PROGRESS_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

#[no_mangle]
pub extern "C" fn set_note_played_callback(callback: NotePlayedCallback) {
    let mut cb = NOTE_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

#[no_mangle]
pub extern "C" fn engine_init(sample_rate: f32) -> *mut Engine<'static> {
    let rx = get_receiver();
    let engine = Engine::new(rx, sample_rate);
    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn set_play_pause(engine: *mut Engine, is_playing: bool) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    engine.is_playing = is_playing;
}

#[no_mangle]
pub extern "C" fn add_event(
    beat_time: f32,
    pitch: i8,
    velocity: i8,
    duration: f32,
    track: i8,
    param1: f32,
    param2: f32,
) {
    let sender = get_sender();
    let event = Event {
        beat_time,
        pitch,
        velocity,
        duration,
        track,
        param1,
        param2,
    };
    sender.send(Message::Schedule(event)).unwrap();
}

#[no_mangle]
pub extern "C" fn note_on(
    engine: *mut Engine,
    pitch: i8,
    velocity: i8,
    track: i8,
    param1: f32,
    param2: f32,
) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    engine.note_on(pitch as u8, velocity as u8, track, param1, param2);
}

#[no_mangle]
pub extern "C" fn note_off(engine: *mut Engine, pitch: i8, track: i8) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    engine.note_off(pitch as u8, track);
}

#[no_mangle]
pub extern "C" fn set_sound(engine: *mut Engine, sound: i8, track: i8) {
    todo!();
}

#[no_mangle]
pub extern "C" fn set_parameter(parameter: i8, value: f32, track: i8) {
    let sender = get_sender();
    sender
        .send(Message::ParameterChange(parameter, value, track))
        .unwrap();
}

#[no_mangle]
pub extern "C" fn clear_events() {
    let sender = get_sender();
    sender.send(Message::Clear).unwrap();
}

#[no_mangle]
pub extern "C" fn render(
    engine: *mut Engine,
    buf_l: *mut c_float,
    buf_r: *mut c_float,
    sample_time: i64,
    tempo: f32,
    num_frames: i32,
) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    let buf_l = unsafe { std::slice::from_raw_parts_mut(buf_l, num_frames as usize) };
    let buf_r = unsafe { std::slice::from_raw_parts_mut(buf_r, num_frames as usize) };
    engine.process(buf_l, buf_r, sample_time, tempo, num_frames);
}

#[no_mangle]
pub extern "C" fn engine_free(ptr: *mut Engine) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
}
