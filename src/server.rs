use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc, time::Duration};

use async_trait::async_trait;
use russh::{server::{Auth, Config, Handle, Handler, Msg, Server, Session}, Channel, ChannelId, CryptoVec};
use russh_keys::key::PublicKey;
use tokio::sync::Mutex;

use crate::char::ANSI_CLEAR_SCREEN;

#[derive(Clone)]
pub struct SshSession {
    clients: Arc<Mutex<HashMap<(usize, ChannelId), Handle>>>,
    uuid: usize
}

impl Server for SshSession {
    type Handler = Self;

    fn new_client(&mut self, _peer_address: Option<SocketAddr>) -> Self {
        let clone = self.clone();
        self.uuid += 1;
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
            .insert((self.uuid, channel.id()), session.handle());
        session.handle().data(channel.id(), CryptoVec::from(format!("{ANSI_CLEAR_SCREEN}"))).await.unwrap();
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
            uuid: 0
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

    pub async fn remove_user(&mut self, uuid: usize, channel: ChannelId) {
        self.clients.lock().await.remove(&(uuid, channel));
        println!("User left");
    }

    pub async fn broadcast_message(&mut self, message: String) {
        // Lock the mutex and collect the necessary data
        let client_data: Vec<(usize, ChannelId, russh::server::Handle)> = {
            let clients = self.clients.lock().await;

            if clients.is_empty() {
                return;
            }

            clients.iter().map(|((uuid, channel), client)| (uuid.clone(), channel.clone(), client.clone())).collect()
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