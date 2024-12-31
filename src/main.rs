mod daemon;
pub mod cli;
pub(crate) mod protocol;
mod client;

use std::{collections::HashMap, convert::identity, future::Future, path::PathBuf};

use cli::{run_cli, CliHandler, CliParams};
use client::send_request;
use daemon::{icon::lookup_icon, launch_daemon};
use hyprland::{data::{Client, Clients}, shared::HyprData};
use anyhow::{Context as _, Result};
use log::debug;
use protocol::Launch;
use tokio::runtime::Builder;

#[derive(Clone, Debug)]
struct Class {
    clients: Vec<Client>,
    icon_path: Option<PathBuf>
}

impl Class {
    pub fn new(class_name: &str) -> Self {
        Self {
            clients: vec!(),
            icon_path: lookup_icon(class_name)
        }
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.push(client)
    }
}

struct CliHandlerImpl {}

impl CliHandler for CliHandlerImpl {
    type Output = Result<()>;

    fn start_daemon(force: bool) -> Self::Output {
        call_async(launch_daemon(force))
            .and_then(identity)
    }

    fn stop_daemon() -> Self::Output {
        send_request(protocol::Request::StopDaemon)
    }

    fn launch(sc: cli::LaunchCommand) -> Self::Output {
        let sub = match sc {
            cli::LaunchCommand::App => Launch::App,
            cli::LaunchCommand::WindowInApp => Launch::WindowInApp
        };
        send_request(protocol::Request::Launch(sub))
    }

    fn move_cursor(direction: cli::Direction) -> Self::Output {
        let direction = match direction {
            cli::Direction::Prev => protocol::Direction::Prev,
            cli::Direction::Next => protocol::Direction::Next
        };
        send_request(protocol::Request::MoveCursor(direction))
    }

    fn show_windows() -> Self::Output {
        send_request(protocol::Request::ShowWindows)
    }

    fn select_current() -> Self::Output {
        send_request(protocol::Request::SelectCurrent)
    }
}

fn main() -> Result<()> {
    env_logger::init();

    run_cli::<CliHandlerImpl>()
        .and_then(identity)
}

fn call_async<F: Future>(f: F) -> Result<F::Output> {
    let runtime = Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .context("Failed to build async runtime")?;

    Ok(runtime.block_on(f))
}
