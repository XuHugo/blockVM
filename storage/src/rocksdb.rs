#[cfg(not(target_os = "windows"))]
use rocksdb::{Options, DB};
use crate::key_value_db::{KeyValueDB, KeyValueDb, WriteBatch};
use std::sync::{Arc, Once};
use parking_lot::{RwLock, Mutex};

/// The central object responsible for handling all the connections.
#[cfg(not(target_os = "windows"))]
pub struct RocksDbDriver {
    key_value_db: DB,
}


#[cfg(not(target_os = "windows"))]
impl RocksDbDriver {
    pub fn open(db_path: &str, max_open_files: u64) -> KeyValueDB {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_num_levels(2);
        opts.set_max_open_files(max_open_files as i32);
        Arc::new(Mutex::new(Box::new(RocksDbDriver {
            key_value_db: DB::open(&opts, db_path).unwrap()
        })))
    }
}

#[cfg(not(target_os = "windows"))]
impl KeyValueDb for RocksDbDriver {
    fn get(&mut self, key: &str, value: &mut String) -> bool {
        let mut ret: bool = true;
        let result = self.key_value_db.get(key.as_bytes());
        if result.is_err() {
            return false;
        }
        match result.unwrap() {
            Some(v) => {
                if let Ok(s) = String::from_utf8(v.clone()) {
                    value.push_str(s.as_str());
                    ret = true;
                } else {
                    ret = false;
                }
            }
            _ => {
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
        let result = self.key_value_db.get(key);
        if result.is_err() {
            return false;
        }
        match result.unwrap() {
            Some(v) => {
                value.extend_from_slice(v.clone().as_slice());
                ret = true;
            }
            _ => {
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

    fn write_batch(&mut self, values: WriteBatch) -> bool {
        if self.key_value_db.write(values).is_err() {
            return false;
        }
        if self.key_value_db.flush().is_err() {
            return false;
        }
        true
    }
}