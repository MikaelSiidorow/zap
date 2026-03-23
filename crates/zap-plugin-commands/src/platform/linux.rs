use std::process::Command;

pub struct LinuxPlatform;

impl super::Platform for LinuxPlatform {
    fn execute(&self, command_id: &str) -> anyhow::Result<()> {
        match command_id {
            "lock-screen" => {
                Command::new("loginctl").arg("lock-session").spawn()?;
            }
            "sleep" => {
                Command::new("systemctl").arg("suspend").spawn()?;
            }
            "restart" => {
                Command::new("systemctl").arg("reboot").spawn()?;
            }
            "shutdown" => {
                Command::new("systemctl").arg("poweroff").spawn()?;
            }
            "logout" => {
                let session_id = std::env::var("XDG_SESSION_ID").unwrap_or_default();
                Command::new("loginctl")
                    .args(["terminate-session", &session_id])
                    .spawn()?;
            }
            "empty-trash" => {
                Command::new("gio").args(["trash", "--empty"]).spawn()?;
            }
            _ => anyhow::bail!("unknown command: {command_id}"),
        }
        Ok(())
    }
}
