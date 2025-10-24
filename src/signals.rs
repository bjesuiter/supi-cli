use anyhow::{Context, Result};
use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

pub enum SignalEvent {
    Terminate, // SIGINT, SIGTERM, SIGQUIT
    Restart,   // Configurable restart signal (e.g., SIGUSR1)
}

pub struct SignalHandler {
    signals: Signals,
}

impl SignalHandler {
    pub fn new(restart_signal_name: &str) -> Result<Self> {
        let restart_signal = parse_signal_name(restart_signal_name)
            .context(format!("Invalid restart signal: {}", restart_signal_name))?;

        // Set up signal handlers for termination signals and restart signal
        let signals = Signals::new(&[SIGINT, SIGTERM, SIGQUIT, restart_signal])
            .context("Failed to create signal handler")?;

        Ok(Self { signals })
    }

    pub async fn next(&mut self) -> Option<SignalEvent> {
        if let Some(signal) = self.signals.next().await {
            match signal {
                SIGINT | SIGTERM | SIGQUIT => Some(SignalEvent::Terminate),
                SIGUSR1 | SIGUSR2 => Some(SignalEvent::Restart),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn parse_signal_name(name: &str) -> Result<i32> {
    match name.to_uppercase().as_str() {
        "SIGUSR1" | "USR1" => Ok(SIGUSR1),
        "SIGUSR2" | "USR2" => Ok(SIGUSR2),
        "SIGHUP" | "HUP" => Ok(SIGHUP),
        _ => anyhow::bail!(
            "Unsupported signal: {}. Supported: SIGUSR1, SIGUSR2, SIGHUP",
            name
        ),
    }
}
