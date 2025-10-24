mod cli;
mod hotkey;
mod output;
mod process;
mod signals;
mod supervisor;

use clap::Parser;
use cli::Cli;
use hotkey::HotkeyListener;
use output::LogColor;
use process::ProcessManager;
use signals::SignalHandler;
use supervisor::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Parse log colors
    let log_color = LogColor::from_str(&args.log_color).map_err(|e| anyhow::anyhow!(e))?;
    let info_color = LogColor::from_str(&args.info_color).map_err(|e| anyhow::anyhow!(e))?;

    sprintln_colored!(log_color, "[supi] Supervisor PID: {}", std::process::id());
    sprintln_colored!(log_color, "[supi] Starting supervisor");
    sprintln_colored!(
        log_color,
        "[supi] Config: restart_signal={}, restart_hotkey='{}', stop_on_child_exit={}",
        args.restart_signal,
        args.restart_hotkey,
        args.stop_on_child_exit
    );

    let process_manager = ProcessManager::new(args.command, args.args, log_color);
    let signal_handler = SignalHandler::new(&args.restart_signal)?;

    // Set up hotkey listener (raw mode will be enabled in supervisorafter command validation)
    let hotkey_listener = match HotkeyListener::new(args.restart_hotkey) {
        Ok(listener) => Some(listener),
        Err(e) => {
            seprintln_colored!(
                log_color,
                "[supi] Warning: Could not set up hotkey listener: {}",
                e
            );
            seprintln_colored!(
                log_color,
                "[supi] Continuing without hotkey support (signals still work)"
            );
            None
        }
    };

    let mut supervisor = Supervisor::new(
        process_manager,
        signal_handler,
        hotkey_listener,
        args.stop_on_child_exit,
        args.restart_signal,
        args.restart_hotkey,
        log_color,
        info_color,
    );

    supervisor.run().await?;

    Ok(())
}
