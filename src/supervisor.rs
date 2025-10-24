use crate::hotkey::HotkeyListener;
use crate::output::LogColor;
use crate::process::ProcessManager;
use crate::signals::{SignalEvent, SignalHandler};
use crate::{seprintln_colored, sprintln_colored};
use anyhow::Result;

pub struct Supervisor {
    process_manager: ProcessManager,
    signal_handler: SignalHandler,
    hotkey_listener: Option<HotkeyListener>,
    stop_on_child_exit: bool,
    log_color: LogColor,
}

impl Supervisor {
    pub fn new(
        process_manager: ProcessManager,
        signal_handler: SignalHandler,
        hotkey_listener: Option<HotkeyListener>,
        stop_on_child_exit: bool,
        log_color: LogColor,
    ) -> Self {
        Self {
            process_manager,
            signal_handler,
            hotkey_listener,
            stop_on_child_exit,
            log_color,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Spawn initial process
        self.process_manager.spawn().await?;

        // Only enable raw mode after successfully spawning the process
        // This prevents raw mode from being activated when the command doesn't exist
        if let Some(ref mut listener) = self.hotkey_listener {
            listener.enable_raw_mode()?;
            sprintln_colored!(
                self.log_color,
                "[supi] Hotkey listener active: press '{}' to restart",
                listener.hotkey()
            );
        }

        loop {
            tokio::select! {
                // Handle signals
                Some(signal_event) = self.signal_handler.next() => {
                    match signal_event {
                        SignalEvent::Terminate(signal_name) => {
                            sprintln_colored!(self.log_color, "[supi] Received {} signal, shutting down...", signal_name);
                            self.process_manager.shutdown().await?;
                            break;
                        }
                        SignalEvent::Restart(signal_name) => {
                            sprintln_colored!(self.log_color, "[supi] Received {} signal", signal_name);
                            if self.process_manager.is_running() {
                                self.process_manager.restart().await?;
                            } else {
                                sprintln_colored!(self.log_color, "[supi] Child process not running, starting...");
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
                    sprintln_colored!(self.log_color, "[supi] Hotkey pressed, restarting...");
                    if self.process_manager.is_running() {
                        self.process_manager.restart().await?;
                    } else {
                        sprintln_colored!(self.log_color, "[supi] Child process not running, starting...");
                        self.process_manager.spawn().await?;
                    }
                }
                // Handle child process exit
                status = self.process_manager.wait(), if self.process_manager.is_running() => {
                    match status {
                        Ok(exit_status) => {
                            sprintln_colored!(self.log_color, "[supi] Child process exited with status: {}", exit_status);

                            if self.stop_on_child_exit {
                                sprintln_colored!(self.log_color, "[supi] Exiting (--stop-on-child-exit is set)");
                                break;
                            } else {
                                sprintln_colored!(self.log_color, "[supi] Child process exited, but supervisor continues running");
                                if self.hotkey_listener.is_some() {
                                    sprintln_colored!(self.log_color, "[supi] (Press Ctrl+C to exit, or press hotkey/send restart signal to restart)");
                                } else {
                                    sprintln_colored!(self.log_color, "[supi] (Press Ctrl+C to exit, or send restart signal to restart)");
                                }
                                // Continue loop, waiting for signals
                            }
                        }
                        Err(e) => {
                            seprintln_colored!(self.log_color, "[supi] Error waiting for child process: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
