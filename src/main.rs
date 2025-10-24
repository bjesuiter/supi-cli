mod cli;
mod hotkey;
mod output;
mod process;
mod signals;
mod supervisor;

use clap::Parser;
use cli::Cli;
use hotkey::HotkeyListener;
use output::{LogColor, Output};
use process::ProcessManager;
use signals::SignalHandler;
use supervisor::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Parse log colors and create Output instance
    let log_color = LogColor::from_str(&args.log_color).map_err(|e| anyhow::anyhow!(e))?;
    let info_color = LogColor::from_str(&args.info_color).map_err(|e| anyhow::anyhow!(e))?;
    let output = Output::new(log_color, info_color, args.silent);

    output.log(&format!("[supi] Supervisor PID: {}", std::process::id()));
    output.log("[supi] Starting supervisor");
    output.log(&format!(
        "[supi] Config: restart_signal={}, restart_hotkey='{}', stop_on_child_exit={}",
        args.restart_signal, args.restart_hotkey, args.stop_on_child_exit
    ));

    let process_manager = ProcessManager::new(args.command, args.args, output.clone());
    let signal_handler = SignalHandler::new(&args.restart_signal)?;

    // Set up hotkey listener (raw mode will be enabled in supervisor after command validation)
    let hotkey_listener = match HotkeyListener::new(args.restart_hotkey) {
        Ok(listener) => Some(listener),
        Err(e) => {
            output.elog(&format!(
                "[supi] Warning: Could not set up hotkey listener: {}",
                e
            ));
            output.elog("[supi] Continuing without hotkey support (signals still work)");
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
        output,
    );

    supervisor.run().await?;

    Ok(())
}
