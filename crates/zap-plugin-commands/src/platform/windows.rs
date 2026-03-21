use std::process::Command;

pub struct WindowsPlatform;

impl super::Platform for WindowsPlatform {
    fn execute(&self, command_id: &str) -> anyhow::Result<()> {
        match command_id {
            "lock-screen" => {
                Command::new("rundll32.exe")
                    .args(["user32.dll,LockWorkStation"])
                    .spawn()?;
            }
            "sleep" => {
                Command::new("rundll32.exe")
                    .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                    .spawn()?;
            }
            "restart" => {
                Command::new("shutdown").args(["/r", "/t", "0"]).spawn()?;
            }
            "shutdown" => {
                Command::new("shutdown").args(["/s", "/t", "0"]).spawn()?;
            }
            "logout" => {
                Command::new("shutdown").arg("/l").spawn()?;
            }
            "empty-trash" => {
                Command::new("powershell")
                    .args(["-Command", "Clear-RecycleBin -Force"])
                    .spawn()?;
            }
            _ => anyhow::bail!("unknown command: {command_id}"),
        }
        Ok(())
    }
}
