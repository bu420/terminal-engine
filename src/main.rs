use char::{AnsiColorMode, CharColor, CharInfo};
use glm::{look_at, mat4_to_mat3, perspective, rotate, vec3};
use raster::{half_block_shader, CharHalf, Framebuf};
use russh::server::{Auth, Config, Handle, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId, CryptoVec};
use russh_keys::key::PublicKey;
use tokio::sync::Mutex;
use vertex::Vertex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use async_trait::async_trait;

pub mod raster;
pub mod char;
pub mod vertex;
pub mod clip;

extern crate nalgebra_glm as glm;

#[derive(Clone)]
struct SSHServer {
    clients: Arc<Mutex<HashMap<(usize, ChannelId), Handle>>>,
    uuid: usize
}

impl Server for SSHServer {
    type Handler = Self;

    fn new_client(&mut self, _peer_address: Option<SocketAddr>) -> Self {
        let clone = self.clone();
        self.uuid += 1;
        clone
    }
}

#[async_trait]
impl Handler for SSHServer {
    type Error = russh::Error;

    async fn auth_none(&mut self, _: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_password(&mut self, _username: &str, _password: &str) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn auth_publickey(&mut self, _user: &str, _public_key: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool, Self::Error> {
        let mut clients = self.clients.lock().await
            .insert((self.uuid, channel.id()), session.handle());
        Ok(true)
    }

    async fn channel_close(&mut self, channel: ChannelId, _: &mut Session) -> Result<(), Self::Error> {
        self.remove_user(self.uuid, channel).await;
        Ok(())
    }

    async fn data(&mut self, _channel: ChannelId, _data: &[u8], _session: &mut Session) -> Result<(), Self::Error> {
        //let data = CryptoVec::from(format!("You sent: \x1b[38;2;0;255;0m{}\x1b[0m\r\n", String::from_utf8_lossy(data)));
        //session.data(channel, data);
        Ok(())
    }
}

impl SSHServer {
    async fn remove_user(&mut self, uuid: usize, channel: ChannelId) {
        self.clients.lock().await.remove(&(uuid, channel));
        println!("User left");
    }

    async fn broadcast_message(&mut self, message: String) {
        // Lock the mutex and collect the necessary data
        let client_data: Vec<(usize, ChannelId, russh::server::Handle)> = {
            let clients = self.clients.lock().await;
            clients.iter().map(|((uuid, channel), client)| (uuid.clone(), channel.clone(), client.clone())).collect()
        };

        // Iterate over the collected data and send the message outside of the lock
        for (uuid, channel, client) in client_data {
            if let Err(_) = client.data(channel.clone(), CryptoVec::from(message.clone())).await {
                self.remove_user(uuid, channel).await;
            }
        }
    }

    async fn num_sessions(&self) -> usize {
        self.clients.lock().await.len()
    }
}

#[tokio::main]
async fn main() {
    let key = russh_keys::load_secret_key(Path::new("key.txt"), None)
        .expect("Failed to open key.txt (SSH private key)");

    let config = Arc::new(Config {
        inactivity_timeout: Some(Duration::from_secs(5)),
        keepalive_interval: Some(Duration::from_secs(5)),
        keepalive_max: 1,
        keys: vec![key],
        ..Default::default()
    });

    let mut server = SSHServer {
        clients: Arc::new(Mutex::new(HashMap::new())),
        uuid: 0
    };

    let time_start = SystemTime::now();

    let (models, _materials) = tobj::load_obj("assets/low-poly-torus.obj", &tobj::GPU_LOAD_OPTIONS)
        .expect("Failed to load file");

    let mut fb = Framebuf::new(48, 48);

    let proj_matrix = perspective(fb.h as f32 / fb.w as f32, 70.0, 0.0001, 1000.0);
    let view_matrix = look_at(&vec3(0.0, 0.0, 4.0), &vec3(0.0, 0.0, 0.0), &vec3(0.0, -1.0, 0.0));

    let mut previous_time = SystemTime::now();

    let server_clone = server.clone();

    tokio::spawn(async move {
        loop {
            //tokio::time::sleep(Duration::from_millis(20)).await;

            let elapsed_time = time_start.elapsed().unwrap().as_millis() as f32;
            let current_time = SystemTime::now();
            let _delta_time = current_time.duration_since(previous_time).unwrap().as_millis();
            previous_time = current_time;

            fb.clear();
            
            let mut model_matrix = rotate(&glm::identity(), elapsed_time / 1000.0, &vec3(0.0, 1.0, 0.0));
            //model_matrix = rotate(&model_matrix, elapsed_time / 1700.0, &vec3(0.0, 1.0, 0.0));
            model_matrix = rotate(&model_matrix, elapsed_time / 1500.0, &vec3(1.0, 0.0, 0.0));

            let mvp_matrix = proj_matrix * view_matrix * model_matrix;
            let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());

            fb.draw_model(&models[0], &mvp_matrix, &normal_matrix, |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf| {
                let r = ((vertex.attributes[0] + 1.0) / 2.0 * 255.0) as u8;
                let g = ((vertex.attributes[1] + 1.0) / 2.0 * 255.0) as u8;
                let b = ((vertex.attributes[2] + 1.0) / 2.0 * 255.0) as u8;
                half_block_shader(c, &half, &CharColor { r, g, b });
            });

            let mut str = "\x1b[?25l\x1b[H".to_owned(); // Make cursor invisible and move cursor to beginning.
            str.push_str(&fb.to_string(&AnsiColorMode::Ansi256));
            str.push_str(&format!("\r\n\x1b[38;2;0;120;215mUsers online: {}\x1b[0m\r\n", server_clone.num_sessions().await));
            
            server_clone.clone().broadcast_message(str).await;
        }
    });

    println!("SSH server started and listening on port 22");
    server.run_on_address(config, ("localhost", 22)).await.unwrap();
}
