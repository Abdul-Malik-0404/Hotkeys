use std::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ShowWindow,
    ToggleGlobalEnabled,
    Quit,
}

#[allow(dead_code)]
pub struct TrayController {
    _tx: Sender<TrayCommand>,
    rx: Receiver<TrayCommand>,
}

impl TrayController {
    #[allow(unused_variables)]
    pub fn spawn(initial_enabled: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        Self { _tx: tx, rx }
    }

    pub fn try_recv(&self) -> Option<TrayCommand> {
        self.rx.try_recv().ok()
    }

    #[allow(unused_variables)]
    pub fn set_global_enabled(&self, enabled: bool) {}
}
