#include <stdio.h>
#include <time.h>

#define WIDTH 88
#define HEIGHT 44

#include "te/linalg.h"
#include "te/rast.h"

float z_buf[WIDTH * HEIGHT];
rgb color_buf[WIDTH * HEIGHT];

char chars[] = " `^\",:;Il!i~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
vec vertices[8] = {{-1,-1,-1},{1,-1,-1},{1,1,-1},{-1,1,-1},{-1,-1,1},{1,-1,1},{1,1,1},{-1,1,1}};
vec tex_coords[4] = {{0,0},{1,0},{1,1},{0,1}};
vec normals[6] = {{0,0,1},{1,0,0},{0,0,-1},{-1,0,0},{0,1,0},{0,-1,0}};
int indices[36] = {0,1,3,3,1,2,1,5,2,2,5,6,5,4,6,6,4,7,4,0,7,7,0,3,3,2,7,7,2,6,4,5,0,0,5,1};
int tex_coords_indices[6] = {0,1,3,3,1,2};

void clear() {
    printf("\033[u");
    memset(z_buf, 0, WIDTH * HEIGHT * sizeof(float));
    memset(color_buf, 0, WIDTH * HEIGHT * sizeof(rgb));
}

int main() {
    printf("\033[s");
    mat proj, view;
    perspective(proj, HEIGHT / (float)WIDTH * 2.0f, 70.0f, 0.0001f, 1000.0f);
    look_at(view, (vec){-2.3f, 2.3f, -2.3f}, (vec){0.0f, 0.0f, 0.0f}, (vec){0.0f, -1.0f, 0.0f});

    while (1) {
        vec vertex_buf[36];
        vec tex_coord_buf[36];
        for (int i = 0; i < 36; i++) {
            memcpy(vertex_buf[i], vertices[indices[i]], 3 * sizeof(float));
            vertex_buf[i][3] = 1.0f;
            memcpy(tex_coord_buf[i], tex_coords[tex_coords_indices[i % 6]], 2 * sizeof(float));
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

        char screen[WIDTH * HEIGHT + HEIGHT + 1];
        int i = 0;

        for (int y = 0; y < HEIGHT; y++) {
            for (int x = 0; x < WIDTH; x++) {
                float color = color_buf[y * WIDTH + x][0];
                int j = (int)((strlen(chars) - 1) * color);
                screen[i++] = chars[j];
            }
            screen[i++] = '\n';
        }
        screen[i++] = '\0';
        puts(screen);
    }
}