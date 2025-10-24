use crate::hotkey::HotkeyListener;
use crate::output::Output;
use crate::process::ProcessManager;
use crate::signals::{SignalEvent, SignalHandler};
use anyhow::Result;

pub struct Supervisor {
    process_manager: ProcessManager,
    signal_handler: SignalHandler,
    hotkey_listener: Option<HotkeyListener>,
    stop_on_child_exit: bool,
    restart_signal: String,
    restart_hotkey: char,
    output: Output,
}

impl Supervisor {
    pub fn new(
        process_manager: ProcessManager,
        signal_handler: SignalHandler,
        hotkey_listener: Option<HotkeyListener>,
        stop_on_child_exit: bool,
        restart_signal: String,
        restart_hotkey: char,
        output: Output,
    ) -> Self {
        Self {
            process_manager,
            signal_handler,
            hotkey_listener,
            stop_on_child_exit,
            restart_signal,
            restart_hotkey,
            output,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Spawn initial process
        self.process_manager.spawn().await?;

        // Only enable raw mode after successfully spawning the process
        // This prevents raw mode from being activated when the command doesn't exist
        if let Some(ref mut listener) = self.hotkey_listener {
            listener.enable_raw_mode()?;
            self.output.info(&format!(
                "[supi] Hotkey listener active: press '{}' to restart",
                self.restart_hotkey
            ));
        }

        loop {
            tokio::select! {

                // Handle signals
                Some(signal_event) = self.signal_handler.next() => {
                    match signal_event {
                        SignalEvent::Terminate(signal_name) => {
                            self.output.log(&format!("[supi] Received {} signal, shutting down...", signal_name));
                            self.process_manager.shutdown().await?;
                            break;
                        }
                        SignalEvent::Restart(signal_name) => {
                            self.output.log(&format!("[supi] Received {} signal", signal_name));
                            if self.process_manager.is_running() {
                                self.process_manager.restart().await?;
                            } else {
                                self.output.log("[supi] Child process not running, starting...");
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
                    self.output.log("[supi] Hotkey pressed, restarting...");
                    if self.process_manager.is_running() {
                        self.process_manager.restart().await?;
                    } else {
                        self.output.log("[supi] Child process not running, starting...");
                        self.process_manager.spawn().await?;
                    }
                }

                // Handle child process exit
                status = self.process_manager.wait(), if self.process_manager.is_running() => {
                    match status {
                        Ok(exit_status) => {
                            self.output.log(&format!("[supi] Child process exited with status: {}", exit_status));

                            if self.stop_on_child_exit {
                                self.output.log("[supi] Exiting (--stop-on-child-exit is set)");
                                break;
                            } else {
                                self.output.log("[supi] Child process exited, but supervisor continues running");
                                if self.hotkey_listener.is_some() {
                                    self.output.info(&format!("[supi] Press Ctrl+C to exit, press hotkey '{}' to restart, or send signal({}) to restart", self.restart_hotkey, self.restart_signal));
                                } else {
                                    self.output.info(&format!("[supi] Press Ctrl+C to exit, or send signal({}) to restart",
                                    self.restart_signal));
                                }
                                // Continue loop, waiting for signals
                            }
                        }
                        Err(e) => {
                            self.output.elog(&format!("[supi] Error waiting for child process: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
