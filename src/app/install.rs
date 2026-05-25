use tokio::task::JoinHandle;

use crate::core::pacman::{Action, LogLine};

pub struct InstallState {
    pub package_name: String,
    pub action: Action,
    pub log: Vec<LogLine>,
    pub scroll: u16,
    pub command_preview: String,
    pub waiter: Option<JoinHandle<()>>,
}
