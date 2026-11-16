use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;
pub const MAX_INPUT: u64 = 1_048_576;

mod coverage;
mod delta;
mod source;
pub use coverage::coverage_metric;
pub use coverage::read_input;
pub use delta::compare;

pub fn evaluate(
    root: &Path,
    config: &Config,
    coverage: Option<&Path>,
    current: &crate::history::ScanEnvelope,
    baseline: Option<&crate::history::ScanEnvelope>,
    base_ref: Option<&str>,
) -> Result<Report, String> {
    config.validate()?;
    let delta = if config.enabled && config.metrics.iter().any(|p| p.id == "new-findings") {
        Some(compare(root, current, baseline, base_ref))
    } else {
        None
    };
    let mut metrics = vec![];
    for p in &config.metrics {
        let metric = if !config.enabled {
            Metric {
                policy: p.clone(),
                value: None,
                state: State::NotApplicable,
                reason: None,
                files: vec![],
                coverage: None,
            }
        } else {
            match p.id.as_str() {
                "lcov-line-coverage" => coverage_metric(root, p, coverage)?,
                "new-findings" => {
                    let d = delta.as_ref().unwrap();
                    let input = if d.state == State::Pass {
                        let included = current
                            .result
                            .findings
                            .iter()
                            .filter(|f| !excluded(&f.file, &p.exclusions));
                        let count =
                            included.filter(|f| d.new.contains(&f.fingerprint)).count() as u64;
                        let files = current
                            .result
                            .findings
                            .iter()
                            .filter(|f| !excluded(&f.file, &p.exclusions))
                            .map(|f| f.file.clone())
                            .collect::<BTreeSet<_>>()
                            .into_iter()
                            .collect();
                        Ok((count, files))
                    } else {
                        Err("baseline comparison unavailable")
                    };
                    source::outcome(p, input)
                }
                _ => source::rust(root, p),
            }
        };
        metrics.push(metric);
    }
    let state = aggregate(&metrics);
    Ok(Report {
        version: VERSION,
        config: config.clone(),
        state,
        metrics,
        delta,
    })
}

fn aggregate(metrics: &[Metric]) -> State {
    if metrics.iter().any(|m| m.state == State::Fail) {
        State::Fail
    } else if metrics.iter().any(|m| m.state == State::Unknown) {
        State::Unknown
    } else if metrics.iter().any(|m| m.state == State::Pass) {
        State::Pass
    } else {
        State::NotApplicable
    }
}
pub fn exit_status(report: &Report) -> i32 {
    match report.state {
        State::Pass | State::NotApplicable => 0,
        State::Fail => 1,
        State::Unknown if report.metrics.iter().any(|m| m.value.is_some()) => 4,
        State::Unknown => 3,
    }
}

pub fn validate_report(report: &Report) -> Result<(), String> {
    report.config.validate()?;
    let bad = || "invalid quality report".to_string();
    if report.version != VERSION
        || report.metrics.len() != report.config.metrics.len()
        || report.state != aggregate(&report.metrics)
    {
        return Err(bad());
    }
    for (p, m) in report.config.metrics.iter().zip(&report.metrics) {
        if &m.policy != p
            || m.files
                .iter()
                .any(|f| !relative_path(f) || excluded(f, &p.exclusions))
            || m.files.len() > 4096
            || m.files.iter().collect::<BTreeSet<_>>().len() != m.files.len()
            || m.reason
                .as_ref()
                .is_some_and(|s| s.len() > 128 || s.chars().any(char::is_control))
        {
            return Err(bad());
        }
        let expected = if !report.config.enabled {
            State::NotApplicable
        } else {
            source::outcome(p, m.value.map(|v| (v, vec![])).ok_or("missing")).state
        };
        if m.state != expected
            || (!report.config.enabled
                && (m.value.is_some() || m.coverage.is_some() || m.reason.is_some()))
            || (m.value.is_none() && report.config.enabled && m.reason.is_none())
        {
            return Err(bad());
        }
        if let Some(c) = &m.coverage {
            if p.id != "lcov-line-coverage"
                || c.format != "lcov-v1-line-records"
                || c.total_lines == 0
                || c.total_lines > 100_000
                || c.covered_lines > c.total_lines
                || c.payload_sha256.len() != 64
                || !c.payload_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || m.value != Some(c.covered_lines * 10_000 / c.total_lines)
            {
                return Err(bad());
            }
        } else if p.id == "lcov-line-coverage" && m.value.is_some() {
            return Err(bad());
        }
    }
    if let Some(d) = &report.delta {
        if !matches!(d.state, State::Pass | State::Unknown)
            || (d.state == State::Unknown
                && (!d.new.is_empty() || !d.resolved.is_empty() || !d.unchanged.is_empty()))
            || (d.state == State::Pass
                && d.baseline
                    .as_ref()
                    .and_then(|b| b.base_commit.as_ref())
                    .is_none())
            || d.new
                .iter()
                .chain(&d.resolved)
                .chain(&d.unchanged)
                .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(bad());
        }
    }
    Ok(())
}

pub fn source_metric(root: &Path, policy: &Policy) -> Result<Metric, String> {
    Config {
        version: VERSION,
        enabled: true,
        metrics: vec![policy.clone()],
    }
    .validate()?;
    if !matches!(
        policy.id.as_str(),
        "rust-function-lines" | "rust-duplicate-blocks"
    ) {
        return Err("not a source metric".into());
    }
    Ok(source::rust(root, policy))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
}
impl State {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Unknown => "unknown",
            Self::NotApplicable => "not-applicable",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Comparison {
    Maximum,
    Minimum,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Missing {
    Unknown,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub id: String,
    pub version: u32,
    pub unit: String,
    pub required: bool,
    pub comparison: Comparison,
    pub threshold: u64,
    pub unknown: Missing,
    pub exclusions: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub enabled: bool,
    pub metrics: Vec<Policy>,
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION || self.metrics.is_empty() || self.metrics.len() > 4 {
            return Err("invalid quality config version or metric count".into());
        }
        let mut ids = BTreeSet::new();
        for p in &self.metrics {
            let expected = match p.id.as_str() {
                "rust-function-lines" => ("lines", Comparison::Maximum),
                "rust-duplicate-blocks" => ("blocks", Comparison::Maximum),
                "lcov-line-coverage" => ("basis-points", Comparison::Minimum),
                "new-findings" => ("findings", Comparison::Maximum),
                _ => return Err("unsupported quality metric".into()),
            };
            if p.version != VERSION
                || p.unit != expected.0
                || p.comparison != expected.1
                || !ids.insert(&p.id)
                || p.exclusions.len() > 128
                || p.exclusions.iter().any(|s| !relative_path(s))
                || (p.id == "lcov-line-coverage" && p.threshold > 10_000)
            {
                return Err("invalid quality metric definition".into());
            }
        }
        Ok(())
    }
}
pub fn parse_config(text: &str) -> Result<Config, String> {
    if text.len() > MAX_INPUT as usize {
        return Err("quality config exceeds input budget".into());
    }
    let config: Config = serde_json::from_str(text).map_err(|_| "invalid quality config")?;
    config.validate()?;
    Ok(config)
}
pub(crate) fn relative_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 4096
        && !s.contains(['\\', ':', '\0', '*', '?'])
        && !s.starts_with('/')
        && s.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
        && Path::new(s)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
pub(crate) fn excluded(path: &str, exclusions: &[String]) -> bool {
    exclusions
        .iter()
        .any(|e| path == e || path.strip_prefix(e).is_some_and(|s| s.starts_with('/')))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub format: String,
    pub payload_sha256: String,
    pub covered_lines: u64,
    pub total_lines: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    pub policy: Policy,
    pub value: Option<u64>,
    pub state: State,
    pub reason: Option<String>,
    pub files: Vec<String>,
    pub coverage: Option<Coverage>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    pub run_id: String,
    pub project_id: String,
    pub schema_version: u32,
    pub rule_version: String,
    pub identities: Vec<(String, String, u32)>,
    pub base_ref: Option<String>,
    pub base_commit: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delta {
    pub state: State,
    pub reason: Option<String>,
    pub baseline: Option<Baseline>,
    pub new: Vec<String>,
    pub unchanged: Vec<String>,
    pub resolved: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub version: u32,
    pub config: Config,
    pub state: State,
    pub metrics: Vec<Metric>,
    pub delta: Option<Delta>,
}
