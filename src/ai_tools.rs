//! AI agent tool registry and artifact renderer.
//!
//! Tool coverage is data: `templates/ai-tools.json` names each tool, its
//! artifacts, and each artifact's format. Adding a tool is a registry entry plus
//! a content file, not a code change. Renderers are keyed by format, so the
//! maintenance surface is the number of formats rather than the number of tools.

use serde::Deserialize;

const REGISTRY: &str = "ai-tools.json";
const CONTENT_DIR: &str = "ai-tools/content";

/// Tool id that receives the cross-tool instruction file (`AGENTS.md`).
pub const CROSS_TOOL_ID: &str = "agents-md";

/// Placeholder in an artifact path, replaced by the artifact's `name` (or the
/// tool id when the entry names none).
const NAME_TOKEN: &str = "{name}";

#[derive(Debug, Deserialize, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Artifact {
    pub path: String,
    pub format: String,
    pub template: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// One artifact resolved to a project-relative path with rendered content.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedArtifact {
    pub tool: String,
    pub path: String,
    pub content: String,
}

/// A destination path claimed by more than one selected tool. The first claim
/// wins; the overlap is reported instead of being resolved by ordering.
#[derive(Debug, Clone, PartialEq)]
pub struct Overlap {
    pub path: String,
    pub kept: String,
    pub skipped: String,
}

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub artifacts: Vec<PlannedArtifact>,
    pub overlaps: Vec<Overlap>,
}

#[derive(Deserialize)]
struct Registry {
    tools: Vec<Tool>,
}

/// Reads the embedded (or locally overridden) tool registry.
pub fn load() -> Result<Vec<Tool>, String> {
    let raw =
        crate::template::read(REGISTRY).ok_or_else(|| format!("missing templates/{REGISTRY}"))?;
    let registry: Registry =
        serde_json::from_str(&raw).map_err(|e| format!("invalid templates/{REGISTRY}: {e}"))?;
    if registry.tools.is_empty() {
        return Err(format!("templates/{REGISTRY} defines no tool"));
    }
    Ok(registry.tools)
}

/// Tool ids in registry order.
pub fn ids(tools: &[Tool]) -> Vec<String> {
    tools.iter().map(|tool| tool.id.clone()).collect()
}

/// Prompt labels in registry order: `<name> (<id>)`.
pub fn display_names(tools: &[Tool]) -> Vec<String> {
    tools
        .iter()
        .map(|tool| format!("{} ({})", tool.name, tool.id))
        .collect()
}

/// Comma-separated selection: `none` (or empty) selects nothing.
pub fn parse_selection(spec: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for token in spec.split(',') {
        let token = token.trim();
        if token.is_empty() || token == "none" || out.iter().any(|id| id == token) {
            continue;
        }
        out.push(token.to_string());
    }
    out
}

/// Renders the artifacts of the selected tools, in registry order, each
/// destination path once.
pub fn plan(tools: &[Tool], selected: &[String]) -> Result<Plan, String> {
    let mut plan = Plan::default();
    for id in selected {
        let tool = tools.iter().find(|tool| &tool.id == id).ok_or_else(|| {
            format!(
                "unknown AI tool '{}' (known: {})",
                id,
                ids(tools).join(", ")
            )
        })?;
        for artifact in &tool.artifacts {
            let path = artifact.resolve_path(&tool.id);
            if let Some(existing) = plan.artifacts.iter().find(|a| a.path == path) {
                plan.overlaps.push(Overlap {
                    path,
                    kept: existing.tool.clone(),
                    skipped: tool.id.clone(),
                });
                continue;
            }
            let body = read_body(&artifact.template)?;
            let content = render(
                &artifact.format,
                &body,
                artifact.name.as_deref().unwrap_or(&tool.id),
                artifact.description.as_deref().unwrap_or(""),
            )?;
            plan.artifacts.push(PlannedArtifact {
                tool: tool.id.clone(),
                path,
                content,
            });
        }
    }
    Ok(plan)
}

/// `plan` against the embedded registry.
pub fn plan_selected(selected: &[String]) -> Result<Plan, String> {
    plan(&load()?, selected)
}

/// The shared guidance body, written to `.ai-workspace/rules.md` as well as to
/// every selected tool's artifacts, so the guidance cannot drift between them.
pub fn shared_content() -> Result<String, String> {
    read_body("shared")
}

/// Format-keyed renderer. An unknown format is an error, never a silent skip.
pub fn render(format: &str, body: &str, name: &str, description: &str) -> Result<String, String> {
    let default_description = format!("{name} project rules");
    let description = if description.is_empty() {
        default_description.as_str()
    } else {
        description
    };
    let rendered = match format {
        "markdown" => body.to_string(),
        "mdc" => format!(
            "---\ndescription: {description}\nglobs: **/*\nalwaysApply: true\n---\n\n{body}"
        ),
        "skill" => {
            format!("---\nname: {name}\ndescription: {description}\n---\n\n{body}")
        }
        "command" => format!("---\ndescription: {description}\n---\n\n{body}"),
        other => return Err(format!("unknown AI artifact format '{}'", other)),
    };
    Ok(rendered)
}

impl Artifact {
    /// Destination path with `{name}` replaced.
    pub fn resolve_path(&self, tool_id: &str) -> String {
        let name = self.name.as_deref().unwrap_or(tool_id);
        self.path.replace(NAME_TOKEN, name)
    }
}

fn read_body(template: &str) -> Result<String, String> {
    let rel = format!("{CONTENT_DIR}/{template}.md");
    crate::template::read(&rel).ok_or_else(|| format!("missing templates/{rel}"))
}
