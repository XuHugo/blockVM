use std::sync::{Arc, Once};
use parking_lot::{RwLock, Mutex};

pub type KeyValueDB = Arc<Mutex<Box<dyn KeyValueDb + Send + 'static>>>;
#[cfg(target_os = "windows")]
pub type WriteBatch = rusty_leveldb::WriteBatch;
#[cfg(not(target_os = "windows"))]
pub type WriteBatch = rocksdb::WriteBatch;

pub trait KeyValueDb {
    fn get(&mut self, key: &str, value: &mut String) -> bool;
    fn put(&mut self, key: &str, value: &str) -> bool;
    fn get_bytes(&mut self, key: &[u8], value: &mut Vec<u8>) -> bool;
    fn put_bytes(&mut self, key: &[u8], value: &Vec<u8>) -> bool;
    fn delete(&mut self, key: &str) -> bool;
    fn write_batch(&mut self, values: WriteBatch) -> bool;
}

pub fn write_batch() -> WriteBatch {
    #[cfg(target_os = "windows")]
        {
            WriteBatch::new()
        }
    #[cfg(not(target_os = "windows"))]
        {
            WriteBatch::default()
        }
}
