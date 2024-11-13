extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate config;
#[macro_use]
extern crate lazy_static;

mod configure;
mod consensus;
mod db;
mod genesis_block;
mod p2p_network;
mod ssl;

use config::*;
pub use configure::Configure;
pub use consensus::Consensus;
pub use db::Db;
pub use genesis_block::GenesisBlock;
pub use p2p_network::P2PNetwork;
pub use ssl::SSL;

use parking_lot::{Mutex, RwLock};
use std::io;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::Once;
lazy_static! {
    pub static ref ConfigureInstanceRef: Arc<Configure> = Arc::new({
        let mut conf = Config::default();
        conf.merge(File::new("src/config", FileFormat::Toml))
            .unwrap();
        let p2p_conf: Configure = conf.try_into().unwrap();
        p2p_conf
    });
}
