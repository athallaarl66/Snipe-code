//! Project-local QA skill contracts; planning uses the real deterministic CLI.

use snipe_code::{ai_tools, ai_workspace, qa, template};
use std::{fs, path::PathBuf, process::Command};

const HOSTS: &[(&str, &str)] = &[
    ("claude-code", ".claude/skills/snipe-qa-planning/SKILL.md"),
    ("opencode", ".opencode/skills/snipe-qa-planning/SKILL.md"),
    ("hermes-agent", ".hermes/skills/snipe-qa-planning/SKILL.md"),
    ("antigravity", ".agents/skills/snipe-qa-planning/SKILL.md"),
];

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("snipe-qa-agent-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn body() -> String {
    template::read("ai-tools/content/testing.md")
        .expect("QA skill body exists")
        .replace("\r\n", "\n")
}

#[test]
fn qa_skill_only_extends_existing_skill_hosts_without_changing_tool_inventory() {
    let tools = ai_tools::load().unwrap();
    let inventory: Vec<_> = tools
        .iter()
        .map(|tool| (tool.id.as_str(), tool.name.as_str()))
        .collect();
    assert_eq!(
        inventory,
        vec![
            ("agents-md", "Cross-tool (AGENTS.md)"),
            ("claude-code", "Claude Code"),
            ("cursor", "Cursor"),
            ("opencode", "OpenCode"),
            ("antigravity", "Antigravity"),
            ("github-copilot", "GitHub Copilot"),
            ("windsurf", "Windsurf"),
            ("hermes-agent", "Hermes Agent"),
            ("codex", "Codex"),
            ("gemini-cli", "Gemini CLI"),
        ]
    );
    for tool in &tools {
        let entries: Vec<_> = tool
            .artifacts
            .iter()
            .filter(|artifact| {
                artifact.template == "testing"
                    || artifact.name.as_deref() == Some("snipe-qa-planning")
            })
            .collect();
        if let Some((_, path)) = HOSTS.iter().find(|(id, _)| *id == tool.id) {
            assert_eq!(entries.len(), 1, "{}", tool.id);
            let entry = entries[0];
            assert_eq!(entry.format, "skill");
            assert_eq!(entry.template, "testing");
            assert_eq!(entry.name.as_deref(), Some("snipe-qa-planning"));
            assert_eq!(entry.resolve_path(&tool.id), *path);
            assert!(entry.description.as_deref().unwrap().contains("planning"));
        } else {
            assert!(entries.is_empty(), "unsupported skill host: {}", tool.id);
        }
    }
}

#[test]
fn every_supported_host_renders_same_read_only_planning_and_approval_contract() {
    let body = body();
    for (id, path) in HOSTS {
        let plan = ai_tools::plan_selected(&[id.to_string()]).unwrap();
        let skill = plan
            .artifacts
            .iter()
            .find(|item| item.path == *path)
            .unwrap();
        let content = skill.content.replace("\r\n", "\n");
        assert!(content.starts_with("---\nname: \"snipe-qa-planning\"\ndescription: \""));
        assert!(content.ends_with(&body));
        for required in [
            "user-selected requirements file",
            "version 1",
            "positive and negative",
            "snipe-code qa plan \"<PROJECT>\" --input \"<SELECTED_FILE>\"",
            "--adapter postman-cli",
            "--export \"<OUTSIDE_PROJECT_NEW_FILE>\"",
            "relative to the current directory",
            "deterministic JSON",
            "does not edit project files",
            "does not run target tests",
            "does not install tools",
            "does not access the network",
            "`method`, `url`, `headers`, `body`, `expected_status`",
            "Collection v2.1",
            "`script.collection`",
            "1.90.0",
            "runtime unverified",
            "`ready-for-review` is not `passed`",
            "separate explicit USER approval",
            "setup approval is not run approval",
            "agent cannot approve on behalf of the user",
            "changed scope requires new approval",
            "install, test execution, endpoint access, and data transmission",
            "local AI/MCP",
            "AI/MCP runtime is deferred",
            "`qa run` and `qa setup` are not available",
            "fake-runner policy library",
            "Do not bypass",
            "direct Postman",
            "`snipe-code verify`",
            "host execution is not a sandbox",
            "Do not claim OS parity",
            "`blocked`",
            "`unsupported`",
        ] {
            assert!(content.contains(required), "{path}: missing {required}");
        }
    }
}

#[test]
fn qa_skill_generation_is_project_local_and_opt_in_per_host() {
    let workspace = Workspace::new();
    let unselected = workspace.0.join("none");
    fs::create_dir(&unselected).unwrap();
    let output = ai_workspace::generate_ai_workspace(unselected.to_str().unwrap(), &[]).unwrap();
    assert!(output.artifacts.is_empty());
    assert!(ai_workspace::preview(&[]).unwrap().artifacts.is_empty());
    for (_, path) in HOSTS {
        assert!(!unselected.join(path).exists());
    }
    for (id, path) in HOSTS {
        let project = workspace.0.join(id);
        fs::create_dir(&project).unwrap();
        let selected = [id.to_string()];
        let preview = ai_workspace::preview(&selected).unwrap();
        assert!(!project.join(path).exists(), "preview must not write");
        let output =
            ai_workspace::generate_ai_workspace(project.to_str().unwrap(), &selected).unwrap();
        assert!(output.artifacts.iter().any(|item| item == path));
        let skill = preview
            .artifacts
            .iter()
            .find(|item| item.path == *path)
            .unwrap();
        assert_eq!(
            fs::read_to_string(project.join(path)).unwrap(),
            skill.content
        );
        for (other, other_path) in HOSTS {
            if id != other {
                assert!(!project.join(other_path).exists());
            }
        }
    }
}

#[test]
fn skill_example_uses_real_cli_and_keeps_postman_readiness_distinct_from_execution() {
    let body = body();
    let example = body
        .split_once("```json\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let workspace = Workspace::new();
    let project = workspace.0.join("project");
    fs::create_dir(&project).unwrap();
    let input = project.join("selected.json");
    fs::write(&input, example).unwrap();
    for adapter in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_snipe-code"));
        command
            .args(["qa", "plan"])
            .arg(&project)
            .arg("--input")
            .arg(&input);
        if adapter {
            command.args(["--adapter", "postman-cli"]);
        }
        let first = command.output().unwrap();
        let second = command.output().unwrap();
        assert!(
            first.status.success(),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );
        assert!(second.status.success());
        assert_eq!(
            first.stdout, second.stdout,
            "planning must be deterministic"
        );
        let actual: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
        let mut expected = qa::plan(&project, &input).unwrap();
        if adapter {
            qa::postman_draft(&mut expected).unwrap();
            assert_eq!(actual["script"]["adapter_version"], "1.90.0");
            assert_eq!(actual["script"]["status"], "ready-for-review");
            assert!(
                actual["script"]["collection"]["info"]["schema"]
                    .as_str()
                    .unwrap()
                    .contains("/v2.1.0/")
            );
        } else {
            assert!(actual.get("script").is_none());
        }
        assert_eq!(actual, serde_json::to_value(expected).unwrap());
        let cases = actual["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 2);
        for case in cases {
            assert_eq!(
                case["status"],
                if adapter { "ready-for-review" } else { "draft" }
            );
        }
        assert_eq!(fs::read_to_string(&input).unwrap(), example);
        assert_eq!(fs::read_dir(&project).unwrap().count(), 1);
    }
}
