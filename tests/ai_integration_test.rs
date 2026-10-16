//! AI agent tool integration: registry, renderers, selection, per-tool
//! artifacts, structured output, and the project-local guarantee.

mod common;

use snipe_code::ai_tools::{self, Artifact, Tool};
use snipe_code::generator::{self, GenerateConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");

/// Registry ids documented in `templates/ai-tools.json` and the README.
const DOCUMENTED_IDS: &[&str] = &[
    "agents-md",
    "claude-code",
    "cursor",
    "opencode",
    "antigravity",
    "github-copilot",
    "windsurf",
    "hermes-agent",
    "codex",
    "gemini-cli",
];

/// Tools that expose a `/` command concept, so they get a command artifact.
const COMMAND_TOOLS: &[&str] = &["claude-code", "opencode", "antigravity"];

/// Agent configuration paths a generator could only write by escaping the
/// project tree. Live writers are left out (`.codex` holds a CLI log/sqlite the
/// tool itself keeps open; IDE install roots churn on their own), because a
/// changed mtime there proves nothing about generation. Their artifact paths are
/// still covered by the per-artifact assertion below.
const HOME_AGENT_PATHS: &[&str] = &[
    ".claude/skills",
    ".claude/commands",
    ".cursor/rules",
    ".opencode/skills",
    ".opencode/commands",
    ".agents/rules",
    ".agents/skills",
    ".agents/workflows",
    ".windsurf/rules",
    ".gemini",
    ".hermes/skills",
];

fn config_for(project: &str, ai_tools: Vec<String>) -> GenerateConfig {
    GenerateConfig {
        project_name: project.to_string(),
        template_id: "none".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools,
    }
}

fn selection(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn planned_paths(tools: &[Tool], id: &str) -> Vec<String> {
    ai_tools::plan(tools, &selection(&[id]))
        .expect("plan resolves")
        .artifacts
        .into_iter()
        .map(|artifact| artifact.path)
        .collect()
}

static SEQ: AtomicU64 = AtomicU64::new(0);

/// A temp directory the binary can be run in, without changing this process's cwd.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("snipe-ai-{}-{}-{}", label, std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs the binary in `dir` with stdin closed (no terminal), returning
/// (stdout, stderr, success).
fn run_bin(dir: &Path, args: &[&str]) -> (String, String, bool) {
    let out = Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("run snipe-code");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .expect("home directory")
}

/// (path, size, mtime) for everything under the given home agent dirs.
fn snapshot(home: &Path) -> Vec<(String, u64, u64)> {
    let mut out = Vec::new();
    for name in HOME_AGENT_PATHS {
        collect_entries(&home.join(name), &mut out);
    }
    out.sort();
    out
}

fn collect_entries(path: &Path, out: &mut Vec<(String, u64, u64)>) {
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if !meta.is_dir() {
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push((path.to_string_lossy().to_string(), meta.len(), modified));
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    children.sort();
    for child in children {
        collect_entries(&child, out);
    }
}

// --- 1.1 registry ---------------------------------------------------------

#[test]
fn test_registry_parses_and_every_entry_resolves() {
    let tools = ai_tools::load().expect("registry parses");
    for id in DOCUMENTED_IDS {
        assert!(
            tools.iter().any(|tool| tool.id == *id),
            "registry must document {id}"
        );
    }
    for tool in &tools {
        assert!(
            !tool.artifacts.is_empty(),
            "{} must name at least one artifact",
            tool.id
        );
        let plan = ai_tools::plan(&tools, &selection(&[&tool.id])).expect("entry resolves");
        assert_eq!(
            plan.artifacts.len(),
            tool.artifacts.len(),
            "{}: every artifact must resolve to a written file",
            tool.id
        );
        for artifact in &plan.artifacts {
            assert!(
                !artifact.content.trim().is_empty(),
                "{}: rendered content is empty",
                artifact.path
            );
        }
    }
}

// --- 1.2 / 1.3 renderers --------------------------------------------------

#[test]
fn test_format_renderers_produce_expected_shape() {
    let body = "Shared body line\n";

    let markdown = normalized(&ai_tools::render("markdown", body, "snipe-code", "d").unwrap());
    assert_eq!(markdown, body, "markdown is the body as-is");

    let mdc = normalized(&ai_tools::render("mdc", body, "snipe-code", "d").unwrap());
    assert!(
        mdc.starts_with("---\ndescription: d\nglobs: **/*\nalwaysApply: true\n---\n\n"),
        "mdc must carry its frontmatter: {mdc}"
    );
    assert!(mdc.ends_with(body));

    let skill = normalized(&ai_tools::render("skill", body, "snipe-code", "d").unwrap());
    assert!(
        skill.starts_with("---\nname: snipe-code\ndescription: d\n---\n\n"),
        "skill must carry name and description frontmatter: {skill}"
    );

    let command = normalized(&ai_tools::render("command", body, "snipe-code", "d").unwrap());
    assert!(
        command.starts_with("---\ndescription: d\n---\n\n"),
        "command must carry its frontmatter: {command}"
    );
    assert!(!command.contains("alwaysApply"));
}

#[test]
fn test_unknown_format_and_unknown_tool_are_errors() {
    let fixture = Tool {
        id: "fixture".to_string(),
        name: "Fixture".to_string(),
        artifacts: vec![Artifact {
            path: "FIXTURE.md".to_string(),
            format: "not-a-format".to_string(),
            template: "shared".to_string(),
            name: None,
            description: None,
        }],
    };
    let error = ai_tools::plan(&[fixture], &selection(&["fixture"])).unwrap_err();
    assert!(
        error.contains("unknown AI artifact format 'not-a-format'"),
        "unknown format must be reported, not skipped: {error}"
    );

    let error = ai_tools::plan(&ai_tools::load().unwrap(), &selection(&["nope"])).unwrap_err();
    assert!(error.contains("unknown AI tool 'nope'"), "{error}");
}

// --- 1.4 overlapping paths ------------------------------------------------

#[test]
fn test_shared_path_is_written_once_and_the_overlap_is_reported() {
    let _ws = common::Workspace::new("ai-overlap");
    let config = config_for("app", selection(&["agents-md", "codex"]));
    let result = generator::generate(&config, false).unwrap();

    assert_eq!(
        result
            .ai_artifacts
            .iter()
            .filter(|path| *path == "AGENTS.md")
            .count(),
        1,
        "AGENTS.md must be reported once: {:?}",
        result.ai_artifacts
    );
    assert_eq!(result.ai_overlaps.len(), 1, "{:?}", result.ai_overlaps);
    assert!(result.ai_overlaps[0].contains("AGENTS.md"));
    assert!(result.ai_overlaps[0].contains("codex"));
    assert!(fs::exists("app/AGENTS.md").unwrap());
    assert_eq!(
        normalized(&fs::read_to_string("app/AGENTS.md").unwrap()),
        normalized(&ai_tools::shared_content().unwrap())
    );
}

#[test]
fn test_gemini_overlap_is_deduplicated_across_antigravity_and_gemini_cli() {
    let _ws = common::Workspace::new("ai-gemini-overlap");
    let config = config_for("app", selection(&["antigravity", "gemini-cli"]));
    let result = generator::generate(&config, false).unwrap();

    assert_eq!(
        result
            .ai_artifacts
            .iter()
            .filter(|path| *path == "GEMINI.md")
            .count(),
        1,
        "{:?}",
        result.ai_artifacts
    );
    assert_eq!(result.ai_overlaps.len(), 1, "{:?}", result.ai_overlaps);
}

// --- 2.1 no selection -----------------------------------------------------

#[test]
fn test_no_selection_writes_no_agent_artifact() {
    let _ws = common::Workspace::new("ai-none");
    let config = config_for("app", Vec::new());
    let result = generator::generate(&config, false).unwrap();

    for path in [
        ".cursorrules",
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        ".claude",
        ".cursor",
        ".opencode",
        ".agents",
        ".windsurf",
        ".hermes",
    ] {
        assert!(
            !Path::new("app").join(path).exists(),
            "{path} must not be written when no tool is selected"
        );
    }
    assert!(result.ai_tools.is_empty());
    assert!(result.ai_artifacts.is_empty());
    assert!(
        !result.files.iter().any(|file| file == "AGENTS.md"),
        "generation report must not claim an agent artifact: {:?}",
        result.files
    );

    // The shared workspace content stays: it is what tools without a dedicated
    // convention read.
    for file in [".ai-workspace/rules.md", ".ai-workspace/rules.mdc"] {
        assert!(fs::exists(format!("app/{file}")).unwrap(), "{file} missing");
        assert!(
            result.files.iter().any(|reported| reported == file),
            "{file} must be reported"
        );
    }
    assert_eq!(
        normalized(&fs::read_to_string("app/.ai-workspace/rules.md").unwrap()),
        normalized(&ai_tools::shared_content().unwrap()),
        "workspace rules render from the same source as the per-tool artifacts"
    );
}

// --- 2.2 / 2.3 producers --------------------------------------------------

#[test]
fn test_non_interactive_run_selects_tools_without_prompting() {
    let dir = TempDir::new("ai-noninteractive");
    let (stdout, stderr, ok) = run_bin(&dir.0, &["new", "--ai-tools", "cursor", "--name", "app"]);
    assert!(ok, "exit status must be success. stderr: {stderr}");
    assert!(
        !stdout.contains("Pilih AI Agent Tools"),
        "no prompt may be issued: {stdout}"
    );

    let project = dir.0.join("app");
    for path in [".cursorrules", ".cursor/rules/snipe-code.mdc"] {
        assert!(project.join(path).exists(), "{path} must exist");
    }
    for path in [
        "AGENTS.md",
        "CLAUDE.md",
        ".claude/commands/snipe-code.md",
        ".opencode/skills/snipe-code/SKILL.md",
    ] {
        assert!(
            !project.join(path).exists(),
            "{path} belongs to an unselected tool"
        );
    }
}

#[test]
fn test_both_selection_producers_yield_identical_generation() {
    let ws = common::Workspace::new("ai-producers");
    let _ = ws;

    let (stdout, stderr, ok) = run_bin(
        Path::new("."),
        &[
            "new",
            "--ai-tools",
            "cursor,claude-code",
            "--name",
            "via-cli",
            "--json",
        ],
    );
    assert!(ok, "exit status must be success. stderr: {stderr}");
    let from_cli: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be structured data");

    let config = config_for(
        "via-config",
        ai_tools::parse_selection("cursor,claude-code"),
    );
    let result = generator::generate(&config, false).unwrap();

    assert_eq!(
        from_cli["files"],
        serde_json::to_value(&result.files).unwrap(),
        "the non-interactive option and the wizard selection must produce the same output"
    );
    assert_eq!(
        from_cli["ai_tools"],
        serde_json::to_value(&result.ai_tools).unwrap()
    );
    assert_eq!(
        from_cli["ai_artifacts"],
        serde_json::to_value(&result.ai_artifacts).unwrap()
    );
}

#[test]
fn test_selection_parsing_accepts_none_and_deduplicates() {
    assert!(ai_tools::parse_selection("none").is_empty());
    assert!(ai_tools::parse_selection("").is_empty());
    assert!(ai_tools::parse_selection("  ").is_empty());
    assert_eq!(
        ai_tools::parse_selection("cursor, cursor ,claude-code"),
        selection(&["cursor", "claude-code"])
    );
    assert_eq!(
        ai_tools::parse_selection("cursor,none"),
        selection(&["cursor"])
    );
}

// --- 3.2 / 3.3 per-tool artifacts -----------------------------------------

#[test]
fn test_each_tool_writes_only_its_own_artifacts() {
    let _ws = common::Workspace::new("ai-per-tool");
    let tools = ai_tools::load().unwrap();

    for tool in &tools {
        let project = format!("app-{}", tool.id);
        let config = config_for(&project, selection(&[&tool.id]));
        let result = generator::generate(&config, false).unwrap();
        let own = planned_paths(&tools, &tool.id);
        assert_eq!(
            result.ai_artifacts, own,
            "{}: reported artifacts must match the registry",
            tool.id
        );
        for path in &own {
            assert!(
                fs::exists(format!("{project}/{path}")).unwrap(),
                "{project}/{path} missing"
            );
        }
        for other in &tools {
            if other.id == tool.id {
                continue;
            }
            for path in planned_paths(&tools, &other.id) {
                if own.contains(&path) {
                    continue; // a shared artifact (AGENTS.md, GEMINI.md)
                }
                assert!(
                    !fs::exists(format!("{project}/{path}")).unwrap(),
                    "{project}/{path} belongs to unselected tool {}",
                    other.id
                );
            }
        }
    }
}

#[test]
fn test_command_artifacts_only_for_tools_with_a_slash_command() {
    let tools = ai_tools::load().unwrap();

    for id in COMMAND_TOOLS {
        let plan = ai_tools::plan(&tools, &selection(&[id])).unwrap();
        let commands: Vec<&str> = plan
            .artifacts
            .iter()
            .filter(|artifact| {
                artifact.path.ends_with("snipe-code.md") || artifact.path.contains("workflows/")
            })
            .map(|artifact| artifact.path.as_str())
            .collect();
        assert!(
            !commands.is_empty(),
            "{id} exposes a slash command, so it must ship a command artifact"
        );
    }

    for id in [
        "agents-md",
        "cursor",
        "github-copilot",
        "windsurf",
        "hermes-agent",
        "codex",
        "gemini-cli",
    ] {
        let plan = ai_tools::plan(&tools, &selection(&[id])).unwrap();
        assert!(
            !plan
                .artifacts
                .iter()
                .any(|artifact| artifact.path.contains("/commands/")
                    || artifact.path.contains("workflows/")),
            "{id} has no slash command concept, so no command artifact may be written"
        );
    }
}

// --- 3.4 dry run ----------------------------------------------------------

#[test]
fn test_dry_run_lists_ai_artifacts_and_writes_nothing() {
    let _ws = common::Workspace::new("ai-dry-run");
    let selected = selection(&["claude-code", "codex"]);
    let config = config_for("app", selected.clone());
    let preview = generator::dry_run_preview(&config);

    for artifact in ai_tools::plan(&ai_tools::load().unwrap(), &selected)
        .unwrap()
        .artifacts
    {
        assert!(
            preview.contains(&artifact.path),
            "preview must list {}",
            artifact.path
        );
    }
    assert!(preview.contains(".ai-workspace/rules.md"));
    assert!(
        !Path::new("app").exists(),
        "a dry run must not create the project"
    );
}

// --- 3.5 project-local guarantee ------------------------------------------

#[test]
fn test_no_home_directory_agent_config_is_created_or_modified() {
    let _ws = common::Workspace::new("ai-home");
    let home = home_dir();
    let before = snapshot(&home);

    let tools = ai_tools::load().unwrap();
    let all: Vec<String> = ai_tools::ids(&tools);
    let config = config_for("app", all);
    generator::generate(&config, false).unwrap();

    let after = snapshot(&home);
    let added: Vec<_> = after
        .iter()
        .filter(|entry| !before.contains(entry))
        .take(10)
        .collect();
    let removed: Vec<_> = before
        .iter()
        .filter(|entry| !after.contains(entry))
        .take(10)
        .collect();
    assert!(
        added.is_empty() && removed.is_empty(),
        "generation touched a home-directory agent configuration — added/changed: {added:?} removed: {removed:?}"
    );

    // The same property stated directly: every artifact the registry can write is
    // project-relative, so none of them may exist under the home directory.
    let plan = ai_tools::plan(&tools, &ai_tools::ids(&tools)).unwrap();
    assert!(!plan.artifacts.is_empty());
    for artifact in &plan.artifacts {
        assert!(
            !home.join(&artifact.path).exists(),
            "{} must never be written under the home directory",
            artifact.path
        );
    }
}

// --- 4.1 / 4.2 structured output and help ---------------------------------

#[test]
fn test_structured_result_on_stdout_keeps_progress_on_stderr() {
    let dir = TempDir::new("ai-json");
    let (stdout, stderr, ok) = run_bin(
        &dir.0,
        &[
            "new",
            "--ai-tools",
            "claude-code",
            "--name",
            "app",
            "--json",
        ],
    );
    assert!(ok, "exit status must be success. stderr: {stderr}");

    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be a single JSON document");
    assert_eq!(value["project"], "app");
    assert_eq!(value["outcome"], "created");
    assert_eq!(value["ai_tools"], serde_json::json!(["claude-code"]));
    let artifacts = value["ai_artifacts"]
        .as_array()
        .expect("ai_artifacts array");
    assert!(
        artifacts
            .iter()
            .any(|path| path == ".claude/skills/snipe-code/SKILL.md"),
        "{artifacts:?}"
    );
    assert!(
        !stdout.contains("Project created") && !stdout.contains("Generating project structure"),
        "progress text must not appear in the structured result: {stdout}"
    );
    assert!(
        stderr.contains("Project created"),
        "progress belongs on stderr: {stderr}"
    );
    assert!(dir.0.join("app/.claude/commands/snipe-code.md").exists());
}

#[test]
fn test_help_documents_the_ai_tool_and_json_options() {
    let dir = TempDir::new("ai-help");
    let (stdout, stderr, ok) = run_bin(&dir.0, &["help", "new"]);
    assert!(ok, "stderr: {stderr}");
    for token in ["--ai-tools", "--json", "--name", "none"] {
        assert!(stdout.contains(token), "help new must document {token}");
    }
    assert!(
        stdout.contains("agents-md") && stdout.contains("claude-code"),
        "help new must list the selectable tool ids"
    );

    let (main_help, _, _) = run_bin(&dir.0, &["help"]);
    for token in ["--ai-tools", "--json"] {
        assert!(main_help.contains(token), "main help must document {token}");
    }
}

// --- 5.2 README stays in sync with the registry ---------------------------

#[test]
fn test_readme_lists_every_registry_tool() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    for tool in ai_tools::load().unwrap() {
        assert!(
            readme.contains(&tool.name),
            "README must list the '{}' tool (registry id {})",
            tool.name,
            tool.id
        );
    }
    for token in ["--ai-tools", "none", "--json", "ai-tools.json"] {
        assert!(readme.contains(token), "README must document {token}");
    }
}
