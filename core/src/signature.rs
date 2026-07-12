//! Port for ed25519 signature verification of installed skills.
//!
//! The ecosystem does not yet support signing skills, so the default adapter
//! ([`NoopSignatureVerifier`]) returns [`VerificationStatus::NotSupported`].
//! When signing is adopted, implement [`SignatureVerifier`] against a public-key
//! infrastructure (OWASP AST01).

use std::path::Path;

/// The result of verifying a skill's signature.
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    /// The skill is not signed (no signature file found).
    Unsigned,
    /// The signature is valid and matches the skill content.
    Valid,
    /// The signature exists but is invalid or content has been tampered with.
    Invalid { reason: String },
    /// Signature verification is not supported in this environment.
    NotSupported,
}

/// Port for verifying the ed25519 signature of an installed skill.
pub trait SignatureVerifier {
    /// Verifies the signature of the skill at the given directory.
    fn verify(&self, skill_dir: &Path) -> VerificationStatus;
}

/// A no-op signature verifier that always reports [`VerificationStatus::NotSupported`].
///
/// Used when no signing infrastructure is available.
pub struct NoopSignatureVerifier;

impl SignatureVerifier for NoopSignatureVerifier {
    fn verify(&self, _skill_dir: &Path) -> VerificationStatus {
        VerificationStatus::NotSupported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn noop_verifier_returns_not_supported() {
        let verifier = NoopSignatureVerifier;
        assert_eq!(
            verifier.verify(Path::new("/tmp/fake-skill")),
            VerificationStatus::NotSupported
        );
    }

    #[test]
    fn verification_status_debug() {
        let status = VerificationStatus::Invalid {
            reason: "signature mismatch".into(),
        };
        let debug = format!("{status:?}");
        assert!(debug.contains("Invalid"));
        assert!(debug.contains("signature mismatch"));
    }

    #[test]
    fn unsigned_vs_not_supported_are_distinct() {
        assert_ne!(
            VerificationStatus::Unsigned,
            VerificationStatus::NotSupported
        );
    }

    #[test]
    fn trait_object_works() {
        let verifier: Box<dyn SignatureVerifier> = Box::new(NoopSignatureVerifier);
        assert_eq!(
            verifier.verify(&PathBuf::from("/tmp/test")),
            VerificationStatus::NotSupported
        );
    }

    #[test]
    fn signature_verifier_trait_is_object_safe() {
        fn take_box(_v: Box<dyn SignatureVerifier>) {}
        take_box(Box::new(NoopSignatureVerifier));
    }
}
