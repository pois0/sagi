mod selectable_frame;

use std::sync::{Arc, Mutex};

use gtk::{gio::spawn_blocking, prelude::*, Application, FlowBox, Frame, Image, Overlay, Window};
use anyhow::{anyhow, Result};
use gtk_layer_shell::{Layer, LayerShell as _};
use tokio::sync::{mpsc::UnboundedReceiver, Notify};

use crate::protocol::{Direction, Launch};

use super::{app_manager::AppManager, icon::lookup_icon};

#[derive(Clone, Debug)]
pub(super) enum GuiOp {
    Launch(Launch),
    MoveCursor(Direction),
    ShowWindows,
    SelectCurrent
}

pub(super) async fn start_gui(app_manager: Arc<Mutex<AppManager>>, mut receiver: UnboundedReceiver<GuiOp>) -> Result<()> {
    spawn_blocking(move || {
        let app = create_application();

        let mut hold_guard = Some(app.hold());
        let activation_notify = Arc::new(Notify::new());
        let activation_notify2 = activation_notify.clone();

        app.connect_activate(move |_| {
            activation_notify.notify_one();
        });

        let app2 = app.clone();
        glib::spawn_future_local(async move {
            let mut window = None;
            activation_notify2.notified().await;
            while let Some(op) = receiver.recv().await {
                match op {
                    GuiOp::Launch(launch) => {
                        window = Some({
                            let it = create_window(&app_manager, &app2);
                            it.show();
                            it
                        });
                        hold_guard = None;
                    }
                    GuiOp::MoveCursor(direction) => todo!(),
                    GuiOp::ShowWindows => todo!(),
                    GuiOp::SelectCurrent => {
                        let Some(window) = window.take() else { continue };
                        window.close();
                        hold_guard = Some(app2.hold());
                    }
                }
            }
        });

        app.run_with_args::<&str>(&[]);
    }).await.map_err(|_| anyhow!("Gui task was failed"))
}

fn create_window(app_manager: &Arc<Mutex<AppManager>>, app: &Application) -> Window {
    let flow_box = FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .orientation(gtk::Orientation::Horizontal)
        .max_children_per_line(16)
        .min_children_per_line(16)
        .build();

    for (class_name, _) in app_manager.lock().unwrap().get_apps() {
        flow_box.insert(&app_frame(class_name), -1);
    }

    let overlay = Overlay::builder()
        .child(&flow_box)
        .build();
    let window = Window::builder()
        .application(app)
        .child(&overlay)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.present();
    window
}

fn app_frame(class_name: &str) -> Frame {
    let icon_path = lookup_icon(class_name).and_then(|it| it.into_os_string().into_string().ok());
    let icon_image = Image::builder()
        .file(&icon_path.unwrap_or("".to_string()))
        .height_request(96)
        .width_request(96)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    Frame::builder()
        .child(&icon_image)
        .build()
}

fn create_application() -> Application {
    Application::builder()
        .application_id("jp.pois.sagi")
        .build()
}
