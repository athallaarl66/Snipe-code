mod common;

use snipe_code::generator::{self, GenerateConfig};
use snipe_code::stack;
use snipe_code::template;
use std::fs;
use std::path::{Path, PathBuf};

/// A `templates/`-shaped directory that overrides built-in content.
struct Override(PathBuf);

impl Override {
    fn new(label: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("snipe-override-{}-{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // SAFETY: the caller holds the process-wide test lock via common::Workspace,
        // so no other test observes the variable while it is set.
        unsafe { std::env::set_var(template::OVERRIDE_ENV, &dir) };
        Self(dir)
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.0.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Override {
    fn drop(&mut self) {
        unsafe { std::env::remove_var(template::OVERRIDE_ENV) };
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fe_config(frontend: &str) -> GenerateConfig {
    GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: frontend.to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
        ai_tools: Vec::new(),
    }
}

fn default_content(stack_id: &str, rel: &str) -> String {
    stack::get_frontend_files(stack_id)
        .unwrap()
        .into_iter()
        .find(|file| file.path == rel)
        .unwrap()
        .content
}

/// 2.1 — a local template directory replaces the built-in file it provides.
#[test]
fn test_local_template_directory_overrides_one_file() {
    let _ws = common::Workspace::new("override-one");
    let builtin = default_content("nextjs", "next.config.js");
    let over = Override::new("one");
    let marker = "module.exports = { marker: 'from-local-templates' };\n";
    over.write("frontends/nextjs/files/next.config.js", marker);

    generator::generate(&fe_config("nextjs"), false).unwrap();

    let generated = fs::read_to_string("app/frontend/next.config.js").unwrap();
    assert_eq!(generated, marker, "override file must win");
    assert_ne!(
        generated, builtin,
        "override must differ from the built-in default"
    );
}

/// 2.2 — files the override directory does not provide keep coming from the defaults.
#[test]
fn test_partial_override_keeps_builtin_remainder() {
    let _ws = common::Workspace::new("override-partial");
    let builtin_defaults: Vec<(String, String)> = stack::get_frontend_files("nextjs")
        .unwrap()
        .into_iter()
        .map(|file| (file.path, file.content))
        .collect();
    let over = Override::new("partial");
    over.write(
        "frontends/nextjs/files/next.config.js",
        "module.exports = { marker: 'partial' };\n",
    );

    let result = generator::generate(&fe_config("nextjs"), false).unwrap();

    for (path, content) in builtin_defaults {
        let file = stack::FileDef { path, content };
        let rel = format!("frontend/{}", file.path);
        assert!(
            result.files.contains(&rel),
            "{} must still be generated",
            rel
        );
        let on_disk = fs::read_to_string(format!("app/{}", rel)).unwrap();
        if file.path == "next.config.js" {
            assert_eq!(on_disk, "module.exports = { marker: 'partial' };\n");
        } else {
            assert_eq!(on_disk, file.content, "{} must stay the built-in copy", rel);
        }
    }
}

/// 2.3 — the reported template source differs between the two cases.
#[test]
fn test_reported_template_source_differs() {
    let _ws = common::Workspace::new("override-report");

    assert_eq!(template::source_label(), "built-in templates");
    generator::generate(&fe_config("nextjs"), false).unwrap();
    assert_eq!(template::source_label(), "built-in templates");

    let over = Override::new("report");
    over.write(
        "frontends/nextjs/files/next.config.js",
        "module.exports = { marker: 'reported' };\n",
    );

    generator::generate(&fe_config("nextjs"), true).unwrap();
    let label = template::source_label();
    assert!(
        label.starts_with("local templates from "),
        "unexpected label: {}",
        label
    );
    assert!(
        label.contains(&over.path().display().to_string()),
        "label must name the override directory: {}",
        label
    );
}
