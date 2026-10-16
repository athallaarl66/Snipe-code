//! Writes the shared `.ai-workspace/` files plus every selected AI tool's
//! artifacts, rendered from the registry in `ai_tools`.

use std::fs;
use std::path::Path;

use crate::ai_tools::{self, Plan};

const SHARED_DESCRIPTION: &str = "Shared SNIPE-CODE rules for tools with no dedicated convention.";

/// What generation wrote and which tool artifacts overlapped.
#[derive(Debug, Default)]
pub struct AiOutput {
    pub tools: Vec<String>,
    pub artifacts: Vec<String>,
    pub overlaps: Vec<String>,
}

/// Writes `.ai-workspace/` (always: it is the shared guidance for tools with no
/// dedicated convention) and the artifacts of `selected`. Nothing is written
/// outside `project_path`.
pub fn generate_ai_workspace(project_path: &str, selected: &[String]) -> Result<AiOutput, String> {
    // Resolved before any directory is created, so a bad registry entry or an
    // unknown tool leaves the destination untouched.
    let plan = ai_tools::plan_selected(selected)?;
    let path = Path::new(project_path);
    let shared = ai_tools::shared_content()?;

    let workspace_dir = path.join(".ai-workspace");
    fs::create_dir_all(&workspace_dir).map_err(|e| e.to_string())?;
    write(
        &workspace_dir.join("rules.md"),
        &ai_tools::render("markdown", &shared, "snipe-code", SHARED_DESCRIPTION)?,
    )?;
    write(
        &workspace_dir.join("rules.mdc"),
        &ai_tools::render("mdc", &shared, "snipe-code", SHARED_DESCRIPTION)?,
    )?;
    let audit_dir = workspace_dir.join("audit");
    fs::create_dir_all(&audit_dir).map_err(|e| e.to_string())?;
    write(&audit_dir.join(".gitkeep"), "")?;

    let mut output = AiOutput::default();
    for artifact in &plan.artifacts {
        let dest = path.join(&artifact.path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        write(&dest, &artifact.content)?;
        output.artifacts.push(artifact.path.clone());
        if !output.tools.contains(&artifact.tool) {
            output.tools.push(artifact.tool.clone());
        }
    }
    output.overlaps = plan
        .overlaps
        .iter()
        .map(|o| format!("{}: kept {}, skipped {}", o.path, o.kept, o.skipped))
        .collect();
    Ok(output)
}

/// Renders the artifacts for `selected` without writing anything (dry run).
pub fn preview(selected: &[String]) -> Result<Plan, String> {
    ai_tools::plan_selected(selected)
}

fn write(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("cannot write '{}': {}", path.display(), e))
}
