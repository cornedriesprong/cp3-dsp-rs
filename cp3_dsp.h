/* CP3 DSP */

#ifndef CP3_DSP_H
#define CP3_DSP_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define SAMPLE_RATE 48000.0

#define A4_FREQ 440.0

#define A4_MIDI 69

#define MAX_BUFFER_SIZE 8192

#define VOICE_COUNT 8

typedef struct Engine Engine;

typedef void (*PlaybackProgressCallback)(float);

void set_playback_progress_callback(PlaybackProgressCallback callback);

struct Engine *engine_init(void);

void add_event(float beat_time,
               int8_t pitch,
               int8_t velocity,
               float duration,
               float param1,
               float param2);

void note_on(struct Engine *engine, int8_t pitch, int8_t velocity, float param1, float param2);

void note_off(struct Engine *engine, int8_t pitch);

void clear_events(void);

void render(struct Engine *engine,
            float *buf_l,
            float *buf_r,
            int64_t sample_time,
            float tempo,
            int32_t num_frames);

void engine_free(struct Engine *ptr);

#endif /* CP3_DSP_H */
