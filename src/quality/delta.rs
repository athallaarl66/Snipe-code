use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::{Command, Stdio};

use super::{Baseline, Delta, State};
use crate::history::ScanEnvelope;

pub fn baseline_reference(base: &ScanEnvelope, base_ref: Option<&str>) -> Baseline {
    let identities: BTreeSet<_> = base
        .result
        .findings
        .iter()
        .map(|f| {
            (
                f.rule_id.clone(),
                f.rule_version
                    .clone()
                    .unwrap_or_else(|| base.result.rule_version.clone()),
                f.fingerprint_version,
            )
        })
        .collect();
    Baseline {
        run_id: base.result.run_id.clone(),
        project_id: base.result.project_id.clone(),
        schema_version: base.schema_version,
        rule_version: base.result.rule_version.clone(),
        identities: identities.into_iter().collect(),
        base_ref: base_ref.map(str::to_owned),
        base_commit: None,
    }
}
fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, &'static str> {
    let output = Command::new("git")
        .arg("--no-pager")
        .args(args)
        .current_dir(root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| "Git unavailable")?;
    if !output.status.success() || output.stdout.len() > 1_048_576 {
        return Err("base reference or Git state unavailable");
    }
    Ok(output.stdout)
}
fn compatible(base: &ScanEnvelope, current: &ScanEnvelope) -> bool {
    if base.schema_version != current.schema_version
        || base.result.project_id != current.result.project_id
        || base.result.rule_version != current.result.rule_version
        || base.result.exclusions != current.result.exclusions
        || base.result.evaluated_rule_groups != current.result.evaluated_rule_groups
        || base.result.detected_scopes != current.result.detected_scopes
    {
        return false;
    }
    let packs = |e: &ScanEnvelope| {
        e.result
            .sast
            .iter()
            .map(|s| (s.language.clone(), s.rule_pack_version.clone()))
            .collect::<BTreeSet<_>>()
    };
    if packs(base) != packs(current) {
        return false;
    }
    let mut versions = BTreeMap::new();
    for f in base.result.findings.iter().chain(&current.result.findings) {
        let v = f
            .rule_version
            .as_deref()
            .unwrap_or(&base.result.rule_version);
        if f.fingerprint_version != 1 || versions.insert(&f.rule_id, v).is_some_and(|old| old != v)
        {
            return false;
        }
    }
    true
}
fn complete(e: &ScanEnvelope) -> bool {
    e.result
        .evaluated_rule_groups
        .iter()
        .all(|g| matches!(g.status.as_str(), "evaluated" | "not-applicable"))
        && e.result.sast.iter().all(|r| {
            matches!(
                r.status,
                crate::sast::SastStatus::Evaluated | crate::sast::SastStatus::NotApplicable
            )
        })
}
pub fn compare(
    root: &Path,
    current: &ScanEnvelope,
    base: Option<&ScanEnvelope>,
    base_ref: Option<&str>,
) -> Delta {
    let mut delta = Delta {
        state: State::Unknown,
        reason: None,
        baseline: base.map(|b| baseline_reference(b, base_ref)),
        new: vec![],
        unchanged: vec![],
        resolved: vec![],
    };
    let checked = (|| {
        let base = base.ok_or("baseline missing or invalid")?;
        let reference = base_ref
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 256
                    && !s.starts_with('-')
                    && !s.contains(['\0', '\n', '\r'])
            })
            .ok_or("base reference missing or invalid")?;
        if !compatible(base, current) {
            return Err("baseline schema, project, rules or fingerprint versions incompatible");
        }
        if !complete(base) || !complete(current) {
            return Err("baseline or current evaluated scope incomplete");
        }
        let resolved = git(
            root,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{reference}^{{commit}}"),
            ],
        )?;
        let commit = std::str::from_utf8(&resolved)
            .map_err(|_| "invalid Git reference output")?
            .trim();
        if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid Git reference output");
        }
        delta.baseline.as_mut().unwrap().base_commit = Some(commit.to_string());
        if !git(root, &["ls-files", "--unmerged", "-z"])?.is_empty() {
            return Err("merge conflict makes baseline ambiguous");
        }
        let changes = git(
            root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--name-status",
                "-z",
                "--find-renames",
                commit,
                "--",
            ],
        )?;
        let fields: Vec<_> = changes.split(|&b| b == 0).collect();
        let mut i = 0;
        while i + 1 < fields.len() {
            if matches!(fields[i].first(), Some(b'R' | b'C' | b'U')) {
                return Err("rename or merge state ambiguous");
            }
            i += 2;
        }
        // Deleted + untracked paths can be a rename Git has not staged yet.
        if fields.iter().any(|s| *s == b"D")
            && !git(root, &["ls-files", "--others", "--exclude-standard", "-z"])?.is_empty()
        {
            return Err("possible unstaged rename ambiguous");
        }
        let fingerprints = |e: &ScanEnvelope| {
            e.result
                .findings
                .iter()
                .map(|f| f.fingerprint.clone())
                .collect::<BTreeSet<_>>()
        };
        let before = fingerprints(base);
        let after = fingerprints(current);
        delta.new = after.difference(&before).cloned().collect();
        delta.resolved = before.difference(&after).cloned().collect();
        delta.unchanged = after.intersection(&before).cloned().collect();
        Ok(())
    })();
    match checked {
        Ok(()) => delta.state = State::Pass,
        Err(reason) => delta.reason = Some(reason.into()),
    }
    delta
}
