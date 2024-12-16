use std::{io::{Read as _, Write as _}, os::unix::net::UnixStream};

use anyhow::{bail, Context as _, Result};
use log::debug;
use crate::protocol::{decode_response, encode_request, get_socket_path, Request, Response};

mod unix;


pub(crate) fn send_request(req: Request) -> Result<()> {
    let mut stream = UnixStream::connect(get_socket_path())
        .context("Failed to connect the daemon")?;
    debug!("Unixstream was created");
    let encode_request = encode_request(req);
    stream.write_all(&encode_request)?;
    let mut buf = vec![0; 1024];
    let size = stream.read(&mut buf)?;
    decode_response(&buf[..size])
        .and_then(|it|
            if it == Response::Accepted {
                Ok(())
            } else {
                bail!("The request was refused.")
            }
        )
}
