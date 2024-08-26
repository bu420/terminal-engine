use glm::{vec4_to_vec3, Mat4, Vec3, Vec4};
use itertools::Itertools;

use crate::vertex::Vertex;

enum ClipComponent {
    X,
    Y,
    Z
}

impl ClipComponent {
    fn to_index(&self) -> usize {
        match self {
            ClipComponent::X => 0,
            ClipComponent::Y => 1,
            ClipComponent::Z => 2
        }
    }
}

/*pub fn clip_line(start: &Vertex, end: &Vertex) -> Option<(Vertex, Vertex)> {
    start.position.x 
}*/

pub fn clip_triangle(vertices: &[&Vertex; 3]) -> Vec<Vertex> {
    let result = clip_component(
        &vec![vertices[0].clone(), vertices[1].clone(), vertices[2].clone()], 
        &ClipComponent::X);

    if result.is_empty() {
        return result;
    }

    let result = clip_component(&result, &ClipComponent::Y);

    if result.is_empty() {
        return result;
    }

    return clip_component(&result, &ClipComponent::Z);
}

fn clip_component(vertices: &Vec<Vertex>, component: &ClipComponent) -> Vec<Vertex> {
    let result = clip_component_signed(&vertices, component, 1.0);
    
    if result.is_empty() {
        return result;
    }

    return clip_component_signed(&result, component, -1.0);
}

fn clip_component_signed(vertices: &Vec<Vertex>, component: &ClipComponent, sign: f32) -> Vec<Vertex> {
    let mut result: Vec<Vertex> = Vec::with_capacity(vertices.len());

    // Iterate all lines and clip if necessary.
    for (a, b) in vertices.iter().circular_tuple_windows() {
        let a_comp: f32 = a.position[component.to_index()] * sign;
        let b_comp: f32 = b.position[component.to_index()] * sign;

        let is_a_visible = a_comp <= a.position.w;
        let is_b_visible = b_comp <= b.position.w;

        if is_a_visible != is_b_visible {
            let amount: f32 = (b.position.w - b_comp) / 
                ((b.position.w - b_comp) - (a.position.w - a_comp));

            result.push(Vertex::lerp(b, a, amount));
        }

        if is_a_visible {
            result.push(a.clone());
        }
    }

    result
}

pub fn should_backface_cull(vertices: &[Vec4; 3], model_matrix: &Mat4, camera_pos: &Vec3) -> bool {
    // Put points in model space
    let points = vertices.iter().map(|p| vec4_to_vec3(&(model_matrix * p))).collect::<Vec<_>>();
    // Pick any one of the triangle's points and calculate direction from camera
    let direction_towards_point = (&points[0] - camera_pos).normalize();
    // Calculate normal from winding order
    let normal = (points[1] - points[0]).cross(&(points[2] - points[0])).normalize();
    
    direction_towards_point.dot(&normal) > 0.0
}