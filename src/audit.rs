use std::fs;
use std::path::Path;

pub fn run_audit(project_path: &str, export_format: &str) -> Result<String, String> {
    let path = Path::new(project_path);
    if !path.exists() {
        return Err(format!("Folder '{}' does not exist", project_path));
    }

    let mut findings = Vec::new();

    // Check for hardcoded secrets
    check_hardcoded_secrets(path, &mut findings);

    // Check for .env files in git
    check_env_in_git(path, &mut findings);

    // Check for SQL injection risks
    check_sql_injection(path, &mut findings);

    // Check for XSS risks
    check_xss_risks(path, &mut findings);

    // Check for missing security configs
    check_security_configs(path, &mut findings);

    // Generate report
    let report = generate_report(&findings, export_format);

    // Save report
    let report_path = path.join("Audit_Report.md");
    fs::write(&report_path, &report).map_err(|e| e.to_string())?;

    Ok(report)
}

#[derive(Debug)]
struct Finding {
    severity: String,
    category: String,
    description: String,
    file: String,
    line: usize,
}

fn check_hardcoded_secrets(path: &Path, findings: &mut Vec<Finding>) {
    let secret_patterns = vec!["password", "secret", "api_key", "apikey", "token", "private_key"];
    let extensions = vec![".ts", ".tsx", ".js", ".jsx", ".env", ".json"];

    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_num, line) in content.lines().enumerate() {
                    for pattern in &secret_patterns {
                        if line.to_lowercase().contains(pattern) && !line.trim().starts_with('#') {
                            // Check if it's an actual hardcoded value (not just a variable name)
                            if line.contains('=') && !line.contains("process.env") && !line.contains("your-") {
                                findings.push(Finding {
                                    severity: "HIGH".to_string(),
                                    category: "Hardcoded Secret".to_string(),
                                    description: format!("Potential hardcoded secret: {}", pattern),
                                    file: file.display().to_string(),
                                    line: line_num + 1,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_env_in_git(path: &Path, findings: &mut Vec<Finding>) {
    let gitignore = path.join(".gitignore");
    if gitignore.exists() {
        if let Ok(content) = fs::read_to_string(&gitignore) {
            if !content.contains(".env") {
                findings.push(Finding {
                    severity: "MEDIUM".to_string(),
                    category: "Git Security".to_string(),
                    description: ".env files not in .gitignore".to_string(),
                    file: ".gitignore".to_string(),
                    line: 0,
                });
            }
        }
    }
}

fn check_sql_injection(path: &Path, findings: &mut Vec<Finding>) {
    let extensions = vec![".ts", ".tsx", ".js", ".jsx"];
    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_num, line) in content.lines().enumerate() {
                    // Check for string concatenation in SQL queries
                    if (line.contains("query") || line.contains("sql")) && line.contains('+') {
                        findings.push(Finding {
                            severity: "HIGH".to_string(),
                            category: "SQL Injection".to_string(),
                            description: "Potential SQL injection: string concatenation in query".to_string(),
                            file: file.display().to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

fn check_xss_risks(path: &Path, findings: &mut Vec<Finding>) {
    let extensions = vec![".tsx", ".jsx", ".vue"];
    for ext in &extensions {
        let files = find_files(path, ext);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_num, line) in content.lines().enumerate() {
                    // Check for dangerouslySetInnerHTML or v-html
                    if line.contains("dangerouslySetInnerHTML") || line.contains("v-html") {
                        findings.push(Finding {
                            severity: "MEDIUM".to_string(),
                            category: "XSS Risk".to_string(),
                            description: "Direct HTML injection detected".to_string(),
                            file: file.display().to_string(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
    }
}

fn check_security_configs(path: &Path, findings: &mut Vec<Finding>) {
    // Check for .env.example
    if !path.join(".env.example").exists() {
        findings.push(Finding {
            severity: "LOW".to_string(),
            category: "Security Config".to_string(),
            description: "Missing .env.example file".to_string(),
            file: ".env.example".to_string(),
            line: 0,
        });
    }

    // Check for .gitignore
    if !path.join(".gitignore").exists() {
        findings.push(Finding {
            severity: "MEDIUM".to_string(),
            category: "Security Config".to_string(),
            description: "Missing .gitignore file".to_string(),
            file: ".gitignore".to_string(),
            line: 0,
        });
    }
}

fn find_files(path: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.file_name().map_or(false, |n| {
                n.to_string_lossy().starts_with('.') || n.to_string_lossy() == "node_modules" || n.to_string_lossy() == "target"
            }) {
                files.extend(find_files(&path, extension));
            } else if path.extension().map_or(false, |e| e.to_string_lossy() == &extension[1..]) {
                files.push(path);
            }
        }
    }
    files
}

fn generate_report(findings: &[Finding], _export_format: &str) -> String {
    let mut report = String::new();

    report.push_str("# Security Audit Report\n\n");
    report.push_str(&format!("**Generated by:** SNIPE-CODE CLI\n"));
    report.push_str(&format!("**Date:** {}\n", chrono_placeholder()));
    report.push_str(&format!("**Total Findings:** {}\n\n", findings.len()));

    // Summary
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    report.push_str("## Summary\n\n");
    report.push_str(&format!("- 🔴 HIGH: {}\n", high));
    report.push_str(&format!("- 🟡 MEDIUM: {}\n", medium));
    report.push_str(&format!("- 🟢 LOW: {}\n\n", low));

    // Findings
    if findings.is_empty() {
        report.push_str("## ✅ No Issues Found\n\n");
        report.push_str("Your codebase looks clean! No security issues detected.\n");
    } else {
        report.push_str("## Findings\n\n");
        for (i, finding) in findings.iter().enumerate() {
            let icon = match finding.severity.as_str() {
                "HIGH" => "🔴",
                "MEDIUM" => "🟡",
                "LOW" => "🟢",
                _ => "⚪",
            };

            report.push_str(&format!("### {} {}. {} [{}]\n", icon, i + 1, finding.category, finding.severity));
            report.push_str(&format!("**File:** `{}` (line {})\n", finding.file, finding.line));
            report.push_str(&format!("**Description:** {}\n\n", finding.description));
        }
    }

    report.push_str("---\n");
    report.push_str("*Report generated by SNIPE-CODE CLI*\n");

    report
}

fn chrono_placeholder() -> String {
    "2026-07-28".to_string()
}
