use snipe_code::verify::report::{self, Format, Request};
use snipe_code::verify::{self, Gate, GateOutcome, GateState, PartOutcome, Report};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn sample(states: Vec<GateState>) -> Report {
    Report {
        root: PathBuf::from("project<&\""),
        parts: vec![PartOutcome {
            label: "part<&\"".into(),
            ecosystem: "npm",
            outcomes: states
                .into_iter()
                .enumerate()
                .map(|(i, state)| GateOutcome {
                    gate: Gate::ORDER[i],
                    state,
                    command: "run <&\"\nline".into(),
                })
                .collect(),
        }],
    }
}

#[test]
fn report_formats_keep_states_counts_and_escaped_data() {
    let report = sample(vec![
        GateState::Passed,
        GateState::Failed("fail <&\"\u{1}".into()),
        GateState::Blocked("denied".into()),
        GateState::Unavailable("missing".into()),
        GateState::NotApplicable("none".into()),
    ]);
    let json: serde_json::Value =
        serde_json::from_str(&verify::render_json(&report).unwrap()).unwrap();
    assert_eq!(json["exit_code"], 1);
    for state in [
        "passed",
        "failed",
        "blocked",
        "unavailable",
        "not-applicable",
    ] {
        assert_eq!(json["counts"][state], 1);
    }
    let md = report::render_markdown(&report);
    for state in [
        "passed",
        "failed",
        "blocked",
        "unavailable",
        "not-applicable",
    ] {
        assert!(md.contains(state));
    }
    assert!(md.contains("fail &lt;&amp;"));
    let xml = report::render_junit(&report);
    for state in [
        "passed",
        "failed",
        "blocked",
        "unavailable",
        "not-applicable",
    ] {
        assert!(xml.contains(&format!("name=\"state\" value=\"{state}\"")));
    }
    assert!(xml.contains("skipped=\"3\""));
    assert!(xml.contains("fail &lt;&amp;&quot;�"));
    assert!(!xml.contains('\u{1}'));
}

#[test]
fn parse_report_flags_rejects_invalid_combinations() {
    for args in [
        vec!["--report", "pdf", "--output", "x"],
        vec!["--report", "json"],
        vec!["--output", "x"],
        vec!["--overwrite"],
        vec!["--report", "--output", "x"],
        vec!["--report", "json", "--output", ""],
        vec!["--report", "json", "--output", "x", "--json"],
        vec!["--report", "md", "--report", "json", "--output", "x"],
    ] {
        assert!(
            report::parse_args(&args.iter().map(|v| v.to_string()).collect::<Vec<_>>()).is_err(),
            "{args:?}"
        );
    }
    let args = [
        "path",
        "--only",
        "test",
        "--report",
        "junit",
        "--output",
        "x",
        "--overwrite",
    ]
    .map(str::to_string);
    let (rest, request) = report::parse_args(&args).unwrap();
    assert_eq!(rest, ["path", "--only", "test"]);
    let request = request.unwrap();
    assert_eq!(request.format, Format::Junit);
    assert!(request.overwrite);
    assert!(verify::Options::parse(&rest).is_ok());
}

#[test]
fn write_collision_failure_and_exit_statuses() {
    let dir = std::env::temp_dir().join(format!(
        "verify-report-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("report.json");
    let mut request = Request {
        format: Format::Json,
        output: path.clone(),
        overwrite: false,
    };
    let cases = [
        (vec![GateState::Passed], 0),
        (vec![GateState::Failed("fail".into())], 1),
        (vec![GateState::Blocked("blocked".into())], 3),
        (
            vec![GateState::Passed, GateState::Unavailable("missing".into())],
            4,
        ),
    ];
    for (states, code) in cases {
        let result = sample(states);
        report::write(&request, &result).unwrap();
        let data: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(data["exit_code"], code);
        assert_eq!(verify::exit_status(&result), code);
        fs::remove_file(&path).unwrap();
    }
    fs::write(&path, "preserve").unwrap();
    assert!(report::write(&request, &Report::default()).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "preserve");
    request.overwrite = true;
    report::write(&request, &Report::default()).unwrap();
    assert!(serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).is_ok());
    request.output = dir.join("missing").join("report");
    assert!(report::write(&request, &Report::default()).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[cfg(unix)]
fn overwrite_replaces_symlink_without_modifying_its_target() {
    use std::os::unix::fs::symlink;

    let dir = std::env::temp_dir().join(format!(
        "verify-report-link-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&dir).unwrap();
    let target = dir.join("target.json");
    let output = dir.join("report.json");
    fs::write(&target, "preserve").unwrap();
    symlink(&target, &output).unwrap();
    let request = Request {
        format: Format::Json,
        output: output.clone(),
        overwrite: true,
    };
    report::write(&request, &Report::default()).unwrap();
    assert_eq!(fs::read_to_string(&target).unwrap(), "preserve");
    assert!(
        !fs::symlink_metadata(&output)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(serde_json::from_slice::<serde_json::Value>(&fs::read(output).unwrap()).is_ok());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn junit_identifies_parts_from_different_ecosystems_uniquely() {
    let mut report = sample(vec![GateState::Passed]);
    report.parts.push(PartOutcome {
        label: report.parts[0].label.clone(),
        ecosystem: "cargo",
        outcomes: vec![GateOutcome {
            gate: Gate::Test,
            state: GateState::Passed,
            command: "cargo test".into(),
        }],
    });
    let xml = report::render_junit(&report);
    assert!(xml.contains("classname=\"part&lt;&amp;&quot; (npm)\" name=\"install\""));
    assert!(xml.contains("classname=\"part&lt;&amp;&quot; (cargo)\" name=\"test\""));
}
