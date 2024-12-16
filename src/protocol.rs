use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum Request {
    Launch(Launch),
    MoveCursor(Direction),
    ShowWindows,
    SelectCurrent,
    StopDaemon
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum Launch {
    App,
    WindowInApp
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum Direction {
    Left,
    Right
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Response {
    Accepted,
    Refused
}

pub(crate) fn encode_request(msg: Request) -> Vec<u8> {
    bincode::serialize(&msg).unwrap()
}

pub(crate) fn decode_request(raw_msg: &[u8]) -> Result<Request> {
    bincode::deserialize(raw_msg).context("Failed to decode the request")
}

pub(crate) fn encode_response(msg: Response) -> Vec<u8> {
    bincode::serialize(&msg).unwrap()
}

pub(crate) fn decode_response(raw_msg: &[u8]) -> Result<Response> {
    bincode::deserialize(raw_msg).context("Failed to decode the response")
}

pub(crate) fn get_socket_path() -> PathBuf {
    let mut buf = if let Ok(runtime_path) = env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_path)
    } else if let Ok(uid) = env::var("UID") {
        PathBuf::from(format!("/run/user/{}", &uid))
    } else {
        PathBuf::from("/tmp")
    };

    buf.push("sagi.socket");
    buf
}
