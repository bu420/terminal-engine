#include <stdio.h>
#include <time.h>

#define WIDTH 64
#define HEIGHT 32

#include "te/linalg.h"
#include "te/rast.h"
#include "assets"

float z_buf[WIDTH * HEIGHT];
char_info char_buf[WIDTH * HEIGHT];

#define CHAR_BLOCK 0x8896e2
#define CHAR_HALF_BLOCK_BOTTOM 0x8496e2
#define CHAR_HALF_BLOCK_TOP 0x8096e2
#define CHAR_DOTTED_HIGH_DENSITY 0x9396e2
#define CHAR_DOTTED_MEDIUM_DENSITY 0x9296e2
#define CHAR_DOTTED_LOW_DENSITY 0x9196e2

int char_info_to_str(char_info info, char out[], int max) {
    char str[5];
    str[0] = (info.ch >> 0) & 0xFF;
    str[1] = (info.ch >> 8) & 0xFF;
    str[2] = (info.ch >> 16) & 0xFF;
    str[3] = (info.ch >> 24) & 0xFF;
    str[4] = '\0';

    int len = snprintf(out, max - 1, "\x1b[38;2;%i;%i;%im\x1b[48;2;%i;%i;%im%s", 
        info.fg.r, info.fg.g, info.fg.b, 
        info.bg.r, info.bg.g, info.bg.b,
        str);
    
    return len;
}

void clear() {
    // Restore cursor position.
    printf("\033[u");

    memset(z_buf, 0, WIDTH * HEIGHT * sizeof(float));

    char_info info = {.ch = ' ', .fg = {0, 0, 0}, .bg = {0, 0, 0}};
    for (int i = 0; i < WIDTH * HEIGHT; i++) {
        char_buf[i] = info;
    }
}

int main() {
    // Make cursor invisible.
    printf("\x1b[?25l");

    // Save cursor position.
    printf("\033[s");

    mat proj, view;
    perspective(proj, HEIGHT / (float)WIDTH * 2.0f, 70.0f, 0.0001f, 1000.0f);
    look_at(view, (vec){-2.3f, 2.3f, -2.3f}, (vec){0.0f, 0.0f, 0.0f}, (vec){0.0f, -1.0f, 0.0f});

    while (1) {
        vec vertex_buf[36];
        vec tex_coord_buf[36];
        for (int i = 0; i < 36; i++) {
            memcpy(vertex_buf[i], cube_vertices[cube_indices[i]], 3 * sizeof(float));
            vertex_buf[i][3] = 1.0f;
            memcpy(tex_coord_buf[i], cube_tex_coords[cube_tex_coords_indices[i % 6]], 2 * sizeof(float));
        }

        clear();
        clock_t uptime = clock() / (CLOCKS_PER_SEC / 1000);
        mat model = IDENTITY, vp, mvp;
        rotate_y(model, (float)(uptime * M_PI / 1000));
        rotate_x(model, (float)(uptime * M_PI / 1700));
        mat_mul_mat(view, proj, vp);
        mat_mul_mat(model, vp, mvp);
        for (int i = 0; i < 12; i++) {
            vec tri[3];
            for (int j = 0; j < 3; j++)
                vec_mul_mat(vertex_buf[i * 3 + j], mvp, tri[j]);
            triangle(tri[0], tri[1], tri[2], tex_coord_buf[i * 3 + 0], tex_coord_buf[i * 3 + 1], tex_coord_buf[i * 3 + 2]);
        }

        char screen[WIDTH * HEIGHT * 40 + 200];
        int i = 0;

        for (int y = 0; y < HEIGHT; y++) {
            for (int x = 0; x < WIDTH; x++) {
                char str[40];
                int len = char_info_to_str(char_buf[y * WIDTH + x], str, 40);
                for (int j = 0; j < len; j++) {
                    screen[i++] = str[j];
                }
            }
            screen[i++] = '\n';
        }
        screen[i++] = '\0';
        puts(screen);
    }

    // Make cursor visible.
    printf("\x1b[?25h");
}