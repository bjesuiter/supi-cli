mod cli;
mod hotkey;
mod output;
mod process;
mod signals;
mod supervisor;

use clap::Parser;
use cli::Cli;
use hotkey::HotkeyListener;
use process::ProcessManager;
use signals::SignalHandler;
use supervisor::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    sprintln!("[supi] Supervisor PID: {}", std::process::id());
    sprintln!("[supi] Starting supervisor");
    sprintln!(
        "[supi] Config: restart_signal={}, restart_hotkey='{}', stop_on_child_exit={}",
        args.restart_signal,
        args.restart_hotkey,
        args.stop_on_child_exit
    );

    let process_manager = ProcessManager::new(args.command, args.args);
    let signal_handler = SignalHandler::new(&args.restart_signal)?;

    // Set up hotkey listener
    let hotkey_listener = match HotkeyListener::new(args.restart_hotkey) {
        Ok(listener) => {
            sprintln!(
                "[supi] Hotkey listener active: press '{}' to restart",
                args.restart_hotkey
            );
            Some(listener)
        }
        Err(e) => {
            seprintln!("[supi] Warning: Could not enable hotkey listener: {}", e);
            seprintln!("[supi] Continuing without hotkey support (signals still work)");
            None
        }
    };

    let mut supervisor = Supervisor::new(
        process_manager,
        signal_handler,
        hotkey_listener,
        args.stop_on_child_exit,
    );

    supervisor.run().await?;

    Ok(())
}
