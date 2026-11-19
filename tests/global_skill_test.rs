use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Home(PathBuf);
impl Home {
    fn new() -> Self {
        let root = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!("snipe-skill-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(args)
            .env("HOME", &self.0)
            .env("USERPROFILE", &self.0)
            .env("HERMES_HOME", self.0.join("hermes-profile"))
            .output()
            .unwrap()
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn preview_install_and_collision_for_supported_agents() {
    let home = Home::new();
    for (agent, relative) in [
        ("claude", ".claude/skills"),
        ("opencode", ".config/opencode/skills"),
        ("codex", ".agents/skills"),
        ("hermes", "hermes-profile/skills"),
        ("cursor", ".cursor/skills"),
        ("github-copilot", ".copilot/skills"),
        ("windsurf", ".codeium/windsurf/skills"),
        ("antigravity", ".gemini/config/skills"),
        ("gemini-cli", ".gemini/skills"),
    ] {
        let path = home.0.join(relative).join("snipe-code/SKILL.md");
        let preview = home.run(&["skill", agent]);
        assert!(preview.status.success(), "{:?}", preview);
        assert!(
            String::from_utf8_lossy(&preview.stdout)
                .replace('\\', "/")
                .contains(&path.to_string_lossy().replace('\\', "/"))
        );
        assert!(!path.exists());
        let installed = home.run(&["skill", agent, "--install"]);
        assert!(installed.status.success(), "{:?}", installed);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("snipe-code analyze <path>"));
        fs::write(&path, "existing user content").unwrap();
        assert!(!home.run(&["skill", agent, "--install"]).status.success());
        assert_eq!(fs::read_to_string(&path).unwrap(), "existing user content");
    }
}

#[test]
fn invalid_arguments_never_write() {
    let home = Home::new();
    for args in [
        vec!["skill"],
        vec!["skill", "unknown-agent", "--install"],
        vec!["skill", "claude", "--install", "extra"],
        vec!["skill", "claude", "--unknown"],
        vec!["skill", "--install"],
    ] {
        let output = home.run(&args);
        assert_eq!(output.status.code(), Some(2), "{:?}", args);
        assert_eq!(fs::read_dir(&home.0).unwrap().count(), 0);
    }
    assert!(home.run(&["help", "skill"]).status.success());
    assert!(home.run(&["skill", "--help"]).status.success());
}

#[test]
fn existing_directory_is_not_modified() {
    let home = Home::new();
    let dir = home.0.join(Path::new(".claude/skills/snipe-code"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("other"), "keep").unwrap();
    assert!(!home.run(&["skill", "claude", "--install"]).status.success());
    assert_eq!(fs::read_to_string(dir.join("other")).unwrap(), "keep");
    assert!(!dir.join("SKILL.md").exists());
}
