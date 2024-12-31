mod css;

use std::sync::{Arc, Mutex};

use css::DEFAULT_CSS;
use either::Either;
use gtk::{style_context_add_provider_for_display, gdk, gio::{spawn_blocking, ApplicationHoldGuard}, prelude::*, Application, ApplicationWindow, CssProvider, FlowBox, Frame, Image, Overlay, STYLE_PROVIDER_PRIORITY_USER};
use anyhow::{anyhow, Result, Context as _};
use gtk_layer_shell::{Layer, LayerShell as _};
use tokio::sync::{mpsc::UnboundedReceiver, Notify};
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

use crate::protocol::{Direction, Launch};

use super::{app_manager::{AppManager, Applications}, icon::lookup_icon, wayland::activate_window};

const CURRENT_ITEM_CLASS: &str = "current-item";

#[derive(Clone, Debug)]
pub(super) enum GuiOp {
    Launch(Launch),
    MoveCursor(Direction),
    ShowWindows,
    SelectCurrent
}

type SwitcherContext = Either<Closed, Open>;

struct Open {
    window: ApplicationWindow,
    apps: Applications,
    frames: Vec<Frame>,
    cursor: usize
}

struct Closed {
    #[allow(dead_code)]
    hold_guard: ApplicationHoldGuard
}

const fn new_open_ctx(window: ApplicationWindow, frames: Vec<Frame>, apps: Applications) -> SwitcherContext {
    Either::Right(Open{
        window,
        apps,
        frames,
        cursor: 0
    })
}

fn new_closed_ctx(app: &Application) -> SwitcherContext {
    Either::Left(Closed {
        hold_guard: app.hold()
    })
}

pub(super) async fn start_gui(app_manager: Arc<Mutex<AppManager>>, receiver: UnboundedReceiver<GuiOp>) -> Result<()> {
    spawn_blocking(move || {
        let app = create_application();

        let ctx = new_closed_ctx(&app);
        let activation_notify = Arc::new(Notify::new());
        let activation_notify2 = activation_notify.clone();

        app.connect_activate(move |_| {
            load_css();
            activation_notify.notify_one();
        });


        let app2 = app.clone();
        glib::spawn_future_local(async move {
            activation_notify2.notified().await;

            UnboundedReceiverStream::new(receiver)
                .fold(ctx, |ctx, op| match op {
                    GuiOp::Launch(launch) => ctx.left_and_then(|_| {
                        let apps = app_manager.lock().unwrap().get_apps().clone();
                        let (window, frames) = create_window(&app2, &apps);
                        window.show();
                        new_open_ctx(window, frames, apps)
                    }),
                    GuiOp::MoveCursor(direction) => ctx.map_right(|Open { apps, frames, cursor, window }| {
                        frames[cursor].remove_css_class(CURRENT_ITEM_CLASS);
                        let cursor = match direction {
                            Direction::Prev => 
                                if cursor == 0 {
                                    apps.len() - 1
                                } else {
                                    cursor - 1
                                },
                            Direction::Next =>
                                if cursor == apps.len() - 1 {
                                    0
                                } else {
                                    cursor + 1
                                },
                        };
                        frames[cursor].add_css_class(CURRENT_ITEM_CLASS);
                        Open {
                            apps,
                            frames,
                            window,
                            cursor
                        }
                    }),
                    GuiOp::ShowWindows => todo!(),
                    GuiOp::SelectCurrent => ctx.right_and_then(|Open { window, apps, cursor, .. }| {
                        let (_, windows) = &apps[cursor];
                        let _ = activate_window(windows[0].addr().clone());
                        window.close();
                        new_closed_ctx(&app2)
                    })
                }).await;

            // while let Some(op) = receiver.recv().await {
            //     match op {
            //         GuiOp::Launch(launch) => {
            //             window = Some({
            //                 let (it, f) = create_window(&app_manager, &app2);
            //                 frames = f;
            //                 it.show();
            //                 it
            //             });
            //             hold_guard = None;
            //         }
            //         GuiOp::MoveCursor(direction) => {
            //             // let Some(app_window) = &window else { continue };
            //             // let overlay = app_window.first_child().unwrap();
            //             // let overlay = overlay.downcast_ref::<Overlay>().unwrap();
            //             // let flow_box = overlay.first_child().unwrap();
            //             // let flow_box = flow_box.downcast_ref::<FlowBox>().unwrap();
            //             // let frame = flow_box.first_child().unwrap();
            //             // let frame = frame.downcast_ref::<Frame>().unwrap();
            //             frames.get(0).unwrap().set_label(Some(&format!("Count: {test}")));
            //             test += 1;
            //         }
            //         GuiOp::ShowWindows => todo!(),
            //         GuiOp::SelectCurrent => {
            //             let Some(window) = window.take() else { continue };
            //             window.close();
            //             hold_guard = Some(app2.hold());
            //         }
            //     }
            // }
        });

        app.run_with_args::<&str>(&[]);
    }).await.map_err(|_| anyhow!("Gui task was failed"))
}

fn create_window(app: &Application, apps: &Applications) -> (ApplicationWindow, Vec<Frame>) {
    let flow_box = FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .orientation(gtk::Orientation::Horizontal)
        .max_children_per_line(16)
        .min_children_per_line(16)
        .build();

    let mut frames = Vec::new();
    for (class_name, _) in apps {
        let app_frame = app_frame(class_name);
        flow_box.insert(&app_frame, -1);
        frames.push(app_frame);
    }
    if let Some(first_frame) = frames.first() {
        first_frame.add_css_class(CURRENT_ITEM_CLASS);
    }

    let overlay = Overlay::builder()
        .child(&flow_box)
        .build();
    let window = ApplicationWindow::builder()
        .application(app)
        .child(&overlay)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.present();
    (window, frames)
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
        .css_classes(vec!["app-frame"])
        .child(&icon_image)
        .build()
}

fn create_application() -> Application {
    Application::builder()
        .application_id("jp.pois.sagi")
        .build()
}

fn load_css() -> Result<()> {
    let css_provider = CssProvider::new();
    css_provider.load_from_data(DEFAULT_CSS);
    let display = &gdk::Display::default().context("Failed to connect to a display")?;

    style_context_add_provider_for_display(
        display,
        &css_provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );

    Ok(())
}
