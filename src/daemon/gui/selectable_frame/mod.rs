mod imp;

use glib::Object;
use gtk::{glib, Frame, Widget, Accessible, Actionable, Buildable};

glib::wrapper! {
    pub struct SelectableFrame(ObjectSubclass<imp::SelectableFrame>)
        @extends Frame, Widget,
        @implements Accessible, Actionable, Buildable;
}

impl SelectableFrame {
    pub fn new() -> Self {
        Object::builder().build()
    }


}
