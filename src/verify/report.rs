//! Explicit verification report requests and deterministic renderers.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{Report, exit_status, render_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Junit,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub format: Format,
    pub output: PathBuf,
    pub overwrite: bool,
}

/// Extract report flags before parsing verification options or running gates.
/// `--json` remains the existing stdout JSON flag and is not consumed here.
pub fn parse_args(args: &[String]) -> Result<(Vec<String>, Option<Request>), String> {
    let mut rest = Vec::new();
    let mut format = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--report" | "--output" => {
                let flag = &args[index];
                let value = args
                    .get(index + 1)
                    .filter(|v| !v.is_empty() && !v.starts_with('-'))
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                if flag == "--report" {
                    if format.is_some() {
                        return Err("duplicate --report".into());
                    }
                    format = Some(match value.as_str() {
                        "json" => Format::Json,
                        "junit" => Format::Junit,
                        "md" => Format::Markdown,
                        _ => {
                            return Err(format!(
                                "unsupported report format {value}; choose json, junit, or md"
                            ));
                        }
                    });
                } else {
                    if output.is_some() {
                        return Err("duplicate --output".into());
                    }
                    output = Some(PathBuf::from(value));
                }
                index += 2;
            }
            "--overwrite" => {
                if overwrite {
                    return Err("duplicate --overwrite".into());
                }
                overwrite = true;
                index += 1;
            }
            _ => {
                rest.push(args[index].clone());
                index += 1;
            }
        }
    }
    match (format, output) {
        (Some(format), Some(output)) => {
            if rest.iter().any(|arg| arg == "--json") {
                return Err("--json cannot be combined with a file report".into());
            }
            Ok((
                rest,
                Some(Request {
                    format,
                    output,
                    overwrite,
                }),
            ))
        }
        (None, None) if !overwrite => Ok((rest, None)),
        _ => {
            Err("--report and --output are required together; --overwrite requires a report".into())
        }
    }
}

/// Write exactly the requested destination. Without overwrite, creation is atomic.
pub fn write(request: &Request, report: &Report) -> Result<(), String> {
    let content = match request.format {
        Format::Json => render_json(report)?,
        Format::Junit => render_junit(report),
        Format::Markdown => render_markdown(report),
    };
    if !request.overwrite {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&request.output)
            .map_err(|e| format!("cannot write {}: {e}", request.output.display()))?;
        return file
            .write_all(content.as_bytes())
            .and_then(|()| file.flush())
            .map_err(|e| format!("cannot write {}: {e}", request.output.display()));
    }

    let parent = request
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = request.output.file_name().ok_or_else(|| {
        format!(
            "cannot write {}: output has no file name",
            request.output.display()
        )
    })?;
    let mut temp = None;
    for attempt in 0..32 {
        let candidate = parent.join(format!(
            ".{}.{}.{}.tmp",
            name.to_string_lossy(),
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temp = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "cannot write {}: {error}",
                    request.output.display()
                ));
            }
        }
    }
    let Some((temp_path, mut file)) = temp else {
        return Err(format!(
            "cannot allocate temporary report beside {}",
            request.output.display()
        ));
    };
    let result = file
        .write_all(content.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "cannot write {}: {error}",
            request.output.display()
        ));
    }
    if let Err(error) = fs::rename(&temp_path, &request.output) {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "cannot replace {}: {error}",
            request.output.display()
        ));
    }
    Ok(())
}

fn xml(value: &str) -> String {
    let mut result = String::new();
    for ch in value.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            '\t' => result.push_str("&#9;"),
            '\n' => result.push_str("&#10;"),
            '\r' => result.push_str("&#13;"),
            c if (' '..='\u{d7ff}').contains(&c)
                || ('\u{e000}'..='\u{fffd}').contains(&c)
                || ('\u{10000}'..='\u{10ffff}').contains(&c) =>
            {
                result.push(c)
            }
            _ => result.push('\u{fffd}'),
        }
    }
    result
}

pub fn render_junit(report: &Report) -> String {
    let tests: usize = report.parts.iter().map(|p| p.outcomes.len()).sum();
    let failures = report.count("failed");
    let skipped =
        report.count("blocked") + report.count("unavailable") + report.count("not-applicable");
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuite name=\"verification\" tests=\"{tests}\" failures=\"{failures}\" skipped=\"{skipped}\">\n  <properties><property name=\"root\" value=\"{}\"/><property name=\"status\" value=\"{}\"/><property name=\"exit_code\" value=\"{}\"/></properties>\n",
        xml(&report.root.to_string_lossy()),
        report.status().name(),
        exit_status(report)
    );
    for part in &report.parts {
        for outcome in &part.outcomes {
            let state = outcome.state.name();
            let detail = outcome.state.detail().unwrap_or("");
            out.push_str(&format!("  <testcase classname=\"{} ({})\" name=\"{}\">\n    <properties><property name=\"state\" value=\"{state}\"/><property name=\"ecosystem\" value=\"{}\"/><property name=\"command\" value=\"{}\"/></properties>\n", xml(&part.label), xml(part.ecosystem), outcome.gate.name(), xml(part.ecosystem), xml(&outcome.command)));
            if state == "failed" {
                out.push_str(&format!("    <failure message=\"{}\"/>\n", xml(detail)));
            } else if state != "passed" {
                out.push_str(&format!(
                    "    <skipped message=\"{}\"/>\n",
                    xml(&format!("{state}: {detail}"))
                ));
            }
            out.push_str("  </testcase>\n");
        }
    }
    out.push_str("</testsuite>\n");
    out
}

fn md(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "\\|")
        .replace('\r', " ")
        .replace('\n', "<br>")
}

pub fn render_markdown(report: &Report) -> String {
    let mut out = format!(
        "# Verification report\n\nTarget: {}\n\n",
        md(&report.root.to_string_lossy())
    );
    for part in &report.parts {
        out.push_str(&format!(
            "## {} ({})\n\n| Gate | State | Command | Detail |\n| --- | --- | --- | --- |\n",
            md(&part.label),
            md(part.ecosystem)
        ));
        for outcome in &part.outcomes {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                outcome.gate.name(),
                outcome.state.name(),
                md(&outcome.command),
                md(outcome.state.detail().unwrap_or(""))
            ));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "## Summary\n\n{}\n\nExit code: {}\n",
        super::summary(report),
        exit_status(report)
    ));
    out
}
