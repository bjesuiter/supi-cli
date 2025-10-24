use anyhow::Result;

pub struct ProcessManager {
    command: String,
    args: Vec<String>,
}

impl ProcessManager {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub async fn spawn(&mut self) -> Result<()> {
        anyhow::bail!("ProcessManager::spawn not implemented yet")
    }

    pub async fn restart(&mut self) -> Result<()> {
        anyhow::bail!("ProcessManager::restart not implemented yet")
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        anyhow::bail!("ProcessManager::shutdown not implemented yet")
    }
}
