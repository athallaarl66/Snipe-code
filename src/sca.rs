//! Offline npm v3 pair-query cache. SHA-256 detects corruption, not authenticity;
//! refresh trusts HTTPS transport to api.osv.dev, not a signed advisory database.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub const MAX_AGE: u64 = 7 * 24 * 60 * 60;
pub const ENDPOINT: &str = "https://api.osv.dev/v1/query";
pub const COVERAGE: &str =
    "npm package-lock.json v3 package/version pairs only; not ecosystem-wide";

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn validate_report(report: &Report) -> Result<(), String> {
    let status_ok = |s: &str| matches!(s, "unknown" | "stale" | "clean" | "vulnerable");
    let provenance_ok = |p: &Provenance| {
        p.source == SOURCE
            && p.license == LICENSE
            && p.endpoint == ENDPOINT
            && p.fetched_at > 0
            && p.fetched_at.checked_add(MAX_AGE) == Some(p.expires_at)
            && p.payload_sha256.len() == 64
            && p.payload_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    };
    if !status_ok(&report.status) || report.snapshot.as_ref().is_some_and(|p| !provenance_ok(p)) {
        return Err("invalid dependency analysis report".into());
    }
    for p in &report.packages {
        if !name_ok(&p.package)
            || !pinned(&p.version)
            || !status_ok(&p.status)
            || !matches!(
                p.severity.as_str(),
                "unknown" | "low" | "medium" | "high" | "critical"
            )
            || p.path.contains('\\')
            || !p.path.starts_with("node_modules/")
            || p.path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
            || installed_name(&p.path) != Some(p.package.as_str())
            || p.advisory_id.as_ref().is_some_and(|id| !ghsa(id))
            || p.source.as_ref().is_some_and(|s| s != SOURCE)
            || p.snapshot != report.snapshot
            || (matches!(p.status.as_str(), "clean" | "vulnerable" | "stale")
                && p.snapshot.is_none())
            || (p.status == "clean" && p.advisory_id.is_some())
            || (p.status == "vulnerable" && p.advisory_id.is_none())
        {
            return Err("invalid dependency analysis package".into());
        }
    }
    Ok(())
}
const SOURCE: &str = "GitHub Advisory Database";
const LICENSE: &str = "CC-BY-4.0";
const LIMIT: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub license: String,
    pub endpoint: String,
    pub fetched_at: u64,
    pub expires_at: u64,
    pub payload_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageResult {
    pub package: String,
    pub version: String,
    pub path: String,
    pub direct: bool,
    pub status: String,
    pub advisory_id: Option<String>,
    pub source: Option<String>,
    pub severity: String,
    pub snapshot: Option<Provenance>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub status: String,
    pub coverage: String,
    pub issues: Vec<String>,
    pub packages: Vec<PackageResult>,
    pub snapshot: Option<Provenance>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    package: String,
    version: String,
    response: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: u32,
    provenance: Provenance,
    payload: Vec<Entry>,
}
#[derive(Clone)]
struct Package {
    name: String,
    version: String,
    path: String,
    direct: bool,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn bounded(path: &Path) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(|_| "cache or lockfile unavailable")?
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|_| "cache or lockfile unreadable")?;
    if data.len() > LIMIT {
        return Err("cache or lockfile too large".into());
    }
    Ok(data)
}
fn name_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 214
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"@/._-".contains(&b))
        && if let Some(rest) = s.strip_prefix('@') {
            rest.split('/').count() == 2 && rest.split('/').all(|p| !p.is_empty())
        } else {
            !s.contains('/')
        }
}
fn pinned(s: &str) -> bool {
    let core = s.split(['-', '+']).next().unwrap_or("");
    core.split('.').count() == 3
        && core
            .split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
}
fn installed_name(path: &str) -> Option<&str> {
    let name = path.rsplit_once("node_modules/")?.1;
    name_ok(name).then_some(name)
}
fn resolve<'a>(
    packages: &'a serde_json::Map<String, Value>,
    from: &str,
    name: &str,
) -> Option<&'a str> {
    let mut base = from;
    loop {
        let candidate = if base.is_empty() {
            format!("node_modules/{name}")
        } else {
            format!("{base}/node_modules/{name}")
        };
        if let Some((key, _)) = packages.get_key_value(&candidate) {
            return Some(key);
        }
        if base.is_empty() {
            return None;
        }
        base = base
            .rsplit_once("/node_modules/")
            .map(|(p, _)| p)
            .unwrap_or("");
    }
}
fn inventory(root: &Path) -> Result<Vec<Package>, String> {
    let root = fs::canonicalize(root).map_err(|_| "project unavailable")?;
    let lock_path = root.join("package-lock.json");
    let canonical = fs::canonicalize(&lock_path).map_err(|_| "npm lockfile unavailable")?;
    if !canonical.starts_with(&root)
        || fs::symlink_metadata(&lock_path)
            .map_err(|_| "npm lockfile unavailable")?
            .file_type()
            .is_symlink()
    {
        return Err("npm lockfile outside scan scope".into());
    }
    let lock: Value = serde_json::from_slice(&bounded(&root.join("package-lock.json"))?)
        .map_err(|_| "malformed npm lockfile")?;
    if lock["lockfileVersion"].as_u64() != Some(3) {
        return Err("only npm package-lock v3 supported".into());
    }
    let packages = lock["packages"]
        .as_object()
        .ok_or("missing npm package graph")?;
    if !packages.get("").is_some_and(Value::is_object) {
        return Err("missing npm graph root".into());
    }
    let mut direct = BTreeSet::new();
    for (path, record) in packages {
        if !record.is_object() {
            return Err("malformed npm package record".into());
        }
        for field in [
            "dependencies",
            "devDependencies",
            "optionalDependencies",
            "peerDependencies",
        ] {
            let Some(deps) = record.get(field) else {
                continue;
            };
            let deps = deps.as_object().ok_or("malformed npm dependency graph")?;
            for (name, spec) in deps {
                let spec = spec
                    .as_str()
                    .ok_or("malformed npm dependency specification")?;
                if !name_ok(name)
                    || [
                        "git:",
                        "git+",
                        "file:",
                        "link:",
                        "workspace:",
                        "npm:",
                        "http:",
                        "https:",
                    ]
                    .iter()
                    .any(|prefix| spec.starts_with(prefix))
                {
                    return Err("unsupported npm dependency source".into());
                }
                let target = resolve(packages, path, name);
                let optional = field == "optionalDependencies"
                    || record["optionalDependencies"].get(name).is_some()
                    || (field == "peerDependencies"
                        && record["peerDependenciesMeta"][name]["optional"] == true);
                if target.is_none() && !optional {
                    return Err("unresolved npm dependency graph".into());
                }
                if path.is_empty() {
                    if let Some(target) = target {
                        direct.insert(target.to_string());
                    }
                }
            }
        }
    }
    let mut output = Vec::new();
    for (path, record) in packages {
        if path.is_empty() {
            continue;
        }
        if path.contains('\\')
            || path
                .split('/')
                .any(|s| s == ".." || s == "." || s.is_empty())
            || !path.starts_with("node_modules/")
            || record["link"] == true
        {
            return Err("unsupported npm package path or link".into());
        }
        let name = installed_name(path).ok_or("unsupported npm package path")?;
        if let Some(explicit) = record.get("name") {
            if explicit.as_str() != Some(name) {
                return Err("unsupported npm alias".into());
            }
        }
        let version = record["version"]
            .as_str()
            .filter(|s| pinned(s))
            .ok_or("unpinned npm package version")?;
        if let Some(resolved) = record.get("resolved") {
            if !resolved
                .as_str()
                .is_some_and(|s| s.starts_with("https://registry.npmjs.org/"))
            {
                return Err("unsupported npm package source".into());
            }
        }
        output.push(Package {
            name: name.into(),
            version: version.into(),
            path: path.clone(),
            direct: direct.contains(path),
        });
    }
    output.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(output)
}
fn timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || !s.ends_with('Z')
    {
        return false;
    }
    let num = |a: usize, z: usize| s.get(a..z).and_then(|v| v.parse::<u32>().ok());
    let (Some(y), Some(m), Some(d), Some(h), Some(min), Some(sec)) = (
        num(0, 4),
        num(5, 7),
        num(8, 10),
        num(11, 13),
        num(14, 16),
        num(17, 19),
    ) else {
        return false;
    };
    let days = match m {
        2 if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    y >= 1970
        && d > 0
        && d <= days
        && h < 24
        && min < 60
        && sec < 60
        && (b.len() == 20
            || (b[19] == b'.' && b.len() > 21 && b[20..b.len() - 1].iter().all(u8::is_ascii_digit)))
}
fn ghsa(id: &str) -> bool {
    id.len() == 19
        && id.starts_with("GHSA-")
        && id[5..].split('-').count() == 3
        && id[5..]
            .split('-')
            .all(|p| p.len() == 4 && p.bytes().all(|b| b"23456789cfghjmpqrvwx".contains(&b)))
}
fn advisories(entry: &Entry) -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_str(&entry.response).map_err(|_| "malformed OSV response")?;
    let object = v.as_object().ok_or("malformed OSV response")?;
    if object.contains_key("error") || object.contains_key("next_page_token") {
        return Err("incomplete OSV response".into());
    }
    let Some(vulns) = object.get("vulns") else {
        if object.is_empty() {
            return Ok(vec![]);
        }
        return Err("unrecognized OSV response".into());
    };
    let mut found = Vec::new();
    for advisory in vulns.as_array().ok_or("malformed OSV advisories")? {
        let id = advisory["id"]
            .as_str()
            .filter(|id| ghsa(id))
            .ok_or("unsupported advisory source")?;
        if !advisory["modified"].as_str().is_some_and(timestamp)
            || !advisory["published"].as_str().is_some_and(timestamp)
        {
            return Err("invalid advisory timestamp".into());
        }
        if advisory.get("withdrawn").is_some() {
            return Err("withdrawn advisory requires review".into());
        }
        if !advisory["references"].as_array().is_some_and(|refs| {
            refs.iter()
                .any(|r| r["url"].as_str() == Some(&format!("https://github.com/advisories/{id}")))
        }) {
            return Err("unverified GHSA provenance".into());
        }
        if !advisory["affected"].as_array().is_some_and(|a| {
            a.iter().any(|p| {
                p["package"]["ecosystem"] == "npm" && p["package"]["name"] == entry.package
            })
        }) {
            return Err("OSV package mismatch".into());
        }
        let severity = match advisory["database_specific"]["severity"].as_str() {
            Some("LOW") => "low",
            Some("MODERATE" | "MEDIUM") => "medium",
            Some("HIGH") => "high",
            Some("CRITICAL") => "critical",
            _ => "unknown",
        };
        found.push((id.into(), severity.into()));
    }
    found.sort();
    found.dedup();
    Ok(found)
}
fn validate(snapshot: &Snapshot, now: u64) -> Result<(), String> {
    let p = &snapshot.provenance;
    if snapshot.schema != 1 || p.source != SOURCE || p.license != LICENSE || p.endpoint != ENDPOINT
    {
        return Err("invalid snapshot provenance".into());
    }
    if p.fetched_at == 0
        || p.fetched_at > now
        || p.expires_at
            != p.fetched_at
                .checked_add(MAX_AGE)
                .ok_or("invalid snapshot timestamp")?
    {
        return Err("invalid snapshot timestamp".into());
    }
    if digest(&serde_json::to_vec(&snapshot.payload).map_err(|_| "invalid snapshot payload")?)
        != p.payload_sha256
    {
        return Err("snapshot SHA-256 mismatch".into());
    }
    let mut pairs = BTreeSet::new();
    for entry in &snapshot.payload {
        if !name_ok(&entry.package)
            || !pinned(&entry.version)
            || !pairs.insert((&entry.package, &entry.version))
        {
            return Err("invalid snapshot package pair".into());
        }
        advisories(entry)?;
    }
    Ok(())
}
/// Read the returned snapshot file directly, or a cache directory containing `current`.
/// No network, writes, installs, or source-code reads occur during analysis.
pub fn analyze(root: &Path, snapshot: Option<&Path>, now: u64) -> Report {
    let mut report = Report {
        status: "unknown".into(),
        coverage: COVERAGE.into(),
        issues: vec![],
        packages: vec![],
        snapshot: None,
    };
    let packages = match inventory(root) {
        Ok(p) => p,
        Err(e) => {
            report.issues.push(e);
            return report;
        }
    };
    let loaded = snapshot
        .ok_or_else(|| "snapshot missing".to_string())
        .and_then(|path| {
            let path = if path.is_dir() {
                active_snapshot(path)?
            } else {
                path.to_path_buf()
            };
            let s: Snapshot =
                serde_json::from_slice(&bounded(&path)?).map_err(|_| "malformed snapshot")?;
            validate(&s, now)?;
            Ok(s)
        });
    let snapshot = match loaded {
        Ok(s) => {
            report.snapshot = Some(s.provenance.clone());
            Some(s)
        }
        Err(e) => {
            report.issues.push(e);
            None
        }
    };
    let stale = snapshot
        .as_ref()
        .is_some_and(|s| now >= s.provenance.expires_at);
    if stale {
        report.issues.push("snapshot stale".into());
    }
    for package in packages {
        let entry = snapshot.as_ref().and_then(|s| {
            s.payload
                .iter()
                .find(|e| e.package == package.name && e.version == package.version)
        });
        let matches = entry.and_then(|e| advisories(e).ok()).unwrap_or_default();
        let status = if stale {
            "stale"
        } else if entry.is_none() {
            "unknown"
        } else if matches.is_empty() {
            "clean"
        } else {
            "vulnerable"
        };
        let matches = if matches.is_empty() {
            vec![(None, "unknown".into())]
        } else {
            matches.into_iter().map(|(id, s)| (Some(id), s)).collect()
        };
        for (id, severity) in matches {
            report.packages.push(PackageResult {
                package: package.name.clone(),
                version: package.version.clone(),
                path: package.path.clone(),
                direct: package.direct,
                status: status.into(),
                source: id.as_ref().map(|_| SOURCE.into()),
                advisory_id: id,
                severity,
                snapshot: report.snapshot.clone(),
            });
        }
    }
    report.status = if stale {
        "stale"
    } else if !report.issues.is_empty() || report.packages.iter().any(|p| p.status == "unknown") {
        "unknown"
    } else if report.packages.iter().any(|p| p.status == "vulnerable") {
        "vulnerable"
    } else {
        "clean"
    }
    .into();
    report
}
/// Fixed endpoint. Request body contains npm package name and pinned version only.
pub fn query_body(package: &str, version: &str) -> Result<Vec<u8>, String> {
    if !name_ok(package) || !pinned(version) {
        return Err("invalid npm package pair".into());
    }
    serde_json::to_vec(&json!({"package":{"name":package,"ecosystem":"npm"},"version":version}))
        .map_err(|_| "query serialization failed".into())
}
/// Consent must be true before the fetch closure is invoked or cache is touched.
/// Invalid/partial refresh leaves the previous `current` pointer intact.
pub fn refresh<F>(
    root: &Path,
    cache_dir: &Path,
    consent: bool,
    mut fetch: F,
    now: u64,
) -> Result<PathBuf, String>
where
    F: FnMut(&str, &[u8]) -> Result<Vec<u8>, String>,
{
    if !consent {
        return Err("refresh requires explicit --yes consent".into());
    }
    if now == 0 {
        return Err("invalid refresh timestamp".into());
    }
    let packages = inventory(root)?;
    let pairs: BTreeSet<_> = packages.iter().map(|p| (&p.name, &p.version)).collect();
    let mut payload = Vec::new();
    for (package, version) in pairs {
        let bytes =
            fetch(ENDPOINT, &query_body(package, version)?).map_err(|_| "OSV request failed")?;
        if bytes.len() > LIMIT {
            return Err("OSV response too large".into());
        }
        let entry = Entry {
            package: package.clone(),
            version: version.clone(),
            response: String::from_utf8(bytes).map_err(|_| "invalid OSV response encoding")?,
        };
        advisories(&entry)?;
        payload.push(entry);
    }
    let provenance = Provenance {
        source: SOURCE.into(),
        license: LICENSE.into(),
        endpoint: ENDPOINT.into(),
        fetched_at: now,
        expires_at: now
            .checked_add(MAX_AGE)
            .ok_or("invalid refresh timestamp")?,
        payload_sha256: digest(
            &serde_json::to_vec(&payload).map_err(|_| "snapshot serialization failed")?,
        ),
    };
    let snapshot = Snapshot {
        schema: 1,
        provenance,
        payload,
    };
    validate(&snapshot, now)?;
    let bytes = serde_json::to_vec(&snapshot).map_err(|_| "snapshot serialization failed")?;
    if bytes.len() > LIMIT {
        return Err("snapshot too large".into());
    }
    fs::create_dir_all(cache_dir).map_err(|_| "cache creation failed")?;
    let filename = format!("{}.json", digest(&bytes));
    let destination = cache_dir.join(&filename);
    let temporary = cache_dir.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    let pointer = cache_dir.join(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "snapshot staging failed")?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "snapshot staging failed")?;
        drop(f);
        fs::rename(&temporary, &destination).map_err(|_| "snapshot publish failed")?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pointer)
            .map_err(|_| "pointer staging failed")?;
        f.write_all(filename.as_bytes())
            .and_then(|_| f.sync_all())
            .map_err(|_| "pointer staging failed")?;
        drop(f);
        // std::fs::rename replaces an existing file on Windows as well as Unix.
        fs::rename(&pointer, cache_dir.join("current"))
            .map_err(|_| "snapshot activation failed")?;
        Ok(destination)
    })();
    let _ = fs::remove_file(temporary);
    let _ = fs::remove_file(pointer);
    result
}
pub fn active_snapshot(cache_dir: &Path) -> Result<PathBuf, String> {
    let bytes = bounded(&cache_dir.join("current"))?;
    let name = std::str::from_utf8(&bytes).map_err(|_| "invalid snapshot pointer")?;
    if !name.is_ascii()
        || name.len() != 69
        || !name.ends_with(".json")
        || !name[..64]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid snapshot pointer".into());
    }
    let path = cache_dir.join(name);
    if digest(&bounded(&path)?) != name[..64] {
        return Err("snapshot file SHA-256 mismatch".into());
    }
    Ok(path)
}
/// Real adapter, called only by an explicitly consented refresh. No redirects.
pub fn fetch_osv(endpoint: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    if endpoint != ENDPOINT {
        return Err("unapproved OSV endpoint".into());
    }
    let mut child = Command::new("curl")
        .args([
            "--disable",
            "--silent",
            "--fail",
            "--proto",
            "=https",
            "--max-redirs",
            "0",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "--max-filesize",
            "4194304",
            "--request",
            "POST",
            "--header",
            "Content-Type: application/json",
            "--data-binary",
            "@-",
            ENDPOINT,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "OSV transport unavailable")?;
    let result = (|| {
        child
            .stdin
            .take()
            .ok_or("OSV transport failed")?
            .write_all(body)
            .map_err(|_| "OSV transport failed")?;
        let mut data = Vec::new();
        child
            .stdout
            .take()
            .ok_or("OSV transport failed")?
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut data)
            .map_err(|_| "OSV transport failed")?;
        if data.len() > LIMIT {
            return Err("OSV response too large".into());
        }
        if !child.wait().map_err(|_| "OSV transport failed")?.success() {
            return Err("OSV request failed".into());
        }
        Ok(data)
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}
