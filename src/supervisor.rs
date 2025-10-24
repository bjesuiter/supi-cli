use crate::hotkey::HotkeyListener;
use crate::process::ProcessManager;
use crate::signals::{SignalEvent, SignalHandler};
use crate::{seprintln, sprintln};
use anyhow::Result;

pub struct Supervisor {
    process_manager: ProcessManager,
    signal_handler: SignalHandler,
    hotkey_listener: Option<HotkeyListener>,
    stop_on_child_exit: bool,
}

impl Supervisor {
    pub fn new(
        process_manager: ProcessManager,
        signal_handler: SignalHandler,
        hotkey_listener: Option<HotkeyListener>,
        stop_on_child_exit: bool,
    ) -> Self {
        Self {
            process_manager,
            signal_handler,
            hotkey_listener,
            stop_on_child_exit,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Spawn initial process
        self.process_manager.spawn().await?;

        loop {
            tokio::select! {
                // Handle signals
                Some(signal_event) = self.signal_handler.next() => {
                    match signal_event {
                        SignalEvent::Terminate(signal_name) => {
                            sprintln!("[supi] Received {} signal, shutting down...", signal_name);
                            self.process_manager.shutdown().await?;
                            break;
                        }
                        SignalEvent::Restart(signal_name) => {
                            sprintln!("[supi] Received {} signal", signal_name);
                            if self.process_manager.is_running() {
                                self.process_manager.restart().await?;
                            } else {
                                sprintln!("[supi] Child process not running, starting...");
                                self.process_manager.spawn().await?;
                            }
                        }
                    }
                }
                // Handle hotkey press
                Some(_) = async {
                    match &mut self.hotkey_listener {
                        Some(listener) => listener.next().await,
                        None => std::future::pending().await,
                    }
                } => {
                    sprintln!("[supi] Hotkey pressed, restarting...");
                    if self.process_manager.is_running() {
                        self.process_manager.restart().await?;
                    } else {
                        sprintln!("[supi] Child process not running, starting...");
                        self.process_manager.spawn().await?;
                    }
                }
                // Handle child process exit
                status = self.process_manager.wait(), if self.process_manager.is_running() => {
                    match status {
                        Ok(exit_status) => {
                            sprintln!("[supi] Child process exited with status: {}", exit_status);

                            if self.stop_on_child_exit {
                                sprintln!("[supi] Exiting (--stop-on-child-exit is set)");
                                break;
                            } else {
                                sprintln!("[supi] Child process exited, but supervisor continues running");
                                if self.hotkey_listener.is_some() {
                                    sprintln!("[supi] (Press Ctrl+C to exit, or press hotkey/send restart signal to restart)");
                                } else {
                                    sprintln!("[supi] (Press Ctrl+C to exit, or send restart signal to restart)");
                                }
                                // Continue loop, waiting for signals
                            }
                        }
                        Err(e) => {
                            seprintln!("[supi] Error waiting for child process: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
