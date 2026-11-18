use snipe_code::{
    audit::{self, AuditResult, Finding},
    history::ScanEnvelope,
};
use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("snipe-parity-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SAFE_DESCRIPTION: &str = "Potential SQL injection: string concatenation in query";
const CANARY: &str = "PARITY_SECRET_CANARY_7391";

fn export(count: usize, unicode: bool) -> (Fixture, ScanEnvelope) {
    let fixture = Fixture::new();
    let findings: Vec<_> = (0..count)
        .map(|i| Finding {
            severity: ["HIGH", "MEDIUM", "LOW"][i % 3].into(),
            category: format!("ParityCategory{i:04}"),
            file: format!(
                "src/{}/file{i:04}_{}{}.rs",
                "nested/".repeat(16),
                "x".repeat(100),
                if unicode { "_🦀" } else { "" }
            ),
            line: i + 1,
            description: if i == 0 {
                format!("{CANARY} {}", "long-description ".repeat(40))
            } else {
                SAFE_DESCRIPTION.into()
            },
        })
        .collect();
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();
    let result = AuditResult {
        exported_files: vec![],
        findings,
        high,
        medium,
        low,
        highest_severity: if high > 0 { Some("HIGH".into()) } else { None },
        policy_failed: high > 0,
        detected_scopes: vec![],
        evaluated_rule_groups: vec![],
        exclusions: vec![],
        sast: vec![],
        sca: None,
    };
    let json = audit::render_scan_json(&fixture.0, result).unwrap();
    assert!(!json.contains(CANARY), "secret leaked into canonical JSON");
    let envelope: ScanEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope.result.findings.len(), count);
    assert_eq!(
        (
            envelope.result.high,
            envelope.result.medium,
            envelope.result.low
        ),
        (high, medium, low)
    );
    if count > 0 {
        // Arbitrary descriptions are deliberately redacted before report export.
        assert_eq!(envelope.result.findings[0].description, "[REDACTED]");
    }
    let input = fixture.0.join("scan.json");
    fs::write(&input, json).unwrap();
    audit::export_scan_json(&input, "md,docx,pdf", &fixture.0).unwrap();
    (fixture, envelope)
}

fn run_extractor(program: &str, args: &[&str], fixture: &Fixture) -> String {
    let output = Command::new(program)
        .current_dir(&fixture.0)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("NOT RUN: {program} extractor unavailable: {error}"));
    assert!(
        output.status.success(),
        "{program} extractor failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("extractor returned non-UTF-8 text")
}

fn docx_text(fixture: &Fixture) -> String {
    // Python's standard library extracts text nodes and XML entities, not raw ZIP bytes.
    let script = "import sys,zipfile,xml.etree.ElementTree as E; z=zipfile.ZipFile('Audit_Report.docx'); r=E.fromstring(z.read('word/document.xml')); n={'w':'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}; sys.stdout.buffer.write(('\\n'.join(''.join(t.text or '' for t in p.findall('.//w:t',n)) for p in r.findall('.//w:p',n))).encode('utf-8'))";
    run_extractor("python", &["-c", script], fixture)
}

fn pdf_text(fixture: &Fixture) -> String {
    run_extractor("pdftotext", &["-raw", "Audit_Report.pdf", "-"], fixture)
}

fn assert_common(text: &str, envelope: &ScanEnvelope, format: &str) {
    let findings = &envelope.result.findings;
    assert!(!text.contains(CANARY), "{format}: secret leaked");
    assert!(
        text.contains(&format!("Total Findings: {}", findings.len())),
        "{format}: total missing"
    );
    for (label, count) in [
        ("HIGH", envelope.result.high),
        ("MEDIUM", envelope.result.medium),
        ("LOW", envelope.result.low),
    ] {
        assert!(
            text.contains(&format!("{label}: {count}")),
            "{format}: {label} count missing"
        );
    }
    assert_eq!(
        text.matches("File: ").count(),
        findings.len(),
        "{format}: finding count"
    );
    for (i, finding) in findings.iter().enumerate() {
        let header = format!("{}. {} [{}]", i + 1, finding.category, finding.severity);
        let location = format!("File: {} (line {})", finding.file, finding.line);
        let description = format!("Description: {}", finding.description);
        let block = format!("{header}\n{location}\n{description}");
        assert!(
            text.contains(&block),
            "{format}: missing complete finding {}",
            i + 1
        );
        assert_eq!(
            text.matches(&header).count(),
            1,
            "{format}: duplicate identity {}",
            i + 1
        );
    }
}

#[test]
fn markdown_matches_canonical_json_for_zero_one_and_many() {
    for count in [0, 1, 75] {
        let (fixture, envelope) = export(count, false);
        let text = fs::read_to_string(fixture.0.join("Audit_Report.md")).unwrap();
        let text = text.replace("**", "").replace('`', "").replace("### ", "");
        assert_common(&text, &envelope, "Markdown");
    }
}

#[test]
fn docx_matches_canonical_json_for_zero_one_and_many() {
    for count in [0, 1, 75] {
        let (fixture, envelope) = export(count, false);
        assert_common(&docx_text(&fixture), &envelope, "DOCX");
    }
}

#[test]
fn pdf_matches_canonical_json_for_zero_one_and_many() {
    for count in [0, 1, 75] {
        let (fixture, envelope) = export(count, false);
        let text = pdf_text(&fixture).replace("Report generated by SNIPE-CODE CLI", "");
        assert!(!text.contains(CANARY), "PDF: secret leaked");
        assert!(text.contains(&format!("Total Findings: {count}")));
        for (label, value) in [
            ("HIGH", envelope.result.high),
            ("MEDIUM", envelope.result.medium),
            ("LOW", envelope.result.low),
        ] {
            assert!(
                text.contains(&format!("{label}: {value}")),
                "PDF: {label} count missing"
            );
        }
        assert_eq!(text.matches("File: ").count(), count, "PDF: finding count");
        // pdftotext inserts line/page breaks at wrapping boundaries.
        let joined = text.replace(['\n', '\r', '\u{c}'], "");
        for (i, finding) in envelope.result.findings.iter().enumerate() {
            let header = format!("{}. {} [{}]", i + 1, finding.category, finding.severity);
            assert_eq!(
                joined.matches(&header).count(),
                1,
                "PDF: identity {}",
                i + 1
            );
            let block = format!(
                "{header}File: {} (line {})Description: {}",
                finding.file, finding.line, finding.description
            );
            assert!(
                joined.contains(&block),
                "PDF: missing complete finding {}",
                i + 1
            );
        }
    }
}

#[test]
fn pdf_escapes_unsupported_unicode_finding_path_reversibly() {
    let (fixture, envelope) = export(1, true);
    let text = pdf_text(&fixture).replace(['\n', '\r', '\u{c}'], "");
    let encoded_path = envelope.result.findings[0].file.replace('🦀', r"\u{1F980}");
    assert!(
        text.contains(&encoded_path),
        "PDF must preserve unsupported path code points as escapes"
    );
}
