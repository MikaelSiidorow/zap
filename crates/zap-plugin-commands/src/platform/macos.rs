use std::process::Command;

pub struct MacOSPlatform;

impl super::Platform for MacOSPlatform {
    fn execute(&self, command_id: &str) -> anyhow::Result<()> {
        match command_id {
            "lock-screen" => {
                Command::new("pmset").arg("displaysleepnow").spawn()?;
            }
            "sleep" => {
                Command::new("pmset").arg("sleepnow").spawn()?;
            }
            "restart" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to restart"])
                    .spawn()?;
            }
            "shutdown" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to shut down"])
                    .spawn()?;
            }
            "logout" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to log out"])
                    .spawn()?;
            }
            "empty-trash" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"Finder\" to empty the trash"])
                    .spawn()?;
            }
            _ => anyhow::bail!("unknown command: {command_id}"),
        }
        Ok(())
    }
}
