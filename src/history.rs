use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::audit::{AuditResult, Finding};

pub const SCHEMA_VERSION: u32 = 2;
pub const RULE_VERSION: &str = "heuristic-1";
const FINGERPRINT_VERSION: u32 = 1;
const MIGRATION_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanEnvelope {
    pub schema_version: u32,
    pub result: ScanResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub run_id: String,
    pub scanned_at_utc: String,
    pub project_id: String,
    pub tool_version: String,
    pub rule_version: String,
    pub detected_scopes: Vec<String>,
    pub evaluated_rule_groups: Vec<RuleStatus>,
    pub exclusions: Vec<String>,
    pub findings: Vec<ScanFinding>,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub highest_severity: Option<String>,
    pub policy_failed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleStatus {
    pub rule_group: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanFinding {
    pub rule_id: String,
    pub fingerprint_version: u32,
    pub fingerprint: String,
    pub severity: String,
    pub category: String,
    pub description: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRow {
    pub run_id: String,
    pub project_id: String,
    pub scanned_at_utc: String,
}

impl ScanEnvelope {
    pub fn from_audit(root: &Path, audit: AuditResult) -> Result<Self, String> {
        let root = canonical_directory(root)?;
        let findings = audit
            .findings
            .into_iter()
            .map(|finding| scan_finding(&root, finding))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            result: ScanResult {
                run_id: Uuid::new_v4().to_string(),
                scanned_at_utc: timestamp_utc(),
                project_id: project_id(&root),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                rule_version: RULE_VERSION.to_string(),
                detected_scopes: audit
                    .detected_scopes
                    .iter()
                    .map(|scope| redact_scope(&root, scope))
                    .collect(),
                evaluated_rule_groups: rule_statuses(&audit.evaluated_rule_groups),
                exclusions: audit.exclusions,
                findings,
                high: audit.high,
                medium: audit.medium,
                low: audit.low,
                highest_severity: audit.highest_severity,
                policy_failed: audit.policy_failed,
            },
        })
    }

    pub fn to_audit_result(&self) -> AuditResult {
        AuditResult {
            exported_files: Vec::new(),
            findings: self
                .result
                .findings
                .iter()
                .map(|finding| Finding {
                    severity: finding.severity.clone(),
                    category: finding.category.clone(),
                    description: finding.description.clone(),
                    file: finding.file.clone(),
                    line: finding.line,
                })
                .collect(),
            high: self.result.high,
            medium: self.result.medium,
            low: self.result.low,
            highest_severity: self.result.highest_severity.clone(),
            policy_failed: self.result.policy_failed,
            detected_scopes: self.result.detected_scopes.clone(),
            evaluated_rule_groups: self
                .result
                .evaluated_rule_groups
                .iter()
                .filter(|status| status.status == "evaluated")
                .map(|status| status.rule_group.clone())
                .collect(),
            exclusions: self.result.exclusions.clone(),
        }
    }
}

fn scan_finding(root: &Path, finding: Finding) -> Result<ScanFinding, String> {
    let source_path = Path::new(&finding.file);
    let relative = if source_path.is_absolute() {
        source_path
            .strip_prefix(root)
            .map_err(|_| "finding path is outside scan root".to_string())?
    } else {
        source_path
    };
    let file = relative.to_string_lossy().replace('\\', "/");
    validate_scan_path(&file)?;
    let mut rule_id = finding.category.to_lowercase().replace(' ', "-");
    if rule_id == "hardcoded-secret" {
        if let Some(pattern) = finding
            .description
            .strip_prefix("Potential hardcoded secret: ")
        {
            if [
                "password",
                "secret",
                "api_key",
                "apikey",
                "token",
                "private_key",
            ]
            .contains(&pattern)
            {
                rule_id.push('-');
                rule_id.push_str(pattern);
            }
        }
    }
    let fingerprint = finding_fingerprint(
        RULE_VERSION,
        &rule_id,
        &file,
        finding.line,
        &finding.severity,
    );
    let description = redact_description(&finding.description);
    Ok(ScanFinding {
        rule_id,
        fingerprint_version: FINGERPRINT_VERSION,
        fingerprint,
        severity: finding.severity,
        category: finding.category,
        description,
        file,
        line: finding.line,
    })
}

fn digest(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

pub fn finding_fingerprint(
    rule_version: &str,
    rule_id: &str,
    file: &str,
    line: usize,
    severity: &str,
) -> String {
    digest(
        format!("{FINGERPRINT_VERSION}\0{rule_version}\0{rule_id}\0{file}\0{line}\0{severity}")
            .as_bytes(),
    )
}

fn redact_description(description: &str) -> String {
    const SAFE: &[&str] = &[
        "Potential hardcoded secret: password",
        "Potential hardcoded secret: secret",
        "Potential hardcoded secret: api_key",
        "Potential hardcoded secret: apikey",
        "Potential hardcoded secret: token",
        "Potential hardcoded secret: private_key",
        ".env files not in .gitignore",
        "Potential SQL injection: string concatenation in query",
        "Direct HTML injection detected",
        "Missing .env.example file",
        "Missing .gitignore file",
        "Heuristic unsafe query or secret pattern in supported stack source",
    ];
    if SAFE.contains(&description) {
        description.to_string()
    } else {
        "[REDACTED]".to_string()
    }
}

fn redact_scope(root: &Path, scope: &str) -> String {
    if let Some((ecosystem, path)) = scope.split_once(" (") {
        let path = path.trim_end_matches(')');
        if let Ok(relative) = Path::new(path).strip_prefix(root) {
            if relative.as_os_str().is_empty() {
                return ecosystem.to_string();
            }
            return format!(
                "{} ({})",
                ecosystem,
                relative.to_string_lossy().replace('\\', "/")
            );
        }
        return ecosystem.to_string();
    }
    scope.to_string()
}

fn rule_statuses(groups: &[String]) -> Vec<RuleStatus> {
    let mut statuses: Vec<_> = groups
        .iter()
        .map(|rule_group| RuleStatus {
            rule_group: rule_group.clone(),
            status: "unknown".to_string(),
        })
        .collect();
    statuses.push(RuleStatus {
        rule_group: "unsupported languages and unevaluated rules".to_string(),
        status: "unknown".to_string(),
    });
    statuses
}

fn validate_scan_path(file: &str) -> Result<(), String> {
    let normalized = file.replace('\\', "/");
    let path = Path::new(&normalized);
    if normalized.is_empty()
        || path.is_absolute()
        || normalized.starts_with('/')
        || normalized.as_bytes().get(1) == Some(&b':')
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("scan finding path must be relative to project root".to_string());
    }
    Ok(())
}

fn project_id(root: &Path) -> String {
    digest(root.to_string_lossy().as_bytes())
}

fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| error.to_string())
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| error.to_string())?;
    set_private_dir(path)
}

fn ensure_private_file(path: &Path) -> Result<(), String> {
    set_private_file(path)
}

fn serialize_envelope(envelope: &ScanEnvelope) -> Result<String, String> {
    serde_json::to_string(envelope).map_err(|error| error.to_string())
}

fn deserialize_envelope(json: &str) -> Result<ScanEnvelope, String> {
    serde_json::from_str(json).map_err(|error| error.to_string())
}

fn verify_schema_version(value: &serde_json::Value) -> Result<(), String> {
    if value["schema_version"].as_u64() != Some(SCHEMA_VERSION as u64) {
        return Err(format!(
            "unsupported scan result schema version: {}",
            value["schema_version"]
        ));
    }
    Ok(())
}

fn history_database_path(data_dir: &Path) -> PathBuf {
    data_dir.join("history.sqlite3")
}

fn utc_timestamp_is_valid(timestamp: &str) -> bool {
    let bytes = timestamp.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || *bytes.last().unwrap_or(&0) != b'Z'
        || ![0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
            .iter()
            .all(|range| bytes[range.clone()].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    if bytes.len() > 20 && bytes.len() < 22 {
        return false;
    }
    if bytes.len() > 20
        && (bytes[19] != b'.' || !bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    let number = |range: std::ops::Range<usize>| {
        std::str::from_utf8(&bytes[range])
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or_default()
    };
    let year = number(0..4);
    let month = number(5..7);
    let day = number(8..10);
    let hour = number(11..13);
    let minute = number(14..16);
    let second = number(17..19);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0 && (1..=days_in_month).contains(&day) && hour < 24 && minute < 60 && second < 60
}

fn validate_finding(finding: &ScanFinding) -> Result<(), String> {
    validate_scan_path(&finding.file)?;
    if finding.fingerprint_version > FINGERPRINT_VERSION {
        return Err("unsupported finding fingerprint version".to_string());
    }
    if finding.fingerprint.is_empty() || finding.rule_id.is_empty() {
        return Err("finding is missing rule identity or fingerprint".to_string());
    }
    if !matches!(finding.severity.as_str(), "HIGH" | "MEDIUM" | "LOW") {
        return Err("finding contains invalid severity".to_string());
    }
    Ok(())
}

fn validate_all_findings(envelope: &ScanEnvelope) -> Result<(), String> {
    for finding in &envelope.result.findings {
        validate_finding(finding)?;
        if finding.fingerprint_version == FINGERPRINT_VERSION
            && finding.fingerprint
                != finding_fingerprint(
                    &envelope.result.rule_version,
                    &finding.rule_id,
                    &finding.file,
                    finding.line,
                    &finding.severity,
                )
        {
            return Err("finding fingerprint does not match its identity".to_string());
        }
    }
    Ok(())
}

fn validate_timestamp(envelope: &ScanEnvelope) -> Result<(), String> {
    if !utc_timestamp_is_valid(&envelope.result.scanned_at_utc) {
        return Err("scan timestamp is not UTC RFC3339".to_string());
    }
    Ok(())
}

fn validate_identity(envelope: &ScanEnvelope) -> Result<(), String> {
    if envelope.result.run_id.is_empty() || envelope.result.project_id.is_empty() {
        return Err("scan result is missing run or project identity".to_string());
    }
    Ok(())
}

fn validate_statuses(envelope: &ScanEnvelope) -> Result<(), String> {
    if envelope.result.evaluated_rule_groups.iter().any(|status| {
        !matches!(
            status.status.as_str(),
            "evaluated" | "not-applicable" | "unknown" | "error"
        )
    }) {
        return Err("scan result contains invalid rule status".to_string());
    }
    Ok(())
}

fn validate_envelope_complete(envelope: &ScanEnvelope) -> Result<(), String> {
    validate_envelope(envelope)?;
    validate_identity(envelope)?;
    validate_timestamp(envelope)?;
    validate_statuses(envelope)?;
    validate_all_findings(envelope)?;
    let findings = &envelope.result.findings;
    let (high, medium, low) = findings.iter().fold((0, 0, 0), |mut counts, finding| {
        match finding.severity.as_str() {
            "HIGH" => counts.0 += 1,
            "MEDIUM" => counts.1 += 1,
            "LOW" => counts.2 += 1,
            _ => {}
        }
        counts
    });
    if (high, medium, low)
        != (
            envelope.result.high,
            envelope.result.medium,
            envelope.result.low,
        )
    {
        return Err("scan result severity counts do not match findings".to_string());
    }
    Ok(())
}

fn redact_scope_path(scope: &str) -> String {
    if let Some((identity, path)) = scope.split_once(" (") {
        let path = path.trim_end_matches(')');
        let normalized = path.replace('\\', "/");
        if Path::new(path).is_absolute()
            || normalized.starts_with('/')
            || (normalized.as_bytes().get(1) == Some(&b':')
                && normalized.as_bytes().get(2) == Some(&b'/'))
        {
            return identity.to_string();
        }
    }
    scope.to_string()
}

fn redact_envelope(envelope: &mut ScanEnvelope) {
    envelope.result.detected_scopes = envelope
        .result
        .detected_scopes
        .iter()
        .map(|scope| redact_scope_path(scope))
        .collect();
    for finding in &mut envelope.result.findings {
        finding.description = redact_description(&finding.description);
    }
}

fn insert_envelope(transaction: &Transaction<'_>, envelope: &ScanEnvelope) -> Result<(), String> {
    let json = serialize_envelope(envelope)?;
    transaction
        .execute(
            "INSERT INTO projects(project_id, created_at_utc) VALUES (?1, ?2) ON CONFLICT(project_id) DO NOTHING",
            (&envelope.result.project_id, &envelope.result.scanned_at_utc),
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "INSERT INTO scan_runs(run_id, project_id, scanned_at_utc, result_json) VALUES (?1, ?2, ?3, ?4)",
            (
                &envelope.result.run_id,
                &envelope.result.project_id,
                &envelope.result.scanned_at_utc,
                json,
            ),
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn query_envelope(connection: &Connection, run_id: &str) -> Result<Option<ScanEnvelope>, String> {
    let json: Option<String> = connection
        .query_row(
            "SELECT result_json FROM scan_runs WHERE run_id = ?1",
            [run_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    json.map(|json| {
        let envelope = deserialize_envelope(&json)?;
        validate_envelope_complete(&envelope)?;
        Ok(envelope)
    })
    .transpose()
}

fn query_history(connection: &Connection) -> Result<Vec<HistoryRow>, String> {
    let mut statement = connection
        .prepare("SELECT run_id, project_id, scanned_at_utc FROM scan_runs ORDER BY scanned_at_utc DESC, run_id")
        .map_err(|error| error.to_string())?;
    statement
        .query_map([], |row| {
            Ok(HistoryRow {
                run_id: row.get(0)?,
                project_id: row.get(1)?,
                scanned_at_utc: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn erase_project(transaction: &Transaction<'_>, project_id: &str) -> Result<usize, String> {
    transaction
        .execute("DELETE FROM projects WHERE project_id = ?1", [project_id])
        .map_err(|error| error.to_string())
}

fn timestamp_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (seconds / 86_400) as i64;
    let (year, month, day) = crate::date::civil_from_days(days);
    let hour = (seconds % 86_400) / 3_600;
    let minute = (seconds % 3_600) / 60;
    let second = seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub fn read_scan(input: &Path) -> Result<ScanEnvelope, String> {
    let text = fs::read_to_string(input).map_err(|error| error.to_string())?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| error.to_string())?;
    let mut result = if value.get("schema_version").is_some() {
        if value["schema_version"].as_u64() == Some(1) {
            let legacy: AuditResult = serde_json::from_value(value["result"].clone())
                .map_err(|error| format!("invalid legacy scan result: {error}"))?;
            legacy_envelope(legacy)
        } else {
            verify_schema_version(&value)?;
            serde_json::from_value::<ScanEnvelope>(value)
                .map_err(|error| format!("invalid scan result: {error}"))?
        }
    } else {
        let legacy: AuditResult = serde_json::from_value(value)
            .map_err(|error| format!("invalid scan result: {error}"))?;
        legacy_envelope(legacy)
    };
    redact_envelope(&mut result);
    validate_envelope_complete(&result)?;
    Ok(result)
}

fn legacy_envelope(audit: AuditResult) -> ScanEnvelope {
    let findings = audit
        .findings
        .into_iter()
        .map(|finding| {
            let rule_id = finding.category.to_lowercase().replace(' ', "-");
            let fingerprint = digest(
                format!(
                    "legacy\0{rule_id}\0{}\0{}\0{}",
                    finding.file, finding.line, finding.severity
                )
                .as_bytes(),
            );
            ScanFinding {
                rule_id,
                fingerprint_version: 0,
                fingerprint,
                severity: finding.severity,
                category: finding.category,
                description: finding.description,
                file: if validate_scan_path(&finding.file).is_ok() {
                    finding.file.replace('\\', "/")
                } else {
                    "[unmapped]".to_string()
                },
                line: finding.line,
            }
        })
        .collect();
    ScanEnvelope {
        schema_version: SCHEMA_VERSION,
        result: ScanResult {
            run_id: Uuid::new_v4().to_string(),
            scanned_at_utc: timestamp_utc(),
            project_id: "legacy".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            rule_version: "legacy".to_string(),
            detected_scopes: audit.detected_scopes,
            evaluated_rule_groups: rule_statuses(&audit.evaluated_rule_groups),
            exclusions: audit.exclusions,
            findings,
            high: audit.high,
            medium: audit.medium,
            low: audit.low,
            highest_severity: audit.highest_severity,
            policy_failed: audit.policy_failed,
        },
    }
}

pub fn default_data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("SNIPE_CODE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    base.map(|path| path.join("snipe-code"))
        .ok_or_else(|| "cannot determine application data directory".to_string())
}

pub struct HistoryStore {
    connection: Connection,
}

impl HistoryStore {
    pub fn open_in(data_dir: &Path) -> Result<Self, String> {
        ensure_private_directory(data_dir)?;
        let path = history_database_path(data_dir);
        let mut connection = Connection::open(&path).map_err(|error| error.to_string())?;
        ensure_private_file(&path)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| error.to_string())?;
        migrate(&mut connection).map_err(|error| error.to_string())?;
        Ok(Self { connection })
    }

    pub fn save(&mut self, envelope: &ScanEnvelope) -> Result<(), String> {
        let mut envelope = envelope.clone();
        redact_envelope(&mut envelope);
        validate_envelope_complete(&envelope)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;
        insert_envelope(&transaction, &envelope)?;
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn list(&self) -> Result<Vec<HistoryRow>, String> {
        query_history(&self.connection)
    }

    pub fn get(&self, run_id: &str) -> Result<Option<ScanEnvelope>, String> {
        query_envelope(&self.connection, run_id)
    }

    pub fn delete_project(&mut self, project_id: &str) -> Result<usize, String> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;
        let runs = transaction
            .query_row(
                "SELECT COUNT(*) FROM scan_runs WHERE project_id = ?1",
                [project_id],
                |row| row.get::<_, usize>(0),
            )
            .map_err(|error| error.to_string())?;
        let _ = erase_project(&transaction, project_id)?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(runs)
    }
}

pub fn validate_envelope(envelope: &ScanEnvelope) -> Result<(), String> {
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported scan result schema version: {}",
            envelope.schema_version
        ));
    }
    if envelope.result.run_id.is_empty() || envelope.result.project_id.is_empty() {
        return Err("scan result is missing run or project identity".to_string());
    }
    Ok(())
}

fn migrate(connection: &mut Connection) -> rusqlite::Result<()> {
    let current: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > MIGRATION_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if current == MIGRATION_VERSION {
        return Ok(());
    }
    let transaction: Transaction<'_> = connection.transaction()?;
    transaction.execute_batch(
        "CREATE TABLE projects (
            project_id TEXT PRIMARY KEY NOT NULL,
            created_at_utc TEXT NOT NULL
        );
        CREATE TABLE scan_runs (
            run_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL REFERENCES projects(project_id) ON DELETE CASCADE,
            scanned_at_utc TEXT NOT NULL,
            result_json TEXT NOT NULL
        );
        CREATE INDEX scan_runs_project_id ON scan_runs(project_id);
        PRAGMA user_version = 1;",
    )?;
    transaction.commit()
}

#[cfg(unix)]
fn set_private_dir(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn set_private_dir(_: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn set_private_file(_: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_envelope(project_id: &str, run_id: &str) -> ScanEnvelope {
        ScanEnvelope {
            schema_version: SCHEMA_VERSION,
            result: ScanResult {
                run_id: run_id.to_string(),
                scanned_at_utc: "2026-01-01T00:00:00Z".to_string(),
                project_id: project_id.to_string(),
                tool_version: "test".to_string(),
                rule_version: RULE_VERSION.to_string(),
                detected_scopes: vec!["common".to_string()],
                evaluated_rule_groups: vec![RuleStatus {
                    rule_group: "common".to_string(),
                    status: "evaluated".to_string(),
                }],
                exclusions: vec![],
                findings: vec![],
                high: 0,
                medium: 0,
                low: 0,
                highest_severity: None,
                policy_failed: false,
            },
        }
    }

    fn fixture_audit(root: &Path, description: &str) -> AuditResult {
        AuditResult {
            exported_files: vec![],
            findings: vec![Finding {
                severity: "HIGH".to_string(),
                category: "Hardcoded Secret".to_string(),
                description: description.to_string(),
                file: root.join("src/config.rs").display().to_string(),
                line: 7,
            }],
            high: 1,
            medium: 0,
            low: 0,
            highest_severity: Some("HIGH".to_string()),
            policy_failed: true,
            detected_scopes: vec![format!("rust ({})", root.display())],
            evaluated_rule_groups: vec!["common".to_string()],
            exclusions: vec![],
        }
    }

    #[test]
    fn fingerprint_is_stable_for_same_rule_and_changes_with_rule_version() {
        let first = finding_fingerprint("rules-1", "hardcoded-secret", "src/a.rs", 3, "HIGH");
        let repeat = finding_fingerprint("rules-1", "hardcoded-secret", "src/a.rs", 3, "HIGH");
        let changed = finding_fingerprint("rules-2", "hardcoded-secret", "src/a.rs", 3, "HIGH");
        assert_eq!(first, repeat);
        assert_ne!(first, changed);
    }

    #[test]
    fn repeated_scan_fixtures_keep_finding_fingerprint() {
        let dir = std::env::temp_dir().join(format!("snipe-scan-stable-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let root = fs::canonicalize(&dir).unwrap();
        let first = ScanEnvelope::from_audit(
            &root,
            fixture_audit(&root, "Potential hardcoded secret: password"),
        )
        .unwrap();
        let second = ScanEnvelope::from_audit(
            &root,
            fixture_audit(&root, "Potential hardcoded secret: password"),
        )
        .unwrap();
        assert_eq!(
            first.result.findings[0].fingerprint,
            second.result.findings[0].fingerprint
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn scan_envelope_redacts_values_and_absolute_paths() {
        let dir = std::env::temp_dir().join(format!("snipe-scan-envelope-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let root = fs::canonicalize(&dir).unwrap();
        let envelope =
            ScanEnvelope::from_audit(&root, fixture_audit(&root, "password=canary-value")).unwrap();
        let finding = &envelope.result.findings[0];
        assert_eq!(finding.file, "src/config.rs");
        assert!(!finding.description.contains("canary-value"));
        assert!(finding.description.contains("[REDACTED]"));
        assert_eq!(envelope.result.detected_scopes, ["rust"]);
        assert!(
            !serde_json::to_string(&envelope)
                .unwrap()
                .contains(root.to_str().unwrap())
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn validation_rejects_invalid_timestamp_and_severity_totals() {
        assert!(utc_timestamp_is_valid("2024-02-29T23:59:59Z"));
        assert!(utc_timestamp_is_valid("2024-02-29T23:59:59.123Z"));
        assert!(!utc_timestamp_is_valid("2023-02-29T23:59:59Z"));
        assert!(!utc_timestamp_is_valid("2024-13-01T00:00:00Z"));
        assert!(!utc_timestamp_is_valid("2024-01-01T24:00:00Z"));
        assert!(!utc_timestamp_is_valid("2024-01-01T00:00:00.Z"));

        let mut envelope = fixture_envelope("project-a", "run-a");
        envelope.result.high = 1;
        assert!(
            validate_envelope_complete(&envelope)
                .unwrap_err()
                .contains("severity counts do not match")
        );
    }

    #[test]
    fn redaction_rejects_untrusted_description_wording() {
        for description in [
            "Credential value: expose-canary-123",
            "Credential found: password=expose-canary-123",
            "expose-canary-123",
        ] {
            assert_eq!(redact_description(description), "[REDACTED]");
        }
    }

    #[test]
    fn read_scan_rejects_unknown_schema_and_reads_legacy_audit_json() {
        let dir = std::env::temp_dir().join(format!("snipe-scan-read-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let unknown = dir.join("unknown.json");
        fs::write(&unknown, r#"{"schema_version":99,"result":{}}"#).unwrap();
        assert!(
            read_scan(&unknown)
                .unwrap_err()
                .contains("unsupported scan result schema version: 99")
        );

        let legacy = dir.join("legacy.json");
        let audit = fixture_audit(&dir, "Potential hardcoded secret: password");
        fs::write(&legacy, serde_json::to_vec(&audit).unwrap()).unwrap();
        let envelope = read_scan(&legacy).unwrap();
        assert_eq!(envelope.result.rule_version, "legacy");
        assert_eq!(envelope.result.findings[0].file, "[unmapped]");
        let mut relative = audit.clone();
        relative.findings[0].file = "frontend/config.js".to_string();
        fs::write(
            &legacy,
            serde_json::to_vec(&serde_json::json!({"schema_version": 1, "result": relative}))
                .unwrap(),
        )
        .unwrap();
        let envelope = read_scan(&legacy).unwrap();
        assert_eq!(envelope.result.findings[0].file, "frontend/config.js");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn migration_rolls_back_partial_schema_on_failure() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute("CREATE TABLE scan_runs (id INTEGER)", [])
            .unwrap();
        connection
            .execute("INSERT INTO scan_runs VALUES (17)", [])
            .unwrap();
        assert!(migrate(&mut connection).is_err());
        let prior_row: i64 = connection
            .query_row("SELECT id FROM scan_runs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(prior_row, 17);
        let projects_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='projects')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!projects_exists);
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 0);
    }

    #[test]
    fn deleting_project_cascades_only_its_runs() {
        let dir = std::env::temp_dir().join(format!("snipe-history-test-{}", Uuid::new_v4()));
        let mut store = HistoryStore::open_in(&dir).unwrap();
        store.save(&fixture_envelope("project-a", "run-a")).unwrap();
        store.save(&fixture_envelope("project-b", "run-b")).unwrap();
        assert_eq!(store.delete_project("project-a").unwrap(), 1);
        let rows = store.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_id, "run-b");
        drop(store);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn saving_history_redacts_data_and_supports_list_show_delete() {
        let dir = std::env::temp_dir().join(format!("snipe-history-redact-{}", Uuid::new_v4()));
        let root = std::env::temp_dir().join(format!("snipe-project-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let mut envelope =
            ScanEnvelope::from_audit(&root, fixture_audit(&root, "password=canary-value")).unwrap();
        envelope.result.run_id = "run-redacted".to_string();
        let project_id = envelope.result.project_id.clone();
        let mut store = HistoryStore::open_in(&dir).unwrap();
        store.save(&envelope).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        let saved = store.get("run-redacted").unwrap().unwrap();
        assert!(
            !serde_json::to_string(&saved)
                .unwrap()
                .contains("canary-value")
        );
        drop(store);

        let raw = fs::read(history_database_path(&dir)).unwrap();
        assert!(!String::from_utf8_lossy(&raw).contains("canary-value"));
        let mut store = HistoryStore::open_in(&dir).unwrap();
        assert_eq!(store.delete_project(&project_id).unwrap(), 1);
        assert!(store.get("run-redacted").unwrap().is_none());
        drop(store);
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn saving_rejects_unsupported_schema_without_creating_run() {
        let dir = std::env::temp_dir().join(format!("snipe-history-schema-{}", Uuid::new_v4()));
        let mut store = HistoryStore::open_in(&dir).unwrap();
        let mut envelope = fixture_envelope("project-a", "run-a");
        envelope.schema_version = SCHEMA_VERSION + 1;
        assert!(
            store
                .save(&envelope)
                .unwrap_err()
                .contains("unsupported scan result schema version")
        );
        assert!(store.list().unwrap().is_empty());
        drop(store);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn history_files_have_private_unix_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("snipe-history-mode-{}", Uuid::new_v4()));
        let store = HistoryStore::open_in(&dir).unwrap();
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(dir.join("history.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(store);
        fs::remove_dir_all(dir).unwrap();
    }
}
