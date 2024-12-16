use std::sync::{Arc, Mutex};

use app_manager::AppManager;
use gui::start_gui;
use log::{debug, info};
use tokio::{select, sync::mpsc::unbounded_channel};
use unix::{exists_socket, remove_socket, ClientListener};
use wayland::{init_windows, create_hypr_listener};
use anyhow::{Context as _, Result};

pub(crate) mod icon;
mod app_manager;
mod wayland;
mod unix;
mod gui;

pub(crate) async fn launch_daemon(force: bool) -> Result<()> {
    if exists_socket()? {
        if force {
            remove_socket()?;
        } else {
            info!("Socket already exists");
            return Ok(())
        }
    }

    let mut app_manager = AppManager::new();
    init_windows(&mut app_manager)?;
    let app_manager = Arc::new(Mutex::new(app_manager));
    let mut hypr_listener = create_hypr_listener(&app_manager);
    let client_listener = ClientListener::new()?;
    let (tx, rx) = unbounded_channel();
    
    select! {
        res = hypr_listener.start_listener_async() => res.context("Hyprland event listener was closed"),
        res = client_listener.listen(tx) => res,
        res = start_gui(app_manager.clone(), rx) => res
    }
}
