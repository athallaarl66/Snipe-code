//! Derives the gates that apply to a project from what the project declares.
//!
//! Recognition is `audit::detect_scopes` (the existing detector) plus the
//! manifest scan below; gate commands come from the manifests only, never from
//! the stack list this tool generates from.

use std::fs;
use std::path::{Path, PathBuf};

use super::{CommandSpec, Gate, GatePlan, PartPlan};
use crate::audit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ecosystem {
    Npm,
    Composer,
    Go,
    Maven,
    Dotnet,
    Cargo,
}

impl Ecosystem {
    fn name(self) -> &'static str {
        match self {
            Ecosystem::Npm => "npm",
            Ecosystem::Composer => "composer",
            Ecosystem::Go => "go",
            Ecosystem::Maven => "maven",
            Ecosystem::Dotnet => "dotnet",
            Ecosystem::Cargo => "cargo",
        }
    }
}

/// Manifest file names, manifest extensions, and the ecosystem each marks.
const MANIFESTS: &[(&[&str], &[&str], Ecosystem)] = &[
    (&["package.json"], &[], Ecosystem::Npm),
    (&["composer.json"], &[], Ecosystem::Composer),
    (&["go.mod"], &[], Ecosystem::Go),
    (&["pom.xml"], &[], Ecosystem::Maven),
    (&[], &["csproj", "sln"], Ecosystem::Dotnet),
    (&["Cargo.toml"], &[], Ecosystem::Cargo),
];

/// Plans the gates of every recognized part below `root`, in the fixed gate
/// order. `security` adds the dependency vulnerability gate.
pub fn plan(root: &Path, security: bool) -> Vec<PartPlan> {
    let mut candidates: Vec<(PathBuf, Ecosystem)> = Vec::new();
    for (names, extensions, ecosystem) in MANIFESTS {
        let manifests = names
            .iter()
            .flat_map(|name| audit::find_named_files(root, &[name]))
            .chain(
                extensions
                    .iter()
                    .flat_map(|extension| audit::find_files(root, &format!(".{extension}"))),
            );
        for manifest in manifests {
            if let Some(dir) = manifest.parent() {
                candidates.push((dir.to_path_buf(), *ecosystem));
            }
        }
    }
    // Also ask the existing detector, so a layout it recognises is verified
    // even when its manifests sit where the scan above does not look.
    if candidates.is_empty() && audit::detect_scopes(root).len() <= 1 {
        return Vec::new();
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.name().cmp(right.1.name())));
    candidates.dedup();
    let candidates = drop_nested_dotnet(candidates);

    let mut parts = Vec::new();
    for (dir, ecosystem) in candidates {
        let gates = Gate::ORDER
            .iter()
            .filter(|gate| **gate != Gate::Security || security)
            .map(|gate| gate_plan(&dir, ecosystem, *gate))
            .collect();
        parts.push(PartPlan {
            dir,
            root: root.to_path_buf(),
            ecosystem: ecosystem.name(),
            gates,
        });
    }
    parts
}

/// A test project nested under the application project is part of that part,
/// not a second part on its own — the .NET SDK layout relies on this.
fn drop_nested_dotnet(candidates: Vec<(PathBuf, Ecosystem)>) -> Vec<(PathBuf, Ecosystem)> {
    let dotnet_dirs: Vec<PathBuf> = candidates
        .iter()
        .filter(|(_, ecosystem)| *ecosystem == Ecosystem::Dotnet)
        .map(|(dir, _)| dir.clone())
        .collect();
    candidates
        .into_iter()
        .filter(|(dir, ecosystem)| {
            *ecosystem != Ecosystem::Dotnet
                || !dotnet_dirs
                    .iter()
                    .any(|outer| outer != dir && dir.starts_with(outer))
        })
        .collect()
}

fn spec(dir: &Path, program: &str, args: &[&str]) -> CommandSpec {
    CommandSpec::new(program, args, dir)
}

fn gate_plan(dir: &Path, ecosystem: Ecosystem, gate: Gate) -> GatePlan {
    use Ecosystem::{Cargo, Composer, Dotnet, Go, Maven, Npm};
    use Gate::{Build, Install, Lint, Security, Test, Typecheck};

    let (commands, not_applicable): (Vec<CommandSpec>, Option<String>) = match (ecosystem, gate) {
        (Npm, Install) => (vec![spec(dir, "npm", &["install"])], None),
        (Npm, Lint | Typecheck | Test | Build) => npm_script(dir, gate.name()),
        (Npm, Security) => (vec![spec(dir, "npm", &["audit"])], None),

        (Composer, Install) => (
            vec![spec(
                dir,
                "composer",
                &["install", "--no-interaction", "--no-progress"],
            )],
            None,
        ),
        (Composer, Test) => composer_test(dir),
        (Composer, Security) => (vec![spec(dir, "composer", &["audit"])], None),

        (Go, Install) => (
            vec![
                spec(dir, "go", &["mod", "download"]),
                spec(dir, "go", &["mod", "tidy"]),
            ],
            None,
        ),
        (Go, Lint) => (vec![spec(dir, "go", &["vet", "./..."])], None),
        (Go, Test) => (vec![spec(dir, "go", &["test", "./..."])], None),
        (Go, Build) => (vec![spec(dir, "go", &["build", "./..."])], None),
        (Go, Typecheck) => (
            vec![],
            Some("go vet covers type checking; no separate command declared".to_string()),
        ),
        (Go, Security) => (
            vec![],
            Some("govulncheck is not bundled with the Go toolchain".to_string()),
        ),

        (Maven, Install) => (
            vec![],
            Some("Maven resolves dependencies during test and package".to_string()),
        ),
        (Maven, Test) => (vec![spec(dir, "mvn", &["test"])], None),
        (Maven, Build) => (vec![spec(dir, "mvn", &["package", "-DskipTests"])], None),
        (Maven, Security) => (
            vec![],
            Some("OWASP dependency-check is not bundled with Maven".to_string()),
        ),

        (Dotnet, Install) => (vec![spec(dir, "dotnet", &["restore"])], None),
        (Dotnet, Test) => {
            if has_test_project(dir) {
                (vec![spec(dir, "dotnet", &["test", "--no-restore"])], None)
            } else {
                (vec![], Some("no test project declared".to_string()))
            }
        }
        (Dotnet, Build) => (vec![spec(dir, "dotnet", &["build", "--no-restore"])], None),
        (Dotnet, Security) => (
            vec![spec(dir, "dotnet", &["list", "package", "--vulnerable"])],
            None,
        ),

        (Cargo, Test) => (vec![spec(dir, "cargo", &["test"])], None),
        (Cargo, Build) => (vec![spec(dir, "cargo", &["build"])], None),
        (Cargo, Security) => (
            vec![],
            Some("cargo-audit is not bundled with the Rust toolchain".to_string()),
        ),

        // Everything left is a gate the ecosystem does not provide.
        (_, other) => (
            vec![],
            Some(format!("no {} command declared", other.name())),
        ),
    };

    GatePlan {
        gate,
        commands,
        not_applicable,
    }
}

/// An npm gate exists only when the manifest declares that script.
fn npm_script(dir: &Path, name: &str) -> (Vec<CommandSpec>, Option<String>) {
    if crate::generator::declared_scripts(dir)
        .iter()
        .any(|script| script == name)
    {
        (vec![spec(dir, "npm", &["run", name])], None)
    } else {
        (vec![], Some(format!("no \"{name}\" script declared")))
    }
}

/// PHP tests come from the project's own runner: Artisan, else PHPUnit.
fn composer_test(dir: &Path) -> (Vec<CommandSpec>, Option<String>) {
    if dir.join("artisan").exists() {
        (vec![spec(dir, "php", &["artisan", "test"])], None)
    } else if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
        (vec![spec(dir, "php", &["vendor/bin/phpunit"])], None)
    } else {
        (
            vec![],
            Some("no PHPUnit configuration declared".to_string()),
        )
    }
}

/// A .NET part has a test gate only when it declares a test project.
fn has_test_project(dir: &Path) -> bool {
    audit::find_files(dir, ".csproj").iter().any(|project| {
        let name = project
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        name.contains("test")
            || fs::read_to_string(project)
                .map(|text| text.contains("Microsoft.NET.Test.Sdk"))
                .unwrap_or(false)
    })
}
