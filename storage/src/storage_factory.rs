use configure::{Configure, ConfigureInstanceRef, Db};
use parking_lot::{Mutex, RwLock};
use std::sync::{Arc, Once};

use crate::key_value_db::KeyValueDB;
#[cfg(target_os = "windows")]
use crate::leveldb::LevelDbDriver;
#[cfg(not(target_os = "windows"))]
use crate::rocksdb::RocksDbDriver;

#[derive(Clone)]
pub struct StorageFactory {
    key_value_db: KeyValueDB,
    ledger_db: KeyValueDB,
    account_db: KeyValueDB,
}

impl StorageFactory {
    pub fn initialize() -> StorageFactory {
        let db_config = &ConfigureInstanceRef.db;
        StorageFactory {
            #[cfg(target_os = "windows")]
            key_value_db: LevelDbDriver::open(
                db_config.key_value_db_path.as_str(),
                db_config.key_vaule_max_open_files,
            ),
            #[cfg(target_os = "windows")]
            ledger_db: LevelDbDriver::open(
                db_config.ledger_db_path.as_str(),
                db_config.key_vaule_max_open_files,
            ),
            #[cfg(target_os = "windows")]
            account_db: LevelDbDriver::open(
                db_config.account_db_path.as_str(),
                db_config.key_vaule_max_open_files,
            ),
            #[cfg(not(target_os = "windows"))]
            key_value_db: RocksDbDriver::open(
                &*db_config.key_value_db_path,
                db_config.key_vaule_max_open_files,
            ),
            #[cfg(not(target_os = "windows"))]
            ledger_db: RocksDbDriver::open(
                &*db_config.ledger_db_path,
                db_config.key_vaule_max_open_files,
            ),
            #[cfg(not(target_os = "windows"))]
            account_db: RocksDbDriver::open(
                &*db_config.account_db_path,
                db_config.key_vaule_max_open_files,
            ),
        }
    }

    //Store other data except account, ledger and transaction.
    pub fn key_value_db(&mut self) -> KeyValueDB {
        self.key_value_db.clone()
    }

    //Store account tree.
    pub fn account_db(&mut self) -> KeyValueDB {
        self.account_db.clone()
    }

    //Store transactions and ledgers.
    pub fn ledger_db(&mut self) -> KeyValueDB {
        self.ledger_db.clone()
    }
}
