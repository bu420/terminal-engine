use glm::{vec2, vec4, Vec2, Vec4};
use itertools::izip;

use crate::{char::{AnsiColorMode, CharColor, CharColorLayer, CharInfo}, clip::clip_triangle, vertex::Vertex};

pub struct Line {
    pub current: Vertex,
    num_steps: usize,
    i: usize,
    increment: Vertex
}

impl Line {
    pub fn new(start: &Vertex, end: &Vertex) -> Self {
        let mut start_clone = start.clone();
        start_clone.position.x = start_clone.position.x.floor();
        start_clone.position.y = start_clone.position.y.floor();

        let mut end_clone = end.clone();
        end_clone.position.x = end_clone.position.x.floor();
        end_clone.position.y = end_clone.position.y.floor();
        
        let mut difference = &end_clone - &start_clone;
        difference.position = difference.position.abs();
        
        let num_steps = 
            if difference.position.x > difference.position.y { difference.position.x } 
            else { difference.position.y } as usize;
        
        Self {
            current: start.clone(),
            num_steps: num_steps as usize,
            i: 0,
            increment: &difference / num_steps
        }
    }

    pub fn step(&mut self) -> bool {
        if self.i < self.num_steps {
            self.current += &self.increment;
            self.i += 1;
            return true;
        }
        false
    }
}

pub fn half_block_shader(c: &mut CharInfo, half: CharHalf, r: u8, g: u8, b: u8) {
    match half {
        CharHalf::Top => {
            c.char_code = '▀';
            c.fg_color = Some(CharColor {
                r,
                g,
                b,
                layer: CharColorLayer::Foreground
            });
        },
        CharHalf::Bottom => {
            match c.fg_color {
                Some(_) => {
                    c.bg_color = Some(CharColor {
                        r,
                        g,
                        b,
                        layer: CharColorLayer::Background
                    });
                },
                None => {
                    c.char_code = '▄';
                    c.fg_color = Some(CharColor {
                        r,
                        g,
                        b,
                        layer: CharColorLayer::Foreground
                    });
                }
            }
        }
    }
}

pub type Shader = fn(&Vertex, &mut CharInfo, CharHalf);

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
        self.z_buf.fill(0.0);
    }

    pub fn print(&self) {
        let mut out = String::with_capacity(self.w * self.h / 2 * 45 + self.h / 2);

        for y in 0..self.h / 2 {
            for x in 0..self.w {
                out.push_str(&self.char_buf[y * self.w + x].to_ansi(&AnsiColorMode::AnsiTrueColor));
            }

            out.push('\n');
        }

        print!("{out}");
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
        let mut line = Line::new(
            &Vertex { position: self.prepare_position(&start.position), attributes: start.attributes.clone() }, 
            &Vertex { position: self.prepare_position(&end.position), attributes: end.attributes.clone() });

        while line.step() {
            let x = line.current.position.x as usize;
            let y = line.current.position.y as usize;

            shader(
                &line.current, 
                &mut self.char_buf[(y / 2) * self.w + x], 
                if y % 2 == 0 { CharHalf::Top } else { CharHalf::Bottom });
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

        let area = calc_area(&p[0].xy(), &p[1].xy(), &p[2].xy());

        for x in 0..self.w {
            for y in 0..self.h {
                let point = vec2(x as f32 + 0.5, y as f32 + 0.5);

                let mut a0 = calc_area(&p[1].xy(), &p[2].xy(), &point);
                let mut a1 = calc_area(&p[2].xy(), &p[0].xy(), &point);
                let mut a2 = calc_area(&p[0].xy(), &p[1].xy(), &point);

                if a0 >= 0.0 && a1 >= 0.0 && a2 >= 0.0 {
                    a0 /= area;
                    a1 /= area;
                    a2 /= area;

                    let vertex = Vertex {
                        position: Default::default(),
                        // Interpolate attributes.
                        attributes: izip!(&vertices[0].attributes, &vertices[1].attributes, &vertices[2].attributes)
                            .map(|(a, b, c)| a * a0 + b * a1 + c * a2).collect()
                    };

                    shader(
                        &vertex, 
                        &mut self.char_buf[(y / 2) * self.w + x], 
                        if y % 2 == 0 { CharHalf::Top } else { CharHalf::Bottom });
                }
            }
        }
    }
}

fn is_point_visible(p: Vec4) -> bool {
    p.x >= -p.w && p.x <= p.w && p.y >= -p.w && p.y <= p.w && p.z >= -p.w && p.z <= p.w
}

fn calc_area(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}
