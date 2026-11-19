use std::fs::{self, OpenOptions};
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

const SKILL: &str = include_str!("../templates/global-snipe-code-skill.md");
pub const HELP: &str = "Usage: snipe-code skill [<claude|opencode|codex|hermes|cursor|github-copilot|windsurf|antigravity|gemini-cli> [--install]]\nWithout --install, preview only. --install creates the displayed skill without overwriting.\nWith no arguments, choose an agent interactively. Hermes requires HERMES_HOME for the intended profile.";

const AGENTS: &[&str] = &[
    "claude",
    "opencode",
    "codex",
    "hermes",
    "cursor",
    "github-copilot",
    "windsurf",
    "antigravity",
    "gemini-cli",
];

fn install(
    root: &Path,
    write: impl FnOnce(&mut fs::File) -> std::io::Result<()>,
) -> std::io::Result<()> {
    fs::create_dir_all(root)?;
    let dir = root.join("snipe-code");
    fs::create_dir(&dir)?;
    let file = dir.join("SKILL.md");
    let mut output = match OpenOptions::new().write(true).create_new(true).open(&file) {
        Ok(output) => output,
        Err(error) => {
            // Only remove an empty directory: never remove a competing writer's file.
            let _ = fs::remove_dir(&dir);
            return Err(error);
        }
    };
    // A failed write may leave a partial file. Keep it: deleting by pathname could
    // remove another process's replacement or edits instead.
    write(&mut output)
}

pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
            eprintln!("{HELP}");
            return 2;
        }
        let choice = match dialoguer::Select::new()
            .with_prompt("Choose AI for global skill")
            .items(AGENTS)
            .default(0)
            .interact_opt()
        {
            Ok(Some(choice)) => choice,
            Ok(None) => return 0,
            Err(error) => {
                eprintln!("Selection failed: {error}");
                return 3;
            }
        };
        let selected = vec![AGENTS[choice].to_string()];
        let preview = run(&selected);
        if preview != 0 {
            return preview;
        }
        match dialoguer::Confirm::new()
            .with_prompt("Install at this location?")
            .default(false)
            .interact()
        {
            Ok(true) => return run(&[selected[0].clone(), "--install".to_string()]),
            Ok(false) => return 0,
            Err(error) => {
                eprintln!("Confirmation failed: {error}");
                return 3;
            }
        }
    }
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("{HELP}");
        return 0;
    }
    if !(args.len() == 1 || (args.len() == 2 && args[1] == "--install")) {
        eprintln!("{HELP}");
        return 2;
    }
    let agent = args[0].as_str();
    if !AGENTS.contains(&agent) {
        eprintln!("Unsupported agent: {agent}\n{HELP}");
        return 2;
    }
    let base = if agent == "hermes" {
        std::env::var_os("HERMES_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    } else {
        #[cfg(windows)]
        let home = std::env::var_os("USERPROFILE");
        #[cfg(not(windows))]
        let home = std::env::var_os("HOME");
        home.filter(|v| !v.is_empty()).map(PathBuf::from)
    };
    let Some(base) = base else {
        eprintln!(
            "Cannot resolve home for {agent}; set {}.",
            if agent == "hermes" {
                "HERMES_HOME"
            } else if cfg!(windows) {
                "USERPROFILE"
            } else {
                "HOME"
            }
        );
        return 3;
    };
    if !base.is_absolute() {
        eprintln!("Home path must be absolute: {}", base.display());
        return 3;
    }
    let root = match agent {
        "claude" => base.join(".claude/skills"),
        "opencode" => base.join(".config/opencode/skills"),
        "codex" => base.join(".agents/skills"),
        "hermes" => base.join("skills"),
        "cursor" => base.join(".cursor/skills"),
        "github-copilot" => base.join(".copilot/skills"),
        "windsurf" => base.join(".codeium/windsurf/skills"),
        "antigravity" => base.join(".gemini/config/skills"),
        "gemini-cli" => base.join(".gemini/skills"),
        _ => unreachable!(),
    };
    let dir = root.join("snipe-code");
    let file = dir.join("SKILL.md");
    println!("Agent: {agent}\nSkill: {}", file.display());
    if args.len() == 1 {
        println!(
            "Preview only; pass --install to create this skill. Existing skill directories are never overwritten."
        );
        return 0;
    }
    let result = install(&root, |output| output.write_all(SKILL.as_bytes()));
    match result {
        Ok(()) => {
            println!("Installed.");
            0
        }
        Err(e) => {
            eprintln!("Installation failed; existing skill directories are not overwritten: {e}");
            3
        }
    }
}
