use std::collections::HashMap;

use hyprland::shared::Address;

#[derive(Clone, Debug)]
pub(crate) struct AppManager {
    windows: HashMap<Address, String>,
    apps: Vec<(String, Vec<Window>)>
}

#[derive(Clone, Debug)]
pub(crate) struct Window {
    title: String,
    addr: Address
}

impl AppManager {
    pub(super) fn new() -> Self {
        Self {
            windows: HashMap::new(),
            apps: Vec::new()
        }
    }

    pub(super) fn add_window(&mut self, class: String, addr: Address, title: String) -> Option<()> {
        self.windows.insert(addr.clone(), class.clone());

        let pos = self.app_pos(&class);
        if let Some(i) = pos {
            let (_, vec) = self.apps.get_mut(i)?;
            vec.push(Window::new(addr, title));
        } else {
            self.apps.push((class, vec![Window::new(addr, title)]));
        }

        Some(())
    }

    pub(super) fn remove_window(&mut self, addr: &Address) -> Option<()> {
        let class = self.windows.remove(&addr)?;

        let app_pos = self.app_pos(&class)?;
        let (_, vec) = self.apps.get_mut(app_pos)?;
        let pos = Self::window_pos(vec, addr)?;
        vec.remove(pos);
        if vec.len() == 0 {
            self.apps.remove(app_pos);
        }

        Some(())
    }

    pub(super) fn move_to_top(&mut self, class: String, addr: Address) -> Option<()> {
        let app_pos = self.app_pos(&class)?;

        let mut app = self.apps.remove(app_pos);

        let (_, app_windows) = &mut app;
        let window_pos = Self::window_pos(app_windows, &addr)?;
        let window = app_windows.remove(window_pos);
        app_windows.insert(0, window);
        
        self.apps.insert(0, app);

        Some(())
    }

    pub(crate) const fn get_apps(&self) -> &Vec<(String, Vec<Window>)> {
        &self.apps
    }

    fn app_pos(&self, class: &str) -> Option<usize> {
        self.apps.iter().position(|(it, _)| class == it)
    }

    fn window_pos(clients: &Vec<Window>, addr: &Address) -> Option<usize> {
        clients.iter().position(|it| &it.addr == addr)
    }
}

impl Window {
    pub(crate) const fn new(addr: Address, title: String) -> Self {
        Self { title, addr }
    }
}
