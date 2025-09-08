use std::time::SystemTime;

use char::AnsiColorMode;
use crossterm::event::{read, Event};
use dialoguer::{theme::ColorfulTheme, Select};

use crate::raster::Framebuf;

pub mod char;
pub mod clip;
pub mod raster;
pub mod scene;
pub mod vertex;

extern crate nalgebra_glm as glm;

fn main() {
    loop {
        let fruits = vec!["Cube", "Laughing Skull", "Bull"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select scene")
            .default(0)
            .items(&fruits)
            .interact()
            .unwrap();

        // Discard leftover input
        let _ = read();

        let scene: Box<dyn scene::Scene> = match selection {
            0 => Box::new(scene::CubeScene::new()),
            1 => Box::new(scene::SkullScene::new()),
            2 => Box::new(scene::BullScene::new()),
            _ => unreachable!(),
        };

        let start_time = SystemTime::now();

        let mut fb = Framebuf::new(48, 48);

        print!("\x1b[?25l\x1b[2J\x1b[H"); // Hide cursor and clear terminal

        loop {
            fb.clear();

            scene.run(&mut fb, start_time.elapsed().unwrap().as_millis() as f32);

            print!("\x1b[H"); // Move cursor to top-left
            println!("{}", fb.to_string(&AnsiColorMode::AnsiTrueColor));

            println!("Press any key to close animation...");
            if crossterm::event::poll(std::time::Duration::from_millis(1)).unwrap() {
                if let Event::Key(_) = read().unwrap() {
                    break;
                }
            }
        }
    }
}
