use glm::{
    identity, look_at, make_vec3, mat4_to_mat3, perspective, rotate, scale, translate, vec3, vec4_to_vec3
};
use tobj::Model;

use crate::{
    char::{CharColor, CharInfo},
    raster::{half_block_shader, CharHalf, Framebuf},
    scene::Scene,
    vertex::Vertex,
};

pub struct SkullScene {
    model: Model,
}

impl SkullScene {
    pub fn new() -> Self {
        let (models, _materials) =
            tobj::load_obj("assets/low-poly-skull.obj", &tobj::GPU_LOAD_OPTIONS)
                .expect("Failed to load file");
        Self {
            model: models[0].clone(),
        }
    }
}

impl Scene for SkullScene {
    fn run(&self, fb: &mut Framebuf, elapsed_time: f32) {
        let camera_pos = vec3(0.0, -0.5, 4.0);
        let proj_matrix = perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
        let view_matrix = look_at(&camera_pos, &vec3(0.0, -0.5, 0.0), &vec3(0.0, -1.0, 0.0));

        let duration = 3000.0; // one full rotation duration in ms
        let t = ((elapsed_time as f32) % duration) / duration; // normalize [0,1]

        // Ease-in-out cubic
        let eased_t = if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        };

        let angle_y = eased_t * std::f32::consts::PI * 2.0; // full 360 degrees

        // Base model matrix
        let mut model_matrix = rotate(&identity(), angle_y, &vec3(0.0, 1.0, 0.0));
        model_matrix = scale(&model_matrix, &vec3(1.8, 1.8, 1.8));

        // Add "laugh/jitter" at start and end (t ~ 0 or t ~ 1)
        let jitter_strength = 0.04; // small up/down movement
        let jitter_frequency = 4.0; // how fast it "laughs"

        let jitter = if t < 0.25 {
            (jitter_frequency * t * std::f32::consts::PI * 2.0).sin() * jitter_strength
        } else if t > 0.75 {
            (jitter_frequency * (t - 0.8) * 5.0 * std::f32::consts::PI * 2.0).sin()
                * jitter_strength
        } else {
            0.0
        };

        // Apply vertical jitter
        model_matrix = translate(&model_matrix, &vec3(0.0, jitter, 0.0));

        let vp_matrix = proj_matrix * view_matrix;
        let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());

        fb.draw_model(
            &self.model,
            &model_matrix,
            &vp_matrix,
            &normal_matrix,
            &camera_pos,
            elapsed_time,
            |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf, _elapsed_time: f32| {
                let light_color = vec3(1.0, 1.0, 1.0);
                let light_pos = vec3(100.0, 0.0, 50.0);
                let light_direction = (light_pos - vec4_to_vec3(&vertex.position)).normalize();

                let ambient_strength = 0.0;
                let ambient = ambient_strength * light_color;

                let normal = make_vec3(&vertex.attributes[2..5]);
                let diffuse = normal.dot(&light_direction).max(0.0f32) * light_color;

                // Fade color based on vertex y position (higher y = lighter color)
                let base_color = vec3(0.2, 0.6, 0.8);
                let y = vertex.position.y;
                let fade = ((y + 1.0) / 2.0).clamp(0.0, 1.0); // Map y from [-1,1] to [0,1]
                let object_color = base_color * (1.0 - fade) + vec3(1.0, 1.0, 1.0) * fade;

                let result = (ambient + diffuse).component_mul(&object_color) + vec3(0.2, 0.2, 0.2);

                half_block_shader(
                    c,
                    &half,
                    &CharColor {
                        r: (result.x * 255.0) as u8,
                        g: (result.y * 255.0) as u8,
                        b: (result.z * 255.0) as u8,
                    },
                );
            },
        );
    }
}
