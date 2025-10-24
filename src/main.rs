mod cli;
mod hotkey;
mod process;
mod signals;
mod supervisor;

use clap::Parser;
use cli::Cli;
use process::ProcessManager;
use signals::SignalHandler;
use supervisor::Supervisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    println!("[supi] Supervisor PID: {}", std::process::id());
    println!(
        "[supi] Starting supervisor for: {} {:?}",
        args.command, args.args
    );
    println!(
        "[supi] Config: restart_signal={}, restart_hotkey={}, stop_on_child_exit={}",
        args.restart_signal, args.restart_hotkey, args.stop_on_child_exit
    );

    let process_manager = ProcessManager::new(args.command, args.args);
    let signal_handler = SignalHandler::new(&args.restart_signal)?;
    let mut supervisor = Supervisor::new(process_manager, signal_handler, args.stop_on_child_exit);

    supervisor.run().await?;

    Ok(())
}
