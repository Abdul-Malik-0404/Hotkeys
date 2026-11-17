use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ShowWindow,
    ToggleGlobalEnabled,
    Quit,
}

pub struct TrayController {
    rx: Receiver<TrayCommand>,
    #[cfg(target_os = "linux")]
    handle: Option<ksni::blocking::Handle<LinuxTray>>,
}

#[cfg(target_os = "linux")]
struct LinuxTray {
    tx: Sender<TrayCommand>,
    global_enabled: bool,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for LinuxTray {
    fn id(&self) -> String {
        "hotkeys".into()
    }

    fn title(&self) -> String {
        if self.global_enabled {
            "Hotkeys (Active)".into()
        } else {
            "Hotkeys (Paused)".into()
        }
    }

    fn icon_name(&self) -> String {
        "preferences-desktop-keyboard-shortcuts".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayCommand::ShowWindow);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::{CheckmarkItem, MenuItem, StandardItem};
        vec![
            StandardItem {
                label: "Open Keybind Settings".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send(TrayCommand::ShowWindow);
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "Enable Global Hotkeys".into(),
                checked: self.global_enabled,
                activate: Box::new(|tray: &mut Self| {
                    tray.global_enabled = !tray.global_enabled;
                    let _ = tray.tx.send(TrayCommand::ToggleGlobalEnabled);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Hotkeys".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.tx.send(TrayCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl TrayController {
    pub fn spawn(initial_enabled: bool) -> Self {
        #[allow(unused_variables)]
        let (tx, rx) = mpsc::channel();

        #[cfg(target_os = "linux")]
        let handle = {
            use ksni::blocking::TrayMethods;
            let tray = LinuxTray {
                tx,
                global_enabled: initial_enabled,
            };
            tray.assume_sni_available(true).spawn().ok()
        };

        Self {
            rx,
            #[cfg(target_os = "linux")]
            handle,
        }
    }

    pub fn try_recv(&self) -> Option<TrayCommand> {
        self.rx.try_recv().ok()
    }

    #[allow(unused_variables)]
    pub fn set_global_enabled(&self, enabled: bool) {
        #[cfg(target_os = "linux")]
        if let Some(handle) = &self.handle {
            let _ = handle.update(|tray| {
                tray.global_enabled = enabled;
            });
        }
    }
}
