//! Review-only run manifests. This module never starts a process or accesses a target.
//!
//! Callers must supply redacted command arguments and targets; environment entries are
//! names only. A manifest is a description, not proof that any boundary is enforced.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{self, IsTerminal, Write};

/// Caller-supplied metadata for fake policy tests, not effective DB discovery.
/// Never read credentials, environment values or target files to populate this seam.
pub struct DbSettings<'a> {
    pub host: &'a str,
    pub name: &'a str,
    pub environment: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DbPolicy {
    Unknown,
    LocalUnverified,
    Remote,
    ProductionLike,
    Ambiguous,
}

impl DbPolicy {
    /// Fixed, credential-free text suitable for the manifest target and result.
    pub fn disclosure(self) -> &'static str {
        match self {
            Self::Unknown => {
                "DB target unknown; explicit approval required; absence of configuration does not prove safety."
            }
            Self::LocalUnverified => {
                "Local DB metadata only; explicit approval required; localhost and a test name are not proof of isolation."
            }
            Self::Remote => {
                "Explicit remote DB target blocked; approval cannot override DB policy."
            }
            Self::ProductionLike => {
                "Production-like DB target blocked; approval cannot override DB policy."
            }
            Self::Ambiguous => {
                "Explicit DB configuration ambiguous or unparseable; blocked without override."
            }
        }
    }
}

/// Conservative classification of supplied host/name/environment, not a URL parser
/// or preflight. Even LocalUnverified makes no claim about ownership or isolation.
pub fn classify_db(settings: Option<DbSettings<'_>>) -> DbPolicy {
    let Some(settings) = settings else {
        return DbPolicy::Unknown;
    };
    if [settings.host, settings.name, settings.environment]
        .iter()
        .any(|value| {
            let value = value.to_ascii_lowercase();
            ["prod", "live", "main"]
                .iter()
                .any(|marker| value.contains(marker))
        })
    {
        return DbPolicy::ProductionLike;
    }
    if settings.host.is_empty()
        || settings.name.is_empty()
        || !settings
            .host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-:".contains(c))
        || !settings
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
        || !settings
            .environment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
    {
        return DbPolicy::Ambiguous;
    }
    let host = settings.host.to_ascii_lowercase();
    if ["localhost", "127.0.0.1", "::1"].contains(&host.as_str()) {
        DbPolicy::LocalUnverified
    } else if host.contains(':') && host.parse::<std::net::IpAddr>().is_err() {
        DbPolicy::Ambiguous
    } else {
        DbPolicy::Remote
    }
}

/// Inputs are owned by the manifest after construction; changing them requires a new manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestScope {
    pub command: Vec<String>,
    pub cwd: String,
    /// Redacted endpoint and database identity, or an explicit unknown disclosure.
    pub target: String,
    /// Environment variable names, never values.
    pub env_names: Vec<String>,
    pub mounts: Vec<Mount>,
    pub filesystem: String,
    pub process: String,
    pub network: String,
    pub output: String,
    pub timeout_seconds: u64,
    /// Explicitly list install effects; use an empty list when there are none.
    pub install_effects: Vec<String>,
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub access: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    Host,
    SandboxRequested,
}

/// Immutable snapshot. No deserialization or mutable access to the scope is provided.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunManifest {
    version: u8,
    scope: ManifestScope,
    execution_disclosure: &'static str,
}

impl RunManifest {
    pub fn new(scope: ManifestScope) -> Result<Self, &'static str> {
        if scope.command.is_empty() || scope.command[0].trim().is_empty() {
            return Err("command is required");
        }
        if [
            &scope.cwd,
            &scope.target,
            &scope.filesystem,
            &scope.process,
            &scope.network,
            &scope.output,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
            || scope.timeout_seconds == 0
        {
            return Err("scope and timeout must be explicit");
        }
        if scope.env_names.iter().any(|name| {
            name.is_empty()
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || name.as_bytes()[0].is_ascii_digit()
        }) {
            return Err("environment entries must be variable names, not values");
        }
        if scope.mounts.iter().any(|mount| {
            mount.source.trim().is_empty()
                || mount.destination.trim().is_empty()
                || mount.access.trim().is_empty()
        }) {
            return Err("mount scope must be explicit");
        }
        let execution_disclosure = match scope.execution_mode {
            ExecutionMode::Host => {
                "Host execution is not sandboxed: filesystem, processes, network, endpoints, secrets and hardcoded database targets are not isolated by a temporary workspace."
            }
            ExecutionMode::SandboxRequested => {
                "Sandbox requested; no isolation boundary is established or verified by this manifest."
            }
        };
        Ok(Self {
            version: 1,
            scope,
            execution_disclosure,
        })
    }

    pub fn scope(&self) -> &ManifestScope {
        &self.scope
    }

    /// Fixed struct field order and ordered lists make this serialization deterministic.
    pub fn preview(&self) -> String {
        serde_json::to_string_pretty(self)
            .expect("manifest contains only serializable strings and integers")
    }

    pub fn hash(&self) -> String {
        let bytes = serde_json::to_vec(self)
            .expect("manifest contains only serializable strings and integers");
        format!("{:x}", Sha256::digest(bytes))
    }
}

/// In-memory, non-cloneable, non-serializable approval. A check consumes it even on mismatch.
#[derive(Debug)]
pub struct Approval {
    manifest_hash: String,
}

impl Approval {
    pub fn consume(self, manifest: &RunManifest) -> bool {
        self.manifest_hash == manifest.hash()
    }
}

/// Preview and obtain a single approval from an interactive terminal. No agent or
/// non-interactive caller can supply a precomputed answer through this API.
/// Terminal presence alone does not prove who typed the response; integration must
/// ensure the terminal is controlled by the user, not an agent or automated PTY.
pub fn request_approval(manifest: &RunManifest) -> io::Result<Option<Approval>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    if !stdin.is_terminal() || !stdout.is_terminal() {
        return Ok(None);
    }
    let hash = manifest.hash();
    writeln!(stdout, "{}\nSHA-256: {}", manifest.preview(), hash)?;
    write!(stdout, "Type 'approve {}' to approve this one run: ", hash)?;
    stdout.flush()?;
    let mut answer = String::new();
    if stdin.read_line(&mut answer)? == 0 {
        return Ok(None);
    }
    Ok(
        (answer.trim_end_matches(['\r', '\n']) == format!("approve {hash}")).then_some(Approval {
            manifest_hash: hash,
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Passed,
    Failed,
    Unavailable,
    Timeout,
    Cancelled,
    Skipped,
    Blocked,
}

/// In-memory fake observations. Intentionally not Debug/Serialize: raw logs and
/// artifact paths must pass through redaction before display or persistence.
pub struct FakeOutcome {
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub artifacts: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FakeRunResult {
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub approval_matched: bool,
    pub db_policy: DbPolicy,
    pub db_disclosure: &'static str,
    pub execution_disclosure: &'static str,
    pub reason: &'static str,
    pub stdout: &'static str,
    pub stderr: &'static str,
    pub artifacts: Vec<&'static str>,
}

/// Fake-only policy seam. Callback must not perform real execution or I/O.
/// Does not inspect effective configuration, scrub child environment, enforce a
/// timeout, create artifacts or establish a sandbox. Manifest strings are only
/// descriptive and caller-redacted; approval cannot authorize real host execution
/// through this API. A future host runner needs its own verified execution binding.
pub fn run_fake(
    manifest: &RunManifest,
    db_policy: DbPolicy,
    approval: Option<Approval>,
    fake: impl FnOnce(&RunManifest) -> FakeOutcome,
) -> FakeRunResult {
    let approval_matched = approval.is_some_and(|approval| approval.consume(manifest));
    let mut result = FakeRunResult {
        status: RunStatus::Blocked,
        exit_code: None,
        approval_matched,
        db_policy,
        db_disclosure: db_policy.disclosure(),
        execution_disclosure: manifest.execution_disclosure,
        reason: "",
        stdout: "",
        stderr: "",
        artifacts: vec![],
    };
    result.reason = if manifest.scope.execution_mode == ExecutionMode::SandboxRequested {
        "Requested sandbox boundary is unavailable; no host fallback."
    } else if matches!(
        db_policy,
        DbPolicy::Remote | DbPolicy::ProductionLike | DbPolicy::Ambiguous
    ) {
        db_policy.disclosure()
    } else if manifest.scope.target != db_policy.disclosure() {
        "DB policy disclosure does not match reviewed manifest target."
    } else if !approval_matched {
        "Missing or stale single-run approval."
    } else {
        ""
    };
    if !result.reason.is_empty() {
        return result;
    }
    let outcome = fake(manifest);
    result.status = match (outcome.status, outcome.exit_code) {
        (RunStatus::Passed, Some(0)) => RunStatus::Passed,
        (RunStatus::Passed, Some(_)) => RunStatus::Failed,
        (RunStatus::Passed, None) => RunStatus::Unavailable,
        (status, _) => status,
    };
    result.exit_code = outcome.exit_code;
    result.reason = "Fake callback observation only; no host execution or isolation verified.";
    // ponytail: omit all untrusted text, preserving artifact count only; add a
    // reviewed, context-aware scrubber before retaining useful logs or paths.
    result.stdout = if outcome.stdout.is_empty() {
        ""
    } else {
        "[REDACTED]"
    };
    result.stderr = if outcome.stderr.is_empty() {
        ""
    } else {
        "[REDACTED]"
    };
    result.artifacts = vec!["[REDACTED]"; outcome.artifacts.len()];
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> ManifestScope {
        ManifestScope {
            command: vec!["tool".into(), "test".into()],
            cwd: "/project".into(),
            target: "DB target unknown; endpoint redacted".into(),
            env_names: vec!["PATH".into()],
            mounts: vec![Mount {
                source: "/src".into(),
                destination: "/work".into(),
                access: "read-only".into(),
            }],
            filesystem: "host access".into(),
            process: "host processes".into(),
            network: "host network".into(),
            output: "/output".into(),
            timeout_seconds: 30,
            install_effects: vec!["none".into()],
            execution_mode: ExecutionMode::Host,
        }
    }

    fn fake_manifest(policy: DbPolicy) -> RunManifest {
        let mut input = scope();
        input.target = policy.disclosure().into();
        RunManifest::new(input).unwrap()
    }

    fn approved(manifest: &RunManifest) -> Option<Approval> {
        Some(Approval {
            manifest_hash: manifest.hash(),
        })
    }

    fn passed() -> FakeOutcome {
        FakeOutcome {
            status: RunStatus::Passed,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            artifacts: vec![],
        }
    }

    #[test]
    fn fake_policy_blocks_before_callback_even_with_approval() {
        for policy in [
            DbPolicy::Remote,
            DbPolicy::ProductionLike,
            DbPolicy::Ambiguous,
        ] {
            let manifest = fake_manifest(policy);
            let result = run_fake(&manifest, policy, approved(&manifest), |_| {
                panic!("must not run")
            });
            assert_eq!(result.status, RunStatus::Blocked);
            assert!(result.approval_matched);
        }
        let manifest = fake_manifest(DbPolicy::Unknown);
        let result = run_fake(&manifest, DbPolicy::Unknown, None, |_| {
            panic!("missing approval")
        });
        assert_eq!(result.status, RunStatus::Blocked);
        assert!(!result.approval_matched);
        let mut changed = manifest.scope().clone();
        changed.command.push("changed".into());
        let changed = RunManifest::new(changed).unwrap();
        let result = run_fake(&changed, DbPolicy::Unknown, approved(&manifest), |_| {
            panic!("stale approval")
        });
        assert_eq!(result.status, RunStatus::Blocked);
        assert!(!result.approval_matched);
        let mut sandbox = manifest.scope().clone();
        sandbox.execution_mode = ExecutionMode::SandboxRequested;
        let sandbox = RunManifest::new(sandbox).unwrap();
        let result = run_fake(&sandbox, DbPolicy::Unknown, approved(&sandbox), |_| {
            panic!("no sandbox")
        });
        assert_eq!(result.status, RunStatus::Blocked);
        let result = run_fake(
            &manifest,
            DbPolicy::LocalUnverified,
            approved(&manifest),
            |_| panic!("policy changed"),
        );
        assert_eq!(result.status, RunStatus::Blocked);
    }

    #[test]
    fn approved_fake_callback_records_actual_outcome_and_redacts_all_untrusted_text() {
        for policy in [DbPolicy::Unknown, DbPolicy::LocalUnverified] {
            let manifest = fake_manifest(policy);
            let calls = std::cell::Cell::new(0);
            let result = run_fake(&manifest, policy, approved(&manifest), |reviewed| {
                calls.set(calls.get() + 1);
                assert_eq!(reviewed, &manifest);
                passed()
            });
            assert_eq!(calls.get(), 1);
            assert_eq!(result.status, RunStatus::Passed);
            assert_eq!(result.exit_code, Some(0));
            assert!(result.approval_matched);
            assert_eq!(result.db_disclosure, policy.disclosure());
        }
        for (status, exit_code, expected_name) in [
            (RunStatus::Passed, Some(0), "passed"),
            (RunStatus::Failed, Some(7), "failed"),
            (RunStatus::Unavailable, None, "unavailable"),
            (RunStatus::Timeout, None, "timeout"),
            (RunStatus::Cancelled, None, "cancelled"),
            (RunStatus::Skipped, None, "skipped"),
            (RunStatus::Blocked, None, "blocked"),
        ] {
            let manifest = fake_manifest(DbPolicy::Unknown);
            let result = run_fake(&manifest, DbPolicy::Unknown, approved(&manifest), |_| {
                FakeOutcome {
                    status,
                    exit_code,
                    stdout: "STDOUT_SECRET_CANARY".into(),
                    stderr: "STDERR_SECRET_CANARY".into(),
                    artifacts: vec!["/results/ARTIFACT_SECRET_CANARY.txt".into()],
                }
            });
            assert_eq!(result.status, status);
            assert_eq!(result.exit_code, exit_code);
            assert_eq!(result.artifacts, ["[REDACTED]"]);
            let json = serde_json::to_value(&result).unwrap();
            assert_eq!(json["status"], expected_name);
            let serialized = serde_json::to_string(&result).unwrap();
            let debug = format!("{result:?}");
            for text in [&serialized, &debug] {
                assert!(!text.contains("SECRET_CANARY"));
                assert!(text.contains("[REDACTED]"));
            }
        }
    }

    #[test]
    fn fake_pass_requires_success_exit_status() {
        let manifest = fake_manifest(DbPolicy::Unknown);
        for (exit_code, expected) in [(Some(9), RunStatus::Failed), (None, RunStatus::Unavailable)]
        {
            let result = run_fake(&manifest, DbPolicy::Unknown, approved(&manifest), |_| {
                FakeOutcome {
                    exit_code,
                    ..passed()
                }
            });
            assert_eq!(result.status, expected);
        }
    }

    #[test]
    fn every_scope_change_invalidates_approval() {
        let original = RunManifest::new(scope()).unwrap();
        let changes: [fn(&mut ManifestScope); 14] = [
            |s| s.command.push("changed".into()),
            |s| s.cwd.push('x'),
            |s| s.target.push('x'),
            |s| s.env_names.push("OTHER".into()),
            |s| s.mounts[0].source.push('x'),
            |s| s.mounts[0].destination.push('x'),
            |s| s.mounts[0].access.push('x'),
            |s| s.filesystem.push('x'),
            |s| s.process.push('x'),
            |s| s.network.push('x'),
            |s| s.output.push('x'),
            |s| s.timeout_seconds += 1,
            |s| s.install_effects.push("install".into()),
            |s| s.execution_mode = ExecutionMode::SandboxRequested,
        ];
        for change in changes {
            let mut changed = scope();
            change(&mut changed);
            let changed = RunManifest::new(changed).unwrap();
            assert_ne!(original.hash(), changed.hash());
            assert!(
                !Approval {
                    manifest_hash: original.hash()
                }
                .consume(&changed)
            );
        }
        assert!(
            Approval {
                manifest_hash: original.hash()
            }
            .consume(&original)
        );
    }
}
