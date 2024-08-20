use std::iter::zip;

use glm::{vec2, vec3, vec4, Mat3, Mat4, Vec2, Vec4};
use itertools::izip;
use tobj::Model;

use crate::{char::{AnsiColorMode, CharColor, CharInfo}, clip::clip_triangle, vertex::Vertex};

pub fn half_block_shader(c: &mut CharInfo, half: &CharHalf, color: &CharColor) {
    match (half, c.char_code) {
        (CharHalf::Top, '▄') => c.bg_color = Some(color.clone()),
        (CharHalf::Bottom, '▀') => c.bg_color = Some(color.clone()),
        (CharHalf::Top, _) => {
            c.char_code = '▀';
            c.fg_color = Some(color.clone());
        },
        (CharHalf::Bottom, _) => {
            c.char_code = '▄';
            c.fg_color = Some(color.clone());
        }
    }
}

pub type Shader = fn(&Vertex, &mut CharInfo, &CharHalf);

pub enum CharHalf {
    Top,
    Bottom
}

pub struct Framebuf {
    pub w: usize,
    pub h: usize,
    pub char_buf: Vec<CharInfo>,
    pub z_buf: Vec<f32>
}

impl Framebuf {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            char_buf: vec![Default::default(); w * (h / 2)],
            z_buf: vec![0.0; w * h]
        }
    }

    pub fn clear(&mut self) {
        self.char_buf.fill(Default::default());
        self.z_buf.fill(f32::INFINITY);
    }

    pub fn to_string(&self, mode: &AnsiColorMode) -> String {
        let mut str = String::with_capacity(self.w * self.h / 2 * 45 + self.h / 2);

        for y in 0..self.h / 2 {
            for x in 0..self.w {
                str.push_str(&self.char_buf[y * self.w + x].to_ansi(mode));
            }

            str.push_str("\r\n");
        }

        str
    }

    fn prepare_position(&self, p: &Vec4) -> Vec4 {
        // W division (homogeneous clip space -> NDC space).
        // Viewport transformation ([-1, 1] -> framebuffer size).
        vec4(
            (p.x / p.w + 1.0) / 2.0 * (self.w - 1) as f32, 
            (p.y / p.w + 1.0) / 2.0 * (self.h - 1) as f32, 
            p.z / p.w, 
            p.w)
    }

    pub fn draw_line(&mut self, start: &Vertex, end: &Vertex, shader: Shader) {        
        if is_point_visible(start.position) && is_point_visible(end.position) {
            self.raster_line(start, end, shader);
        }

        
    }

    // This function assumes the entire line is visible.
    fn raster_line(&mut self, start: &Vertex, end: &Vertex, shader: Shader) {
        let start_pos = self.prepare_position(&start.position);
        
        let difference = Vertex { 
            position: self.prepare_position(&end.position) - start_pos, 
            attributes: zip(&end.attributes, &start.attributes).map(|(a, b)| a - b).collect() 
        };
        
        let num_steps = difference.position.x.abs().max(difference.position.y.abs()) as usize;
        
        let mut current = Vertex { position: start_pos, attributes: start.attributes.clone() };
        let increment = &difference / num_steps;
        
        for _ in 0..num_steps {
            let x = current.position.x as usize;
            let y = current.position.y as usize;

            // TODO: replace with proper clipping.
            if x >= self.w || y >= self.h {
                continue;
            }

            shader(
                &current, 
                &mut self.char_buf[(y / 2) * self.w + x], 
                if y % 2 == 0 { &CharHalf::Top } else { &CharHalf::Bottom });

            current += &increment;
        }
    }

    pub fn draw_triangle(&mut self, vertices: &[&Vertex; 3], shader: Shader) {
        // Raster triangle without clipping if all vertices are visible.
        if !vertices.iter().map(|v| is_point_visible(v.position)).collect::<Vec<bool>>().contains(&false) {
            self.raster_triangle(&vertices, shader);
            return;
        }

        let clipped = clip_triangle(vertices);

        if clipped.len() < 3 {
            return;
        }

        for i in 0..clipped.len() - 1 {
            self.raster_triangle(&[&clipped[0], &clipped[i], &clipped[i + 1]], shader);
        }
    }

    // This function assumes the entire triangle is visible.
    fn raster_triangle(&mut self, vertices: &[&Vertex; 3], shader: Shader) {
        let p: Vec<Vec4> = vertices.iter().map(|v| self.prepare_position(&v.position)).collect();
        let area_inv = 1.0 / calc_area(&p[0].xy(), &p[1].xy(), &p[2].xy());

        // Calculate bounding box.
        let min_x = p.iter().map(|p| p.x).fold(f32::INFINITY, f32::min).max(0.0) as usize;
        let max_x = p.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max).min(self.w as f32 - 1.0) as usize;
        let min_y = p.iter().map(|p| p.y).fold(f32::INFINITY, f32::min).max(0.0) as usize;
        let max_y = p.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max).min(self.h as f32 - 1.0) as usize;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let point = vec2(x as f32 + 0.5, y as f32 + 0.5);
                let a0 = calc_area(&p[1].xy(), &p[2].xy(), &point) * area_inv;
                let a1 = calc_area(&p[2].xy(), &p[0].xy(), &point) * area_inv;
                let a2 = calc_area(&p[0].xy(), &p[1].xy(), &point) * area_inv;

                if a0 >= 0.0 && a1 >= 0.0 && a2 >= 0.0 {
                    let z = p[0].z * a0 + p[1].z * a1 + p[2].z * a2;

                    if z < self.z_buf[y * self.w + x] {
                        self.z_buf[y * self.w + x] = z;

                        let attributes = izip!(&vertices[0].attributes, &vertices[1].attributes, &vertices[2].attributes)
                        .map(|(a, b, c)| a * a0 + b * a1 + c * a2)
                        .collect();

                        let vertex = Vertex {
                            position: Default::default(),
                            attributes
                        };

                        shader(
                            &vertex, 
                            &mut self.char_buf[(y / 2) * self.w + x], 
                            if y % 2 == 0 { &CharHalf::Top } else { &CharHalf::Bottom });
                    }
                }
            }
        }
    }

    pub fn draw_model(&mut self, model: &Model, mvp_matrix: &Mat4, normal_matrix: &Mat3, shader: Shader) {
        let mesh = &model.mesh;

        for indices in mesh.indices.chunks(3) {            
            let get_position = |i| mvp_matrix * vec4(
                mesh.positions[3 * i as usize],
                mesh.positions[3 * i as usize + 1],
                mesh.positions[3 * i as usize + 2],
                1.0,
            );

            let get_normal = |i| {
                let normal = normal_matrix * vec3(
                    mesh.normals[3 * i as usize], 
                    mesh.normals[3 * i as usize + 1], 
                    mesh.normals[3 * i as usize + 2]
                );
                vec![normal[0], normal[1], normal[2]]
            };

            let v0 = Vertex { position: get_position(indices[0]), attributes: get_normal(indices[0]) };
            let v1 = Vertex { position: get_position(indices[1]), attributes: get_normal(indices[1]) };
            let v2 = Vertex { position: get_position(indices[2]), attributes: get_normal(indices[2]) };

            self.draw_triangle(&[&v0, &v1, &v2], shader);
        }
    }
}

fn is_point_visible(p: Vec4) -> bool {
    p.x >= -p.w && p.x <= p.w && p.y >= -p.w && p.y <= p.w && p.z >= -p.w && p.z <= p.w
}

fn calc_area(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}
