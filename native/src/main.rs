#[macro_use]
mod macros;

use std::collections::HashMap;

use error::Error;
use indexer::Indexer;
use native_messaging::{stdin, stdout_error, stdout_ready, stdout_reply};

mod error;
mod indexer;
mod native_messaging;

pub(crate) struct BoosterPack {
    indexer: HashMap<String, Indexer>,
}

fn main() {
    let mut pack = BoosterPack {
        indexer: HashMap::new(),
    };
    stdout_ready();
    loop {
        let (rpc_id, message_in) = match stdin() {
            Ok(stdin) => stdin,
            Err(error) => {
                stdout_error(-1, error);
                continue;
            }
        };

        let reply = indexer::handle_message(&mut pack, message_in);

        match reply {
            Ok(reply) => stdout_reply(rpc_id, reply),
            Err(error) => stdout_error(rpc_id, error),
        };
    }
}
