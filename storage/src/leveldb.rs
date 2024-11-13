#[cfg(target_os = "windows")]
use rusty_leveldb::{Options, DB};
use crate::key_value_db::{KeyValueDB, KeyValueDb, WriteBatch};
use std::sync::{Arc, Once};
use parking_lot::{RwLock, Mutex};

/// The central object responsible for handling all the connections.
#[cfg(target_os = "windows")]
pub struct LevelDbDriver {
    key_value_db: DB,
}


#[cfg(target_os = "windows")]
impl LevelDbDriver {
    pub fn open(db_path: &str, max_open_files: u64) -> KeyValueDB {
        let mut opt = Options::default();
        opt.reuse_logs = false;
        opt.reuse_manifest = false;
        opt.compression_type = rusty_leveldb::CompressionType::CompressionNone;
        opt.max_file_size = max_open_files as usize;
        Arc::new(Mutex::new(Box::new(LevelDbDriver { key_value_db: DB::open(db_path, opt).unwrap() })))
    }
}

#[cfg(target_os = "windows")]
impl KeyValueDb for LevelDbDriver {
    fn get(&mut self, key: &str, value: &mut String) -> bool {
        let mut ret: bool = true;
        match self.key_value_db.get(key.as_bytes()) {
            Some(v) => {
                if let Ok(s) = String::from_utf8(v.clone()) {
                    value.push_str(s.as_str());
                    ret = true;
                } else {
                    ret = false;
                }
            }
            None => {
                ret = false;
            }
        }
        ret
    }

    fn put(&mut self, key: &str, value: &str) -> bool {
        if self.key_value_db.put(key.as_bytes(), value.as_bytes()).is_err() {
            return false;
        }
        if self.key_value_db.flush().is_err() {
            return false;
        }
        true
    }

    fn get_bytes(&mut self, key: &[u8], value: &mut Vec<u8>) -> bool {
        let mut ret: bool = true;
        match self.key_value_db.get(key) {
            Some(v) => {
                value.extend_from_slice(v.clone().as_slice());
                ret = true;
            }
            None => {
                ret = false;
            }
        }
        ret
    }

    fn put_bytes(&mut self, key: &[u8], value: &Vec<u8>) -> bool {
        if self.key_value_db.put(key, value).is_err() {
            return false;
        }
        if self.key_value_db.flush().is_err() {
            return false;
        }
        true
    }

    fn delete(&mut self, key: &str) -> bool {
        if self.key_value_db.delete(key.as_bytes()).is_err() {
            return false;
        }
        if self.key_value_db.flush().is_err() {
            return false;
        }
        true
    }

    fn write_batch(&mut self, mut values:WriteBatch) -> bool {
        if self.key_value_db.write(values, false).is_err() {
            return false;
        }
        if self.key_value_db.flush().is_err() {
            return false;
        }
        true
    }
}