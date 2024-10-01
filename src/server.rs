use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc, time::Duration};

use async_trait::async_trait;
use russh::{server::{Auth, Config, Handle, Handler, Msg, Server, Session}, Channel, ChannelId, CryptoVec};
use russh_keys::key::PublicKey;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{char::ANSI_CLEAR_SCREEN, client::Client};

#[derive(Clone)]
pub struct SshSession {
    clients: Arc<Mutex<HashMap<(Uuid, ChannelId), Client>>>,
    uuid: Uuid
}

impl Server for SshSession {
    type Handler = Self;

    fn new_client(&mut self, _peer_address: Option<SocketAddr>) -> Self {
        let clone = self.clone();
        self.uuid = Uuid::new_v4();
        clone
    }
}

#[async_trait]
impl Handler for SshSession {
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
        self.clients.lock().await
            .insert((self.uuid, channel.id()), Client::new(session.handle(), self.uuid));
        
        session.handle().data(channel.id(), CryptoVec::from(format!("{ANSI_CLEAR_SCREEN}"))).await.unwrap();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(5)).await;
    
                /*fb.clear();
    
                let elapsed_time = start_time.elapsed().unwrap().as_millis() as f32;
                
                let mut model_matrix = rotate(&glm::identity(), elapsed_time / 600.0, &vec3(0.0, 1.0, 0.0));
                //model_matrix = rotate(&model_matrix, elapsed_time / 1500.0, &vec3(1.0, 0.0, 0.0));
                model_matrix = scale(&model_matrix, &vec3(1.8, 1.8, 1.8));
    
                let vp_matrix = proj_matrix * view_matrix;
                let normal_matrix = mat4_to_mat3(&model_matrix.try_inverse().unwrap().transpose());
    
                fb.draw_model(&models[0], &model_matrix, &vp_matrix, &normal_matrix, &camera_pos, |vertex: &Vertex, c: &mut CharInfo, half: &CharHalf| {
                    let light_color = vec3(1.0, 1.0, 1.0);
                    let light_pos = vec3(100.0, 0.0, 50.0);
                    let light_direction = (light_pos - vec4_to_vec3(&vertex.position)).normalize();
                    
                    let ambient_strength = 0.0;
                    let ambient = ambient_strength * light_color;
                    
                    let normal = make_vec3(&vertex.attributes[2..5]);
                    let diffuse = normal.dot(&light_direction).max(0.0) * light_color;
    
                    let tex_coord = make_vec2(&vertex.attributes[0..2]);
                    let size = 8.0;
                    let pattern: bool = ((tex_coord[0] * size % 1.0) > 0.5) ^ ((tex_coord[1] * size % 1.0) < 0.5);
                    let object_color = if pattern { vec3(0.8, 0.85, 1.0) } else { vec3(0.2, 0.6, 0.8) };
                    let result = (ambient + diffuse).component_mul(&object_color) + vec3(0.2, 0.2, 0.2);
                    
                    half_block_shader(c, &half, 
                        &CharColor { r: (result.x * 255.0) as u8, g: (result.y * 255.0) as u8, b: (result.z * 255.0) as u8 });
                });
    
                let mut str = "\x1b[?25l\x1b[H".to_owned(); // Make cursor invisible and move cursor to beginning.
                str.push_str(&fb.to_string(&AnsiColorMode::AnsiTrueColor));
                str.push_str(&format!("Users online: {}\r\n", server_clone.num_sessions().await));
                
                server_clone.clone().broadcast_message(str).await;*/
            }
        });

        Ok(true)
    }

    async fn channel_close(&mut self, channel: ChannelId, _: &mut Session) -> Result<(), Self::Error> {
        self.remove_user(self.uuid, channel).await;
        Ok(())
    }

    async fn data(&mut self, _channel: ChannelId, data: &[u8], _session: &mut Session) -> Result<(), Self::Error> {
        //let data = CryptoVec::from(format!("You sent: \x1b[38;2;0;255;0m{}\x1b[0m\r\n", String::from_utf8_lossy(data)));
        //session.data(channel, data);
        println!("User sent: {:#04x?}", data);
        Ok(())
    }
}

impl SshSession {
    pub fn new() -> Self {
        SshSession {
            clients: Arc::new(Mutex::new(HashMap::new())),
            uuid: Uuid::nil()
        }
    }

    pub async fn run(&mut self) {
        let key = russh_keys::load_secret_key(Path::new("key.txt"), None)
            .expect("Failed to open key.txt (SSH private key)");

        let config = Arc::new(Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            keepalive_interval: Some(Duration::from_secs(5)),
            keepalive_max: 1,
            keys: vec![key],
            ..Default::default()
        });

        self.run_on_address(config, ("localhost", 22)).await.unwrap();
    }

    pub async fn remove_user(&mut self, uuid: Uuid, channel: ChannelId) {
        self.clients.lock().await.remove(&(uuid, channel));
        println!("User left");
    }

    pub async fn broadcast_message(&mut self, message: String) {
        // Lock the mutex and collect the necessary data
        let client_data: Vec<(Uuid, ChannelId, Handle)> = {
            let handles = self.clients.lock().await;

            if handles.is_empty() {
                return;
            }

            handles.iter().map(|((uuid, channel), client)| (uuid.clone(), channel.clone(), client.handle.clone())).collect()
        };

        // Iterate over the collected data and send the message outside of the lock
        for (uuid, channel, client) in client_data {
            if let Err(_) = client.data(channel.clone(), CryptoVec::from(message.clone())).await {
                self.remove_user(uuid, channel).await;
            }
        }
    }

    pub async fn num_sessions(&self) -> usize {
        self.clients.lock().await.len()
    }
}