use crate::raster::Framebuf;

pub mod cube_scene;
pub mod skull_scene;
pub mod bull_scene;

pub use crate::scene::cube_scene::CubeScene;
pub use crate::scene::skull_scene::SkullScene;
pub use crate::scene::bull_scene::BullScene;

pub trait Scene {
    fn run(&self, fb: &mut Framebuf, elapsed_time: f32);
}
