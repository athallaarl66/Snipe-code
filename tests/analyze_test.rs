//! `snipe-code analyze` — the read-only project report.
//!
//! Each test builds a fixture with the exact content it asserts on, so a failure
//! names the behaviour that broke rather than a missing file.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use snipe_code::analyze::{self, LanguageSize};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");
static SEQ: AtomicU64 = AtomicU64::new(0);

/// A throwaway directory with the exact content a test needs.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let seq = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "snipe-analyze-{label}-{}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        // Canonicalise: MSYS hands out 8.3 short temp paths, which would leak
        // into the reported paths.
        let dir = fs::canonicalize(&dir).expect("canonicalise fixture");
        Fixture { dir }
    }

    fn write(&self, relative: &str, content: &str) -> PathBuf {
        let path = self.dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, content).expect("write fixture file");
        path
    }

    fn write_bytes(&self, relative: &str, content: &[u8]) -> PathBuf {
        let path = self.dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, content).expect("write fixture file");
        path
    }

    fn path(&self) -> &str {
        self.dir.to_str().expect("fixture path is utf-8")
    }

    fn report(&self) -> analyze::AnalysisReport {
        analyze::analyze(&self.dir).expect("analyze fixture")
    }

    fn run(&self, args: &[&str]) -> (String, String, Option<i32>) {
        let out = Command::new(BIN)
            .args(args)
            .output()
            .expect("run snipe-code");
        (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code(),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn language(report: &analyze::AnalysisReport, name: &str) -> Option<LanguageSize> {
    report
        .languages
        .iter()
        .find(|entry| entry.language == name)
        .cloned()
}

// --- 1.1 languages and size ------------------------------------------------

#[test]
fn test_language_totals_over_a_multi_language_fixture() {
    let fixture = Fixture::new("languages");
    fixture.write("src/main.rs", "fn main() {}\n");
    fixture.write(
        "src/lib.rs",
        "pub fn a() {}\npub fn b() {}\npub fn c() {}\n",
    );
    fixture.write("web/app.ts", "const a = 1;\nexport default a;\n");
    fixture.write("web/style.css", "body { margin: 0; }\n");

    let report = fixture.report();

    assert_eq!(
        language(&report, "Rust"),
        Some(LanguageSize {
            language: "Rust".to_string(),
            files: 2,
            lines: 4,
        })
    );
    assert_eq!(
        language(&report, "TypeScript"),
        Some(LanguageSize {
            language: "TypeScript".to_string(),
            files: 1,
            lines: 2,
        })
    );
    assert_eq!(
        language(&report, "CSS"),
        Some(LanguageSize {
            language: "CSS".to_string(),
            files: 1,
            lines: 1,
        })
    );
    assert!(
        language(&report, "Go").is_none(),
        "a language with no files must not be reported: {:?}",
        report.languages
    );
}

// --- 1.2 excluded directories ---------------------------------------------

#[test]
fn test_excluded_directories_are_not_counted() {
    let fixture = Fixture::new("excluded");
    fixture.write("src/app.js", "export const a = 1;\n");
    for (dir, file) in [
        ("node_modules/react", "index.js"),
        ("target/debug", "build.js"),
        ("dist", "bundle.js"),
        ("build", "out.js"),
        ("vendor", "lib.js"),
        ("coverage", "report.js"),
        (".next", "page.js"),
    ] {
        fixture.write(&format!("{dir}/{file}"), "const one = 1;\nconst two = 2;\n");
    }

    let report = fixture.report();

    assert_eq!(
        language(&report, "JavaScript"),
        Some(LanguageSize {
            language: "JavaScript".to_string(),
            files: 1,
            lines: 1,
        }),
        "only src/app.js may be counted: {:?}",
        report.languages
    );
}

// --- 1.3 structure and entry points ---------------------------------------

#[test]
fn test_source_directory_bin_is_not_treated_as_build_output() {
    let fixture = Fixture::new("src-bin");
    fixture.write("src/main.rs", "fn main() {}\n");
    fixture.write("src/bin/tool.rs", "fn main() {}\n");
    fixture.write("bin/tool.rs", "fn main() {}\n");

    let report = fixture.report();

    assert_eq!(
        language(&report, "Rust"),
        Some(LanguageSize {
            language: "Rust".to_string(),
            files: 2,
            lines: 2,
        }),
        "src/bin holds real source; the top-level bin does not: {:?}",
        report.languages
    );
}

#[test]
fn test_structure_and_entry_point_are_reported() {
    let fixture = Fixture::new("structure");
    fixture.write("frontend/package.json", "{\"name\": \"app\"}\n");
    fixture.write(
        "frontend/src/main.tsx",
        "export default function App() {}\n",
    );
    fixture.write("src/main.rs", "fn main() {}\n");

    let report = fixture.report();

    assert!(
        report.structure.top_level.contains(&"frontend".to_string()),
        "top level must name frontend: {:?}",
        report.structure.top_level
    );
    assert!(
        report.structure.parts.iter().any(|part| part == "frontend"),
        "frontend must be a recognized part: {:?}",
        report.structure.parts
    );
    assert!(
        report
            .entry_points
            .iter()
            .any(|entry| entry.part == "frontend" && entry.path == "frontend/src/main.tsx"),
        "frontend entry point missing: {:?}",
        report.entry_points
    );
    assert!(
        report
            .entry_points
            .iter()
            .any(|entry| entry.path == "src/main.rs"),
        "root entry point missing: {:?}",
        report.entry_points
    );
    assert!(
        !report
            .limits
            .iter()
            .any(|limit| limit.contains("no entry point")),
        "entry points were found, so that limit must not be claimed: {:?}",
        report.limits
    );
}

// --- 1.4 manifest dependencies --------------------------------------------

#[test]
fn test_declared_dependencies_are_listed_per_manifest() {
    let fixture = Fixture::new("manifests");
    fixture.write(
        "package.json",
        "{\"dependencies\": {\"react\": \"^18\"}, \"devDependencies\": {\"vitest\": \"^2\"}}\n",
    );
    fixture.write(
        "composer.json",
        "{\"require\": {\"laravel/framework\": \"^12\"}, \"require-dev\": {\"phpunit/phpunit\": \"^11\"}}\n",
    );
    fixture.write(
        "go.mod",
        "module example\n\ngo 1.21\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.0\n)\n",
    );
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"example\"\n\n[dependencies]\nserde = \"1\"\nserde_json = \"1\"\n",
    );
    fixture.write(
        "pom.xml",
        "<project>\n  <artifactId>example</artifactId>\n  <dependencies>\n    <dependency>\n      <groupId>org.springframework.boot</groupId>\n      <artifactId>spring-boot-starter-web</artifactId>\n    </dependency>\n  </dependencies>\n</project>\n",
    );
    fixture.write(
        "app.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <ItemGroup>\n    <PackageReference Include=\"xunit\" Version=\"2.6.0\" />\n  </ItemGroup>\n</Project>\n",
    );

    let report = fixture.report();
    let dependencies_of = |name: &str| -> Vec<String> {
        report
            .dependencies
            .iter()
            .find(|manifest| manifest.manifest == name)
            .unwrap_or_else(|| panic!("manifest {name} missing: {:?}", report.dependencies))
            .dependencies
            .clone()
    };

    assert_eq!(dependencies_of("package.json"), ["react", "vitest"]);
    assert_eq!(
        dependencies_of("composer.json"),
        ["laravel/framework", "phpunit/phpunit"]
    );
    assert_eq!(dependencies_of("go.mod"), ["github.com/gin-gonic/gin"]);
    assert_eq!(dependencies_of("Cargo.toml"), ["serde", "serde_json"]);
    assert_eq!(
        dependencies_of("pom.xml"),
        ["spring-boot-starter-web"],
        "the project's own artifactId must not be read as a dependency"
    );
    assert_eq!(dependencies_of("app.csproj"), ["xunit"]);

    let scanned = report
        .dependencies
        .iter()
        .filter(|manifest| !manifest.parsed)
        .map(|manifest| manifest.manifest.as_str())
        .collect::<Vec<_>>();
    assert_eq!(scanned, ["app.csproj", "pom.xml"]);
}

#[test]
fn test_manifest_declaring_nothing_says_so() {
    let fixture = Fixture::new("empty-manifest");
    fixture.write("composer.json", "{\"name\": \"example/app\"}\n");

    let report = fixture.report();
    assert_eq!(report.dependencies.len(), 1, "{:?}", report.dependencies);
    assert!(report.dependencies[0].dependencies.is_empty());
    assert!(
        analyze::render_human(&report).contains("composer.json (parsed)"),
        "the manifest must still be named: {}",
        analyze::render_human(&report)
    );
    assert!(
        analyze::render_human(&report).contains("declares none"),
        "an empty manifest must say it declares none: {}",
        analyze::render_human(&report)
    );
}

// --- 1.5 limits ------------------------------------------------------------

#[test]
fn test_limits_name_content_that_is_not_analyzed() {
    let fixture = Fixture::new("limits");
    fixture.write("data/dump.zzz", "not a language\n");
    fixture.write("notes.tmp", "not a language\n");
    fixture.write_bytes("broken.rs", &[0xff, 0xfe, 0x00, 0x01]);
    fixture.write("src/main.rs", "fn main() {}\n");

    let report = fixture.report();
    let limits = report.limits.join("\n");

    assert!(
        limits.contains("content not analyzed") && limits.contains(".zzz"),
        "unrecognized content must be named: {limits}"
    );
    assert!(
        limits.contains("could not be read"),
        "unreadable content must be named: {limits}"
    );
    assert!(
        limits.contains("line counts"),
        "the line-count caveat must be stated: {limits}"
    );
}

#[test]
fn test_xml_manifest_scan_is_declared_as_a_limit() {
    let fixture = Fixture::new("xml-limit");
    fixture.write(
        "pom.xml",
        "<project>\n  <dependencies>\n    <dependency>\n      <artifactId>x</artifactId>\n    </dependency>\n  </dependencies>\n</project>\n",
    );

    let report = fixture.report();
    assert!(
        report.limits.iter().any(|limit| limit.contains("scanned")),
        "a scanned manifest must be reported as a limit: {:?}",
        report.limits
    );
}

// --- 1.6 the command itself ------------------------------------------------

#[test]
fn test_analyze_command_prints_the_human_report() {
    let fixture = Fixture::new("cli");
    fixture.write("src/main.rs", "fn main() {}\n");

    let (stdout, _, status) = fixture.run(&["analyze", fixture.path()]);

    assert_eq!(status, Some(0), "{stdout}");
    for expected in [
        "Languages",
        "Rust",
        "1 file(s)",
        "1 lines",
        "Structure",
        "Entry points",
        "src/main.rs",
        "Dependencies",
        "Limits",
    ] {
        assert!(
            stdout.contains(expected),
            "report is missing {expected}: {stdout}"
        );
    }
}

#[test]
fn test_analyze_exit_status_contract() {
    let fixture = Fixture::new("status");
    fixture.write("src/main.rs", "fn main() {}\n");

    let missing_path = format!("{}/nope", fixture.path());
    let (_, stderr, missing) = fixture.run(&["analyze", &missing_path]);
    assert_eq!(
        missing,
        Some(3),
        "a missing folder is a runtime error: {stderr}"
    );

    let (_, stderr, unknown) = fixture.run(&["analyze", fixture.path(), "--bogus"]);
    assert_eq!(
        unknown,
        Some(2),
        "an unknown option is a usage error: {stderr}"
    );
    assert!(stderr.contains("--bogus"), "{stderr}");
}

// --- 2.1 structured output -------------------------------------------------

#[test]
fn test_structured_report_matches_the_human_report() {
    let fixture = Fixture::new("structured");
    fixture.write(
        "frontend/package.json",
        "{\"dependencies\": {\"vue\": \"^3\"}}\n",
    );
    fixture.write("frontend/src/main.ts", "export const a = 1;\n");
    fixture.write("src/main.rs", "fn main() {}\n");
    fixture.write("data/blob.zzz", "unknown\n");

    let (stdout, _, status) = fixture.run(&["analyze", fixture.path(), "--json"]);
    assert_eq!(status, Some(0), "{stdout}");

    let structured: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("stdout is not JSON: {error}"));
    let report = fixture.report();
    assert_eq!(
        structured,
        serde_json::to_value(&report).expect("report serializes"),
        "structured output must carry the same findings as the report"
    );

    let human = analyze::render_human(&report);
    let languages = structured["languages"].as_array().expect("languages array");
    for entry in languages {
        let name = entry["language"].as_str().expect("language name");
        let files = entry["files"].as_u64().expect("file count");
        assert!(
            human.contains(name) && human.contains(&format!("{files} file(s)")),
            "human report disagrees with structured output for {name}: {human}"
        );
    }
    for part in structured["structure"]["parts"]
        .as_array()
        .expect("parts array")
    {
        assert!(
            human.contains(part.as_str().expect("part")),
            "human report is missing part {part}: {human}"
        );
    }
    for dependency in structured["dependencies"].as_array().expect("dependencies") {
        let manifest = dependency["manifest"].as_str().expect("manifest");
        assert!(
            human.contains(manifest),
            "human report is missing manifest {manifest}: {human}"
        );
    }
    for limit in structured["limits"].as_array().expect("limits array") {
        assert!(
            human.contains(limit.as_str().expect("limit")),
            "human report is missing limit {limit}: {human}"
        );
    }
}

#[test]
fn test_analyze_does_not_modify_the_project() {
    let fixture = Fixture::new("read-only");
    fixture.write("src/main.rs", "fn main() {}\n");
    let before = snapshot(&fixture.dir);

    let (_, _, status) = fixture.run(&["analyze", fixture.path()]);
    assert_eq!(status, Some(0));

    assert_eq!(
        snapshot(&fixture.dir),
        before,
        "analyze wrote into the project"
    );
}

/// Every relative path and size under `dir`, for the read-only assertion.
fn snapshot(dir: &std::path::Path) -> Vec<(String, u64)> {
    let mut entries = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).expect("read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(dir)
                    .expect("inside fixture")
                    .to_string_lossy()
                    .to_string();
                entries.push((relative, entry.metadata().expect("metadata").len()));
            }
        }
    }
    entries.sort();
    entries
}
