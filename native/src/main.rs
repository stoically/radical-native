// Copyright 2019 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
mod macros;

mod error;
mod indexer;
mod native_messaging;

use error::Error;
use indexer::Indexer;
use native_messaging::{stdin, stdout_error, stdout_ready, stdout_reply};

pub(crate) struct BoosterPack {
    indexer: Option<Indexer>,
}

fn main() {
    let mut pack = BoosterPack { indexer: None };
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
