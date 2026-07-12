//! Port for SSH-based remote machine management.
//!
//! Defines the [`SshConnector`] trait that allows inspecting and syncing
//! skills across a fleet of machines. Real I/O lives in `ai-skill-adapters`
//! (shells out to `ssh`); this crate provides the domain types and the
//! [`NoopSshConnector`] default.

/// A remote machine reachable via SSH.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteHost {
    /// User-facing label (e.g. "prod", "dev-box").
    pub label: String,
    /// SSH destination (`hostname`, `user@host`, or `Host` alias).
    pub host: String,
    /// Optional non-standard port.
    pub port: Option<u16>,
}

impl RemoteHost {
    pub fn new(label: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            host: host.into(),
            port: None,
        }
    }
}

/// A skill installed on a remote machine.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteSkill {
    pub name: String,
    pub path: String,
    pub managed: bool,
}

/// An SSH connection test result.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    /// Successfully connected and got a reply.
    Connected,
    /// Connection was refused by the remote host.
    Refused,
    /// The connection timed out.
    Timeout,
    /// The host is unreachable (DNS, network, or SSH binary missing).
    Unreachable,
}

/// Port for connecting to remote machines via SSH.
pub trait SshConnector {
    /// Test connectivity to a remote host.
    fn check_connection(&self, host: &RemoteHost) -> ConnectionStatus;
    /// List skills installed on a remote host.
    fn list_skills(&self, host: &RemoteHost) -> Result<Vec<RemoteSkill>, String>;
}

/// A no-op SSH connector that always reports [`ConnectionStatus::Unreachable`].
pub struct NoopSshConnector;

impl SshConnector for NoopSshConnector {
    fn check_connection(&self, _host: &RemoteHost) -> ConnectionStatus {
        ConnectionStatus::Unreachable
    }

    fn list_skills(&self, _host: &RemoteHost) -> Result<Vec<RemoteSkill>, String> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_host_construction() {
        let host = RemoteHost::new("dev", "dev.example.com");
        assert_eq!(host.label, "dev");
        assert_eq!(host.host, "dev.example.com");
        assert!(host.port.is_none());
    }

    #[test]
    fn remote_host_with_port() {
        let host = RemoteHost {
            label: "prod".into(),
            host: "prod.example.com".into(),
            port: Some(2222),
        };
        assert_eq!(host.port, Some(2222));
    }

    #[test]
    fn remote_skill_fields_accessible() {
        let skill = RemoteSkill {
            name: "my-skill".into(),
            path: "~/.claude/skills/my-skill".into(),
            managed: true,
        };
        assert_eq!(skill.name, "my-skill");
        assert!(skill.managed);
    }

    #[test]
    fn noop_connector_returns_unreachable() {
        let connector = NoopSshConnector;
        let host = RemoteHost::new("test", "test.local");
        assert_eq!(
            connector.check_connection(&host),
            ConnectionStatus::Unreachable
        );
    }

    #[test]
    fn noop_connector_returns_empty_skills() {
        let connector = NoopSshConnector;
        let host = RemoteHost::new("test", "test.local");
        let skills = connector.list_skills(&host).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn connection_status_variants_are_distinct() {
        assert_ne!(ConnectionStatus::Connected, ConnectionStatus::Refused);
        assert_ne!(ConnectionStatus::Refused, ConnectionStatus::Timeout);
        assert_ne!(ConnectionStatus::Timeout, ConnectionStatus::Unreachable);
    }

    #[test]
    fn ssh_connector_trait_is_object_safe() {
        fn take_box(_v: Box<dyn SshConnector>) {}
        take_box(Box::new(NoopSshConnector));
    }
}
