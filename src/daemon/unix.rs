use std::fs;

use anyhow::{Context as _, Result};
use log::debug;
use tokio::{io::{AsyncReadExt as _, AsyncWriteExt as _}, net::{UnixListener, UnixStream}, sync::mpsc::UnboundedSender};

use crate::protocol::{decode_request, encode_response, get_socket_path, Response};

use super::gui::GuiOp;

pub(super) struct ClientListener {
    unix_listener: UnixListener
}

impl ClientListener {
    pub(super) fn new() -> Result<Self> {
        let socket_path = get_socket_path();
        let msg = format!("The socket is created at: {}", socket_path.to_str().unwrap_or("Unknown"));
        let listener = UnixListener::bind(socket_path)?;
        debug!("{}", msg);
        Ok(Self {
            unix_listener: listener
        })
    }

    pub(super) async fn listen(self, mut sender: UnboundedSender<GuiOp>) -> Result<()> {
        let listener = &self.unix_listener;

        loop {
            match listener.accept().await {
                Ok((mut stream, address)) => {
                    debug!("Connected a client: {address:?}");
                    match handle(&mut stream, &mut sender).await {
                        Ok(cont) => {
                            response_and_shutdown(&mut stream, Response::Accepted).await?;
                            if !cont {
                                return Ok(())
                            }
                        }
                        Err(e) => {
                            response_and_shutdown(&mut stream, Response::Refused).await?;
                            return Err(e)
                        }
                    }
                }
                Err(e) => return Err(e).context("Failed to listen events")
            }
        }
    }
}

impl Drop for ClientListener {
    fn drop(&mut self) {
        let _ = remove_socket();
    }
}

pub(super) fn exists_socket() -> Result<bool> {
    fs::exists(get_socket_path()).context("Failed to check the socket exists")
}

pub(super) fn remove_socket() -> Result<()> {
    debug!("Removing the socket");
    fs::remove_file(get_socket_path()).context("Failed to remove the socket")
}

async fn response_and_shutdown(stream: &mut UnixStream, response: Response) -> Result<()> {
    debug!("Sending a response: {response:?}");
    stream.write_all(&encode_response(response)).await.context("Failed to write the response")?;
    let result = stream.shutdown().await.context("Failed to close the stream");
    debug!("Stream was shutdowned");
    result
}

async fn handle(stream: &mut UnixStream, sender: &mut UnboundedSender<GuiOp>) -> Result<bool> {
    let mut buf = vec![0; 1024];
    let size = stream.read(&mut buf).await.context("Failed to read the request")?;
    let req = decode_request(&buf[..size])?;
    let op = match req {
        crate::protocol::Request::Launch(launch) => GuiOp::Launch(launch),
        crate::protocol::Request::MoveCursor(d) => GuiOp::MoveCursor(d),
        crate::protocol::Request::ShowWindows => GuiOp::ShowWindows,
        crate::protocol::Request::SelectCurrent => GuiOp::SelectCurrent,
        crate::protocol::Request::StopDaemon => {
            return Ok(false)
        }
    };
    sender.send(op)?;
    Ok(true)
}

