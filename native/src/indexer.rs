use std::path::PathBuf;

use serde_json::{json, Value};
use seshat::{
    CheckpointDirection, Config, Connection, CrawlerCheckpoint, Database, Event, EventType,
    Language, LoadConfig, LoadDirection, Profile, SearchConfig, SearchResult,
};

use crate::Error;
use crate::Radical;

enum MessageMethod {
    InitEventIndex,
    LoadCheckpoints,
    IsEventIndexEmpty,
    CommitLiveEvents,
    AddEventToIndex,
    AddCrawlerCheckpoint,
    RemoveCrawlerCheckpoint,
    AddHistoricEvents,
    SearchEventIndex,
    LoadFileEvents,
    CloseEventIndex,
    DeleteEventIndex,
    GetStats,
    Unknown,
}

pub(crate) fn handle_message(radical: &mut Radical, message: Value) -> Result<Value, Error> {
    let method_str = as_str!(message, "method");
    let method = method_to_enum(&method_str);
    let event_store = message["eventStore"].as_str().unwrap_or("default");
    let indexer = radical.indexer.get_mut(event_store);

    let res = match indexer {
        None => match method {
            MessageMethod::InitEventIndex => {
                let indexer = Indexer::new(event_store, &message)?;
                radical.indexer.insert(event_store.to_owned(), indexer);
                json!(null)
            }
            MessageMethod::DeleteEventIndex => Indexer::delete_event_index(event_store)?,
            _ => return Err(Error::IndexNotInitialized),
        },
        Some(indexer) => match method {
            MessageMethod::LoadCheckpoints => indexer.load_checkpoints()?,
            MessageMethod::IsEventIndexEmpty => indexer.is_event_index_empty()?,
            MessageMethod::CommitLiveEvents => indexer.commit_live_events()?,
            MessageMethod::AddEventToIndex => indexer.add_event_to_index(message)?,
            MessageMethod::AddCrawlerCheckpoint => indexer.add_crawler_checkpoint(message)?,
            MessageMethod::AddHistoricEvents => indexer.add_history_events(message)?,
            MessageMethod::RemoveCrawlerCheckpoint => indexer.remove_crawler_checkpoint(message)?,
            MessageMethod::SearchEventIndex => indexer.search_event_index(message)?,
            MessageMethod::LoadFileEvents => indexer.load_file_events(message)?,
            MessageMethod::GetStats => indexer.get_stats()?,
            MessageMethod::CloseEventIndex => {
                radical.indexer.remove(event_store);
                json!(null)
            }
            MessageMethod::DeleteEventIndex => return Err(Error::CloseIndexBeforeDelete),
            MessageMethod::InitEventIndex => json!(null), // no-op
            MessageMethod::Unknown => {
                return Err(Error::UnknownMethod {
                    error: format!("Unknown method: {}", method_str),
                })
            }
        },
    };

    Ok(res)
}

pub(crate) struct Indexer {
    database: Database,
    connection: Connection,
}

impl Indexer {
    pub fn new(event_store: &str, message: &Value) -> Result<Indexer, Error> {
        let mut config = Config::new();
        config = config.set_passphrase(
            message["passphrase"]
                .as_str()
                .unwrap_or("DEFAULT_PASSPHRASE"),
        );
        if let Some(language) = message["language"].as_str() {
            let language = Language::from(language);
            config = config.set_language(&language);
        }

        let path = Indexer::event_store_path(event_store)?;
        std::fs::create_dir_all(&path)?;

        let database = Database::new_with_config(path, &config)?;
        let connection = database.get_connection()?;
        Ok(Indexer {
            database,
            connection,
        })
    }

    fn event_store_path(event_store: &str) -> Result<PathBuf, Error> {
        let mut path = match dirs::data_dir() {
            Some(path) => path,
            None => return Err(Error::UserDataDirNotFound),
        };
        path.push("radical-native");
        path.push("EventStore");
        path.push(event_store);
        Ok(path)
    }

    fn delete_event_index(event_store: &str) -> Result<Value, Error> {
        let path = Indexer::event_store_path(event_store)?;
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }

        Ok(json!(null))
    }

    fn load_checkpoints(&mut self) -> Result<Value, Error> {
        let checkpoints = self.connection.load_checkpoints()?;
        let mut checkpoints_json = Vec::new();
        for checkpoint in checkpoints {
            let direction = match checkpoint.direction {
                CheckpointDirection::Backwards => "b",
                CheckpointDirection::Forwards => "f",
            };
            checkpoints_json.push(json!({
                "roomId": checkpoint.room_id,
                "token": checkpoint.token,
                "fullCrawl": checkpoint.full_crawl,
                "direction": direction,
            }));
        }
        Ok(serde_json::to_value(&checkpoints_json)?)
    }

    fn is_event_index_empty(&mut self) -> Result<Value, Error> {
        let empty = self.connection.is_empty()?;
        Ok(serde_json::to_value(empty)?)
    }

    fn commit_live_events(&mut self) -> Result<Value, Error> {
        self.database.commit_no_wait().recv()??;
        Ok(json!(true))
    }

    fn add_event_to_index(&mut self, message: Value) -> Result<Value, Error> {
        let message_content = get!(message, "content");
        let event_json = get!(message_content, "ev");
        let profile_json = get!(message_content, "profile");
        let (event, profile) = parse_event(&event_json, &profile_json)?;
        self.database.add_event(event, profile);
        Ok(json!(null))
    }

    fn add_crawler_checkpoint(&mut self, message: Value) -> Result<Value, Error> {
        let message_content = get!(message, "content");
        let new_checkpoint: Option<CrawlerCheckpoint> =
            Some(parse_checkpoint(get!(message_content, "checkpoint"))?);
        let old_checkpoint: Option<CrawlerCheckpoint> = None;
        let events: Vec<(Event, Profile)> = Vec::new();
        self.database
            .add_historic_events(events, new_checkpoint, old_checkpoint)
            .recv()??;
        Ok(json!(null))
    }

    fn add_history_events(&mut self, message: Value) -> Result<Value, Error> {
        let message_content = get!(message, "content");
        let new_checkpoint: Option<CrawlerCheckpoint> =
            Some(parse_checkpoint(get!(message_content, "checkpoint"))?);
        let old_checkpoint: Option<CrawlerCheckpoint> = None;
        let mut events: Vec<(Event, Profile)> = Vec::new();
        let events_json = message_content["events"].as_array();
        match events_json {
            Some(events_json) => {
                for event in events_json {
                    let event = parse_event(get!(event, "event"), get!(event, "profile"))?;
                    events.push(event);
                }
            }
            None => (),
        };
        self.database
            .add_historic_events(events, new_checkpoint, old_checkpoint)
            .recv()??;
        Ok(json!(null))
    }

    fn remove_crawler_checkpoint(&mut self, message: Value) -> Result<Value, Error> {
        let message_content = get!(message, "content");
        let new_checkpoint: Option<CrawlerCheckpoint> = None;
        let old_checkpoint: Option<CrawlerCheckpoint> =
            Some(parse_checkpoint(get!(message_content, "checkpoint"))?);
        let events: Vec<(Event, Profile)> = Vec::new();
        self.database
            .add_historic_events(events, new_checkpoint, old_checkpoint)
            .recv()??;
        Ok(json!(null))
    }

    fn search_event_index(&mut self, message: Value) -> Result<Value, Error> {
        let message_content = get!(message, "content");
        let search_config = get!(message_content, "searchConfig");
        let (term, config) = parse_search_object(&search_config)?;
        let searcher = self.database.get_searcher();
        let search_results = searcher.search(&term, &config)?;
        let mut json_results = Vec::new();
        for result in search_results {
            let result: SearchResult = result;
            let event: Value = serde_json::from_str(&result.event_source)?;
            let mut before = Vec::new();
            for event in result.events_before.iter() {
                let event: Value = serde_json::from_str(event)?;
                before.push(event);
            }
            let mut after = Vec::new();
            for event in result.events_after.iter() {
                let event: Value = serde_json::from_str(event)?;
                after.push(event);
            }
            let json_result = json!({
                "rank": result.score,
                "result": event,
                "context": {
                    "events_before": before,
                    "events_after": after,
                    "profile_info": result.profile_info
                }
            });
            json_results.push(json_result);
        }
        Ok(json!({
            "count": json_results.len(),
            "results": json_results,
            "highlights": []
        }))
    }

    fn load_file_events(&mut self, message: Value) -> Result<Value, Error> {
        let args = get!(get!(message, "content"), "args");
        let room_id = as_str!(args, "roomId");
        let mut config = LoadConfig::new(room_id);
        let limit = as_i64!(args, "limit");
        config = config.limit(limit as usize);
        if let Some(event) = args["fromEvent"].as_str() {
            config = config.from_event(event);
        }
        if let Some(direction) = args["direction"].as_str() {
            let direction = parse_load_direction(direction)?;
            config = config.direction(direction);
        }
        let ret = self.connection.load_file_events(&config)?;
        let mut results = Vec::new();
        for (source, profile) in ret {
            let event: Value = serde_json::from_str(&source)?;
            results.push(json!({
                "event": event,
                "profile": profile
            }));
        }
        Ok(json!(results))
    }

    fn get_stats(&mut self) -> Result<Value, Error> {
        let ret = self.connection.get_stats()?;
        Ok(json!({
            "eventCount": ret.event_count,
            "roomCount": ret.room_count,
            "size": ret.size
        }))
    }
}

fn parse_event(event_json: &Value, profile_json: &Value) -> Result<(Event, Profile), Error> {
    let event_content = get!(event_json, "content");
    let event_type = as_str!(event_json, "type");
    let (event_type, content_value, msgtype) = match event_type.as_ref() {
        "m.room.message" => (
            EventType::Message,
            as_str!(event_content, "body"),
            Some(as_str!(event_content, "msgtype")),
        ),
        "m.room.name" => (EventType::Name, as_str!(event_content, "name"), None),
        "m.room.topic" => (EventType::Topic, as_str!(event_content, "topic"), None),
        _ => {
            return Err(Error::ParseEvent {
                error: format!("Unknown event type: {}", event_type),
            });
        }
    };

    let event_id = as_str!(event_json, "event_id");
    let sender = as_str!(event_json, "sender");
    let server_ts = as_i64!(event_json, "origin_server_ts");
    let room_id = as_str!(event_json, "room_id");
    let source = serde_json::to_string(event_json)?;

    let event = Event {
        event_type,
        content_value,
        msgtype,
        event_id,
        sender,
        server_ts,
        room_id,
        source,
    };
    let profile: Profile = serde_json::from_value(profile_json.clone())?;

    Ok((event, profile))
}

fn parse_checkpoint_direction(direction: &str) -> Result<CheckpointDirection, Error> {
    let direction = match direction.to_lowercase().as_ref() {
        "backwards" | "backward" | "b" => CheckpointDirection::Backwards,
        "forwards" | "forward" | "f" => CheckpointDirection::Forwards,
        "" => CheckpointDirection::Backwards,
        d => {
            return Err(Error::ParseCheckpointDirection {
                error: format!("Unknown checkpoint direction {}", d),
            });
        }
    };

    Ok(direction)
}

fn parse_load_direction(direction: &str) -> Result<LoadDirection, Error> {
    let direction = match direction.to_lowercase().as_ref() {
        "backwards" | "backward" | "b" => LoadDirection::Backwards,
        "forwards" | "forward" | "f" => LoadDirection::Forwards,
        "" => LoadDirection::Backwards,
        d => {
            return Err(Error::ParseLoadDirection {
                error: format!("Unknown checkpoint direction {}", d),
            });
        }
    };

    Ok(direction)
}

fn parse_checkpoint(checkpoint_json: &Value) -> Result<CrawlerCheckpoint, Error> {
    let room_id = as_str!(checkpoint_json, "roomId");
    let token = as_str!(checkpoint_json, "token");
    let full_crawl = checkpoint_json["fullCrawl"].as_bool().unwrap_or(false);

    let direction = &checkpoint_json["direction"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let direction = parse_checkpoint_direction(&direction)?;

    Ok(CrawlerCheckpoint {
        room_id,
        token,
        full_crawl,
        direction,
    })
}

fn parse_search_object(search_config: &Value) -> Result<(String, SearchConfig), Error> {
    let term = as_str!(search_config, "search_term");

    let mut config = SearchConfig::new();

    if let Some(limit) = search_config["limit"].as_i64() {
        config.limit(limit as usize);
    }

    if let Some(before_limit) = search_config["before_limit"].as_i64() {
        config.before_limit(before_limit as usize);
    }

    if let Some(after_limit) = search_config["after_limit"].as_i64() {
        config.after_limit(after_limit as usize);
    }

    if let Some(order_by_recency) = search_config["order_by_recency"].as_bool() {
        config.order_by_recency(order_by_recency);
    }

    if let Some(room_id) = search_config["room_id"].as_str() {
        config.for_room(&room_id);
    }

    if let Some(keys) = search_config["keys"].as_array() {
        for key in keys {
            if let Some(key) = key.as_str() {
                match key.as_ref() {
                    "content.body" => config.with_key(EventType::Message),
                    "content.topic" => config.with_key(EventType::Topic),
                    "content.name" => config.with_key(EventType::Name),
                    _ => {
                        return Err(Error::ParseSearchObject {
                            error: format!("Invalid search key {}", key),
                        });
                    }
                };
            }
        }
    }

    Ok((term, config))
}

fn method_to_enum(method: &String) -> MessageMethod {
    if method == "initEventIndex" {
        return MessageMethod::InitEventIndex;
    }
    if method == "loadCheckpoints" {
        return MessageMethod::LoadCheckpoints;
    }
    if method == "isEventIndexEmpty" {
        return MessageMethod::IsEventIndexEmpty;
    }
    if method == "commitLiveEvents" {
        return MessageMethod::CommitLiveEvents;
    }
    if method == "addEventToIndex" {
        return MessageMethod::AddEventToIndex;
    }
    if method == "addCrawlerCheckpoint" {
        return MessageMethod::AddCrawlerCheckpoint;
    }
    if method == "removeCrawlerCheckpoint" {
        return MessageMethod::RemoveCrawlerCheckpoint;
    }
    if method == "addHistoricEvents" {
        return MessageMethod::AddHistoricEvents;
    }
    if method == "searchEventIndex" {
        return MessageMethod::SearchEventIndex;
    }
    if method == "loadFileEvents" {
        return MessageMethod::LoadFileEvents;
    }
    if method == "closeEventIndex" {
        return MessageMethod::CloseEventIndex;
    }
    if method == "deleteEventIndex" {
        return MessageMethod::DeleteEventIndex;
    }
    if method == "getStats" {
        return MessageMethod::GetStats;
    }

    return MessageMethod::Unknown;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event_room_message_text() -> Value {
        json!({
            "type": "m.room.message",
            "room_id": "!FDVbSkWZSIcwvBFMdt:localhost",
            "sender": "@example2:localhost",
            "content": {
                "body": "Test message",
                "msgtype": "m.text"
            },
            "origin_server_ts": 1580728702628 as usize,
            "unsigned": {
                "age": 949499816 as usize
            },
            "event_id": "$lp49H7iDTNWQxD-fiZ6sDE6vT70DlYdKdoujEB5QtLM",
            "user_id": "@example2:localhost",
            "age": 949499816 as usize
        })
    }

    fn checkpoint() -> Value {
        json!({
            "checkpoint": {
                "roomId": "!FDVbSkWZSIcwvBFMdt:localhost",
                "token": "123",
                "fullCrawl": false,
                "direction": "b"
            }
        })
    }

    fn setup() {
        use tempfile::tempdir;
        let tmpdir = tempdir().unwrap();
        std::env::set_var("HOME", tmpdir.path());
    }

    #[test]
    fn crawler_checkpoints() {
        setup();
        let mut indexer = Indexer::new("test_passphrase", &json!({})).expect("indexer");
        let checkpoint = checkpoint();

        indexer
            .add_crawler_checkpoint(json!({
                "content": checkpoint.clone()
            }))
            .expect("add_crawler_checkpoint");
        indexer
            .remove_crawler_checkpoint(json!({
                "content": checkpoint.clone()
            }))
            .expect("remove_crawler_checkpoint");

        let checkpoints = indexer.load_checkpoints().expect("load_checkpoints");
        let count = checkpoints.as_array().expect("checkpoints.as_array").len();
        assert_eq!(count, 0);
    }

    #[test]
    fn json_messages() {
        setup();
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
                "content": checkpoint
            }),
        )
        .expect("addCrawlerCheckpoint");

        handle_message(
            &mut radical,
            json!({
                "method": "removeCrawlerCheckpoint",
                "content": checkpoint
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
