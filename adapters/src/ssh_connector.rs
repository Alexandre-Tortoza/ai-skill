//! Adapter that connects to remote machines via the `ssh` command.
//!
//! Shells out to the `ssh` binary — the same tool users already manage via
//! `~/.ssh/config`, key-based auth, etc. No extra dependencies needed.

use ai_skill_core::{ConnectionStatus, RemoteHost, RemoteSkill, SshConnector};
use std::process::Command;

/// SSH connector that shells out to the local `ssh` binary.
pub struct SshCommandConnector;

impl SshCommandConnector {
    fn build_command(&self, host: &RemoteHost, remote_cmd: &str) -> Command {
        let mut cmd = Command::new("ssh");
        cmd.arg(&host.host);
        if let Some(port) = host.port {
            cmd.arg("-p").arg(port.to_string());
        }
        cmd.arg(remote_cmd);
        cmd
    }
}

impl SshConnector for SshCommandConnector {
    fn check_connection(&self, host: &RemoteHost) -> ConnectionStatus {
        let output = self.build_command(host, "echo connected").output();
        match output {
            Ok(out) if out.status.success() => ConnectionStatus::Connected,
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("Connection refused") {
                    ConnectionStatus::Refused
                } else if stderr.contains("timed out") || stderr.contains("Operation timed out") {
                    ConnectionStatus::Timeout
                } else {
                    ConnectionStatus::Unreachable
                }
            }
            Err(_) => ConnectionStatus::Unreachable,
        }
    }

    fn list_skills(&self, host: &RemoteHost) -> Result<Vec<RemoteSkill>, String> {
        let output = self
            .build_command(host, "ls -1 ~/.claude/skills/ 2>/dev/null || echo ''")
            .output()
            .map_err(|e| format!("ssh failed: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ssh command failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let skills: Vec<RemoteSkill> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| {
                let entry = line.trim().to_string();
                let is_disabled = entry.ends_with(".disabled");
                let name = if is_disabled {
                    entry
                        .strip_suffix(".disabled")
                        .unwrap_or(&entry)
                        .to_string()
                } else {
                    entry.clone()
                };
                RemoteSkill {
                    name,
                    path: format!("~/.claude/skills/{entry}"),
                    managed: !is_disabled,
                }
            })
            .collect();
        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_connector_trait_object_works() {
        let connector: Box<dyn SshConnector> = Box::new(SshCommandConnector);
        let host = RemoteHost::new("local", "127.0.0.1");
        // Should not panic; on systems without sshd this returns Unreachable
        let status = connector.check_connection(&host);
        assert!(matches!(
            status,
            ConnectionStatus::Unreachable | ConnectionStatus::Refused | ConnectionStatus::Timeout
        ));
    }
}
