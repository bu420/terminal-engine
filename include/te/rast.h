#pragma once

#include <stdint.h>
#include <stdbool.h>
#include <assert.h>

typedef struct {
    uint8_t r, g, b;
} rgb;

typedef struct {
    vec p, tc, n;
} vertex;

#define CHAR_INFO_DEFAULT_FG 1 << 0
#define CHAR_INFO_DEFAULT_BG 1 << 1

// Each char_info essentially contains 2 pixels since characters in the terminal are twice as high as wide.
typedef struct {
    uint32_t ch;
    uint8_t flags;
    rgb fg, bg;
} char_info;

typedef struct {
    char_info * char_buf;
    float * z_buf;
} framebuf;

typedef char_info (* shader_callback)(vertex v, char_info c, bool overwriting);

float edge(vec a, vec b, vec c) {
    return (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0]);
}

void triangle(framebuf fb, vertex v0, vertex v1, vertex v2, shader_callback shader) {
    v0.p[0] /= v0.p[2]; v0.p[1] /= v0.p[2];
    v1.p[0] /= v1.p[2]; v1.p[1] /= v1.p[2];
    v2.p[0] /= v2.p[2]; v2.p[1] /= v2.p[2];
    
    v0.p[0] = (1 + v0.p[0]) * 0.5 * WIDTH; v0.p[1] = (1 + v0.p[1]) * 0.5 * HEIGHT;
    v1.p[0] = (1 + v1.p[0]) * 0.5 * WIDTH; v1.p[1] = (1 + v1.p[1]) * 0.5 * HEIGHT;
    v2.p[0] = (1 + v2.p[0]) * 0.5 * WIDTH; v2.p[1] = (1 + v2.p[1]) * 0.5 * HEIGHT;

    v0.tc[0] /= v0.p[2]; v0.tc[1] /= v0.p[2]; 
    v1.tc[0] /= v1.p[2]; v1.tc[1] /= v1.p[2]; 
    v2.tc[0] /= v2.p[2]; v2.tc[1] /= v2.p[2]; 

    v0.p[2] = 1 / v0.p[2]; 
    v1.p[2] = 1 / v1.p[2]; 
    v2.p[2] = 1 / v2.p[2]; 

    float area = edge(v0.p, v1.p, v2.p);

    for (int y = 0; y < HEIGHT; ++y) {
        for (int x = 0; x < WIDTH; ++x) {
            vec p = {x + 0.5f, y + 0.5f};

            float w0 = edge(v1.p, v2.p, p);
            float w1 = edge(v2.p, v0.p, p);
            float w2 = edge(v0.p, v1.p, p);

            if (w0 >= 0 && w1 >= 0 && w2 >= 0) {
                w0 /= area; 
                w1 /= area; 
                w2 /= area;

                float z = 1 / (w0 * v0.p[2] + w1 * v1.p[2] + w2 * v2.p[2]);

                if (z > fb.z_buf[y * WIDTH + x]) {
                    fb.z_buf[y * WIDTH + x] = z;

                    vertex v;
                    for (int i = 0; i < 3; i++) {
                        v.p[i] = v0.p[i] * w0 + v1.p[i] * w1 + v2.p[i] * w2;
                        v.tc[i] = z * (v0.tc[i] * w0 + v1.tc[i] * w1 + v2.tc[i] * w2);
                    }
                    
                    fb.char_buf[(y / 2) * WIDTH + x] = shader(v, fb.char_buf[(y / 2) * WIDTH + x], y % 2 != 0);
                }
            }
        }
    }
}