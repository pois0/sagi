use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
use gtk::{subclass::{frame::FrameImpl, widget::WidgetImpl}, Frame};

#[derive(Default)]
pub struct SelectableFrame;

#[glib::object_subclass]
impl ObjectSubclass for SelectableFrame {
    const NAME: &'static str = "SagiSelectableFrame";

    type Type = super::SelectableFrame;
    type ParentType = Frame;
}

impl ObjectImpl for SelectableFrame {}

impl WidgetImpl for SelectableFrame {}

impl FrameImpl for SelectableFrame {}
