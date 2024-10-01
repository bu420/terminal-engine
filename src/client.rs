use russh::server::Handle;
use uuid::Uuid;

use crate::raster::Framebuf;

pub struct Client {
    pub handle: Handle,
    pub uuid: Uuid,
    pub fb: Framebuf
}

impl Client {
    pub fn new(handle: Handle, uuid: Uuid) -> Self {
        Self {
            handle,
            uuid,
            fb: Framebuf::new(96, 96)
        }
    }
}