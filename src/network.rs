use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone)]
pub struct NetworkSession {
    pub server_running: bool,
    pub client_connected: bool,
    pub host_password: String,
    pub connection_ip: Option<String>,
    pub connection_port: Option<String>,
}

impl NetworkSession {
    pub fn reset_client(&mut self) {
        self.client_connected = false;
        self.connection_ip = None;
        self.connection_port = None;
    }

    pub fn is_connected(&self) -> bool {
        self.server_running && self.client_connected
    }
}
