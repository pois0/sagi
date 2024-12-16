use std::path::PathBuf;

pub(crate) fn lookup_icon(class_name: &str) -> Option<PathBuf> {
    lookup_icon_inner(class_name)
        .or_else(|| lookup_icon_inner(&class_name.to_ascii_lowercase()))
}

fn lookup_icon_inner(class_name: &str) -> Option<PathBuf> {
    freedesktop_icons::lookup(class_name)
        .with_size(96)
        .find()
}
