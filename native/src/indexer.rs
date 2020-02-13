use serde_json::{json, Value};
use seshat::{
    CheckpointDirection, Connection, CrawlerCheckpoint, Database, Event, EventType, LoadConfig,
    LoadDirection, Profile, SearchConfig, SearchResult,
};
use tempfile::tempdir;

use crate::Error;

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

pub(crate) struct Indexer {
    database: Database,
    connection: Connection,
    tmpdir: tempfile::TempDir,
}

impl Indexer {
    pub fn new() -> Self {
        let tmpdir = tempdir().unwrap();
        let database = Database::new(tmpdir.path()).unwrap();
        let connection = database.get_connection().unwrap();
        Indexer {
            database,
            connection,
            tmpdir,
        }
    }

    fn load_checkpoints(&mut self) -> Result<Value, Error> {
        let checkpoints = self.connection.load_checkpoints()?;
        Ok(serde_json::to_value(&checkpoints)?)
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

    fn close_event_index(&mut self) -> Result<Value, Error> {
        // drop(connection);
        // drop(database);
        Ok(json!(null))
    }

    fn get_stats(&mut self) -> Result<Value, Error> {
        let ret = self.connection.get_stats()?;
        Ok(json!({
            "eventCount": ret.event_count,
            "roomCount": ret.room_count,
            "size": ret.size
        }))
    }

    pub fn handle_message(&mut self, message: Value) -> Result<Value, Error> {
        let method = as_str!(message, "method");

        let value = match method_to_enum(&method) {
            MessageMethod::InitEventIndex => json!(null),
            MessageMethod::LoadCheckpoints => self.load_checkpoints()?,
            MessageMethod::IsEventIndexEmpty => self.is_event_index_empty()?,
            MessageMethod::CommitLiveEvents => self.commit_live_events()?,
            MessageMethod::AddEventToIndex => self.add_event_to_index(message)?,
            MessageMethod::AddCrawlerCheckpoint => self.add_crawler_checkpoint(message)?,
            MessageMethod::AddHistoricEvents => self.add_history_events(message)?,
            MessageMethod::RemoveCrawlerCheckpoint => self.remove_crawler_checkpoint(message)?,
            MessageMethod::SearchEventIndex => self.search_event_index(message)?,
            MessageMethod::LoadFileEvents => self.load_file_events(message)?,
            MessageMethod::CloseEventIndex => self.close_event_index()?,
            MessageMethod::GetStats => self.get_stats()?,
            MessageMethod::DeleteEventIndex => {
                // delete folders
                std::process::exit(0);
            }
            MessageMethod::Unknown => {
                return Err(Error::UnknownMethod {
                    error: format!("Unknown method: {}", method),
                })
            }
        };
        Ok(value)
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
            eprintln!("[extract_event] Unknown event type: {}", event_type);
            return Err(Error::ParseEvent);
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
            eprintln!("Unknown checkpoint direction {}", d);
            return Err(Error::ParseCheckpointDirection);
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
            eprintln!("Unknown checkpoint direction {}", d);
            return Err(Error::ParseLoadDirection);
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
