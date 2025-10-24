use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{Duration, timeout};

pub struct ProcessManager {
    command: String,
    args: Vec<String>,
    child: Option<Child>,
}

impl ProcessManager {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            command,
            args,
            child: None,
        }
    }

    pub async fn spawn(&mut self) -> Result<()> {
        if self.child.is_some() {
            anyhow::bail!("Process already running");
        }

        println!("[supi] Starting process: {} {:?}", self.command, self.args);

        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn child process")?;

        // Get stdout/stderr handles
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture child stdout")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture child stderr")?;

        // Spawn tasks to forward output
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("{}", line);
            }
        });

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("{}", line);
            }
        });

        self.child = Some(child);
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        if let Some(child) = &mut self.child {
            let status = child.wait().await.context("Failed to wait on child")?;
            self.child = None;
            Ok(status)
        } else {
            anyhow::bail!("No process running")
        }
    }

    pub async fn restart(&mut self) -> Result<()> {
        println!("[supi] Restarting process...");
        self.shutdown().await?;
        self.spawn().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            println!("[supi] Stopping process gracefully...");

            // Try graceful shutdown with SIGTERM first
            #[cfg(unix)]
            {
                use nix::sys::signal::{Signal, kill};
                use nix::unistd::Pid;

                if let Some(pid) = child.id() {
                    // Send SIGTERM for graceful shutdown
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);

                    // Wait up to 5 seconds for graceful exit
                    match timeout(Duration::from_secs(5), child.wait()).await {
                        Ok(Ok(_status)) => {
                            println!("[supi] Process stopped gracefully");
                            return Ok(());
                        }
                        Ok(Err(e)) => {
                            eprintln!("[supi] Error waiting for process: {}", e);
                        }
                        Err(_) => {
                            println!("[supi] Process didn't stop gracefully, forcing...");
                        }
                    }
                }
            }

            // Force kill if graceful shutdown failed or on non-Unix platforms
            child.kill().await.context("Failed to kill child process")?;
            let _ = child.wait().await;
            println!("[supi] Process stopped");
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}
