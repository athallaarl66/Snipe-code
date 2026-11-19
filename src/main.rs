use std::io::IsTerminal;
use std::path::Path;

use snipe_code::analyze;
use snipe_code::audit;
use snipe_code::commands::new;
use snipe_code::verify;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    dispatch(&args);
}

fn dispatch(args: &[String]) {
    let code = std::process::exit;
    match args.get(1).map(|s| s.as_str()) {
        None => {
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                run_launcher();
            } else {
                usage_error(
                    "Usage: snipe-code <command> [options]. Run 'snipe-code help' for commands.",
                );
            }
        }
        Some("help") | Some("--help") | Some("-h") => match args.get(2) {
            Some(cmd) => print_command_help(cmd),
            None => print_help(),
        },
        Some("faq") => print_faq(),
        Some("version") | Some("--version") => println!("{}", env!("CARGO_PKG_VERSION")),
        Some("new") => run_new_cmd(args),
        Some("analyze") => run_analyze_cmd(args),
        Some("audit") => run_audit_cmd(args),
        Some("scan") => run_scan_cmd(args),
        Some("quality") => run_quality_cmd(args),
        Some("qa") => run_qa_cmd(args),
        Some("advisory") => run_advisory_cmd(args),
        Some("export") => run_export_cmd(args),
        Some("history") => run_history_cmd(args),
        Some("verify") => run_verify_cmd(args),
        // Backward compatible: snipe-code --dry-run
        Some("--dry-run") => run_new_cmd(args),
        Some(other) if other.starts_with('-') => {
            println!("Unknown option: {}", other);
            println!("Run 'snipe-code help' for the command list.");
            code(2);
        }
        Some(other) => {
            println!("Unknown command: {}", other);
            println!("Run 'snipe-code help' for the command list.");
            code(2);
        }
    }
}

const LAUNCHER_ITEMS: [&str; 6] = [
    "Create a new project",
    "Understand an existing project",
    "Run tests and quality checks",
    "Audit project security",
    "View help",
    "Exit",
];

fn select(prompt: &str, items: &[&str]) -> Option<usize> {
    println!("Use arrow keys to choose, Enter to select, Esc to cancel.");
    match dialoguer::Select::new()
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact_opt()
    {
        Ok(choice) => choice,
        Err(error) => {
            eprintln!("Selection cancelled: {error}");
            None
        }
    }
}

fn input(prompt: &str, default: Option<&str>) -> Option<String> {
    let mut field = dialoguer::Input::<String>::new().with_prompt(prompt);
    if let Some(value) = default {
        field = field.default(value.to_owned());
    }
    match field.interact_text() {
        Ok(value) => Some(value),
        Err(error) => {
            eprintln!("Input cancelled: {error}");
            None
        }
    }
}

fn launch_command(command: &str, arguments: &[&str]) {
    let mut args = vec!["snipe-code".to_owned(), command.to_owned()];
    args.extend(arguments.iter().map(|arg| (*arg).to_owned()));
    dispatch(&args);
}

fn run_launcher() {
    loop {
        match select("What would you like to do?", &LAUNCHER_ITEMS) {
            Some(0) => launch_command("new", &[]),
            Some(1) => launch_project_command("analyze"),
            Some(2) => {
                println!(
                    "Verify runs declared commands and may modify files or use the network; it is not a sandbox. Future QA and quality-gate execution is unavailable."
                );
                launch_project_command("verify");
            }
            Some(3) => launch_security(),
            Some(4) => launch_command("help", &[]),
            Some(5) | None => return,
            _ => unreachable!(),
        }
    }
}

fn launch_project_command(command: &str) {
    if let Some(target) = input("Project path", Some(".")) {
        println!("Target: {target}");
        launch_command(command, &[&target]);
    }
}

fn launch_security() {
    let choices = [
        "Scan project (read-only)",
        "Export a saved scan JSON to a security report",
        "View scan history",
        "Legacy audit (writes reports into the target)",
        "Back",
    ];
    match select("Security operation", &choices) {
        Some(0) => launch_project_command("scan"),
        Some(1) => {
            println!("Export writes a security report to the specified directory.");
            let Some(source) = input("Scan JSON path", None) else {
                return;
            };
            let Some(format) = input("Format (md, pdf, or docx)", Some("md")) else {
                return;
            };
            let Some(output) = input("Output directory", None) else {
                return;
            };
            launch_command(
                "export",
                &[&source, "--format", &format, "--output", &output],
            );
        }
        Some(2) => {
            println!(
                "History opens the local history store and may create its database. Use 'snipe-code help history' for all history operations."
            );
            if let Some(0) = select("History", &["List saved scans", "Back"]) {
                launch_command("history", &["list"]);
            }
        }
        Some(3) => {
            println!(
                "Legacy audit writes security report files into the target directory and may replace existing reports."
            );
            launch_project_command("audit");
        }
        _ => {}
    }
}

/// Parses the `new` options. The wizard itself is interactive; these options let
/// a script or CI run it without a terminal.
fn run_new_cmd(args: &[String]) {
    let mut options = new::Options {
        dry_run: args.iter().any(|arg| arg == "--dry-run"),
        json: args.iter().any(|arg| arg == "--json"),
        ..Default::default()
    };

    let mut i = 2;
    while i < args.len() {
        let option = args[i].as_str();
        if option == "--dry-run" || option == "--json" {
            i += 1;
            continue;
        }
        let value = |name: &str| {
            args.get(i + 1)
                .filter(|v| !v.starts_with('-'))
                .cloned()
                .ok_or_else(|| format!("Usage error: {} requires a value.", name))
        };
        match option {
            "--ai-tools" => {
                options.ai_tools = Some(value(option).unwrap_or_else(|e| usage_error(&e)))
            }
            "--name" => options.name = Some(value(option).unwrap_or_else(|e| usage_error(&e))),
            "--template" => {
                options.template = Some(value(option).unwrap_or_else(|e| usage_error(&e)))
            }
            "--frontend" => {
                options.frontend = Some(value(option).unwrap_or_else(|e| usage_error(&e)))
            }
            "--backend" => {
                options.backend = Some(value(option).unwrap_or_else(|e| usage_error(&e)))
            }
            "--database" => {
                options.database = Some(value(option).unwrap_or_else(|e| usage_error(&e)))
            }
            other => usage_error(&format!("Unknown option or positional argument: {}", other)),
        }
        i += 2;
    }

    if let Err(e) = new::run(options) {
        eprintln!("snipe-code: {}", e);
        let usage = e.starts_with("unknown ")
            || e.starts_with("Select at least one frontend or backend")
            || e.contains("valid project name")
            || e.contains("requires");
        std::process::exit(if usage { 2 } else { 1 });
    }
}

fn usage_error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2);
}

fn run_qa_cmd(args: &[String]) {
    let help = "snipe-code qa plan <PROJECT> --input <REQUIREMENTS.json> [--export <FILE>] [--adapter postman-cli]\nPaths are relative to the current directory. Input must be inside PROJECT. Example JSON:\n{\"version\":1,\"requirements\":[{\"id\":\"REQ-1\",\"scenarios\":[{\"id\":\"ok\",\"category\":\"positive\",\"objective\":\"Allow\",\"precondition\":\"Ready\",\"input\":\"Valid\",\"expected_result\":\"Accepted\"},{\"id\":\"bad\",\"category\":\"negative\",\"objective\":\"Reject\",\"precondition\":\"Ready\",\"input\":\"Invalid\",\"expected_result\":\"Denied\"}]}]}\nNo tools or tests run. Export must be a new file outside the project.\n--adapter postman-cli adds a statically reviewed Collection v2.1 under script.collection; target CLI 1.90.0, runtime unverified. Each scenario needs request: {method, url, headers, body, expected_status}. Only synthetic non-secret fixtures; credential headers are rejected. No runner or setup command is available.";
    if matches!(args.get(2).map(String::as_str), Some("--help" | "-h")) && args.len() == 3 {
        println!("{help}");
        return;
    }
    if args.get(2).map(String::as_str) != Some("plan") {
        usage_error(help);
    }
    if args.len() == 4 && matches!(args[3].as_str(), "--help" | "-h") {
        println!("{help}");
        return;
    }
    let mut project = None;
    let mut input = None;
    let mut export = None;
    let mut adapter = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--input" | "--export" | "--adapter" => {
                let option = args[i].as_str();
                i += 1;
                let value = args
                    .get(i)
                    .filter(|s| !s.starts_with('-'))
                    .unwrap_or_else(|| usage_error(help));
                let slot = match option {
                    "--input" => &mut input,
                    "--export" => &mut export,
                    _ => &mut adapter,
                };
                if slot.replace(value.as_str()).is_some() {
                    usage_error(help);
                }
            }
            option if option.starts_with('-') => usage_error(help),
            path if project.is_none() => project = Some(path),
            _ => usage_error(help),
        }
        i += 1;
    }
    if adapter.is_some() && adapter != Some("postman-cli") {
        usage_error("unsupported QA adapter; only postman-cli draft generation is available");
    }
    let project = Path::new(project.unwrap_or_else(|| usage_error(help)));
    let input = Path::new(input.unwrap_or_else(|| usage_error(help)));
    let mut plan = snipe_code::qa::plan(project, input).unwrap_or_else(|error| {
        eprintln!("qa plan: {error}");
        std::process::exit(3);
    });
    if adapter.is_some() {
        snipe_code::qa::postman_draft(&mut plan).unwrap_or_else(|error| {
            eprintln!("qa plan: {error}");
            std::process::exit(3);
        });
    };
    if let Some(destination) = export {
        snipe_code::qa::export(project, Path::new(destination), &plan).unwrap_or_else(|error| {
            eprintln!("qa plan: {error}");
            std::process::exit(3);
        });
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&plan).expect("serialize plan")
    );
}

fn run_scan_cmd(args: &[String]) {
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let mut path = ".";
    let mut path_seen = false;
    let mut json = false;
    let mut save = false;
    let mut snapshot = None;
    let mut tokens = rest.iter();
    while let Some(token) = tokens.next() {
        match token.as_str() {
            "--snapshot" => {
                if snapshot.is_some() {
                    usage_error("duplicate --snapshot");
                }
                snapshot = Some(
                    tokens
                        .next()
                        .filter(|s| !s.starts_with('-'))
                        .unwrap_or_else(|| usage_error("missing --snapshot path")),
                );
            }
            "--json" if !json => json = true,
            "--json" => usage_error("duplicate --json"),
            "--save" if !save => save = true,
            "--save" => usage_error("duplicate --save"),
            option if option.starts_with('-') => usage_error(&format!("unknown option {option}")),
            value if !path_seen => {
                path = value;
                path_seen = true;
            }
            _ => usage_error("multiple scan paths"),
        }
    }
    let mut result = audit::run_audit_scan(path).unwrap_or_else(|error| {
        eprintln!("scan: {error}");
        std::process::exit(3);
    });
    if let Some(snapshot) = snapshot {
        result.sca = Some(snipe_code::sca::analyze(
            Path::new(path),
            Some(Path::new(snapshot)),
            snipe_code::sca::now(),
        ));
    }
    let envelope = snipe_code::history::ScanEnvelope::from_audit(Path::new(path), result)
        .unwrap_or_else(|error| {
            eprintln!("scan: {error}");
            std::process::exit(3);
        });
    if save {
        let data_dir = snipe_code::history::default_data_dir().unwrap_or_else(|error| {
            eprintln!("history: {error}");
            std::process::exit(3);
        });
        let mut store =
            snipe_code::history::HistoryStore::open_in(&data_dir).unwrap_or_else(|error| {
                eprintln!("history: {error}");
                std::process::exit(3);
            });
        store.save(&envelope).unwrap_or_else(|error| {
            eprintln!("history: {error}");
            std::process::exit(3);
        });
        eprintln!("Saved scan history {}", envelope.result.run_id);
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).expect("serialize scan result")
        );
    } else {
        println!(
            "Audit scan: {} HIGH, {} MEDIUM, {} LOW",
            envelope.result.high, envelope.result.medium, envelope.result.low
        );
        if let Some(report) = &envelope.result.sca {
            println!(
                "Dependency analysis: {} ({})",
                report.status, report.coverage
            );
        }
    }
}

fn run_quality_cmd(args: &[String]) {
    use snipe_code::{history, quality};
    let rest = &args[2..];
    if rest.len() == 1 && matches!(rest[0].as_str(), "--help" | "-h") {
        println!("{QUALITY_HELP}");
        return;
    }
    let mut flags = std::collections::BTreeMap::new();
    let mut path = None;
    let mut i = 0;
    while i < rest.len() {
        let token = rest[i].as_str();
        if matches!(token, "--json" | "--save") {
            if flags.insert(token, "").is_some() {
                usage_error("duplicate quality option");
            }
        } else if matches!(
            token,
            "--config" | "--coverage" | "--baseline" | "--base-ref"
        ) {
            i += 1;
            let value = rest
                .get(i)
                .filter(|s| !s.is_empty() && !s.starts_with('-'))
                .unwrap_or_else(|| usage_error("quality option requires a value"));
            if flags.insert(token, value).is_some() {
                usage_error("duplicate quality option");
            }
        } else if token.starts_with('-') || path.is_some() {
            usage_error("invalid quality arguments");
        } else {
            path = Some(token);
        }
        i += 1;
    }
    let config_path = flags
        .get("--config")
        .unwrap_or_else(|| usage_error("quality requires --config"));
    if flags.contains_key("--baseline") != flags.contains_key("--base-ref") {
        usage_error("--baseline and --base-ref must be supplied together");
    }
    let text = quality::read_input(Path::new(config_path)).unwrap_or_else(|e| usage_error(e));
    let config = quality::parse_config(&text).unwrap_or_else(|e| usage_error(&e));
    let root = Path::new(path.unwrap_or("."));
    let result = audit::run_audit_scan(root.to_str().unwrap_or(".")).unwrap_or_else(|_| {
        eprintln!("quality: source root unavailable");
        std::process::exit(3);
    });
    let mut envelope = history::ScanEnvelope::from_audit(root, result).unwrap_or_else(|_| {
        eprintln!("quality: invalid current scan");
        std::process::exit(3);
    });
    let baseline = flags.get("--baseline").and_then(|p| {
        quality::read_input(Path::new(p)).ok()?;
        history::read_scan(Path::new(p)).ok()
    });
    let report = quality::evaluate(
        root,
        &config,
        flags.get("--coverage").map(Path::new),
        &envelope,
        baseline.as_ref(),
        flags.get("--base-ref").copied(),
    )
    .unwrap_or_else(|e| usage_error(&e));
    let code = quality::exit_status(&report);
    if !flags.contains_key("--json") {
        println!("Quality gate: {}", report.state.name());
        for metric in &report.metrics {
            println!(
                "{}: {}, value {:?} {}{}",
                metric.policy.id,
                metric.state.name(),
                metric.value,
                metric.policy.unit,
                metric
                    .reason
                    .as_ref()
                    .map(|r| format!(" ({r})"))
                    .unwrap_or_default()
            );
        }
        if let Some(delta) = &report.delta {
            if let Some(reason) = &delta.reason {
                println!("Baseline: {} ({reason})", delta.state.name());
            }
        }
    }
    envelope.result.quality = Some(report);
    if flags.contains_key("--save") {
        let saved = history::default_data_dir()
            .and_then(|p| history::HistoryStore::open_in(&p))
            .and_then(|mut store| store.save(&envelope));
        if saved.is_err() {
            eprintln!("quality: cannot save history");
            std::process::exit(3);
        }
        eprintln!("Saved scan history {}", envelope.result.run_id);
    }
    if flags.contains_key("--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).expect("serialize quality")
        );
    }
    std::process::exit(code);
}

fn run_advisory_cmd(args: &[String]) {
    if args.len() == 3 && matches!(args[2].as_str(), "--help" | "-h") {
        println!("{ADVISORY_HELP}");
        return;
    }
    if !matches!(args.len(), 6 | 7)
        || args[2] != "refresh"
        || args[4] != "--cache"
        || args[3].starts_with('-')
        || args[5].starts_with('-')
        || (args.len() == 7 && args[6] != "--yes")
    {
        usage_error("usage: snipe-code advisory refresh <PATH> --cache <DIR> --yes");
    }
    if args.len() != 7 {
        usage_error("refresh requires explicit --yes consent");
    }
    let snapshot = snipe_code::sca::refresh(
        Path::new(&args[3]),
        Path::new(&args[5]),
        true,
        snipe_code::sca::fetch_osv,
        snipe_code::sca::now(),
    )
    .unwrap_or_else(|error| {
        eprintln!("advisory: {error}");
        std::process::exit(3);
    });
    println!("Activated snapshot {}", snapshot.display());
}

fn run_history_cmd(args: &[String]) {
    let operation = args
        .get(2)
        .map(String::as_str)
        .unwrap_or_else(|| usage_error("history requires list, show, export, or delete"));
    if !matches!(
        (operation, args.len()),
        ("list", 3) | ("show", 4) | ("export", 6) | ("delete", 4)
    ) || (operation == "export" && args[4] != "--output")
        || (operation != "list" && args[3].starts_with('-'))
        || (operation == "export" && args[5].starts_with('-'))
    {
        usage_error("invalid history arguments");
    }
    let data_dir = snipe_code::history::default_data_dir().unwrap_or_else(|error| {
        eprintln!("history: {error}");
        std::process::exit(3);
    });
    let mut store = snipe_code::history::HistoryStore::open_in(&data_dir).unwrap_or_else(|error| {
        eprintln!("history: {error}");
        std::process::exit(3);
    });
    match operation {
        "list" if args.len() == 3 => {
            let rows = store.list().unwrap_or_else(|error| history_error(&error));
            for row in rows {
                println!("{}\t{}\t{}", row.run_id, row.scanned_at_utc, row.project_id);
            }
        }
        "show" if args.len() == 4 => {
            let result = store
                .get(&args[3])
                .unwrap_or_else(|error| history_error(&error));
            match result {
                Some(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
                None => history_error("unknown run id"),
            }
        }
        "export" if args.len() == 6 && args[4] == "--output" => {
            let result = store
                .get(&args[3])
                .unwrap_or_else(|error| history_error(&error));
            let result = result.unwrap_or_else(|| history_error("unknown run id"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[5])
                .unwrap_or_else(|error| history_error(&error.to_string()));
            use std::io::Write;
            serde_json::to_writer_pretty(&mut file, &result)
                .and_then(|()| file.write_all(b"\n").map_err(serde_json::Error::io))
                .unwrap_or_else(|error| history_error(&error.to_string()));
            println!("Exported {}", args[5]);
        }
        "delete" if args.len() == 4 => {
            let deleted = store
                .delete_project(&args[3])
                .unwrap_or_else(|error| history_error(&error));
            if deleted == 0 {
                history_error("unknown project id");
            }
            println!("Deleted {deleted} scan run(s) for project {}", args[3]);
        }
        "list" | "show" | "export" | "delete" => usage_error("invalid history arguments"),
        _ => usage_error("unknown history operation"),
    }
}

fn history_error(message: &str) -> ! {
    eprintln!("history: {message}");
    std::process::exit(3);
}

fn run_export_cmd(args: &[String]) {
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let mut input = None;
    let mut format = None;
    let mut output = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--format" | "--output" => {
                let option = rest[i].clone();
                let value = rest
                    .get(i + 1)
                    .filter(|value| !value.starts_with('-'))
                    .unwrap_or_else(|| usage_error(&format!("missing value for {option}")));
                let slot = if option == "--format" {
                    &mut format
                } else {
                    &mut output
                };
                if slot.replace(value.clone()).is_some() {
                    usage_error(&format!("duplicate {option}"));
                }
                i += 2;
            }
            token if token.starts_with('-') => usage_error(&format!("unknown option {token}")),
            token if input.is_none() => {
                input = Some(token.to_string());
                i += 1;
            }
            _ => usage_error("multiple scan result paths"),
        }
    }
    let input = input.unwrap_or_else(|| usage_error("missing scan result path"));
    let format = format.unwrap_or_else(|| usage_error("missing --format"));
    let output = output.unwrap_or_else(|| usage_error("missing --output"));
    audit::export_scan_json(Path::new(&input), &format, Path::new(&output)).unwrap_or_else(
        |error| {
            eprintln!("export: {error}");
            std::process::exit(3);
        },
    );
    println!("Exported report to {}", output);
}

fn run_audit_cmd(args: &[String]) {
    if let Err(error) = audit::validate_audit_args(&args[2..]) {
        usage_error(&format!("Usage error: {error}"));
    }
    let mut path = ".".to_string();
    let mut export_opt: Option<String> = None;
    let mut json = false;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--export" {
            if let Some(val) = args.get(i + 1) {
                export_opt = Some(val.clone());
                i += 2;
            } else {
                eprintln!("Usage error: --export membutuhkan nilai.");
                std::process::exit(2);
            }
        } else if args[i] == "--json" {
            json = true;
            i += 1;
        } else {
            path = args[i].clone();
            i += 1;
        }
    }

    let export_format = match export_opt {
        Some(f) if !f.trim().is_empty() => f,
        Some(_) => {
            eprintln!("Usage error: --export tidak boleh kosong.");
            std::process::exit(2);
        }
        None => {
            if console::Term::stdout().is_term() {
                match audit::select_export_formats() {
                    Ok(Some(formats)) => formats.join(","),
                    Ok(None) => {
                        println!("Audit cancelled.");
                        return;
                    }
                    Err(error) => {
                        eprintln!("audit: {error}");
                        std::process::exit(3);
                    }
                }
            } else {
                if json {
                    eprintln!("Terminal non-interaktif — default export: markdown.");
                } else {
                    println!("Terminal non-interaktif — default export: markdown.");
                }
                "md".to_string()
            }
        }
    };

    eprintln!("Compatibility notice: audit writes report files into target directory.");
    match audit::run_audit_report(&path, &export_format) {
        Ok(result) => {
            if json {
                // stdout stays parseable; every human-facing line goes to stderr.
                match audit::render_json(&result) {
                    Ok(text) => println!("{}", text),
                    Err(error) => {
                        eprintln!("Audit failed: {}", error);
                        std::process::exit(3);
                    }
                }
                eprintln!("Audit complete! Exported files:");
                for file in &result.exported_files {
                    eprintln!("  - {}/{}", path, file);
                }
            } else {
                println!("Audit complete! Exported files:");
                for file in &result.exported_files {
                    println!("  - {}/{}", path, file);
                }
            }
            if result.policy_failed {
                eprintln!("Audit policy failed: {} HIGH finding(s)", result.high);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Audit failed: {}", e);
            let code = if e.starts_with("unsupported export format")
                || e.contains("empty token")
                || e.contains("at least one export")
            {
                2
            } else {
                3
            };
            std::process::exit(code);
        }
    }
}

fn run_analyze_cmd(args: &[String]) {
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    if rest.iter().any(|token| token == "--help" || token == "-h") {
        println!("{}", ANALYZE_HELP);
        return;
    }

    let mut path = ".".to_string();
    let mut json = false;
    for token in &rest {
        match token.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => {
                eprintln!("Usage error: opsi tidak dikenal: {}", other);
                std::process::exit(2);
            }
            other => path = other.to_string(),
        }
    }

    let report = match analyze::analyze(Path::new(&path)) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("analyze: {}", error);
            std::process::exit(3);
        }
    };
    if json {
        match analyze::render_json(&report) {
            Ok(text) => println!("{}", text),
            Err(error) => {
                eprintln!("analyze: {}", error);
                std::process::exit(3);
            }
        }
    } else {
        print!("{}", analyze::render_human(&report));
    }
}

fn run_verify_cmd(args: &[String]) {
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    if rest.iter().any(|token| token == "--help" || token == "-h") {
        println!("{}", VERIFY_HELP);
        return;
    }
    let (options_args, request) = verify::report::parse_args(&rest).unwrap_or_else(|error| {
        usage_error(&format!("Usage error: {error}"));
    });
    let json = options_args.iter().any(|token| token == "--json");
    let options_args: Vec<String> = options_args
        .into_iter()
        .filter(|token| token != "--json")
        .collect();
    let options = match verify::Options::parse(&options_args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Usage error: {}", error);
            std::process::exit(2);
        }
    };
    let runner: verify::Runner<'_> = if json || request.is_some() {
        &verify::exec::run_stderr
    } else {
        &verify::exec::run
    };
    let report = match if json || request.is_some() {
        verify::run_stderr(&options.path, &options, runner)
    } else {
        verify::run(&options.path, &options, runner)
    } {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Usage error: {}", error);
            std::process::exit(2);
        }
    };
    if report.parts.is_empty() {
        eprintln!("verify: no gates found in {}", options.path.display());
    }
    if let Some(request) = request {
        if let Err(error) = verify::report::write(&request, &report) {
            eprintln!("verify: report export failed: {error}");
            std::process::exit(3);
        }
        println!("{}", request.output.display());
        std::process::exit(verify::exit_status(&report));
    }
    if json {
        match verify::render_json(&report) {
            Ok(text) => println!("{text}"),
            Err(error) => {
                eprintln!("verify: cannot render JSON: {error}");
                std::process::exit(1);
            }
        }
    } else {
        verify::print_report(&report);
    }
    std::process::exit(verify::exit_status(&report));
}

fn print_help() {
    println!("{}", HELP_TEXT);
    println!();
    print_faq();
}

fn print_faq() {
    println!("{}", FAQ_TEXT);
}

fn print_command_help(cmd: &str) {
    match cmd {
        "analyze" => println!("{}", ANALYZE_HELP),
        "audit" => println!("{}", AUDIT_HELP),
        "scan" => println!(
            "snipe-code scan [PATH] [--json] [--save] [--snapshot <FILE|DIR>] - offline audit; --save stores local history"
        ),
        "advisory" => println!("{ADVISORY_HELP}"),
        "quality" => println!("{QUALITY_HELP}"),
        "qa" => run_qa_cmd(&[String::new(), "qa".into(), "--help".into()]),
        "export" => println!("snipe-code export <scan.json> --format <md,pdf,docx> --output <DIR>"),
        "history" => println!(
            "snipe-code history list | show <run-id> | export <run-id> --output <FILE> | delete <project-id>"
        ),
        "verify" => println!("{}", VERIFY_HELP),
        "new" => println!("{}", NEW_HELP),
        "help" => print_help(),
        _ => {
            println!("Unknown topic: {}", cmd);
            println!("Run 'snipe-code help' for the command list.");
        }
    }
}

const HELP_TEXT: &str = r#"SNIPE-CODE CLI - Rust scaffolding and heuristic security audit

USAGE
    snipe-code [COMMAND] [OPTIONS]

An interactive invocation without a command opens the goal launcher. Without
an interactive terminal, select a command; no project is created.

COMMANDS
    new        Create a project with the generation wizard
    analyze    Report project structure and dependencies (read-only, offline)
    audit      Legacy security audit and report export
    scan       Scan without writing; --save stores local history
    advisory   Refresh the OSV snapshot explicitly using curl and HTTPS
    quality    Analyze Rust metrics, imported coverage, and baseline policy
    qa         Plan reviewable test cases from structured requirements
    export     Export a saved scan to an explicit directory
    history    List, view, export, or delete local scan history
    verify     Run declared project quality gates
    help       Show help (aliases: --help, -h)
    version    Show the compiled version (alias: --version)
    faq        Show installation and troubleshooting guidance

OPTIONS
    --dry-run        (new) Preview without creating files
    --ai-tools <ids> (new) Comma-separated tool IDs, or none
    --name <name>    (new) Set the project name
    --template <id>  (new) Preset: none, company-profile, portfolio, blog
    --frontend <id>  (new) nextjs, react-vite, vue, nuxtjs, none
    --backend <id>   (new) dotnet8, nestjs, go-gin, spring-boot, laravel, none
    --database <id>  (new) postgresql, mysql, none
    --json           Print a structured result where supported
    --export <f>     (audit) Report formats: md, pdf, docx
    --save           (scan) Save the result to local history
    --only <gate>    (verify) Run one gate
    --security       (verify) Include dependency security checks
    --no-install     (verify) Skip the install gate
    --report <f>     (verify) Write a json, junit, or md report with --output
    --version        Show the compiled version

EXAMPLES
    snipe-code                          Open the interactive launcher
    snipe-code new --dry-run            Preview project files
    snipe-code new --ai-tools none --name demo --json
    snipe-code analyze . --json         Read-only project report
    snipe-code scan . --json            Read-only security scan
    snipe-code audit . --export pdf,docx
    snipe-code verify . --only test
    snipe-code verify . --report junit --output verify.xml

Legacy audit writes security reports in the target. Verify runs project tools,
which can change files or access the network. Neither operation needs AI.
Use 'snipe-code help <command>' for details.
"#;

const FAQ_TEXT: &str = r#"FAQ

Q: 'snipe-code' is not recognized as an internal or external command
A: Build with 'cargo build --release' and run the binary from target/release/,
   or install it on PATH with 'cargo install --path . --locked'. The lockfile
   retains the yanked zip dependency required by docx-rs.

Q: How do I uninstall it?
A: Run 'cargo uninstall snipe-code', or delete the portable binary.

Q: Where should I run commands?
A: 'new' creates <current directory>/<project name>. Run it from the desired
   parent directory. Analyze, audit, and verify accept PATH (default: .).

Q: What writes to the project?
A: Legacy audit writes security reports into its target. Verify runs project
   tools that may change files and access the network. For read-only inspection,
   use 'snipe-code analyze .' or 'snipe-code scan .'.
"#;

const NEW_HELP: &str = r#"snipe-code new - Create a project

USAGE
    snipe-code new [--dry-run] [--ai-tools <ids|none>] [--name <n>] [--json]
                   [--template <id>] [--frontend <id>] [--backend <id>] [--database <id>]

WIZARD
    Choose a preset, frontend, backend, database, name, and AI integrations.
    Review the destination and Git choices, then preview, create, go back, or
    cancel. Git initialization and the initial commit require separate choices.
    Presets add files to the framework stack; they are not frameworks.
    Agent selection writes project-local artifacts; it does not run a model,
    authenticate a provider, store credentials, or require an API key.

OPTIONS
    --dry-run            Show planned files without creating them
    --ai-tools <ids>     Generate artifacts for comma-separated agent IDs:
                         agents-md, claude-code, cursor, opencode, antigravity,
                         github-copilot, windsurf, hermes-agent, codex, gemini-cli
                         Use 'none' to skip agent artifacts and the prompt.
    --name <n>           Project name; skip the name prompt
    --template <id>      Preset: none, company-profile, portfolio, blog.
                         Skip the preset prompt.
    --frontend <id>      nextjs, react-vite, vue, nuxtjs, or none
    --backend <id>       dotnet8, nestjs, go-gin, spring-boot, laravel, or none
    --database <id>      postgresql, mysql, or none
    --json               Print the generation result as JSON on stdout

EXAMPLES
    snipe-code new
    snipe-code new --dry-run
    snipe-code new --ai-tools cursor,claude-code
    snipe-code new --ai-tools none --name demo
    snipe-code new --ai-tools claude-code --name demo --json
    snipe-code new --template blog --frontend nextjs --backend none \
                   --database none --name site
"#;

const QUALITY_HELP: &str = "snipe-code quality [PATH] --config <FILE> [--coverage <LCOV>] [--baseline <SCAN_JSON> --base-ref <REF>] [--json] [--save]
Read-only analysis; never runs target tests or installs dependencies.
Coverage is imported, not computed. Baseline Git checks are local/read-only.
Only --save writes local history. Outcomes: pass/fail/unknown/not-applicable.
Exit: 0 pass/disabled, 1 fail, 2 usage/config, 3 runtime/no metric, 4 incomplete.";

const ADVISORY_HELP: &str = "snipe-code advisory refresh <PATH> --cache <DIR> --yes
Sends npm package names and pinned versions to https://api.osv.dev/v1/query.
Requires curl with HTTPS. No source or credentials sent. GHSA records only,
GitHub Advisory Database, CC-BY-4.0. Seven-day snapshot freshness; SHA-256
checks corruption, not authenticity. Scan stays offline; select cache or file
with scan --snapshot <FILE|DIR>. Invalid refresh retains previous snapshot.";

const AUDIT_HELP: &str = r#"snipe-code audit - Legacy project security audit

USAGE
    snipe-code audit [PATH] [--export <md,pdf,docx>] [--json]

DESCRIPTION
    Scan the project and write security reports into the target directory.
    Without --export, choose formats interactively; a non-interactive run
    defaults to Markdown. Existing report files may be replaced.

ARGUMENTS
    PATH         Project directory (default: current directory)

OPTIONS
    --export <md,pdf,docx>   Comma-separated formats, without a prompt
    --json                   Print the result as JSON on stdout; report files
                             are still written and progress goes to stderr.

EXAMPLES
    snipe-code audit .
    snipe-code audit . --export pdf,docx
    snipe-code audit ../app --export md
    snipe-code audit . --export md --json
"#;

const ANALYZE_HELP: &str = r#"snipe-code analyze - Report project structure

USAGE
    snipe-code analyze [PATH] [--json]

DESCRIPTION
    Analyze read-only and offline: languages, line counts, structure, entry
    points, dependencies, and Limits. Dependency, build, and hidden directories
    are skipped; src/bin is retained for Rust source.

ARGUMENTS
    PATH            Project directory (default: current directory)

OPTIONS
    --json          Print the report as JSON on stdout
    -h, --help      Show this help

EXIT STATUS
    0   analysis completed
    2   usage error
    3   missing path or not a directory

NOTES
    Counts include blank lines and comments; they are not lines of code.
    pom.xml and *.csproj are scanned for dependency names. The other supported
    manifests are parsed.

EXAMPLES
    snipe-code analyze .
    snipe-code analyze ../app --json
"#;

const VERIFY_HELP: &str = r#"snipe-code verify - Run project quality gates

USAGE
    snipe-code verify [PATH] [--only <gate>] [--security] [--no-install]
                      [--json | --report <json|junit|md> --output <FILE> [--overwrite]]

DESCRIPTION
    Reads manifests and runs declared install, lint, typecheck, test, and build
    gates. Missing gates are reported as not-applicable. Project commands can
    modify files and access the network. Verification is not a sandbox.
    For read-only inspection use 'snipe-code analyze' or 'snipe-code scan'.

ARGUMENTS
    PATH            Project directory (default: current directory)

OPTIONS
    --only <gate>   Run one applicable gate
    --security      Include dependency checks: npm audit, composer audit,
                    dotnet list package --vulnerable
    --no-install    Skip install; its state is not-applicable
    --json          Print the structured result to stdout
    --report <f>    Write a json, junit, or md verification report
    --output <file> Explicit report destination; required with --report
    --overwrite     Replace an existing report at the explicit destination
    -h, --help      Show this help

SAFETY
    Gates show commands, directories, and known effects before approval.
    Without interactive approval, gates are blocked. Install and test gates
    block explicit remote, production-like, or ambiguous database targets.
    An undetected target is unknown and requires explicit approval; it is not
    proof that the project avoids databases. Preflight does not sandbox project
    commands or prove all their effects.

EXIT STATUS
    0   all applicable gates passed
    1   a gate failed
    2   usage error
    3   no gate executed, or report write failed
    4   incomplete: a gate was unavailable or blocked

EXAMPLES
    snipe-code verify .
    snipe-code verify ../app --only test
    snipe-code verify . --security
    snipe-code verify . --no-install
    snipe-code verify . --report junit --output verify.xml
"#;
