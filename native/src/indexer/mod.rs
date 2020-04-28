use anyhow::{bail, Result};
use serde_json::{json, Value};
use seshat::{Config, Connection, Database, Event, EventType, Language, LoadConfig, Profile};
use std::path::PathBuf;

mod message;

use crate::Radical;
use message::{
    AddEventToIndex, AddHistoricEvents, DeleteEvent, FileEvent, InitEventIndex, Message,
    SearchEventIndex, SearchResult, SearchResultContext, SearchResults,
};

pub(crate) fn handle_message(radical: &mut Radical, message_in: Value) -> Result<Value> {
    let event_store = match message_in.get("eventStore") {
        Some(res) => res.to_string(),
        None => "default".to_owned(),
    };
    let message: Message = serde_json::from_value(message_in)?;

    let res = match radical.indexer.get_mut(&event_store) {
        None => match message {
            Message::InitEventIndex(message) => {
                let config = config(message);
                let indexer = Indexer::new(&event_store, config)?;
                radical.indexer.insert(event_store.to_owned(), indexer);
                json!(null)
            }
            Message::DeleteEventIndex => Indexer::delete_event_index(&event_store)?,
            _ => bail!("index not initialized"),
        },
        Some(indexer) => match message {
            Message::LoadCheckpoints => indexer.load_checkpoints()?,
            Message::IsEventIndexEmpty => indexer.is_event_index_empty()?,
            Message::CommitLiveEvents => indexer.commit_live_events()?,
            Message::AddEventToIndex { content } => indexer.add_event_to_index(content)?,
            Message::AddCrawlerCheckpoint { content } => indexer.add_history_events(content)?,
            Message::AddHistoricEvents { content } => indexer.add_history_events(content)?,
            Message::RemoveCrawlerCheckpoint { content } => indexer.add_history_events(content)?,
            Message::SearchEventIndex { content } => indexer.search_event_index(content)?,
            Message::LoadFileEvents { content } => indexer.load_file_events(content)?,
            Message::DeleteEvent { content } => indexer.delete_event(content)?,
            Message::GetStats => indexer.get_stats()?,
            Message::CloseEventIndex => {
                radical.indexer.remove(&event_store);
                json!(null)
            }
            Message::DeleteEventIndex => {
                radical.indexer.remove(&event_store);
                Indexer::delete_event_index(&event_store)?
            }
            Message::InitEventIndex(_) => json!(null), // no-op
        },
    };

    Ok(res)
}

pub(crate) struct Indexer {
    database: Database,
    connection: Connection,
}

impl Indexer {
    pub fn new(event_store: &str, config: Config) -> Result<Indexer> {
        let path = Indexer::event_store_path(event_store)?;
        std::fs::create_dir_all(&path)?;

        Ok(Indexer::new_in_path(path, config)?)
    }

    pub fn new_in_path(path: PathBuf, config: Config) -> Result<Indexer> {
        let database = Database::new_with_config(path, &config)?;
        let connection = database.get_connection()?;
        Ok(Indexer {
            database,
            connection,
        })
    }

    fn event_store_path(event_store: &str) -> Result<PathBuf> {
        let mut path = match dirs::data_dir() {
            Some(path) => path,
            None => bail!("userdata dir not found"),
        };
        path.push("radical-native");
        path.push("EventStore");
        path.push(event_store);
        Ok(path)
    }

    fn delete_event_index(event_store: &str) -> Result<Value> {
        let path = Indexer::event_store_path(event_store)?;
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }

        Ok(json!(null))
    }

    fn load_checkpoints(&mut self) -> Result<Value> {
        let checkpoints = self.connection.load_checkpoints()?;
        Ok(json!(checkpoints))
    }

    fn is_event_index_empty(&mut self) -> Result<Value> {
        let empty = self.connection.is_empty()?;
        Ok(serde_json::to_value(empty)?)
    }

    fn commit_live_events(&mut self) -> Result<Value> {
        self.database.commit_no_wait().recv()??;
        Ok(json!(true))
    }

    fn add_event_to_index(&mut self, content: AddEventToIndex) -> Result<Value> {
        let event = convert_event(content.ev)?;
        self.database.add_event(event, content.profile);
        Ok(json!(null))
    }

    fn add_history_events(&mut self, message: AddHistoricEvents) -> Result<Value> {
        let mut events: Vec<(Event, Profile)> = Vec::new();
        if let Some(events_in) = message.events {
            for event_in in events_in {
                let profile = event_in.profile;
                let event = convert_event(event_in.event)?;
                events.push((event, profile));
            }
        }

        self.database
            .add_historic_events(events, message.checkpoint, message.old_checkpoint)
            .recv()??;
        Ok(json!(null))
    }

    fn search_event_index(&mut self, message: SearchEventIndex) -> Result<Value> {
        let searcher = self.database.get_searcher();
        let (count, search_results) = searcher.search(&message.term, &message.config)?;

        let mut results = Vec::new();
        for result in search_results {
            let event: Value = serde_json::from_str(&result.event_source)?;
            let mut events_before = Vec::new();
            for event in result.events_before.iter() {
                let event: Value = serde_json::from_str(event)?;
                events_before.push(event);
            }
            let mut events_after = Vec::new();
            for event in result.events_after.iter() {
                let event: Value = serde_json::from_str(event)?;
                events_after.push(event);
            }

            results.push(SearchResult {
                rank: result.score,
                result: event,
                context: SearchResultContext {
                    events_before,
                    events_after,
                    profile_info: result.profile_info,
                },
            });
        }

        let res = SearchResults {
            count,
            results,
            highlights: vec![],
        };

        Ok(json!(res))
    }

    fn load_file_events(&mut self, message: LoadConfig) -> Result<Value> {
        let ret = self.connection.load_file_events(&message)?;
        let mut results = Vec::new();
        for (source, profile) in ret {
            let event: Value = serde_json::from_str(&source)?;
            results.push(FileEvent { event, profile });
        }
        Ok(json!(results))
    }

    fn get_stats(&mut self) -> Result<Value> {
        let res = self.connection.get_stats()?;
        Ok(json!(res))
    }

    fn delete_event(&mut self, message: DeleteEvent) -> Result<Value> {
        self.database.delete_event(&message.event_id).recv()??;
        Ok(json!(null))
    }
}

fn config(message: InitEventIndex) -> Config {
    let mut config = Config::new();

    let passphrase = match message.passphrase {
        Some(passphrase) => passphrase,
        None => "DEFAULT_PASSPHRASE".to_owned(),
    };
    config = config.set_passphrase(passphrase);

    if let Some(language) = message.language {
        let language = Language::from(language.as_str());
        config = config.set_language(&language);
    }

    config
}

fn convert_event(event: Value) -> Result<seshat::Event> {
    let source = serde_json::to_string(&event)?;
    let event: message::Event = serde_json::from_value(event)?;
    let res = match event {
        message::Event::Message(ev) => Event {
            event_type: EventType::Message,
            content_value: ev.content.body,
            msgtype: ev.content.msgtype,
            event_id: ev.event_id,
            sender: ev.sender,
            server_ts: ev.server_ts,
            room_id: ev.room_id,
            source,
        },
        message::Event::Name(ev) => Event {
            event_type: EventType::Name,
            content_value: ev.name,
            msgtype: None,
            event_id: ev.event_id,
            sender: ev.sender,
            server_ts: ev.server_ts,
            room_id: ev.room_id,
            source,
        },
        message::Event::Topic(ev) => Event {
            event_type: EventType::Message,
            content_value: ev.topic,
            msgtype: None,
            event_id: ev.event_id,
            sender: ev.sender,
            server_ts: ev.server_ts,
            room_id: ev.room_id,
            source,
        },
    };

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn event_room_message_text() -> Value {
        json!({
            "type": "m.room.message",
            "room_id": "!FDVbSkWZSIcwvBFMdt:localhost",
            "sender": "@example2:localhost",
            "content": {
                "body": "Test message",
                "msgtype": "m.text"
            },
            "origin_server_ts": 1_580_728_702_628 as usize,
            "unsigned": {
                "age": 949_499_816 as usize
            },
            "event_id": "$lp49H7iDTNWQxD-fiZ6sDE6vT70DlYdKdoujEB5QtLM",
            "user_id": "@example2:localhost",
            "age": 949_499_816 as usize
        })
    }

    fn checkpoint() -> Value {
        json!({
            "roomId": "!FDVbSkWZSIcwvBFMdt:localhost",
            "token": "123",
            "direction": "b"
        })
    }

    fn checkpoint2() -> Value {
        json!({
            "roomId": "!FDVbSkWZSIcwvBFMdt:localhost",
            "token": "456",
            "direction": "b"
        })
    }

    fn indexer(tmpdir: &std::path::Path) -> Indexer {
        let mut config = Config::new();
        config = config.set_passphrase("TEST_PASS");
        Indexer::new_in_path(tmpdir.to_path_buf(), config).expect("indexer")
    }

    #[test]
    fn crawler_checkpoints() {
        let tmpdir = tempdir().expect("tempdir");
        let mut indexer = indexer(tmpdir.path());
        let checkpoint = checkpoint();

        let message: AddHistoricEvents = serde_json::from_value(json!({
            "checkpoint": checkpoint.clone()
        }))
        .unwrap();
        indexer
            .add_history_events(message)
            .expect("add_crawler_checkpoint");

        let message: AddHistoricEvents =
            serde_json::from_value(json!({ "oldCheckpoint": checkpoint })).unwrap();
        indexer
            .add_history_events(message)
            .expect("remove_crawler_checkpoint");

        let checkpoints = indexer.load_checkpoints().expect("load_checkpoints");
        let count = checkpoints.as_array().expect("checkpoints.as_array").len();
        assert_eq!(count, 0);
    }

    #[test]
    fn initial_crawl() {
        let tmpdir = tempdir().expect("tempdir");
        let mut indexer = indexer(tmpdir.path());
        let checkpoint = checkpoint();
        let profile = Profile::new("Alice", "");

        let message: AddHistoricEvents = serde_json::from_value(json!({
            "checkpoint": checkpoint.clone()
        }))
        .unwrap();
        indexer
            .add_history_events(message)
            .expect("add_crawler_checkpoint");

        let message: AddHistoricEvents = serde_json::from_value(json!({
            "checkpoint": checkpoint2(),
            "events": [
                {
                    "event": event_room_message_text(),
                    "profile": profile
                }
            ],
            "oldCheckpoint": checkpoint
        }))
        .unwrap();
        indexer
            .add_history_events(message)
            .expect("add_history_events");

        assert_eq!(
            indexer
                .load_checkpoints()
                .expect("load_checkpoints")
                .as_array()
                .expect("load_checkpoints.as_array")
                .len(),
            1
        );
    }

    #[test]
    fn add_event() {
        let tmpdir = tempdir().expect("tempdir");
        let mut indexer = indexer(tmpdir.path());

        let profile = Profile::new("Alice", "");
        let payload: message::AddEventToIndex = serde_json::from_value(json!({
            "ev": event_room_message_text(),
            "profile": profile
        }))
        .unwrap();
        indexer
            .add_event_to_index(payload)
            .expect("add_event_to_index");

        indexer.commit_live_events().expect("commit_live_events");

        let reply = indexer.get_stats().expect("get_stats");
        assert_eq!(reply["eventCount"].as_i64().expect("eventCount"), 1);
    }

    #[test]
    fn json_messages() {
        let tmpdir = tempdir().expect("tempdir");
        // make sure that we have only one test that modifies the environment
        // since tests run in parallel
        std::env::set_var("HOME", tmpdir.path().to_str().expect("tmpdir path"));
        use std::collections::HashMap;
        let mut radical = Radical {
            indexer: HashMap::new(),
        };
        handle_message(
            &mut radical,
            json!({
                "method": "initEventIndex"
            }),
        )
        .expect("initEventIndex");

        let profile = Profile::new("Alice", "");
        handle_message(
            &mut radical,
            json!({
                "method": "addEventToIndex",
                "content": {
                    "ev": event_room_message_text(),
                    "profile": profile
                }
            }),
        )
        .expect("addEventToIndex");

        handle_message(
            &mut radical,
            json!({
                "method": "commitLiveEvents"
            }),
        )
        .expect("commitLiveEvents");

        let checkpoint = checkpoint();
        handle_message(
            &mut radical,
            json!({
                "method": "addCrawlerCheckpoint",
                "content": {
                    "checkpoint": checkpoint
                }
            }),
        )
        .expect("addCrawlerCheckpoint");

        handle_message(
            &mut radical,
            json!({
                "method": "removeCrawlerCheckpoint",
                "content": {
                    "oldCheckpoint": checkpoint
                }
            }),
        )
        .expect("removeCrawlerCheckpoint");

        let checkpoints = handle_message(
            &mut radical,
            json!({
                "method": "loadCheckpoints"
            }),
        )
        .expect("loadCheckpoints");

        let reply = handle_message(
            &mut radical,
            json!({
                "method": "getStats"
            }),
        )
        .expect("getStats");

        assert_eq!(checkpoints.as_array().expect("checkpoints").len(), 0);
        assert_eq!(reply["eventCount"].as_i64().expect("eventCount"), 1);
    }
}
