use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

mod indexer;
mod native_messaging;
mod secrets;

use indexer::IndexerMap;
use native_messaging::{stdin, stdout_error, stdout_ready, stdout_reply};

pub struct Radical {
    indexer: IndexerMap,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Message {
    Seshat(Value),
    Keytar(Value),
}

fn main() {
    let mut radical = Radical {
        indexer: HashMap::new(),
    };

    stdout_ready();
    loop {
        let (rpc_id, message) = match stdin() {
            Ok(stdin) => stdin,
            Err(error) => {
                stdout_error(-1, error);
                continue;
            }
        };

        let message = serde_json::from_value(message);
        let reply = match message {
            Ok(Message::Seshat(message)) => indexer::handle_message(&mut radical, message),
            Ok(Message::Keytar(message)) => secrets::handle_message(message),
            _ => Err(anyhow!("handling message failed: {:?}", message)),
        };

        match reply {
            Ok(reply) => stdout_reply(rpc_id, reply),
            Err(error) => stdout_error(rpc_id, error),
        };
    }
}
