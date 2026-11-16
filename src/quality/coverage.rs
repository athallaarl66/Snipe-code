use super::{
    Config, Coverage, MAX_INPUT, Metric, Policy, VERSION, excluded, relative_path, source,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn read_input(path: &Path) -> Result<String, &'static str> {
    let meta = fs::symlink_metadata(path).map_err(|_| "report missing or unreadable")?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err("report must be a regular file");
    }
    let mut data = String::new();
    fs::File::open(path)
        .map_err(|_| "report unreadable")?
        .take(MAX_INPUT + 1)
        .read_to_string(&mut data)
        .map_err(|_| "report is not UTF-8")?;
    if data.len() > MAX_INPUT as usize {
        return Err("report exceeds input budget");
    }
    Ok(data)
}
pub fn coverage_metric(root: &Path, p: &Policy, report: Option<&Path>) -> Result<Metric, String> {
    Config {
        version: VERSION,
        enabled: true,
        metrics: vec![p.clone()],
    }
    .validate()?;
    if p.id != "lcov-line-coverage" {
        return Err("not a coverage metric".into());
    }
    let parsed = report
        .ok_or("coverage report missing")
        .and_then(read_input)
        .and_then(|text| parse(root, p, &text));
    Ok(match parsed {
        Ok((provenance, files)) => {
            let value = provenance.covered_lines * 10_000 / provenance.total_lines;
            let mut m = source::outcome(p, Ok((value, files)));
            m.coverage = Some(provenance);
            m
        }
        Err(reason) => source::outcome(p, Err(reason)),
    })
}
fn number(s: &str) -> Result<u64, &'static str> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid LCOV count");
    }
    s.parse().map_err(|_| "invalid LCOV count")
}
fn source_path(root: &Path, s: &str) -> Result<String, &'static str> {
    let input = Path::new(s);
    let root = root
        .canonicalize()
        .map_err(|_| "coverage root unavailable")?;
    let rel = if input.is_absolute() {
        input
            .strip_prefix(&root)
            .map_err(|_| "coverage path outside root")?
    } else {
        input
    };
    let rel = rel
        .to_str()
        .ok_or("invalid coverage path")?
        .replace('\\', "/");
    if !relative_path(&rel) {
        return Err("invalid coverage path");
    }
    let mut current = root.clone();
    for part in rel.split('/') {
        current.push(part);
        let meta = fs::symlink_metadata(&current).map_err(|_| "coverage source missing")?;
        if meta.file_type().is_symlink() {
            return Err("coverage symlink rejected");
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("coverage reparse point rejected");
            }
        }
    }
    if !current.is_file()
        || !current
            .canonicalize()
            .map_err(|_| "coverage source missing")?
            .starts_with(&root)
    {
        return Err("coverage path outside root");
    }
    Ok(rel)
}
fn parse(root: &Path, p: &Policy, text: &str) -> Result<(Coverage, Vec<String>), &'static str> {
    let mut active = None;
    let mut lines = BTreeMap::new();
    let mut summaries = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut files = vec![];
    let (mut covered, mut total) = (0u64, 0u64);
    for (index, row) in text.lines().enumerate() {
        if index >= 100_000 {
            return Err("LCOV record budget exceeded");
        }
        if row == "end_of_record" {
            let path: String = active.take().ok_or("LCOV missing SF")?;
            if !seen.insert(path.clone()) || seen.len() > 4096 || lines.is_empty() {
                return Err("invalid or duplicate LCOV file record");
            }
            let count = lines.len() as u64;
            let hits = lines.values().filter(|&&n| n > 0).count() as u64;
            if summaries.get("LF").is_some_and(|&n| n != count)
                || summaries.get("LH").is_some_and(|&n| n != hits)
            {
                return Err("LCOV summaries disagree with DA records");
            }
            let source = crate::audit::read_bounded_within_root(root, &root.join(&path), MAX_INPUT)
                .ok_or("coverage source unreadable or too large")?;
            if lines
                .keys()
                .any(|&line| line > source.lines().count() as u64)
            {
                return Err("LCOV line outside source file");
            }
            if !excluded(&path, &p.exclusions) {
                total += count;
                covered += hits;
                files.push(path);
            }
            lines.clear();
            summaries.clear();
            continue;
        }
        let (key, value) = row
            .split_once(':')
            .ok_or("unsupported or malformed LCOV report")?;
        match key {
            "TN" if active.is_none() => {}
            "SF" if active.is_none() => {
                active = Some(source_path(root, value)?);
            }
            "DA" if active.is_some() => {
                let parts: Vec<_> = value.split(',').collect();
                if !(2..=3).contains(&parts.len()) {
                    return Err("invalid LCOV DA record");
                }
                let line = number(parts[0])?;
                let count = number(parts[1])?;
                if line == 0 || lines.insert(line, count).is_some() {
                    return Err("invalid or duplicate LCOV line");
                }
            }
            "LF" | "LH" | "FNF" | "FNH" | "BRF" | "BRH" if active.is_some() => {
                if summaries.insert(key, number(value)?).is_some() {
                    return Err("duplicate LCOV summary");
                }
            }
            "FN" | "FNDA" | "BRDA" if active.is_some() && !value.is_empty() => {}
            _ => return Err("unsupported or malformed LCOV report"),
        }
    }
    if active.is_some() || total == 0 {
        return Err("LCOV incomplete or no included lines");
    }
    files.sort();
    Ok((
        Coverage {
            format: "lcov-v1-line-records".into(),
            payload_sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
            covered_lines: covered,
            total_lines: total,
        },
        files,
    ))
}
