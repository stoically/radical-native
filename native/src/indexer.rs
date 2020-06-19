use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use seshat::{
    Config, Connection, CrawlerCheckpoint, Database, EventType, Language, LoadConfig, Profile,
    SearchConfig,
};
use std::{collections::HashMap, path::PathBuf};

use crate::Radical;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "method")]
pub enum Message {
    InitEventIndex(InitEventIndex),
    LoadCheckpoints,
    IsEventIndexEmpty,
    CommitLiveEvents,
    AddEventToIndex { content: AddEventToIndex },
    AddCrawlerCheckpoint { content: AddHistoricEvents },
    AddHistoricEvents { content: AddHistoricEvents },
    RemoveCrawlerCheckpoint { content: AddHistoricEvents },
    SearchEventIndex { content: SearchEventIndex },
    LoadFileEvents { content: LoadConfig },
    GetUserVersion,
    SetUserVersion { content: SetUserVersion },
    DeleteEvent { content: DeleteEvent },
    GetStats,
    CloseEventIndex,
    DeleteEventIndex,
}

#[derive(Debug, Deserialize)]
pub struct InitEventIndex {
    pub passphrase: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddEventToIndex {
    pub ev: Value,
    pub profile: Profile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddHistoricEvents {
    pub checkpoint: Option<CrawlerCheckpoint>,
    pub old_checkpoint: Option<CrawlerCheckpoint>,
    pub events: Option<Vec<Events>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchEventIndex {
    pub term: String,
    pub config: SearchConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteEvent {
    pub event_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUserVersion {
    pub version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub count: usize,
    pub results: Vec<SearchResult>,
    pub highlights: Vec<SearchHighlight>,
    pub next_batch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub rank: f32,
    pub result: Value,
    pub context: SearchResultContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultContext {
    pub events_before: Vec<Value>,
    pub events_after: Vec<Value>,
    pub profile_info: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHighlight {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEvent {
    pub event: Value,
    pub profile: Profile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Events {
    pub event: Value,
    pub profile: Profile,
}

// remove duplication once specialization lands
// https://github.com/rust-lang/rust/issues/31844
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "m.room.message")]
    Message(EventMessage),
    #[serde(rename = "m.room.name")]
    Name(EventName),
    #[serde(rename = "m.room.topic")]
    Topic(EventTopic),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMessage {
    pub event_id: String,
    pub sender: String,
    #[serde(rename = "origin_server_ts")]
    pub server_ts: i64,
    pub room_id: String,
    pub content: EventMessageContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMessageContent {
    pub body: String,
    pub msgtype: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventName {
    pub event_id: String,
    pub sender: String,
    #[serde(rename = "origin_server_ts")]
    pub server_ts: i64,
    pub room_id: String,
    pub content: EventNameContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventNameContent {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTopic {
    pub event_id: String,
    pub sender: String,
    #[serde(rename = "origin_server_ts")]
    pub server_ts: i64,
    pub room_id: String,
    pub content: EventTopicContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTopicContent {
    pub topic: String,
}

pub fn handle_message(radical: &mut Radical, message_in: Value) -> Result<Value> {
    let event_store = match message_in.get("eventStore") {
        Some(res) => res.as_str().context("eventStore.as_str() failed")?,
        None => "default",
    }
    .to_owned();
    let message: Message = serde_json::from_value(message_in)?;

    let res = match radical.indexer.get_mut(&event_store) {
        None => match message {
            Message::InitEventIndex(message) => {
                let config = Indexer::config(message);
                let indexer = Indexer::new(&event_store, config)?;
                radical.indexer.insert(event_store.to_owned(), indexer);
                json!(null)
            }
            Message::DeleteEventIndex => Indexer::delete(&event_store, &mut radical.indexer)?,
            Message::CloseEventIndex => json!(null), // no-op
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
            Message::GetUserVersion => indexer.get_user_version()?,
            Message::SetUserVersion { content } => indexer.set_user_version(content)?,
            Message::DeleteEvent { content } => indexer.delete_event(content)?,
            Message::GetStats => indexer.get_stats()?,
            Message::CloseEventIndex => Indexer::shutdown(&event_store, &mut radical.indexer)?,
            Message::DeleteEventIndex => Indexer::delete(&event_store, &mut radical.indexer)?,
            Message::InitEventIndex(_) => json!(null), // no-op
        },
    };

    Ok(res)
}

pub type IndexerMap = HashMap<String, Indexer>;

pub struct Indexer {
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
        let mut path = match dirs::data_local_dir() {
            Some(path) => path,
            None => bail!("userdata dir not found"),
        };
        path.push("radical-native");
        path.push("EventStore");
        path.push(event_store);
        Ok(path)
    }

    fn shutdown(event_store: &str, indexer: &mut IndexerMap) -> Result<Value> {
        if let Some(indexer) = indexer.remove(event_store) {
            indexer.database.shutdown().recv()??;
        }
        Ok(json!(null))
    }

    fn delete(event_store: &str, indexer: &mut IndexerMap) -> Result<Value> {
        Indexer::shutdown(event_store, indexer)?;

        let path = Indexer::event_store_path(event_store)?;
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }

        Ok(json!(null))
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
        let event: Event = serde_json::from_value(event)?;
        let res = match event {
            Event::Message(ev) => seshat::Event {
                event_type: EventType::Message,
                content_value: ev.content.body,
                msgtype: ev.content.msgtype,
                event_id: ev.event_id,
                sender: ev.sender,
                server_ts: ev.server_ts,
                room_id: ev.room_id,
                source,
            },
            Event::Name(ev) => seshat::Event {
                event_type: EventType::Name,
                content_value: ev.content.name,
                msgtype: None,
                event_id: ev.event_id,
                sender: ev.sender,
                server_ts: ev.server_ts,
                room_id: ev.room_id,
                source,
            },
            Event::Topic(ev) => seshat::Event {
                event_type: EventType::Message,
                content_value: ev.content.topic,
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

    fn load_checkpoints(&self) -> Result<Value> {
        let checkpoints = self.connection.load_checkpoints()?;
        Ok(json!(checkpoints))
    }

    fn is_event_index_empty(&self) -> Result<Value> {
        let res = self.connection.is_empty()?;
        Ok(json!(res))
    }

    fn commit_live_events(&mut self) -> Result<Value> {
        self.database.commit_no_wait().recv()??;
        Ok(json!(true))
    }

    fn add_event_to_index(&self, message: AddEventToIndex) -> Result<Value> {
        let event = Indexer::convert_event(message.ev)?;
        self.database.add_event(event, message.profile);
        Ok(json!(null))
    }

    fn add_history_events(&self, message: AddHistoricEvents) -> Result<Value> {
        let mut events: Vec<(seshat::Event, Profile)> = Vec::new();
        if let Some(events_in) = message.events {
            for event_in in events_in {
                let profile = event_in.profile;
                let event = Indexer::convert_event(event_in.event)?;
                events.push((event, profile));
            }
        }

        let res = self
            .database
            .add_historic_events(events, message.checkpoint, message.old_checkpoint)
            .recv()??;

        Ok(json!(res))
    }

    fn search_event_index(&self, message: SearchEventIndex) -> Result<Value> {
        let searcher = self.database.get_searcher();
        let search_batch = searcher.search(&message.term, &message.config)?;

        let mut results = Vec::new();
        for result in search_batch.results {
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

        let next_batch = match search_batch.next_batch {
            Some(next_batch) => Some(next_batch.to_hyphenated().to_string()),
            None => None,
        };

        let res = SearchResults {
            count: search_batch.count,
            results,
            highlights: vec![],
            next_batch,
        };

        Ok(json!(res))
    }

    fn load_file_events(&self, message: LoadConfig) -> Result<Value> {
        let ret = self.connection.load_file_events(&message)?;
        let mut results = Vec::new();
        for (source, profile) in ret {
            let event: Value = serde_json::from_str(&source)?;
            results.push(FileEvent { event, profile });
        }
        Ok(json!(results))
    }

    fn get_stats(&self) -> Result<Value> {
        let res = self.connection.get_stats()?;
        Ok(json!(res))
    }

    fn get_user_version(&self) -> Result<Value> {
        let res = self.connection.get_user_version()?;
        Ok(json!(res))
    }

    fn set_user_version(&self, content: SetUserVersion) -> Result<Value> {
        self.connection.set_user_version(content.version)?;
        Ok(json!(null))
    }

    fn delete_event(&self, message: DeleteEvent) -> Result<Value> {
        let res = self.database.delete_event(&message.event_id).recv()??;
        Ok(json!(res))
    }
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
        let indexer = indexer(tmpdir.path());
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
        let indexer = indexer(tmpdir.path());
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
        let payload: AddEventToIndex = serde_json::from_value(json!({
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
