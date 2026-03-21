pub struct SystemCommand {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub keywords: &'static str,
}

pub static COMMANDS: &[SystemCommand] = &[
    SystemCommand {
        id: "lock-screen",
        title: "Lock Screen",
        subtitle: "Lock the session",
        keywords: "lock display session screen",
    },
    SystemCommand {
        id: "sleep",
        title: "Sleep",
        subtitle: "Suspend the machine",
        keywords: "suspend hibernate standby",
    },
    SystemCommand {
        id: "restart",
        title: "Restart",
        subtitle: "Reboot the machine",
        keywords: "reboot restart",
    },
    SystemCommand {
        id: "shutdown",
        title: "Shutdown",
        subtitle: "Power off the machine",
        keywords: "power off shutdown halt",
    },
    SystemCommand {
        id: "logout",
        title: "Log Out",
        subtitle: "End the session",
        keywords: "logout log out sign out session end",
    },
    SystemCommand {
        id: "empty-trash",
        title: "Empty Trash",
        subtitle: "Clear the trash",
        keywords: "trash recycle bin empty clear delete",
    },
];
