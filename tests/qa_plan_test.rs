use snipe_code::qa::{CaseStatus, Category, TestCase};

#[test]
fn test_case_schema_round_trips_categories_and_trace_refs() {
    let categories = [
        (Category::Positive, "positive"),
        (Category::Negative, "negative"),
        (Category::Boundary, "boundary"),
        (Category::Authorization, "authorization"),
        (Category::Validation, "validation"),
        (Category::Regression, "regression"),
    ];
    for (category, expected_category) in categories {
        let case = TestCase {
            id: format!("case-{expected_category}"),
            objective: "Check requirement".into(),
            precondition: "Fixture exists".into(),
            input: "Perform action".into(),
            expected_result: "Expected outcome".into(),
            category,
            requirement_refs: vec!["REQ-1".into()],
            source_refs: vec!["src/example.rs:1".into()],
            adapter: None,
            adapter_version: None,
            status: CaseStatus::Draft,
        };
        let json = serde_json::to_value(&case).unwrap();
        assert_eq!(json["category"], expected_category);
        assert_eq!(json["requirement_refs"][0], "REQ-1");
        assert_eq!(json["source_refs"][0], "src/example.rs:1");
        assert_eq!(json["status"], "draft");
        assert_eq!(serde_json::from_value::<TestCase>(json).unwrap(), case);
    }
    assert!(serde_json::from_str::<CaseStatus>("\"passed\"").is_err());
}

#[test]
fn planning_preserves_tree_and_ignores_executable_project_content() {
    use std::collections::BTreeMap;
    use std::fs;
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
        fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                if path.is_dir() {
                    files.insert(relative, None);
                    visit(root, &path, files);
                } else {
                    files.insert(relative, Some(fs::read(path).unwrap()));
                }
            }
        }
        let mut files = BTreeMap::new();
        visit(root, root, &mut files);
        files
    }

    let base = std::env::temp_dir().join(format!("snipe-qa-{}", uuid::Uuid::new_v4()));
    let project = base.join("project");
    fs::create_dir_all(project.join("src/nested")).unwrap();
    fs::write(project.join("src/nested/canary.txt"), "unchanged").unwrap();
    // A local observer, not a target service: planning must never contact it.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/must-not-request", listener.local_addr().unwrap());
    let input = project.join("requirements.json");
    let scenarios: Vec<_> = [("ok", "positive"), ("bad", "negative")]
        .into_iter()
        .map(|(id, category)| {
            serde_json::json!({
                "id": id, "category": category, "objective": "Review only",
                "precondition": "No execution approved", "input": endpoint,
                "expected_result": "No request during planning"
            })
        })
        .collect();
    fs::write(
        &input,
        serde_json::to_vec(&serde_json::json!({
            "version": 1, "requirements": [{"id": "REQ-1", "scenarios": scenarios}]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        project.join("package.json"),
        r#"{"scripts":{"preinstall":"exit 99","test":"exit 99"}}"#,
    )
    .unwrap();
    let before = snapshot(&base);
    let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .args(["qa", "plan"])
        .arg(&project)
        .arg("--input")
        .arg(&input)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(snapshot(&base), before);
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["cases"][0]["input"], endpoint);
    assert_eq!(value["cases"][0]["status"], "draft");
    // Static evidence complements the runtime canaries: this module has no
    // process or network boundary, including fully qualified calls.
    let source = include_str!("../src/qa.rs");
    for forbidden in [
        "Command",
        "std::process",
        "std::net",
        "reqwest",
        "TcpStream",
    ] {
        assert!(
            !source.contains(forbidden),
            "unexpected effect boundary: {forbidden}"
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn invalid_cli_shape_is_rejected_before_input_or_export_io() {
    use std::process::Command;
    for args in [
        vec!["qa"],
        vec!["qa", "run"],
        vec!["qa", "plan", "missing", "--input"],
        vec![
            "qa",
            "plan",
            "missing",
            "--input",
            "missing",
            "--input",
            "duplicate",
        ],
        vec![
            "qa", "plan", "missing", "--input", "missing", "--export", "out", "--export", "other",
        ],
        vec!["qa", "plan", "missing", "--input", "missing", "extra"],
        vec!["qa", "plan", "missing", "--input", "missing", "--yes"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("qa plan: project:"));
    }
}

#[test]
fn postman_draft_maps_explicit_requests_without_running_them() {
    use std::{fs, process::Command};
    let base = std::env::temp_dir().join(format!("snipe-postman-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&base).unwrap();
    let input = base.join("requirements.json");
    let scenarios: Vec<_> = [("ok", "positive", 200), ("bad", "negative", 403)]
        .into_iter()
        .map(|(id, category, status)| {
            serde_json::json!({
                "id": id, "category": category, "objective": "Explicit HTTP behavior",
                "precondition": "Reviewed fixture", "input": "Send request",
                "expected_result": "Explicit response status",
                "request": {"method": if id == "ok" { "GET" } else { "POST" }, "url": "http://127.0.0.1:9/never-contact",
                    "headers": {"Accept": "application/json"}, "body": if id == "ok" { None } else { Some("{\"fixture\":true}") },
                    "expected_status": status}
            })
        })
        .collect();
    fs::write(
        &input,
        serde_json::to_vec(&serde_json::json!({
            "version": 1, "requirements": [{"id": "REQ-HTTP", "scenarios": scenarios}]
        }))
        .unwrap(),
    )
    .unwrap();
    let run = |adapter: &str| {
        Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(["qa", "plan"])
            .arg(&base)
            .arg("--input")
            .arg(&input)
            .args(["--adapter", adapter])
            .env("PATH", "")
            .output()
            .unwrap()
    };
    let output = run("postman-cli");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["script"]["adapter_version"], "1.90.0");
    assert_eq!(plan["script"]["status"], "ready-for-review");
    let items = plan["script"]["collection"]["item"].as_array().unwrap();
    for (i, status) in [200, 403].into_iter().enumerate() {
        assert_eq!(items[i]["id"], plan["cases"][i]["id"]);
        assert_eq!(
            items[i]["request"]["method"],
            if i == 0 { "GET" } else { "POST" }
        );
        assert_eq!(
            items[i]["request"]["url"],
            "http://127.0.0.1:9/never-contact"
        );
        assert_eq!(items[i]["request"]["header"][0]["key"], "Accept");
        assert_eq!(
            items[i]["request"]["header"][0]["value"],
            "application/json"
        );
        assert_eq!(items[i]["event"][0]["listen"], "test");
        let code = items[i]["event"][0]["script"]["exec"][0].as_str().unwrap();
        assert!(code.contains(&format!("pm.response.to.have.status({status})")));
        assert!(!code.contains("sendRequest"));
        assert!(!code.contains("install"));
    }
    assert!(items[0]["request"].get("body").is_none());
    assert_eq!(items[1]["request"]["body"]["mode"], "raw");
    assert_eq!(items[1]["request"]["body"]["raw"], "{\"fixture\":true}");
    assert_eq!(fs::read_dir(&base).unwrap().count(), 1);
    let export = base.with_extension("plan.json");
    let exported = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .args(["qa", "plan"])
        .arg(&base)
        .arg("--input")
        .arg(&input)
        .args(["--adapter", "postman-cli", "--export"])
        .arg(&export)
        .output()
        .unwrap();
    assert!(exported.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&export).unwrap()).unwrap(),
        plan
    );
    fs::remove_file(&export).unwrap();
    let original = fs::read_to_string(&input).unwrap();
    for invalid in [
        original.replace("\"GET\"", "\"GET; touch injected\""),
        original.replace("\"expected_status\":200", "\"expected_status\":0"),
        original.replace("http://127.0.0.1:9/never-contact", "file:///outside"),
        original.replace("http://127.0.0.1:9/never-contact", "http://?bad"),
        original.replace(
            "http://127.0.0.1:9/never-contact",
            "https://host.invalid:wrong/",
        ),
        original.replace(
            "http://127.0.0.1:9/never-contact",
            "https://user:password@host.invalid/",
        ),
        original.replace("http://127.0.0.1:9/never-contact", "https://{{target}}/"),
        original.replace(
            "\"Accept\":\"application/json\"",
            "\"Authorization\":\"Bearer qa-canary-secret\"",
        ),
    ] {
        fs::write(&input, invalid).unwrap();
        let rejected = run("postman-cli");
        assert_eq!(rejected.status.code(), Some(3));
        assert!(rejected.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&rejected.stderr).contains("qa-canary-secret"));
        let failed_export = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .args(["qa", "plan"])
            .arg(&base)
            .arg("--input")
            .arg(&input)
            .args(["--adapter", "postman-cli", "--export"])
            .arg(&export)
            .output()
            .unwrap();
        assert_eq!(failed_export.status.code(), Some(3));
        assert!(!export.exists());
    }
    fs::write(&input, original).unwrap();
    let collection = &plan["script"]["collection"];
    assert_eq!(
        collection["info"]["schema"],
        "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
    );
    assert_eq!(collection["item"].as_array().unwrap().len(), 2);
    for adapter in ["unit", "regression", "playwright", "bruno", "maestro"] {
        let unsupported = run(adapter);
        assert_eq!(unsupported.status.code(), Some(2), "{adapter}");
        assert!(unsupported.stdout.is_empty(), "{adapter}");
        assert!(
            String::from_utf8_lossy(&unsupported.stderr).contains("unsupported QA adapter"),
            "{adapter}"
        );
    }
    let mut missing: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&input).unwrap()).unwrap();
    missing["requirements"][0]["scenarios"][0]
        .as_object_mut()
        .unwrap()
        .remove("request");
    fs::write(&input, serde_json::to_vec(&missing).unwrap()).unwrap();
    let rejected = run("postman-cli");
    assert_eq!(rejected.status.code(), Some(3));
    assert!(rejected.stdout.is_empty());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn cli_planning_is_read_only_and_export_requires_outside_destination() {
    use std::fs;
    use std::process::Command;
    let base = std::env::temp_dir().join(format!("snipe-qa-{}", uuid::Uuid::new_v4()));
    let project = base.join("project");
    fs::create_dir_all(&project).unwrap();
    let input = project.join("requirements.json");
    let source = r#"{"version":1,"requirements":[{"id":"REQ-1","scenarios":[{"id":"ok","category":"positive","objective":"allowed","precondition":"ready","input":"good","expected_result":"accepted"},{"id":"bad","category":"negative","objective":"reject","precondition":"ready","input":"bad","expected_result":"denied"}]}]}"#;
    fs::write(&input, source).unwrap();
    let run = |extra: &[&std::ffi::OsStr]| {
        Command::new(env!("CARGO_BIN_EXE_snipe-code"))
            .arg("qa")
            .arg("plan")
            .arg(&project)
            .arg("--input")
            .arg(&input)
            .args(extra)
            .output()
            .unwrap()
    };
    let first = run(&[]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["cases"][0]["expected_result"], "accepted");
    assert_eq!(value["cases"][1]["expected_result"], "denied");
    assert_eq!(value["cases"][0]["requirement_refs"][0], "REQ-1");
    assert_eq!(value["cases"][0]["source_refs"][0], "requirements.json");
    assert_eq!(first.stdout, run(&[]).stdout);
    assert_eq!(fs::read_to_string(&input).unwrap(), source);
    assert_eq!(fs::read_dir(&project).unwrap().count(), 1);
    let inside = project.join("out.json");
    let denied = run(&["--export".as_ref(), inside.as_os_str()]);
    assert!(!denied.status.success());
    assert!(denied.stdout.is_empty());
    assert!(!inside.exists());
    let invalid_destination = base.join("missing").join("plan.json");
    let failed = run(&["--export".as_ref(), invalid_destination.as_os_str()]);
    assert_eq!(failed.status.code(), Some(3));
    assert!(failed.stdout.is_empty());
    assert!(!invalid_destination.exists());
    let output = base.join("plan.json");
    assert!(
        run(&["--export".as_ref(), output.as_os_str()])
            .status
            .success()
    );
    assert_eq!(
        fs::read(&output).unwrap(),
        first.stdout[..first.stdout.len() - 1]
    );
    assert!(
        !run(&["--export".as_ref(), output.as_os_str()])
            .status
            .success()
    );
    for invalid_source in [
        source.replace("\"denied\"", "\"\""),
        source.replace("\"negative\"", "\"positive\""),
        source.replace("\"id\":\"bad\"", "\"id\":\"ok\""),
        source.replace("\"version\":1", "\"version\":2"),
        "not json".to_string(),
    ] {
        fs::write(&input, invalid_source).unwrap();
        let invalid = run(&[]);
        assert!(!invalid.status.success());
        assert!(invalid.stdout.is_empty());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .args(["help", "qa"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("snipe-code qa plan"));
    fs::remove_dir_all(base).unwrap();
}
