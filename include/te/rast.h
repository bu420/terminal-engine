#pragma once

#include <stdint.h>

typedef struct {
    uint8_t r, g, b;
} rgb;

typedef struct {
    uint32_t ch;
    rgb fg, bg;
} char_info;

extern float z_buf[WIDTH * HEIGHT];
extern char_info char_buf[WIDTH * HEIGHT];

float edge(vec a, vec b, vec c) {
    return (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0]);
}

void triangle(vec v0, vec v1, vec v2, vec tc0, vec tc1, vec tc2) {
    v0[0] /= v0[2]; v0[1] /= v0[2];
    v1[0] /= v1[2]; v1[1] /= v1[2];
    v2[0] /= v2[2]; v2[1] /= v2[2];
    
    v0[0] = (1 + v0[0]) * 0.5 * WIDTH; v0[1] = (1 + v0[1]) * 0.5 * HEIGHT;
    v1[0] = (1 + v1[0]) * 0.5 * WIDTH; v1[1] = (1 + v1[1]) * 0.5 * HEIGHT;
    v2[0] = (1 + v2[0]) * 0.5 * WIDTH; v2[1] = (1 + v2[1]) * 0.5 * HEIGHT;

    tc0[0] /= v0[2]; tc0[1] /= v0[2]; 
    tc1[0] /= v1[2]; tc1[1] /= v1[2]; 
    tc2[0] /= v2[2]; tc2[1] /= v2[2]; 

    v0[2] = 1 / v0[2]; 
    v1[2] = 1 / v1[2]; 
    v2[2] = 1 / v2[2]; 

    float area = edge(v0, v1, v2);

    for (int j = 0; j < HEIGHT; ++j) {
        for (int i = 0; i < WIDTH; ++i) {
            vec p = {i + 0.5f, j + 0.5f};

            float w0 = edge(v1, v2, p);
            float w1 = edge(v2, v0, p);
            float w2 = edge(v0, v1, p);

            if (w0 >= 0 && w1 >= 0 && w2 >= 0) {
                w0 /= area; 
                w1 /= area; 
                w2 /= area;

                float z = 1 / (w0 * v0[2] + w1 * v1[2] + w2 * v2[2]);

                if (z > z_buf[j * WIDTH + i]) {
                    z_buf[j * WIDTH + i] = z;

                    float s = (w0 * tc0[0] + w1 * tc1[0] + w2 * tc2[0]) * z;
                    float t = (w0 * tc0[1] + w1 * tc1[1] + w2 * tc2[1]) * z;

                    clock_t uptime = clock() / (CLOCKS_PER_SEC / 1000);
                    float M = 2.0f;
                    float pattern = (fmod(s * M, 1.0) > 0.5) ^ (fmod(t * M, 1.0) < 0.5);
                    if (pattern < 0.5f) pattern = 0.4f;

                    char_info info;
                    info.ch = 0x8896e2;
                    info.fg = (rgb){255, 0, 0};
                    info.bg = (rgb){0, 0, 255};

                    char_buf[j * WIDTH + i] = info;
                }
            }
        }
    }
}
