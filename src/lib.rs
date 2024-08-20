use crossbeam::channel;
use engine::Engine;
use lazy_static::lazy_static;
use sequencer::{Event, Message, Note};
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
pub mod plot;
pub mod reverb;
pub mod sequencer;
pub mod subtractive;
pub mod synth;
pub mod utils;

lazy_static! {
    static ref CHANNEL: Mutex<(channel::Sender<Message>, channel::Receiver<Message>)> =
        Mutex::new(channel::unbounded());
    static ref PROGRESS_CALLBACK: Mutex<Option<PlaybackProgressCallback>> = Mutex::new(None);
}

fn get_sender() -> channel::Sender<Message> {
    CHANNEL.lock().unwrap().0.clone()
}

fn get_receiver() -> channel::Receiver<Message> {
    CHANNEL.lock().unwrap().1.clone()
}

// Callback type definition
type PlaybackProgressCallback = extern "C" fn(f32);

#[no_mangle]
pub extern "C" fn set_playback_progress_callback(callback: PlaybackProgressCallback) {
    let mut cb = PROGRESS_CALLBACK.lock().unwrap();
    *cb = Some(callback);
}

// This function should be called from your audio processing code
// whenever the playback progress changes
pub fn update_playback_progress(progress: f32) {
    if let Some(callback) = *PROGRESS_CALLBACK.lock().unwrap() {
        callback(progress);
    }
}

#[no_mangle]
pub extern "C" fn engine_init() -> *mut Engine {
    let rx = get_receiver();
    let engine = Engine::new(rx);
    Box::into_raw(Box::new(engine))
}

#[no_mangle]
pub extern "C" fn add_event(
    beat_time: f32,
    pitch: i8,
    velocity: i8,
    duration: f32,
    param1: f32,
    param2: f32,
) {
    let sender = get_sender();
    let event = Event {
        beat_time,
        pitch,
        velocity,
        duration,
        param1,
        param2,
    };
    sender.send(Message::Schedule(event)).unwrap();
}

#[no_mangle]
pub extern "C" fn note_on(engine: *mut Engine, pitch: i8, velocity: i8, param1: f32, param2: f32) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    let note_on = Note::NoteOn {
        pitch,
        velocity,
        param1,
        param2,
    };
    engine.note_on(pitch as u8, velocity as u8, param1, param2);
}

#[no_mangle]
pub extern "C" fn note_off(engine: *mut Engine, pitch: i8) {
    let engine = unsafe {
        assert!(!engine.is_null());
        &mut *engine
    };
    let note_off = Note::NoteOff { pitch };
    engine.note_off(pitch as u8);
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
