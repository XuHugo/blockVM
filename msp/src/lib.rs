pub mod hash;
pub mod signing;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
use parking_lot::{RwLock, Mutex};
pub use signing::{hex_str_to_bytes,bytes_to_hex_str};

lazy_static! {
    pub static ref HashInstanceRef: RwLock<hash::Hash> = RwLock::new(hash::Hash::default());
}