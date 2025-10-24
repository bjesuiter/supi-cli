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
        // Spawn initial process
        self.process_manager.spawn().await?;

        // Wait for process to exit
        let status = self.process_manager.wait().await?;

        println!("[supi] Process exited with status: {}", status);

        if self.stop_on_child_exit {
            println!("[supi] Exiting (--stop-on-child-exit is set)");
        } else {
            println!("[supi] Process exited, but supervisor continues running");
            println!("[supi] (Press Ctrl+C to exit, or send restart signal to restart)");
            // TODO: In next phase, add event loop for signals/hotkeys
            // For now, just exit
        }

        Ok(())
    }
}
