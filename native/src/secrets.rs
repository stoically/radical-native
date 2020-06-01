use anyhow::Result;
use base64::{encode_config, STANDARD_NO_PAD};
use keytar::{delete_password, get_password, set_password, Password};
use rand::random;
use serde::Deserialize;
use serde_json::{json, Value};

const SERVICE: &str = "riot.im";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "method")]
enum Message {
    GetPickleKey { content: MessageContent },
    CreatePickleKey { content: MessageContent },
    DestroyPickleKey { content: MessageContent },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageContent {
    user_id: String,
    device_id: String,
}

pub fn handle_message(message_in: Value) -> Result<Value> {
    let message: Message = serde_json::from_value(message_in)?;
    let res = match message {
        Message::GetPickleKey { content } => {
            let Password { success, password } = get_password(
                SERVICE.to_owned(),
                account_string(content.user_id, content.device_id),
            )?;

            if success {
                json!(password)
            } else {
                json!(null)
            }
        }
        Message::CreatePickleKey { content } => {
            let random_vec: Vec<u8> = (0..32).into_iter().map(|_| random::<u8>()).collect();
            let pickle_key = encode_config(&random_vec, STANDARD_NO_PAD);
            let res = json!(pickle_key);

            set_password(
                SERVICE.to_owned(),
                account_string(content.user_id, content.device_id),
                pickle_key,
            )?;

            res
        }
        Message::DestroyPickleKey { content } => {
            let success = delete_password(
                SERVICE.to_owned(),
                account_string(content.user_id, content.device_id),
            )?;

            json!(success)
        }
    };

    Ok(res)
}

fn account_string(user_id: String, device_id: String) -> String {
    format!("{}|{}", user_id, device_id)
}
