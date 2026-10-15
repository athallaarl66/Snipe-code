//! Gate detection, gate states, and the exit status contract.

use snipe_code::verify::exec::{CommandSpec, RunError};
use snipe_code::verify::{self, Gate, GateState, Options, PartPlan, Report};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A throwaway project directory, removed when the guard drops.
struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let path = std::env::temp_dir().join(format!(
            "snipe-verify-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).expect("create fixture");
        Fixture { path }
    }

    fn write(&self, relative: &str, content: &str) -> &Fixture {
        let target = self.path.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).expect("create fixture dir");
        }
        fs::write(target, content).expect("write fixture file");
        self
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn part<'a>(parts: &'a [PartPlan], ecosystem: &str) -> &'a PartPlan {
    parts
        .iter()
        .find(|part| part.ecosystem == ecosystem)
        .unwrap_or_else(|| panic!("no {ecosystem} part in {parts:?}"))
}

fn commands(part: &PartPlan, gate: Gate) -> Vec<String> {
    part.gates
        .iter()
        .find(|planned| planned.gate == gate)
        .unwrap_or_else(|| panic!("no {} gate", gate.name()))
        .commands
        .iter()
        .map(CommandSpec::display)
        .collect()
}

fn reason(part: &PartPlan, gate: Gate) -> String {
    part.gates
        .iter()
        .find(|planned| planned.gate == gate)
        .unwrap_or_else(|| panic!("no {} gate", gate.name()))
        .not_applicable
        .clone()
        .unwrap_or_else(|| panic!("{} gate is applicable", gate.name()))
}

fn outcomes(report: &Report) -> Vec<(String, String)> {
    report
        .parts
        .iter()
        .flat_map(|part| &part.outcomes)
        .map(|outcome| {
            (
                outcome.gate.name().to_string(),
                outcome.state.name().to_string(),
            )
        })
        .collect()
}

/// A runner that records what ran and fails the command named in `fail_on`.
fn fake_runner<'a>(
    seen: &'a std::cell::RefCell<Vec<String>>,
    fail_on: Option<&'a str>,
    unavailable_on: Option<&'a str>,
) -> impl Fn(&CommandSpec, &str) -> Result<(), RunError> + 'a {
    move |spec, _label| {
        let display = spec.display();
        seen.borrow_mut().push(display.clone());
        if unavailable_on == Some(spec.program.as_str()) {
            return Err(RunError::Unavailable(format!(
                "{} unavailable: not found",
                spec.program
            )));
        }
        if fail_on.is_some_and(|needle| display.contains(needle)) {
            return Err(RunError::Failed(format!(
                "{} {:?} exited with exit code: 1",
                spec.program, spec.args
            )));
        }
        Ok(())
    }
}

#[test]
fn npm_gates_come_from_declared_scripts() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"lint":"eslint .","test":"vitest run","build":"vite build"}}"#,
    );
    let parts = verify::detect::plan(fixture.path(), false);
    let npm = part(&parts, "npm");
    assert_eq!(commands(npm, Gate::Install), ["npm install"]);
    assert_eq!(commands(npm, Gate::Lint), ["npm run lint"]);
    assert_eq!(commands(npm, Gate::Test), ["npm run test"]);
    assert_eq!(commands(npm, Gate::Build), ["npm run build"]);
    assert!(reason(npm, Gate::Typecheck).contains("typecheck"));
    // The security gate is opt-in, so it is not planned by default.
    assert!(
        !npm.gates
            .iter()
            .any(|planned| planned.gate == Gate::Security)
    );
}

#[test]
fn composer_gates_follow_the_projects_own_test_runner() {
    let phpunit = Fixture::new();
    phpunit.write("composer.json", r#"{"name":"app/php"}"#);
    phpunit.write("phpunit.xml", "<phpunit/>");
    let parts = verify::detect::plan(phpunit.path(), false);
    let composer = part(&parts, "composer");
    assert_eq!(
        commands(composer, Gate::Install),
        ["composer install --no-interaction --no-progress"]
    );
    assert_eq!(commands(composer, Gate::Test), ["php vendor/bin/phpunit"]);

    let laravel = Fixture::new();
    laravel.write("composer.json", r#"{"name":"app/laravel"}"#);
    laravel.write("artisan", "#!/usr/bin/env php\n");
    let parts = verify::detect::plan(laravel.path(), false);
    assert_eq!(
        commands(part(&parts, "composer"), Gate::Test),
        ["php artisan test"]
    );

    let bare = Fixture::new();
    bare.write("composer.json", r#"{"name":"app/bare"}"#);
    let parts = verify::detect::plan(bare.path(), false);
    assert!(reason(part(&parts, "composer"), Gate::Test).contains("PHPUnit"));
}

#[test]
fn go_gates_cover_install_lint_test_build() {
    let fixture = Fixture::new();
    fixture.write("go.mod", "module example.com/app\n");
    let parts = verify::detect::plan(fixture.path(), true);
    let go = part(&parts, "go");
    assert_eq!(
        commands(go, Gate::Install),
        ["go mod download", "go mod tidy"]
    );
    assert_eq!(commands(go, Gate::Lint), ["go vet ./..."]);
    assert_eq!(commands(go, Gate::Test), ["go test ./..."]);
    assert_eq!(commands(go, Gate::Build), ["go build ./..."]);
}

#[test]
fn maven_gates_separate_test_from_build() {
    let fixture = Fixture::new();
    fixture.write("pom.xml", "<project/>");
    let parts = verify::detect::plan(fixture.path(), false);
    let maven = part(&parts, "maven");
    assert_eq!(commands(maven, Gate::Test), ["mvn test"]);
    assert_eq!(commands(maven, Gate::Build), ["mvn package -DskipTests"]);
    assert!(reason(maven, Gate::Install).contains("Maven"));
}

#[test]
fn dotnet_test_project_is_part_of_the_application_part() {
    let fixture = Fixture::new();
    fixture.write("app.csproj", "<Project Sdk=\"Microsoft.NET.Sdk.Web\"/>");
    fixture.write(
        "tests/app.Tests.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\"><ItemGroup>\
         <PackageReference Include=\"Microsoft.NET.Test.Sdk\" Version=\"17.0.0\"/>\
         </ItemGroup></Project>",
    );
    let parts = verify::detect::plan(fixture.path(), false);
    assert_eq!(
        parts.len(),
        1,
        "nested test project must not be its own part"
    );
    let dotnet = part(&parts, "dotnet");
    assert_eq!(commands(dotnet, Gate::Install), ["dotnet restore"]);
    assert_eq!(commands(dotnet, Gate::Test), ["dotnet test --no-restore"]);
    assert_eq!(commands(dotnet, Gate::Build), ["dotnet build --no-restore"]);

    let without_tests = Fixture::new();
    without_tests.write("app.csproj", "<Project Sdk=\"Microsoft.NET.Sdk.Web\"/>");
    let parts = verify::detect::plan(without_tests.path(), false);
    assert!(reason(part(&parts, "dotnet"), Gate::Test).contains("test project"));
}

#[test]
fn multiple_parts_run_against_their_own_directory() {
    let fixture = Fixture::new();
    fixture.write(
        "frontend/package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    fixture.write("backend/go.mod", "module example.com/app\n");
    let parts = verify::detect::plan(fixture.path(), false);
    assert_eq!(parts.len(), 2);
    let frontend = part(&parts, "npm");
    let backend = part(&parts, "go");
    assert!(frontend.dir.ends_with("frontend"));
    assert!(backend.dir.ends_with("backend"));
    assert_eq!(frontend.label(), "frontend");
    assert_eq!(backend.label(), "backend");
    assert_eq!(commands(backend, Gate::Test), ["go test ./..."]);
}

#[test]
fn unrecognized_directory_has_no_gates_and_nothing_executes() {
    let fixture = Fixture::new();
    fixture.write("notes.txt", "not a project\n");
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report =
        verify::run(fixture.path(), &options, &fake_runner(&seen, None, None)).expect("run");
    assert!(report.parts.is_empty());
    assert!(seen.borrow().is_empty(), "no command may run");
    assert_eq!(verify::exit_status(&report), 3);
}

#[test]
fn gate_states_are_recorded_in_the_fixed_order() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run","build":"vite build"}}"#,
    );
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report = verify::run(
        fixture.path(),
        &options,
        &fake_runner(&seen, Some("run build"), None),
    )
    .expect("run");

    assert_eq!(
        outcomes(&report)
            .iter()
            .map(|(gate, _)| gate.clone())
            .collect::<Vec<_>>(),
        ["install", "lint", "typecheck", "test", "build"]
    );
    assert_eq!(
        outcomes(&report)
            .iter()
            .map(|(_, state)| state.clone())
            .collect::<Vec<_>>(),
        [
            "passed",
            "not-applicable",
            "not-applicable",
            "passed",
            "failed"
        ]
    );
    let (part_label, gate, command, message) = report.first_failure().expect("a failing gate");
    assert_eq!(part_label, ".");
    assert_eq!(gate, Gate::Build);
    assert_eq!(command, "npm run build");
    assert!(
        message.contains("build"),
        "message names the command: {message}"
    );
    assert_eq!(verify::exit_status(&report), 1);
}

#[test]
fn unavailable_gate_is_reported_but_only_blocks_success_when_nothing_ran() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: false,
    };

    // A missing npm means nothing ran at all, which is not success.
    let seen = std::cell::RefCell::new(Vec::new());
    let report = verify::run(
        fixture.path(),
        &options,
        &fake_runner(&seen, None, Some("npm")),
    )
    .expect("run");
    assert_eq!(report.count("unavailable"), 2);
    assert_eq!(verify::exit_status(&report), 3, "nothing executed");

    let fixture = Fixture::new();
    fixture.write("go.mod", "module example.com/app\n");
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report = verify::run(
        fixture.path(),
        &options,
        &fake_runner(&seen, None, Some("go")),
    )
    .expect("run");
    assert!(
        report.parts[0]
            .outcomes
            .iter()
            .all(|outcome| !matches!(outcome.state.name(), "passed" | "failed"))
    );
    assert_eq!(report.count("unavailable"), 4);
    assert_eq!(report.executed(), 0);
    assert_eq!(verify::exit_status(&report), 3);
}

#[test]
fn only_selector_restricts_the_run_and_rejects_a_gate_that_does_not_apply() {
    let fixture = Fixture::new();
    fixture.write(
        "package.json",
        r#"{"name":"app","scripts":{"test":"vitest run"}}"#,
    );
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: Some(Gate::Test),
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report =
        verify::run(fixture.path(), &options, &fake_runner(&seen, None, None)).expect("run");
    assert_eq!(seen.borrow().as_slice(), ["npm run test"]);
    assert_eq!(
        outcomes(&report),
        [("test".to_string(), "passed".to_string())]
    );
    assert_eq!(verify::exit_status(&report), 0);

    let rejected = Options {
        path: fixture.path().to_path_buf(),
        only: Some(Gate::Build),
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let error = verify::run(fixture.path(), &rejected, &fake_runner(&seen, None, None))
        .expect_err("selector must be rejected");
    assert!(error.contains("build"), "{error}");
    assert!(error.contains("does not apply"), "{error}");
    assert!(seen.borrow().is_empty(), "rejected selector runs nothing");
}

#[test]
fn security_gate_uses_the_scanner_each_ecosystem_ships() {
    let cases = [
        ("package.json", r#"{"name":"app"}"#, "npm audit"),
        ("composer.json", r#"{"name":"app/php"}"#, "composer audit"),
        (
            "app.csproj",
            "<Project Sdk=\"Microsoft.NET.Sdk.Web\"/>",
            "dotnet list package --vulnerable",
        ),
    ];
    for (manifest, content, expected) in cases {
        let fixture = Fixture::new();
        fixture.write(manifest, content);
        let parts = verify::detect::plan(fixture.path(), true);
        let planned = parts[0]
            .gates
            .iter()
            .find(|planned| planned.gate == Gate::Security)
            .expect("security gate");
        assert_eq!(
            planned
                .commands
                .iter()
                .map(CommandSpec::display)
                .collect::<Vec<_>>(),
            [expected],
            "{manifest}"
        );
    }
}

#[test]
fn ecosystems_without_a_bundled_scanner_state_why() {
    let go = Fixture::new();
    go.write("go.mod", "module example.com/app\n");
    let parts = verify::detect::plan(go.path(), true);
    let go_reason = reason(part(&parts, "go"), Gate::Security);
    assert!(go_reason.contains("govulncheck"), "{go_reason}");

    let java = Fixture::new();
    java.write("pom.xml", "<project/>");
    let parts = verify::detect::plan(java.path(), true);
    let java_reason = reason(part(&parts, "maven"), Gate::Security);
    assert!(java_reason.contains("dependency-check"), "{java_reason}");
}

#[test]
fn failing_scan_fails_the_run_and_the_gate_is_absent_unless_requested() {
    let fixture = Fixture::new();
    fixture.write("package.json", r#"{"name":"app"}"#);

    let without = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report = verify::run(
        fixture.path(),
        &without,
        &fake_runner(&seen, Some("audit"), None),
    )
    .expect("run");
    assert!(
        !report.parts[0]
            .outcomes
            .iter()
            .any(|outcome| outcome.gate == Gate::Security),
        "security gate must not appear when it was not requested"
    );

    let with = Options {
        path: fixture.path().to_path_buf(),
        only: Some(Gate::Security),
        security: true,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report = verify::run(
        fixture.path(),
        &with,
        &fake_runner(&seen, Some("npm audit"), None),
    )
    .expect("run");
    assert_eq!(seen.borrow().as_slice(), ["npm audit"]);
    assert_eq!(
        outcomes(&report),
        [("security".to_string(), "failed".to_string())]
    );
    assert_eq!(verify::exit_status(&report), 1);
}

#[test]
fn passing_scan_passes_the_gate() {
    let fixture = Fixture::new();
    fixture.write("package.json", r#"{"name":"app"}"#);
    let options = Options {
        path: fixture.path().to_path_buf(),
        only: None,
        security: true,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let report =
        verify::run(fixture.path(), &options, &fake_runner(&seen, None, None)).expect("run");
    let security = report.parts[0]
        .outcomes
        .iter()
        .find(|outcome| outcome.gate == Gate::Security)
        .expect("security gate");
    assert_eq!(security.state, GateState::Passed);
    assert_eq!(security.command, "npm audit");
    assert_eq!(verify::exit_status(&report), 0);
}

#[test]
fn options_parse_path_selectors_and_usage_errors() {
    let none = Options::parse(&[]).expect("defaults");
    assert_eq!(none.path, PathBuf::from("."));
    assert_eq!(none.only, None);
    assert!(!none.security);

    let full = Options::parse(&[
        "..".to_string(),
        "--only".to_string(),
        "test".to_string(),
        "--security".to_string(),
    ])
    .expect("parse");
    assert_eq!(full.path, PathBuf::from(".."));
    assert_eq!(full.only, Some(Gate::Test));
    assert!(full.security);

    // Selecting the security gate implies requesting it.
    let implied = Options::parse(&["--only".to_string(), "security".to_string()]).expect("parse");
    assert!(implied.security);

    for (args, expected) in [
        (vec!["--only".to_string()], "missing value"),
        (vec!["--nope".to_string()], "unknown option"),
        (
            vec!["--only".to_string(), "deploy".to_string()],
            "unknown gate",
        ),
        (
            vec!["a".to_string(), "b".to_string()],
            "unexpected argument",
        ),
    ] {
        let error = Options::parse(&args).expect_err("usage error expected");
        assert!(error.contains(expected), "{args:?} gave {error}");
    }
}

#[test]
fn missing_directory_is_a_usage_error() {
    let options = Options {
        path: PathBuf::from("snipe-code-no-such-directory"),
        only: None,
        security: false,
    };
    let seen = std::cell::RefCell::new(Vec::new());
    let error = verify::run(&options.path, &options, &fake_runner(&seen, None, None))
        .expect_err("missing path");
    assert!(error.contains("not a directory"), "{error}");
}
