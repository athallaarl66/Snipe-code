use snipe_code::verify::exec::{self, CommandSpec};
use std::process::Command;

#[test]
fn reports_redact_real_fixture_without_mutating_report() {
    use snipe_code::verify::{
        Gate, GateOutcome, GateState, PartOutcome, Report, render_json, report,
    };
    let dir = exec::temp_workspace("redaction-report").unwrap();
    let secret = "fixtureReportCredential472";
    let dotenv = format!("DB_PASSWORD={secret}\n");
    assert!(dotenv.contains(secret));
    std::fs::write(dir.join(".env"), dotenv).unwrap();
    let report = Report {
        root: dir.clone(),
        parts: vec![PartOutcome {
            label: ".".into(),
            ecosystem: "npm",
            outcomes: vec![GateOutcome {
                gate: Gate::Test,
                state: GateState::Failed(format!("failure {secret}")),
                command: format!("tool {secret}"),
            }],
        }],
    };
    for text in [
        render_json(&report).unwrap(),
        report::render_junit(&report),
        report::render_markdown(&report),
    ] {
        assert!(!text.contains(secret));
        assert!(text.contains("[redacted]"));
    }
    assert!(report.parts[0].outcomes[0].command.contains(secret));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn child_output_redacts_known_database_secrets_and_keeps_diagnostics() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "redaction_helper", "--nocapture"])
        .env("SNIPE_REDACTION_HELPER", "1")
        .env("DB_PASSWORD", "fixturePassword123")
        .env(
            "DATABASE_URL",
            "postgres://fixtureUser:fixtureUrlPassword@localhost/app_test",
        )
        .output()
        .unwrap();
    assert!(output.status.success());
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(text.contains("diagnostic retained"));
        assert!(!text.contains("fixturePassword123"));
        assert!(!text.contains("fixtureUrlPassword"));
        assert!(text.contains("[redacted]"));
    }
}

#[test]
fn redaction_helper() {
    if std::env::var_os("SNIPE_REDACTION_HELPER").is_none() {
        return;
    }
    if std::env::var_os("SNIPE_REDACTION_CHILD").is_some() {
        println!("diagnostic retained fixturePassword123 fixtureUrlPassword");
        eprintln!("diagnostic retained fixturePassword123 fixtureUrlPassword");
        return;
    }
    unsafe {
        std::env::set_var("SNIPE_REDACTION_CHILD", "1");
    }
    let exe = std::env::current_exe().unwrap();
    let spec = CommandSpec::new(
        exe.to_str().unwrap(),
        &["--exact", "redaction_helper", "--nocapture"],
        std::path::Path::new("."),
    );
    exec::run(&spec, "fixture").unwrap();
}

#[test]
fn child_output_routes_encoded_dotenv_and_split_writes() {
    for route in ["normal", "stderr", "generated"] {
        let dir = exec::temp_workspace("redaction-test").unwrap();
        std::fs::write(
            dir.join(".env.test"),
            "DB_PASSWORD=fixtureDotenvSecret123\n",
        )
        .unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "stream_helper", "--nocapture"])
            .env("STREAM_ROUTE", route)
            .env("STREAM_DIR", &dir)
            .env("DB_PASSWORD", "fixturePass@word123")
            .env(
                "DATABASE_URL",
                "postgres://fixtureUser:fixtureUrl%40Password123@localhost/app_test",
            )
            .output()
            .unwrap();
        std::fs::remove_dir_all(dir).unwrap();
        assert!(output.status.success(), "helper failed for {route}");
        for bytes in [&output.stdout, &output.stderr] {
            let text = String::from_utf8_lossy(bytes);
            for secret in [
                "fixturePass@word123",
                "fixtureUrl@Password123",
                "fixtureDotenvSecret123",
                "fixtureUser",
                "fixtureUrl%40Password123",
                "fixturePass%40word123",
            ] {
                assert!(!text.contains(secret), "credential leaked in {route}");
            }
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("diagnostic retained") && stderr.contains("[redacted]"));
        if route == "stderr" {
            assert!(!stdout.contains("diagnostic retained"));
        } else {
            assert!(stdout.contains("diagnostic retained") && stdout.contains("[redacted]"));
        }
    }
}

#[test]
fn stream_helper() {
    use std::io::Write;
    let Ok(route) = std::env::var("STREAM_ROUTE") else {
        return;
    };
    if std::env::var_os("STREAM_CHILD").is_some() {
        for mut stream in [
            Box::new(std::io::stdout()) as Box<dyn Write>,
            Box::new(std::io::stderr()),
        ] {
            for _ in 0..128 {
                stream.write_all(&[b'x'; 1024]).unwrap();
            }
            stream
                .write_all(b" diagnostic retained fixturePass@")
                .unwrap();
            stream.flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
            stream.write_all(b"word123 fixtureUrl@Password123 fixtureDotenvSecret123 fixtureUser fixtureUrl%40Password123 fixturePass%40word123").unwrap();
            stream.flush().unwrap();
        }
        return;
    }
    unsafe {
        std::env::set_var("STREAM_CHILD", "1");
    }
    let exe = std::env::current_exe().unwrap();
    let dir = std::path::PathBuf::from(std::env::var_os("STREAM_DIR").unwrap());
    let spec = CommandSpec::new(
        exe.to_str().unwrap(),
        &["--exact", "stream_helper", "--nocapture"],
        &dir,
    );
    match route.as_str() {
        "normal" => exec::run(&spec, "fixture"),
        "stderr" => exec::run_stderr(&spec, "fixture"),
        "generated" => exec::run_generated(&spec, "fixture"),
        _ => panic!("invalid route"),
    }
    .unwrap();
}

#[test]
fn exact_credential_whitespace_and_quotes_are_redacted() {
    let dir = exec::temp_workspace("credential-exact").unwrap();
    let secret = "  'fixture exact secret'  ";
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "exact_credential_helper", "--nocapture"])
        .env("EXACT_DIR", &dir)
        .env("DB_PASSWORD", secret)
        .output()
        .unwrap();
    std::fs::remove_dir_all(dir).unwrap();
    assert!(output.status.success());
    for bytes in [&output.stdout, &output.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(secret));
        assert!(text.contains("[redacted]"));
    }
}

#[test]
fn exact_credential_helper() {
    let Some(dir) = std::env::var_os("EXACT_DIR") else {
        return;
    };
    if std::env::var_os("EXACT_CHILD").is_some() {
        println!("credential: {}", std::env::var("DB_PASSWORD").unwrap());
        eprintln!("credential: {}", std::env::var("DB_PASSWORD").unwrap());
        return;
    }
    unsafe {
        std::env::set_var("EXACT_CHILD", "1");
    }
    let exe = std::env::current_exe().unwrap();
    let spec = CommandSpec::new(
        exe.to_str().unwrap(),
        &["--exact", "exact_credential_helper", "--nocapture"],
        std::path::Path::new(&dir),
    );
    exec::run(&spec, "fixture").unwrap();
}

#[test]
fn interactive_child_prompt_arrives_before_input() {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;
    let mut helper = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "interactive_prompt_helper", "--nocapture"])
        .env("PROMPT_HELPER", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = helper.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut one = [0];
        while output.read(&mut one).unwrap() != 0 {
            bytes.push(one[0]);
            if bytes.ends_with(b"Input required: ") {
                let _ = tx.send(());
                break;
            }
        }
        let mut rest = Vec::new();
        let _ = output.read_to_end(&mut rest);
    });
    let received = rx.recv_timeout(Duration::from_secs(5));
    if received.is_err() {
        let _ = helper.kill();
    } else {
        helper.stdin.take().unwrap().write_all(b"ok\n").unwrap();
    }
    let status = helper.wait().unwrap();
    reader.join().unwrap();
    assert!(received.is_ok(), "prompt not visible before timeout");
    assert!(status.success());
}

#[test]
fn interactive_prompt_helper() {
    use std::io::{BufRead, Write};
    if std::env::var_os("PROMPT_HELPER").is_none() {
        return;
    }
    if std::env::var_os("PROMPT_CHILD").is_some() {
        print!("Input required: ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line).unwrap();
        assert_eq!(line, "ok\n");
        return;
    }
    unsafe {
        std::env::set_var("PROMPT_CHILD", "1");
    }
    let exe = std::env::current_exe().unwrap();
    let spec = CommandSpec::new(
        exe.to_str().unwrap(),
        &["--exact", "interactive_prompt_helper", "--nocapture"],
        std::path::Path::new("."),
    );
    exec::run(&spec, "fixture").unwrap();
}

#[test]
fn preview_and_human_report_redact_real_fixture_path() {
    use std::process::Stdio;
    let root = exec::temp_workspace("preview-boundary").unwrap();
    let secret = "fixturePreviewCredential479";
    let dir = root.join(secret);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("package.json"), r#"{"scripts":{"test":"unused"}}"#).unwrap();
    std::fs::write(
        dir.join(".env"),
        format!("DB_PASSWORD={secret}\nDB_CONNECTION=sqlite\nDB_DATABASE=:memory:\n"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_snipe-code"))
        .args(["verify", dir.to_str().unwrap(), "--only", "install"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(secret));
    assert!(!stderr.contains(secret));
    assert!(stdout.contains("Preflight:") || stderr.contains("Preflight:"));
    assert!(stdout.contains("verify:"));
    assert!(stdout.contains("[redacted]"));
}
