#pragma once

#include <string.h>
#define _USE_MATH_DEFINES
#include <math.h>

#define LENGTH(v) sqrtf(v[0]*v[0] + v[1]*v[1] + v[2]*v[2])
#define NORMALIZE(v) {v[0] / LENGTH(v), v[1] / LENGTH(v), v[2] / LENGTH(v)}
#define CROSS(v0, v1) {v0[1]*v1[2]-v0[2]*v1[1], v0[2]*v1[0]-v0[0]*v1[2], v0[0]*v1[1]-v0[1]*v1[0]}
#define DOT(v0, v1) (v0[0]*v1[0]+v0[1]*v1[1]+v0[2]*v1[2])
#define IDENTITY {{1.0f,0.0f,0.0f,0.0f},{0.0f,1.0f,0.0f,0.0f},{0.0f,0.0f,1.0f,0.0f},{0.0f,0.0f,0.0f,1.0f}}

typedef float vec[4];
typedef float mat[4][4];

void perspective(mat m, float aspect, float fov, float near, float far) {
    memset(m, 0, 16 * sizeof(float));
    float half_tan = tanf(fov / 2);
    m[0][0] = 1 / (half_tan * aspect);      m[1][1] = 1 / half_tan;
    m[2][2] = -(far + near) / (far - near); m[2][3] = -1;
    m[3][2] = -(2 * far * near) / (far - near);
}

void look_at(mat m, vec pos, vec target, vec up) {
    memset(m, 0, 16 * sizeof(float));
    vec diff = {target[0] - pos[0], target[1] - pos[1], target[2] - pos[2]};
    vec forward = NORMALIZE(diff);
    vec right = NORMALIZE(((vec)CROSS(forward, up)));
    vec local_up = NORMALIZE(((vec)CROSS(right, forward)));
    m[0][0] = right[0];         m[1][0] = right[1];             m[2][0] = right[2];
    m[0][1] = local_up[0];      m[1][1] = local_up[1];          m[2][1] = local_up[2];
    m[0][2] = -forward[0];      m[1][2] = -forward[1];          m[2][2] = -forward[2];
    m[3][0] = -DOT(right, pos); m[3][1] = -DOT(local_up, pos);  m[3][2] = DOT(forward, pos);    m[3][3] = 1.0f;
}

void vec_mul_mat(vec v, mat m, vec out) {
    for (int y = 0; y < 4; y++) {
        float i = 0;
        for (int x = 0; x < 4; x++)
            i += m[x][y] * v[x];
        out[y] = i;
    }
}

void mat_mul_mat(mat m0, mat m1, mat out) {
    memset(out, 0, 16 * sizeof(float));
    for (int x = 0; x < 4; x++)
        for (int y = 0; y < 4; y++)
            for (int i = 0; i < 4; i++)
                out[x][y] += m0[x][i] * m1[i][y];
}

void rotate_x(mat m, float angle) {
    mat x = IDENTITY;
    x[1][1] = cosf(angle);  x[1][2] = -sinf(angle);
    x[2][1] = sinf(angle);  x[2][2] = cosf(angle);
    mat temp;
    memcpy(temp, m, 16 * sizeof(float));
    mat_mul_mat(temp, x, m);
}

void rotate_y(mat m, float angle) {
    mat y = IDENTITY;
    y[0][0] = cosf(angle);  y[0][2] = sinf(angle);
    y[2][0] = -sinf(angle); y[2][2] = cosf(angle);
    mat temp;
    memcpy(temp, m, 16 * sizeof(float));
    mat_mul_mat(temp, y, m);
}

void rotate_z(mat m, float angle) {
    mat z = IDENTITY;
    z[0][0] = cosf(angle);  z[0][1] = -sinf(angle);
    z[1][0] = sinf(angle);  z[1][1] = cosf(angle);
    mat temp;
    memcpy(temp, m, 16 * sizeof(float));
    mat_mul_mat(temp, z, m);
}
