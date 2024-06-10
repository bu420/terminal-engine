#include <stdio.h>
#include <time.h>
#include <string.h>

#define WIDTH 64
#define HEIGHT 64
#define HALF_HEIGHT (HEIGHT / 2)

#include "te/linalg.h"
#include "te/rast.h"
#include "assets"

#define CHAR_BLOCK 0x8896e2
#define CHAR_HALF_BLOCK_TOP 0x8096e2
#define CHAR_HALF_BLOCK_BOTTOM 0x8496e2
#define CHAR_DOTTED_HIGH_DENSITY 0x9396e2
#define CHAR_DOTTED_MEDIUM_DENSITY 0x9296e2
#define CHAR_DOTTED_LOW_DENSITY 0x9196e2

int char_info_to_str(char_info info, char out[], int max) {
    if (info.flags & CHAR_INFO_DEFAULT_FG && info.flags & CHAR_INFO_DEFAULT_BG) {
        memcpy(out, "\x1b[0m ", 5);
        return 5;
    }

    char str[5];
    str[0] = (info.ch >> 0) & 0xFF;
    str[1] = (info.ch >> 8) & 0xFF;
    str[2] = (info.ch >> 16) & 0xFF;
    str[3] = (info.ch >> 24) & 0xFF;
    str[4] = '\0';

    int len = 0;

    if (info.flags & CHAR_INFO_DEFAULT_FG) {
        len = snprintf(out, max, "\x1b[0m\x1b[48;2;%i;%i;%im%s", 
            info.bg.r, info.bg.g, info.bg.b,
            str);
    }
    else if (info.flags & CHAR_INFO_DEFAULT_BG) {
        len = snprintf(out, max, "\x1b[0m\x1b[38;2;%i;%i;%im%s", 
            info.fg.r, info.fg.g, info.fg.b, 
            str);
    }
    else {
        len = snprintf(out, max, "\x1b[38;2;%i;%i;%im\x1b[48;2;%i;%i;%im%s", 
            info.fg.r, info.fg.g, info.fg.b, 
            info.bg.r, info.bg.g, info.bg.b,
            str);
    }
    
    return len;
}

char_info shader(vertex v, char_info c, bool overwriting) {
    float M = 2.0f;
    float pattern = (fmod(v.tc[0] * M, 1.0) > 0.5) ^ (fmod(v.tc[1] * M, 1.0) < 0.5);
    rgb color = pattern ? (rgb){200, 180, 80} : (rgb){70, 160, 180};

    if (!overwriting) {
        c.flags &= ~CHAR_INFO_DEFAULT_FG;
        c.ch = CHAR_HALF_BLOCK_TOP;
        c.fg = color;
    }
    else {
        if (c.flags & CHAR_INFO_DEFAULT_FG) {
            c.flags &= ~CHAR_INFO_DEFAULT_FG;
            c.ch = CHAR_HALF_BLOCK_BOTTOM;
            c.fg = color;
        }
        else {
            c.flags &= ~CHAR_INFO_DEFAULT_BG;
            c.bg = color;
        }
    }

    return c;
}

int main() {
    // Make cursor invisible.
    puts("\x1b[?25l");

    // Save cursor position.
    puts("\x1b[s");

    mat proj, view;
    perspective(proj, HEIGHT / (float)WIDTH, 70.0f, 0.0001f, 1000.0f);
    look_at(view, (vec){-2.3f, 2.3f, -2.3f}, (vec){0.0f, 0.0f, 0.0f}, (vec){0.0f, -1.0f, 0.0f});

    while (1) {
        // Restore cursor position.
        puts("\x1b[u");

        float z_buf[WIDTH * HEIGHT] = {0};

        char_info char_buf[WIDTH * HALF_HEIGHT];

        char_info info;
        info.flags = CHAR_INFO_DEFAULT_FG | CHAR_INFO_DEFAULT_BG;

        for (int i = 0; i < WIDTH * HALF_HEIGHT; i++) {
            char_buf[i] = info;
        }

        vec vertex_buf[36];
        vec tex_coord_buf[36];
        for (int i = 0; i < 36; i++) {
            memcpy(vertex_buf[i], cube_vertices[cube_indices[i]], 3 * sizeof(float));
            vertex_buf[i][3] = 1.0f;
            memcpy(tex_coord_buf[i], cube_tex_coords[cube_tex_coords_indices[i % 6]], 2 * sizeof(float));
        }

        clock_t uptime = clock() / (CLOCKS_PER_SEC / 1000);
        mat model = IDENTITY, vp, mvp;
        rotate_y(model, (float)(uptime * M_PI / 1000));
        rotate_x(model, (float)(uptime * M_PI / 1700));
        mat_mul_mat(view, proj, vp);
        mat_mul_mat(model, vp, mvp);
        for (int i = 0; i < 12; i++) {
            vertex tri[3];
            for (int j = 0; j < 3; j++) {
                vec_mul_mat(vertex_buf[i * 3 + j], mvp, tri[j].p);
                memcpy(tri[j].tc, tex_coord_buf[i * 3 + j], sizeof(vec));
            }
            triangle((framebuf){.z_buf = z_buf, .char_buf = char_buf}, tri[0], tri[1], tri[2], shader);
        }

#define MAX_STR_SIZE 45

        char screen[WIDTH * HALF_HEIGHT * MAX_STR_SIZE + HALF_HEIGHT + 1];
        int i = 0;

        for (int y = 0; y < HALF_HEIGHT; y++) {
            for (int x = 0; x < WIDTH; x++) {
                char str[MAX_STR_SIZE];
                int len = char_info_to_str(char_buf[y * WIDTH + x], str, MAX_STR_SIZE);
                memcpy(screen + i, str, len);
                i += len;
            }
            screen[i++] = '\n';
        }
        screen[i++] = '\0';

        puts(screen);
    }

    // Make cursor visible.
    puts("\x1b[?25h");
}