use std::collections::HashMap;

mod indexer;
mod native_messaging;

use indexer::Indexer;
use native_messaging::{stdin, stdout_error, stdout_ready, stdout_reply};

pub(crate) struct Radical {
    indexer: HashMap<String, Indexer>,
}

fn main() {
    let mut radical = Radical {
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

        let reply = indexer::handle_message(&mut radical, message_in);

        match reply {
            Ok(reply) => stdout_reply(rpc_id, reply),
            Err(error) => stdout_error(rpc_id, error),
        };
    }
}
