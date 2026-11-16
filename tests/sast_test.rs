use std::fs;
use std::path::Path;

use snipe_code::sast::{self, SastStatus};

fn fixture(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sast/rust")
            .join(name),
    )
    .expect("read SAST fixture")
}

#[test]
fn rule_catalog_has_versioned_scopes_evidence_and_paired_fixtures() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../src/sast/rules.json")).expect("valid rule catalog");
    let rules = catalog["rules"].as_array().expect("rules array");
    assert!(!rules.is_empty());
    for rule in rules {
        for field in [
            "id",
            "version",
            "scope",
            "evidence_policy",
            "severity",
            "confidence",
        ] {
            assert!(
                rule[field].as_str().is_some_and(|value| !value.is_empty()),
                "missing {field}: {rule}"
            );
        }
        for field in ["positive_fixture", "negative_fixture"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(rule[field].as_str().expect("fixture path"));
            assert!(path.is_file(), "missing {}", path.display());
            let source = fs::read_to_string(path).expect("read fixture");
            assert!(
                syn::parse_file(&source).is_ok(),
                "invalid Rust fixture: {source}"
            );
        }
        if rule["state"] == "released" {
            assert_eq!(catalog["supported_languages"], serde_json::json!(["rust"]));
        }
    }
}

#[test]
fn paired_rust_fixtures_report_candidates_and_keep_safe_cases_clean() {
    let cases = [
        ("shell-tainted.rs", "sast-rust-shell-tainted", true),
        ("shell-literal-safe.rs", "sast-rust-shell-tainted", false),
        (
            "sql-tainted-format.rs",
            "sast-rust-sql-tainted-format",
            true,
        ),
        (
            "sql-parameterized-safe.rs",
            "sast-rust-sql-tainted-format",
            false,
        ),
        (
            "html-tainted-output.rs",
            "sast-rust-html-tainted-output",
            true,
        ),
        (
            "html-ammonia-sanitized.rs",
            "sast-rust-html-tainted-output",
            false,
        ),
    ];

    for (file, expected_rule, should_find) in cases {
        let report = sast::analyze_rust_source(file, &fixture(file));
        assert_eq!(report.status, SastStatus::Evaluated, "{file}");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.rule_id == expected_rule)
            .collect();
        assert_eq!(
            !findings.is_empty(),
            should_find,
            "unexpected result for {file}: {:?}",
            report
        );
        for finding in findings {
            assert_eq!(finding.rule_version, "1.0.0");
            assert_eq!(finding.confidence, "medium");
            assert!(finding.candidate);
            assert!(!finding.evidence.is_empty());
            assert!(finding.evidence.iter().all(|item| item.line > 0));
            assert!(
                !serde_json::to_string(finding)
                    .unwrap()
                    .contains("SAST_CANARY_DO_NOT_LEAK")
            );
        }
    }
}

#[test]
fn escaping_is_html_only_and_literal_braces_are_not_input() {
    let escaped = r#"fn handler() {
        let input = std::env::var("INPUT").unwrap();
        let safe = html_escape::encode_safe(&input);
        axum::response::Html(safe);
    }"#;
    assert!(
        sast::analyze_rust_source("safe.rs", escaped)
            .findings
            .is_empty()
    );
    let shell = escaped.replace(
        "axum::response::Html(safe)",
        "std::process::Command::new(\"sh\").arg(\"-c\").arg(safe).status()",
    );
    assert!(
        sast::analyze_rust_source("unsafe.rs", &shell)
            .findings
            .iter()
            .any(|f| f.rule_id == "sast-rust-shell-tainted")
    );
    let literal = r#"fn run() {
        let input = std::env::var("INPUT").unwrap();
        std::process::Command::new("sh").arg("-c").arg(format!("echo {{input}}")).status();
    }"#;
    assert!(
        sast::analyze_rust_source("literal.rs", literal)
            .findings
            .is_empty()
    );
}

#[test]
fn shell_positional_input_is_not_shell_source() {
    let source = r#"fn run() {
        let input = std::env::var("INPUT").unwrap();
        std::process::Command::new("sh").arg("-c").arg("printf '%s' \"$1\"").arg("fixed").arg(input).status();
    }"#;
    let report = sast::analyze_rust_source("positional.rs", source);
    assert!(report.findings.is_empty());
}

#[test]
fn assignments_control_flow_and_parser_budgets_are_unknown() {
    for source in [
        "fn f() { let mut v = String::new(); v = std::env::var(\"INPUT\").unwrap(); }".to_string(),
        "fn f() { if true { let v = std::env::var(\"INPUT\").unwrap(); } }".to_string(),
        " ".repeat(262_145),
        format!("fn f() {{ {}0{}; }}", "(".repeat(65), ")".repeat(65)),
    ] {
        assert_eq!(
            sast::analyze_rust_source("budget.rs", &source).status,
            SastStatus::Unknown
        );
    }
    let report = sast::analyze_rust_source("invalid.rs", "fn broken() { SAST_CANARY_DO_NOT_LEAK");
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("SAST_CANARY_DO_NOT_LEAK")
    );
}

#[test]
fn unsupported_tainted_transform_and_parse_error_remain_unknown_or_error() {
    let unsupported = r#"
fn run() {
    let input = std::env::var("SNIPE_INPUT").unwrap();
    let transformed = custom_normalize(&input);
    std::process::Command::new("sh")
        .arg("-c")
        .arg(transformed)
        .status()
        .unwrap();
}
"#;
    let report = sast::analyze_rust_source("unsupported.rs", unsupported);
    assert_eq!(report.status, SastStatus::Unknown);
    assert!(report.findings.is_empty());

    let invalid = sast::analyze_rust_source("invalid.rs", "fn broken( {");
    assert_eq!(invalid.status, SastStatus::Error);
    assert!(invalid.findings.is_empty());
}
