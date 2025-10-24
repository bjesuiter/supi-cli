use anyhow::Result;

pub async fn setup_signal_handlers(restart_signal: String) -> Result<()> {
    let _ = restart_signal;
    anyhow::bail!("setup_signal_handlers not implemented yet")
}
