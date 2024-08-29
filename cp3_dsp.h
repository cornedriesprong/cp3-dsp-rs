/* CP3 DSP */

#ifndef CP3_DSP_H
#define CP3_DSP_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define A4_FREQ 440.0

#define A4_MIDI 69

#define MAX_BUFFER_SIZE 8192

#define VOICE_COUNT 1

typedef struct Engine Engine;

typedef void (*PlaybackProgressCallback)(float);

typedef void (*NotePlayedCallback)(bool, int8_t, int8_t);

void set_playback_progress_callback(PlaybackProgressCallback callback);

void set_note_played_callback(NotePlayedCallback callback);

struct Engine *engine_init(float sample_rate);

void set_play_pause(struct Engine *engine, bool is_playing);

void add_event(float beat_time,
               int8_t pitch,
               int8_t velocity,
               float duration,
               int8_t track,
               float param1,
               float param2);

void note_on(struct Engine *engine,
             int8_t pitch,
             int8_t velocity,
             int8_t track,
             float param1,
             float param2);

void note_off(struct Engine *engine, int8_t pitch, int8_t track);

void set_sound(struct Engine *engine, int8_t sound, int8_t track);

void set_parameter(int8_t parameter, float value, int8_t track);

void clear_events(void);

void render(struct Engine *engine,
            float *buf_l,
            float *buf_r,
            int64_t sample_time,
            float tempo,
            int32_t num_frames);

void engine_free(struct Engine *ptr);

#endif /* CP3_DSP_H */
