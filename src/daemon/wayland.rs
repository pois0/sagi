use std::sync::{Arc, Mutex};

use hyprland::{data::{Client, Clients}, dispatch::{Dispatch, DispatchType, WindowIdentifier}, event_listener::EventListener, shared::{Address, HyprData, HyprError}};
use anyhow::Result;
use log::debug;

use super::app_manager::AppManager;

pub(super) fn create_hypr_listener(app_manager: &Arc<Mutex<AppManager>>) -> EventListener {
    let mut listener = EventListener::new();

    let am = Arc::clone(&app_manager);
    listener.add_window_opened_handler(move |e| {
        debug!("Window opened: {e:?}");
        am.lock().unwrap()
            .add_window(e.window_class, e.window_address, e.window_title);
    });

    let am = Arc::clone(&app_manager);
    listener.add_window_closed_handler(move |addr| {
        debug!("Window closed: {addr:?}");
        am.lock().unwrap()
            .remove_window(&addr);
    });

    let am = Arc::clone(&app_manager);
    listener.add_active_window_changed_handler(move |e| {
        if let Some(e) = e {
            debug!("Active window changed: {e:?}");
            am.lock().unwrap() 
                .move_to_top(e.class, e.address);
        }
    });

    listener
}

pub(super) fn activate_window(addr: Address) -> Result<(), HyprError> {
    Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(addr)))
}

pub(super) fn init_windows(app_manager: &mut AppManager) -> Result<()> {
    let clients = Clients::get()?;
    debug!("Opened windows:");
    clients.into_iter()
        .for_each(|Client { class, address, title, .. }| {
            debug!("    {{ Class: \"{class}\", Title: \"{title}\", Address: {address} }}");
            app_manager.add_window(class, address, title);
        });

    Ok(())
}

