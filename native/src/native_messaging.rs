use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use serde_json::{json, Value};
use std::io::{self, prelude::*, Cursor};

use crate::Error;

pub(crate) fn stdin() -> Result<(i64, Value), Error> {
    let mut buffer = [0; 4];
    io::stdin().read_exact(&mut buffer)?;
    let mut buf = Cursor::new(&buffer);
    let size = buf.read_u32::<NativeEndian>()?;

    let mut data_buffer = vec![0u8; size as usize];
    io::stdin().read_exact(&mut data_buffer)?;
    let message: Value = serde_json::from_slice(&data_buffer)?;
    let rpc_id = as_i64!(message, "rpc_id");

    Ok((rpc_id, message))
}

pub(crate) fn stdout_ready() {
    stdout(json!({
        "ready": true
    }))
    .unwrap_or_else(|error| eprintln!("{:?}", error));
}

pub(crate) fn stdout_reply(rpc_id: i64, reply: Value) {
    stdout(json!({
        "rpc_id": rpc_id,
        "reply": reply,
    }))
    .unwrap_or_else(|error| eprintln!("{:?}", error));
}

pub(crate) fn stdout_error(rpc_id: i64, error: Error) {
    stdout(json!({
        "rpc_id": rpc_id,
        "error": format!("{:?}", error),
    }))
    .unwrap_or_else(|error| eprintln!("{:?}", error));
}

fn stdout(message: Value) -> Result<(), Error> {
    let message = serde_json::to_string(&message)?;
    let mut size = Vec::default();
    size.write_u32::<NativeEndian>(message.len() as u32)?;
    io::stdout().write(&size)?;
    io::stdout().write(&message.into_bytes())?;
    Ok(io::stdout().flush()?)
}
