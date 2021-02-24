use crate::storage::*;
use rusty_tarantool::tarantool::{Client, ClientConfig, IteratorType};
use std::str;
use tokio::runtime::Runtime;
use v_onto::individual::*;
use v_onto::parser::*;

pub struct TTStorage {
    rt: Runtime,
    client: Client,
}

const INDIVIDUALS_SPACE_ID: i32 = 512;
const TICKETS_SPACE_ID: i32 = 513;
const AZ_SPACE_ID: i32 = 514;

impl TTStorage {
    pub fn new(tt_uri: String, login: &str, pass: &str) -> TTStorage {
        TTStorage {
            rt: Runtime::new().unwrap(),
            client: ClientConfig::new(tt_uri, login, pass).set_timeout_time_ms(1000).set_reconnect_time_ms(10000).build(),
        }
    }
}

impl Storage for TTStorage {
    fn get_individual_from_db(&mut self, storage: StorageId, uri: &str, iraw: &mut Individual) -> bool {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        let key = (uri,);

        match self.rt.block_on(self.client.select(space, 0, &key, 0, 100, IteratorType::EQ)) {
            Ok(v) => {
                iraw.set_raw(&v.data[5..]);

                if parse_raw(iraw).is_ok() {
                    return true;
                }
            }
            Err(e) => {
                error!("TTStorage: fail get [{}] from tarantool, err={:?}", uri, e);
            }
        }

        false
    }

    fn remove(&mut self, storage: StorageId, key: &str) -> bool {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        let tuple = (key,);

        if let Err(e) = self.rt.block_on(self.client.delete(space, &tuple)) {
            error!("tarantool: fail remove, db [{:?}], err = {:?}", storage, e);
            false
        } else {
            true
        }
    }

    fn put_kv(&mut self, storage: StorageId, key: &str, val: &str) -> bool {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        let tuple = (key, val);

        if let Err(e) = self.rt.block_on(self.client.replace(space, &tuple)) {
            error!("tarantool: fail replace, db [{:?}], err = {:?}", storage, e);
            false
        } else {
            true
        }
    }

    fn put_kv_raw(&mut self, storage: StorageId, _key: &str, val: Vec<u8>) -> bool {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        if let Err(e) = self.rt.block_on(self.client.replace_raw(space, val)) {
            error!("tarantool: fail replace, db [{:?}], err = {:?}", storage, e);
            false
        } else {
            true
        }
    }

    fn get_v(&mut self, storage: StorageId, key: &str) -> Option<String> {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        let key = (key,);

        if let Ok(v) = self.rt.block_on(self.client.select(space, 0, &key, 0, 100, IteratorType::EQ)) {
            if let Ok(s) = std::str::from_utf8(&v.data[5..]) {
                return Some(s.to_string());
            }
        }

        None
    }

    fn get_raw(&mut self, storage: StorageId, key: &str) -> Vec<u8> {
        let space = if storage == StorageId::Tickets {
            TICKETS_SPACE_ID
        } else if storage == StorageId::Az {
            AZ_SPACE_ID
        } else {
            INDIVIDUALS_SPACE_ID
        };

        let key = (key,);

        if let Ok(v) = self.rt.block_on(self.client.select(space, 0, &key, 0, 100, IteratorType::EQ)) {
            return v.data[5..].to_vec();
        }

        Vec::default()
    }
}
