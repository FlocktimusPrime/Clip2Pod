// Opens the inbound TCP port the LAN feed server listens on, so a phone's
// podcatcher app can actually reach it. Elevates per-OS; always returns a
// message rather than panicking — a failed/cancelled elevation must not
// crash the app.

use crate::feed::FEED_PORT;

pub fn open_port() -> Result<String, String> {
    imp::open_port()
}

#[cfg(target_os = "windows")]
mod imp {
    use super::FEED_PORT;
    use std::process::Command;

    pub fn open_port() -> Result<String, String> {
        let rule = format!(
            "advfirewall firewall add rule name=\"Clip2Pod Feed\" dir=in action=allow protocol=TCP localport={FEED_PORT}"
        );
        let ps_cmd = format!(
            "Start-Process netsh -ArgumentList '{rule}' -Verb RunAs -Wait -PassThru | Select-Object -ExpandProperty ExitCode"
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .map_err(|e| format!("could not launch elevation prompt: {e}"))?;
        let code = String::from_utf8_lossy(&output.stdout);
        if code.trim() == "0" {
            Ok(format!("Firewall rule added for port {FEED_PORT}."))
        } else {
            Err("firewall rule not added (elevation declined or netsh failed)".into())
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::FEED_PORT;
    use std::process::Command;

    fn on_path(bin: &str) -> bool {
        Command::new("which").arg(bin).output().is_ok_and(|o| o.status.success())
    }

    pub fn open_port() -> Result<String, String> {
        let script = if on_path("ufw") {
            format!("ufw allow {FEED_PORT}/tcp")
        } else if on_path("firewall-cmd") {
            format!(
                "firewall-cmd --permanent --add-port={FEED_PORT}/tcp && firewall-cmd --reload"
            )
        } else {
            return Err("no supported firewall tool found (ufw or firewall-cmd); open the port manually".into());
        };
        let status = Command::new("pkexec")
            .args(["sh", "-c", &script])
            .status()
            .map_err(|e| format!("could not launch elevation prompt: {e}"))?;
        if status.success() {
            Ok(format!("Firewall rule added for port {FEED_PORT}."))
        } else {
            Err("firewall rule not added (elevation declined or command failed)".into())
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::FEED_PORT;
    use std::process::Command;

    pub fn open_port() -> Result<String, String> {
        let exe = std::env::current_exe().map_err(|e| format!("could not locate app binary: {e}"))?;
        let exe = exe.to_string_lossy().replace('"', "\\\"");
        let inner = format!(
            "/usr/libexec/ApplicationFirewall/socketfilterfw --add \\\"{exe}\\\" && /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp \\\"{exe}\\\""
        );
        let script = format!("do shell script \"{inner}\" with administrator privileges");
        let status = Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map_err(|e| format!("could not launch elevation prompt: {e}"))?;
        if status.success() {
            Ok(format!("Clip2Pod allowed through the Application Firewall (port {FEED_PORT})."))
        } else {
            Err("firewall change not applied (elevation declined or command failed)".into())
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod imp {
    pub fn open_port() -> Result<String, String> {
        Err("automatic firewall configuration is not supported on this platform".into())
    }
}
