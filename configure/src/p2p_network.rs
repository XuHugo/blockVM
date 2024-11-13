use serde::{Deserialize, Serialize};#[derive(Deserialize)]#[derive(Debug)]pub struct P2PNetwork {    pub heartbeat_interval: u64,    pub listen_addr: String,    pub target_peer_connection: u64,    pub max_connection: u64,    pub connect_timeout: u64,    pub known_peers: Vec<String>,}impl Default for P2PNetwork {    fn default() -> Self {        Self {            heartbeat_interval: Default::default(),            listen_addr: Default::default(),            target_peer_connection: Default::default(),            max_connection: Default::default(),            connect_timeout: Default::default(),            known_peers: Default::default(),        }    }}impl Clone for P2PNetwork {    fn clone(&self) -> Self {        Self {            heartbeat_interval: self.heartbeat_interval,            listen_addr: self.listen_addr.clone(),            target_peer_connection: self.target_peer_connection,            max_connection: self.max_connection,            connect_timeout: self.connect_timeout,            known_peers: self.known_peers.clone(),        }    }}