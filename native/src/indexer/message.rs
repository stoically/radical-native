use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(deserialize_with = "method_to_enum")]
    pub method: Method,
    #[serde(default = "event_store_default")]
    pub event_store: String,
    #[serde(default)]
    pub content: Value,
    #[serde(flatten)]
    pub raw: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Method {
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
    DeleteEvent,
    GetStats,
    Unknown,
}

struct MethodVisitor;

impl<'de> Visitor<'de> for MethodVisitor {
    type Value = Method;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let method = match value {
            _ if value == "initEventIndex" => Method::InitEventIndex,
            _ if value == "loadCheckpoints" => Method::LoadCheckpoints,
            _ if value == "isEventIndexEmpty" => Method::IsEventIndexEmpty,
            _ if value == "commitLiveEvents" => Method::CommitLiveEvents,
            _ if value == "addEventToIndex" => Method::AddEventToIndex,
            _ if value == "addCrawlerCheckpoint" => Method::AddCrawlerCheckpoint,
            _ if value == "removeCrawlerCheckpoint" => Method::RemoveCrawlerCheckpoint,
            _ if value == "addHistoricEvents" => Method::AddHistoricEvents,
            _ if value == "searchEventIndex" => Method::SearchEventIndex,
            _ if value == "loadFileEvents" => Method::LoadFileEvents,
            _ if value == "closeEventIndex" => Method::CloseEventIndex,
            _ if value == "deleteEventIndex" => Method::DeleteEventIndex,
            _ if value == "deleteEvent" => Method::DeleteEvent,
            _ if value == "getStats" => Method::GetStats,
            _ => Method::Unknown,
        };

        Ok(method)
    }
}

fn method_to_enum<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(MethodVisitor)
}

fn event_store_default() -> String {
    "default".to_owned()
}
