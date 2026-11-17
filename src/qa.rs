use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// A reviewable test case; recording a plan does not imply execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub objective: String,
    pub precondition: String,
    pub input: String,
    pub expected_result: String,
    pub category: Category,
    pub requirement_refs: Vec<String>,
    pub source_refs: Vec<String>,
    pub adapter: Option<String>,
    pub adapter_version: Option<String>,
    pub status: CaseStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Positive,
    Negative,
    Boundary,
    Authorization,
    Validation,
    Regression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseStatus {
    Draft,
    ReadyForReview,
    Unsupported,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirements {
    version: u32,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    id: String,
    category: Category,
    objective: String,
    precondition: String,
    input: String,
    expected_result: String,
    #[serde(default)]
    request: Option<HttpRequest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpRequest {
    method: String,
    url: String,
    headers: std::collections::BTreeMap<String, String>,
    body: Option<String>,
    expected_status: u16,
}

#[derive(Debug, Serialize)]
pub struct DraftScript {
    pub adapter: &'static str,
    pub adapter_version: &'static str,
    pub status: CaseStatus,
    pub environment: &'static str,
    pub collection: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub version: u32,
    pub cases: Vec<TestCase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<DraftScript>,
    #[serde(skip)]
    requests: Vec<Option<HttpRequest>>,
}

/// Read an explicitly selected requirement file within the project, without running it.
pub fn plan(project: &Path, input: &Path) -> Result<Plan, String> {
    let root = project
        .canonicalize()
        .map_err(|e| format!("project: {e}"))?;
    if !root.is_dir() {
        return Err("project must be a directory".into());
    }
    let source = input
        .canonicalize()
        .map_err(|e| format!("requirements: {e}"))?;
    let relative = source
        .strip_prefix(&root)
        .map_err(|_| "requirements must be inside the project")?;
    let file = fs::File::open(&source).map_err(|e| format!("requirements: {e}"))?;
    let mut content = Vec::new();
    file.take(1_048_577)
        .read_to_end(&mut content)
        .map_err(|e| format!("requirements: {e}"))?;
    if content.len() > 1_048_576 {
        return Err("requirements exceed 1 MiB".into());
    }
    let document: Requirements =
        serde_json::from_slice(&content).map_err(|e| format!("requirements JSON: {e}"))?;
    if document.version != 1 || document.requirements.is_empty() {
        return Err("requirements require version 1 and at least one requirement".into());
    }
    let source_ref = relative.to_string_lossy().replace('\\', "/");
    let mut ids = HashSet::new();
    let mut cases = Vec::new();
    let mut requests = Vec::new();
    for requirement in document.requirements {
        if requirement.id.trim().is_empty() || !ids.insert(requirement.id.clone()) {
            return Err("requirement IDs must be nonempty and unique".into());
        }
        let mut scenario_ids = HashSet::new();
        let mut positive = false;
        let mut negative = false;
        for scenario in requirement.scenarios {
            if [
                &scenario.id,
                &scenario.objective,
                &scenario.precondition,
                &scenario.input,
                &scenario.expected_result,
            ]
            .iter()
            .any(|value| value.trim().is_empty())
                || !scenario_ids.insert(scenario.id.clone())
            {
                return Err(
                    "scenario fields must be nonempty and scenario IDs unique per requirement"
                        .into(),
                );
            }
            positive |= scenario.category == Category::Positive;
            negative |= scenario.category == Category::Negative;
            let identity =
                serde_json::to_vec(&[&requirement.id, &scenario.id]).map_err(|e| e.to_string())?;
            let id = format!("qa-{:x}", Sha256::digest(identity));
            requests.push(scenario.request);
            cases.push(TestCase {
                id,
                objective: scenario.objective,
                precondition: scenario.precondition,
                input: scenario.input,
                expected_result: scenario.expected_result,
                category: scenario.category,
                requirement_refs: vec![requirement.id.clone()],
                source_refs: vec![source_ref.clone()],
                adapter: None,
                adapter_version: None,
                status: CaseStatus::Draft,
            });
        }
        if !positive || !negative {
            return Err(format!(
                "requirement {} needs positive and negative scenarios",
                requirement.id
            ));
        }
    }
    Ok(Plan {
        version: 1,
        cases,
        script: None,
        requests,
    })
}

/// Deterministic collection generation only; does not resolve URLs or run JavaScript.
pub fn postman_draft(plan: &mut Plan) -> Result<(), String> {
    use serde_json::json;
    let mut items = Vec::new();
    for (case, request) in plan.cases.iter().zip(&plan.requests) {
        let request = request
            .as_ref()
            .ok_or("postman-cli requires a structured request for every case")?;
        if !matches!(
            request.method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
        ) || !(100..=599).contains(&request.expected_status)
            || !(request.url.starts_with("http://") || request.url.starts_with("https://"))
            || request
                .url
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            || request.url.contains(['{', '}', '@'])
            || request
                .url
                .split_once("://")
                .map(|(_, s)| {
                    // ponytail: DNS/IPv4 authority only; add IPv6/IDNA with an adapter-specific URL contract.
                    let authority = s.split(['/', '?', '#']).next().unwrap_or("");
                    let (host, port) = authority.split_once(':').unwrap_or((authority, ""));
                    host.is_empty()
                        || !host
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                        || (authority.contains(':') && port.parse::<u16>().map_or(true, |p| p == 0))
                })
                .unwrap_or(true)
            || request.headers.iter().any(|(name, value)| {
                name.is_empty()
                    || matches!(
                        name.to_ascii_lowercase().as_str(),
                        "authorization"
                            | "proxy-authorization"
                            | "cookie"
                            | "set-cookie"
                            | "x-api-key"
                    )
                    || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    || value.chars().any(char::is_control)
            })
        {
            return Err("invalid HTTP request: explicit method, HTTP DNS/IPv4 URL, non-credential headers and status 100-599 required".into());
        }
        let headers: Vec<_> = request
            .headers
            .iter()
            .map(|(key, value)| json!({"key": key, "value": value}))
            .collect();
        let mut http = json!({"method": request.method, "url": request.url, "header": headers});
        if let Some(body) = &request.body {
            http["body"] = json!({"mode": "raw", "raw": body});
        }
        items.push(json!({
            "id": case.id, "name": case.id,
            "request": http,
            "event": [{"listen": "test", "script": {"type": "text/javascript", "exec": [format!("pm.test('expected status', function () {{ pm.response.to.have.status({}); }});", request.expected_status)]}}]
        }));
    }
    let collection = json!({
        "info": {"name": "QA draft", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
        "item": items
    });
    // The only script is a fixed template with a validated integer status.
    // Request strings remain JSON data, not JavaScript source.
    for case in &mut plan.cases {
        case.adapter = Some("postman-cli".into());
        case.adapter_version = Some("1.90.0".into());
        case.status = CaseStatus::ReadyForReview;
    }
    plan.script = Some(DraftScript {
        adapter: "postman-cli",
        adapter_version: "1.90.0",
        status: CaseStatus::ReadyForReview,
        environment: "Postman CLI 1.90.0; Collection v2.1 HTTP; explicit endpoint approval required before execution; runtime compatibility unverified; no login or install performed",
        collection,
    });
    Ok(())
}

/// Export only to a new file outside the project; never overwrite existing data.
pub fn export(project: &Path, destination: &Path, plan: &Plan) -> Result<(), String> {
    let root = project
        .canonicalize()
        .map_err(|e| format!("project: {e}"))?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|e| format!("destination: {e}"))?;
    if parent.starts_with(&root) {
        return Err("destination must be outside the project".into());
    }
    let name = destination
        .file_name()
        .ok_or("destination must name a file")?;
    let destination = parent.join(name);
    let bytes = serde_json::to_vec_pretty(plan).map_err(|e| format!("serialize plan: {e}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|e| format!("destination: {e}"))?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.flush()) {
        drop(file);
        let cleanup = fs::remove_file(&destination);
        return Err(format!(
            "export failed: {error}; partial file cleanup: {cleanup:?}"
        ));
    }
    Ok(())
}
