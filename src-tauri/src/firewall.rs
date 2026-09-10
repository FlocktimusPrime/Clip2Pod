// The LAN feed server needs an inbound-TCP firewall exception so a phone's
// podcatcher can reach it. Earlier versions tried to add the rule automatically
// by elevating (netsh via RunAs, pkexec, osascript) — on Windows that silently
// fails even when the app is already running as admin. So we don't touch the
// firewall at all now: hand the user the exact command to paste, plus the
// non-obvious fixes that a firewall rule alone won't solve.

use crate::feed::FEED_PORT;
use serde::Serialize;

#[derive(Serialize)]
pub struct FirewallHelp {
    /// Where to run `command` (e.g. an elevated shell).
    pub shell_hint: String,
    /// One copy-paste line that adds the inbound exception.
    pub command: String,
    /// Common reasons a phone still can't reach the feed after the rule is added.
    pub tips: Vec<String>,
}

const AP_ISOLATION_TIP: &str = "Some routers isolate wireless clients from each \
other (\"AP isolation\", or a separate guest network). Put the phone and this \
computer on the same normal Wi-Fi network.";

pub fn firewall_help() -> FirewallHelp {
    imp::firewall_help()
}

#[cfg(target_os = "windows")]
mod imp {
    use super::{FirewallHelp, AP_ISOLATION_TIP, FEED_PORT};

    pub fn firewall_help() -> FirewallHelp {
        FirewallHelp {
            shell_hint: "Run in an Administrator PowerShell (right-click the Start \
button → \"Terminal (Admin)\")."
                .into(),
            command: format!(
                "netsh advfirewall firewall add rule name=\"Clip2Pod Feed\" \
dir=in action=allow protocol=TCP localport={FEED_PORT}"
            ),
            tips: vec![
                "If it still doesn't work, your Wi-Fi network is set to \"Public\". \
Open Settings → Network & Internet → Wi-Fi → (your network) and set \"Network \
profile type\" to Private."
                    .into(),
                AP_ISOLATION_TIP.into(),
            ],
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::{FirewallHelp, AP_ISOLATION_TIP, FEED_PORT};
    use std::process::Command;

    fn on_path(bin: &str) -> bool {
        Command::new("which")
            .arg(bin)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    pub fn firewall_help() -> FirewallHelp {
        let ufw = format!("sudo ufw allow {FEED_PORT}/tcp");
        let firewalld = format!(
            "sudo firewall-cmd --permanent --add-port={FEED_PORT}/tcp && \
sudo firewall-cmd --reload"
        );
        let command = if on_path("ufw") {
            ufw
        } else if on_path("firewall-cmd") {
            firewalld.clone()
        } else {
            // Neither tool detected — show both and let the user pick.
            format!("# ufw:\n{ufw}\n# firewalld:\n{firewalld}")
        };
        FirewallHelp {
            shell_hint: "Run in a terminal.".into(),
            command,
            tips: vec![
                format!("Using firewalld instead of ufw? Run: {firewalld}"),
                AP_ISOLATION_TIP.into(),
            ],
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::{FirewallHelp, AP_ISOLATION_TIP, FEED_PORT};

    pub fn firewall_help() -> FirewallHelp {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/Applications/Clip2Pod.app/Contents/MacOS/clip2pod2".into());
        let fw = "/usr/libexec/ApplicationFirewall/socketfilterfw";
        FirewallHelp {
            shell_hint: "Run in Terminal (you'll be asked for your password).".into(),
            command: format!(
                "sudo {fw} --add \"{exe}\" && sudo {fw} --unblockapp \"{exe}\""
            ),
            tips: vec![
                format!(
                    "macOS's firewall is usually off by default (System Settings → \
Network → Firewall). If it's off, nothing is blocking port {FEED_PORT} locally — \
the problem is on the network."
                ),
                AP_ISOLATION_TIP.into(),
            ],
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod imp {
    use super::{FirewallHelp, AP_ISOLATION_TIP, FEED_PORT};

    pub fn firewall_help() -> FirewallHelp {
        FirewallHelp {
            shell_hint: "Allow inbound TCP on this port in your firewall.".into(),
            command: format!("# allow inbound TCP {FEED_PORT}"),
            tips: vec![AP_ISOLATION_TIP.into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_names_the_port() {
        let help = firewall_help();
        assert!(help.command.contains(&FEED_PORT.to_string()));
        assert!(!help.shell_hint.is_empty());
        assert!(!help.tips.is_empty());
    }
}
