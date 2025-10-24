use crate::process::ProcessManager;
use anyhow::Result;

pub struct Supervisor {
    process_manager: ProcessManager,
    stop_on_child_exit: bool,
}

impl Supervisor {
    pub fn new(process_manager: ProcessManager, stop_on_child_exit: bool) -> Self {
        Self {
            process_manager,
            stop_on_child_exit,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let _ = self.stop_on_child_exit;
        anyhow::bail!("Supervisor::run not implemented yet")
    }
}
