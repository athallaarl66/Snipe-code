use serde_json::{Value, json};
use snipe_code::quality::*;

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("snipe-quality-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn function_line_golden_boundaries_exclusions_and_unknown() {
    let f = Fixture::new();
    f.write("src/main.rs", "fn main() {\n let x = 1;\n}\n");
    let mut p = policy("rust-function-lines", "lines", 3);
    let m = source_metric(&f.0, &p).unwrap();
    assert_eq!(m.value, Some(3));
    assert_eq!(m.state, State::Pass);
    p.threshold = 2;
    assert_eq!(source_metric(&f.0, &p).unwrap().state, State::Fail);
    p.threshold = 4;
    assert_eq!(source_metric(&f.0, &p).unwrap().state, State::Pass);
    f.write("ignored/bad.rs", "not Rust");
    p.exclusions.push("ignored".into());
    assert_eq!(source_metric(&f.0, &p).unwrap().files, ["src/main.rs"]);
    f.write("src/other.py", "print(1)");
    assert_eq!(source_metric(&f.0, &p).unwrap().state, State::Unknown);
    p.exclusions.push("src".into());
    assert_eq!(source_metric(&f.0, &p).unwrap().value, None);
    let f = Fixture::new();
    f.write(
        "impl.rs",
        "struct A; impl A { fn method() {\n}\n} trait B { fn f() {\n}\n }",
    );
    assert_eq!(
        source_metric(&f.0, &policy("rust-function-lines", "lines", 2))
            .unwrap()
            .value,
        Some(2)
    );
}

fn policy(id: &str, unit: &str, threshold: u64) -> Policy {
    Policy {
        id: id.into(),
        version: 1,
        unit: unit.into(),
        required: true,
        comparison: if id == "lcov-line-coverage" {
            Comparison::Minimum
        } else {
            Comparison::Maximum
        },
        threshold,
        unknown: Missing::Unknown,
        exclusions: vec![],
    }
}
fn config(p: Policy) -> Config {
    Config {
        version: 1,
        enabled: true,
        metrics: vec![p],
    }
}
#[test]
fn config_schema_requires_every_definition_and_roundtrips() {
    let c = config(policy("rust-function-lines", "lines", 20));
    let value = serde_json::to_value(&c).unwrap();
    assert_eq!(parse_config(&value.to_string()).unwrap(), c);
    for field in [
        "id",
        "version",
        "unit",
        "required",
        "comparison",
        "threshold",
        "unknown",
        "exclusions",
    ] {
        let mut v = value.clone();
        v["metrics"][0].as_object_mut().unwrap().remove(field);
        assert!(parse_config(&v.to_string()).is_err(), "{field}");
    }
    for (field, bad) in [
        ("version", json!(2)),
        ("unit", json!("percent")),
        ("exclusions", json!(["../escape"])),
        ("id", json!("magic-score")),
    ] {
        let mut v = value.clone();
        v["metrics"][0][field] = bad;
        assert!(parse_config(&v.to_string()).is_err());
    }
    assert!(parse_config(&json!({"version":1,"metrics":[]}).to_string()).is_err());
    let mut v: Value = value;
    v["extra"] = json!(true);
    assert!(parse_config(&v.to_string()).is_err());
}

#[test]
fn duplicate_blocks_golden_comments_literals_and_exclusions() {
    let f = Fixture::new();
    let code = format!("fn main() {{ {} }}", "let x = 1;".repeat(10));
    f.write("a.rs", &code);
    let mut p = policy("rust-duplicate-blocks", "blocks", 0);
    assert_eq!(source_metric(&f.0, &p).unwrap().value, Some(0));
    f.write("b.rs", &code.replace("let", "/* comment */ let\n"));
    let m = source_metric(&f.0, &p).unwrap();
    assert_eq!(m.value, Some(1));
    assert_eq!(m.state, State::Fail);
    p.threshold = 1;
    assert_eq!(source_metric(&f.0, &p).unwrap().state, State::Pass);
    p.exclusions.push("b.rs".into());
    assert_eq!(source_metric(&f.0, &p).unwrap().value, Some(0));
    p.exclusions.clear();
    f.write("b.rs", &code.replace("1", "2"));
    assert_eq!(source_metric(&f.0, &p).unwrap().value, Some(0));
    f.write("b.rs", "fn broken(");
    assert_eq!(source_metric(&f.0, &p).unwrap().state, State::Unknown);
}

#[test]
fn coverage_import_valid_paths_provenance_invalid_and_budget() {
    let f = Fixture::new();
    f.write("src/main.rs", "fn main() {\n let x=1;\n}\n");
    let p = policy("lcov-line-coverage", "basis-points", 5000);
    let report = f.0.join("lcov.info");
    let valid = "TN:test\nSF:src/main.rs\nDA:1,1\nDA:2,0\nLF:2\nLH:1\nend_of_record\n";
    f.write("lcov.info", valid);
    let m = coverage_metric(&f.0, &p, Some(&report)).unwrap();
    assert_eq!(m.value, Some(5000));
    assert_eq!(m.state, State::Pass);
    assert_eq!(m.files, ["src/main.rs"]);
    assert_eq!(m.coverage.as_ref().unwrap().payload_sha256.len(), 64);
    assert_eq!(m.coverage.as_ref().unwrap().covered_lines, 1);
    assert_eq!(
        coverage_metric(&f.0, &p, None).unwrap().state,
        State::Unknown
    );
    for text in [
        "<coverage/>",
        "SF:src/main.rs\nDA:1,1",
        "SF:../escape\nDA:1,1\nend_of_record\n",
        "SF:src/main.rs\nDA:999,1\nend_of_record\n",
        "SF:src/main.rs\nDA:1,1\nDA:1,0\nend_of_record\n",
    ] {
        f.write("lcov.info", text);
        assert_eq!(
            coverage_metric(&f.0, &p, Some(&report)).unwrap().state,
            State::Unknown
        );
    }
    f.write("lcov.info", &valid.repeat(2));
    assert_eq!(
        coverage_metric(&f.0, &p, Some(&report)).unwrap().state,
        State::Unknown
    );
    f.write("lcov.info", &"x".repeat(MAX_INPUT as usize + 1));
    assert_eq!(
        coverage_metric(&f.0, &p, Some(&report)).unwrap().state,
        State::Unknown
    );
    f.write("lcov.info", valid);
    let mut excluded = p.clone();
    excluded.exclusions.push("src".into());
    assert_eq!(
        coverage_metric(&f.0, &excluded, Some(&report))
            .unwrap()
            .state,
        State::Unknown
    );
    let abs = f.0.join("src/main.rs").canonicalize().unwrap();
    f.write(
        "lcov.info",
        &valid.replace("src/main.rs", abs.to_str().unwrap()),
    );
    assert_eq!(
        coverage_metric(&f.0, &p, Some(&report)).unwrap().value,
        Some(5000)
    );
}
