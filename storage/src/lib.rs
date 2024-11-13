pub mod leveldb;
pub mod storage_factory;
pub mod key_value_db;
pub mod rocksdb;

pub use storage_factory::StorageFactory;
use parking_lot::{RwLock, Mutex};
use std::io::ErrorKind;

pub use key_value_db::write_batch;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    pub static ref StorageInstanceRef: RwLock<StorageFactory> = RwLock::new(StorageFactory::initialize());
}

