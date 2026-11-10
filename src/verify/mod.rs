//! Verification engine: one code path for the `verify` command and for the CI
//! harness that checks generated projects.

pub mod detect;
pub mod exec;

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use exec::{CommandSpec, RunError};

/// The gates, in the fixed order they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Install,
    Lint,
    Typecheck,
    Test,
    Build,
    Security,
}

impl Gate {
    pub fn name(self) -> &'static str {
        match self {
            Gate::Install => "install",
            Gate::Lint => "lint",
            Gate::Typecheck => "typecheck",
            Gate::Test => "test",
            Gate::Build => "build",
            Gate::Security => "security",
        }
    }

    pub fn parse(token: &str) -> Option<Gate> {
        match token {
            "install" => Some(Gate::Install),
            "lint" => Some(Gate::Lint),
            "typecheck" => Some(Gate::Typecheck),
            "test" => Some(Gate::Test),
            "build" => Some(Gate::Build),
            "security" => Some(Gate::Security),
            _ => None,
        }
    }

    pub const ORDER: [Gate; 6] = [
        Gate::Install,
        Gate::Lint,
        Gate::Typecheck,
        Gate::Test,
        Gate::Build,
        Gate::Security,
    ];
}

/// A gate's plan: the commands it would run, or why it does not apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatePlan {
    pub gate: Gate,
    pub commands: Vec<CommandSpec>,
    /// Set when the project declares nothing for this gate.
    pub not_applicable: Option<String>,
}

impl GatePlan {
    pub fn applies(&self) -> bool {
        self.not_applicable.is_none()
    }
}

/// One recognized part of a project: a directory and the ecosystem read there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartPlan {
    pub dir: PathBuf,
    pub root: PathBuf,
    pub ecosystem: &'static str,
    pub gates: Vec<GatePlan>,
}

impl PartPlan {
    /// How the report names this part: its directory relative to the root.
    pub fn label(&self) -> String {
        match self.dir.strip_prefix(&self.root) {
            Ok(relative) if relative.as_os_str().is_empty() => ".".to_string(),
            Ok(relative) => relative.display().to_string(),
            Err(_) => self.dir.display().to_string(),
        }
    }
}

/// A gate's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateState {
    Passed,
    /// The message is the command failure, without the part it belongs to.
    Failed(String),
    /// The gate was refused by a safety preflight or approval policy.
    Blocked(String),
    /// The project declares nothing for this gate.
    NotApplicable(String),
    /// The tool is not installed on this machine.
    Unavailable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverallStatus {
    Passed,
    Failed,
    Incomplete,
    NoExecution,
}

impl OverallStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
            Self::NoExecution => "no-execution",
        }
    }
}

impl GateState {
    pub fn name(&self) -> &'static str {
        match self {
            GateState::Passed => "passed",
            GateState::Failed(_) => "failed",
            GateState::Blocked(_) => "blocked",
            GateState::NotApplicable(_) => "not-applicable",
            GateState::Unavailable(_) => "unavailable",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            GateState::Passed => None,
            GateState::Failed(message)
            | GateState::Blocked(message)
            | GateState::NotApplicable(message)
            | GateState::Unavailable(message) => Some(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    pub gate: Gate,
    pub state: GateState,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartOutcome {
    pub label: String,
    pub ecosystem: &'static str,
    pub outcomes: Vec<GateOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    pub root: PathBuf,
    pub parts: Vec<PartOutcome>,
}

impl Report {
    /// Gates that actually ran to completion or failure.
    pub fn executed(&self) -> usize {
        self.parts
            .iter()
            .flat_map(|part| &part.outcomes)
            .filter(|outcome| matches!(outcome.state, GateState::Passed | GateState::Failed(_)))
            .count()
    }

    pub fn count(&self, state: &str) -> usize {
        self.parts
            .iter()
            .flat_map(|part| &part.outcomes)
            .filter(|outcome| outcome.state.name() == state)
            .count()
    }

    pub fn status(&self) -> OverallStatus {
        if self.count("failed") > 0 {
            OverallStatus::Failed
        } else if self.executed() == 0 {
            OverallStatus::NoExecution
        } else if self.count("unavailable") > 0 || self.count("blocked") > 0 {
            OverallStatus::Incomplete
        } else {
            OverallStatus::Passed
        }
    }

    pub fn blocked_reason(&self) -> Option<(&str, Gate, &str, &str)> {
        self.parts.iter().find_map(|part| {
            part.outcomes
                .iter()
                .find_map(|outcome| match &outcome.state {
                    GateState::Blocked(message) => Some((
                        part.label.as_str(),
                        outcome.gate,
                        outcome.command.as_str(),
                        message.as_str(),
                    )),
                    _ => None,
                })
        })
    }

    pub fn incomplete_reason(&self) -> Option<(&str, Gate, &str, &str)> {
        self.parts.iter().find_map(|part| {
            part.outcomes
                .iter()
                .find_map(|outcome| match &outcome.state {
                    GateState::Unavailable(message) => Some((
                        part.label.as_str(),
                        outcome.gate,
                        outcome.command.as_str(),
                        message.as_str(),
                    )),
                    _ => None,
                })
        })
    }

    /// The first failing gate, as (part, gate, command, message).
    pub fn first_failure(&self) -> Option<(&str, Gate, &str, &str)> {
        self.parts.iter().find_map(|part| {
            part.outcomes
                .iter()
                .find_map(|outcome| match &outcome.state {
                    GateState::Failed(message) => Some((
                        part.label.as_str(),
                        outcome.gate,
                        outcome.command.as_str(),
                        message.as_str(),
                    )),
                    _ => None,
                })
        })
    }
}

/// `0` complete pass, `1` failure, `2` usage error, `3` no execution,
/// `4` partial or blocked verification.
pub fn exit_status(report: &Report) -> i32 {
    match report.status() {
        OverallStatus::Passed => 0,
        OverallStatus::Failed => 1,
        OverallStatus::NoExecution => 3,
        OverallStatus::Incomplete => 4,
    }
}

/// Recorded on the install gate when `--no-install` skips it.
pub const SKIP_INSTALL_REASON: &str = "skipped by --no-install";

/// How the `verify` command was invoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub path: PathBuf,
    pub only: Option<Gate>,
    pub security: bool,
    /// Leave the install gate alone and run everything else.
    pub skip_install: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            only: None,
            security: false,
            skip_install: false,
        }
    }
}

impl Options {
    /// Parses the arguments that follow the `verify` command.
    pub fn parse(args: &[String]) -> Result<Options, String> {
        let mut options = Options::default();
        let mut path_seen = false;
        let mut index = 0;
        while index < args.len() {
            let token = args[index].as_str();
            match token {
                "--security" => {
                    options.security = true;
                    index += 1;
                }
                "--no-install" => {
                    options.skip_install = true;
                    index += 1;
                }
                "--only" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "missing value for --only".to_string())?;
                    options.only =
                        Some(Gate::parse(value).ok_or_else(|| format!("unknown gate {value}"))?);
                    index += 2;
                }
                other if other.starts_with('-') => {
                    return Err(format!("unknown option {other}"));
                }
                other => {
                    if path_seen {
                        return Err(format!("unexpected argument {other}"));
                    }
                    options.path = PathBuf::from(other);
                    path_seen = true;
                    index += 1;
                }
            }
        }
        if options.only == Some(Gate::Security) {
            options.security = true;
        }
        if options.skip_install && options.only == Some(Gate::Install) {
            return Err("--no-install cannot be combined with --only install".to_string());
        }
        Ok(options)
    }
}

/// A substituted command runner, so the gate contract can be tested without
/// spawning a real toolchain.
pub type Runner<'a> = &'a dyn Fn(&CommandSpec, &str) -> Result<(), RunError>;

/// Detects the applicable parts and runs the selected gates.
///
/// `Err` is a usage error: a selector naming a gate that does not apply to the
/// project. An empty report means nothing recognizable was found.
pub type Approval<'a> = &'a dyn Fn(&GatePlan, &PartPlan, Option<&str>) -> bool;

pub fn run(root: &Path, options: &Options, runner: Runner<'_>) -> Result<Report, String> {
    run_with_approval(root, options, runner, &approve_interactively)
}

pub fn run_with_approval(
    root: &Path,
    options: &Options,
    runner: Runner<'_>,
    approval: Approval<'_>,
) -> Result<Report, String> {
    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }
    let mut plans = detect::plan(
        root,
        options.security || options.only == Some(Gate::Security),
    );
    if options.skip_install {
        for part in &mut plans {
            for gate in &mut part.gates {
                if gate.gate == Gate::Install {
                    gate.commands.clear();
                    gate.not_applicable = Some(SKIP_INSTALL_REASON.to_string());
                }
            }
        }
    }
    if let Some(only) = options.only {
        let applies = plans.iter().any(|part| {
            part.gates
                .iter()
                .any(|gate| gate.gate == only && gate.applies())
        });
        if !applies {
            return Err(format!(
                "gate {} does not apply to {}",
                only.name(),
                root.display()
            ));
        }
    }
    let mut report = execute(&plans, options.only, runner, approval);
    report.root = root.to_path_buf();
    Ok(report)
}

fn approve_interactively(plan: &GatePlan, part: &PartPlan, database_target: Option<&str>) -> bool {
    println!(
        "Preflight: {} gate in {}",
        plan.gate.name(),
        part.dir.display()
    );
    for command in &plan.commands {
        println!("  command: {}", command.display());
        println!("  cwd: {}", command.dir.display());
    }
    println!("  effects: {}", gate_effects(plan.gate));
    if let Some(target) = database_target {
        println!("  database target: {target}");
    }
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return false;
    }
    dialoguer::Confirm::new()
        .with_prompt("Approve this exact gate scope?")
        .default(false)
        .interact()
        .unwrap_or(false)
}

fn gate_effects(gate: Gate) -> &'static str {
    match gate {
        Gate::Install => "may download dependencies, write project files, and run install hooks",
        Gate::Test => "may write files, access configured databases, or use network",
        Gate::Security => "may access network to query vulnerability data",
        Gate::Lint | Gate::Typecheck | Gate::Build => {
            "project-defined command; may write files or use network"
        }
    }
}

fn classify_database_target(directory: &Path, gate: Gate) -> Result<String, String> {
    let mut settings = BTreeMap::new();
    for name in [".env", ".env.test", ".env.testing"] {
        if gate == Gate::Install && name != ".env" {
            continue;
        }
        if let Ok(text) = fs::read_to_string(directory.join(name)) {
            settings.extend(parse_env(&text));
        }
    }
    let environment: BTreeMap<_, _> = std::env::vars_os()
        .filter_map(|(key, value)| {
            let key = key.to_str()?.to_string();
            is_database_setting(&key).then(|| (key, value.to_string_lossy().into_owned()))
        })
        .collect();
    settings.extend(environment);
    classify_settings(directory, &settings)
}

fn parse_env(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let (key, value) = line
                .trim()
                .strip_prefix("export ")
                .unwrap_or(line.trim())
                .split_once('=')?;
            let key = key.trim();
            if !is_database_setting(key) {
                return None;
            }
            let value = value.trim().trim_matches(['\'', '"']);
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn is_database_setting(key: &str) -> bool {
    matches!(
        key,
        "DATABASE_URL"
            | "DB_URL"
            | "DB_CONNECTION"
            | "DB_HOST"
            | "DB_DATABASE"
            | "PGHOST"
            | "PGDATABASE"
    )
}

fn classify_settings(
    directory: &Path,
    settings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(url) = settings
        .get("DATABASE_URL")
        .or_else(|| settings.get("DB_URL"))
    {
        if url.starts_with("sqlite:") {
            return Ok(format!(
                "local SQLite target {}",
                database_target_label(directory, url)
            ));
        }
        let Some((host, database)) = parse_database_url(url) else {
            return Err("database URL target is ambiguous; gate blocked".to_string());
        };
        return classify_host_database(&host, &database);
    }

    if settings
        .get("DB_CONNECTION")
        .is_some_and(|value| value.eq_ignore_ascii_case("sqlite"))
    {
        let database = settings
            .get("DB_DATABASE")
            .map(String::as_str)
            .unwrap_or(":memory:");
        if looks_production(database) {
            return Err("production-like database target; gate blocked".to_string());
        }
        return Ok(format!(
            "local SQLite target {}",
            database_target_label(directory, database)
        ));
    }

    let host = settings.get("DB_HOST").or_else(|| settings.get("PGHOST"));
    let database = settings
        .get("DB_DATABASE")
        .or_else(|| settings.get("PGDATABASE"));
    let (Some(host), Some(database)) = (host, database) else {
        return Err("database target unknown; gate blocked".to_string());
    };
    classify_host_database(host, database)
}

fn classify_host_database(host: &str, database: &str) -> Result<String, String> {
    if looks_production(host) || looks_production(database) {
        return Err("production-like database target; gate blocked".to_string());
    }
    if !matches!(
        host.to_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    ) {
        return Err("remote or ambiguous database host; gate blocked".to_string());
    }
    Ok(format!("local database {database} at {host}"))
}

fn database_target_label(directory: &Path, target: &str) -> String {
    if target == ":memory:" || target.ends_with(":memory:") {
        return ":memory:".to_string();
    }
    let path = Path::new(target);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    };
    resolved
        .canonicalize()
        .unwrap_or(resolved)
        .display()
        .to_string()
}

fn parse_database_url(url: &str) -> Option<(String, String)> {
    let (_, rest) = url.split_once("://")?;
    let authority_and_path = rest.split(['?', '#']).next()?;
    let (authority, path) = authority_and_path.split_once('/')?;
    let authority = authority.rsplit('@').next()?;
    let host = if authority.starts_with('[') {
        authority
            .split(']')
            .next()?
            .trim_start_matches('[')
            .to_string()
    } else {
        authority.split(':').next()?.to_string()
    };
    let database = path.split('/').next()?.to_string();
    if host.is_empty() || database.is_empty() {
        return None;
    }
    Some((host, database))
}

fn looks_production(value: &str) -> bool {
    value
        .to_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| matches!(token, "prod" | "production" | "live" | "main"))
}

/// Runs each plan's gates in the fixed order, recording one outcome per gate.
fn execute(
    plans: &[PartPlan],
    only: Option<Gate>,
    runner: Runner<'_>,
    approval: Approval<'_>,
) -> Report {
    let mut parts = Vec::new();
    for plan in plans {
        let label = plan.label();
        let mut outcomes = Vec::new();
        for gate in Gate::ORDER {
            if !plan.gates.iter().any(|planned| planned.gate == gate) {
                continue;
            }
            if let Some(selected) = only {
                if selected != gate {
                    continue;
                }
            }
            let planned = plan
                .gates
                .iter()
                .find(|planned| planned.gate == gate)
                .expect("gate presence checked above");
            let command = planned
                .commands
                .iter()
                .map(CommandSpec::display)
                .collect::<Vec<_>>()
                .join(" && ");
            if let Some(reason) = &planned.not_applicable {
                outcomes.push(GateOutcome {
                    gate,
                    state: GateState::NotApplicable(reason.clone()),
                    command: String::new(),
                });
                continue;
            }
            let database_target = if matches!(gate, Gate::Install | Gate::Test) {
                match classify_database_target(&plan.dir, gate) {
                    Ok(target) => Some(target),
                    Err(reason) => {
                        outcomes.push(GateOutcome {
                            gate,
                            state: GateState::Blocked(reason),
                            command,
                        });
                        continue;
                    }
                }
            } else {
                None
            };
            if !approval(planned, plan, database_target.as_deref()) {
                outcomes.push(GateOutcome {
                    gate,
                    state: GateState::Blocked(
                        "explicit approval for this exact scope was not granted".to_string(),
                    ),
                    command,
                });
                continue;
            }
            let mut state = GateState::Passed;
            for spec in &planned.commands {
                match runner(spec, &label) {
                    Ok(()) => {}
                    Err(RunError::Failed(message)) => {
                        state = GateState::Failed(message);
                        break;
                    }
                    Err(RunError::Unavailable(message)) => {
                        state = GateState::Unavailable(message);
                        break;
                    }
                }
            }
            outcomes.push(GateOutcome {
                gate,
                state,
                command,
            });
        }
        parts.push(PartOutcome {
            label,
            ecosystem: plan.ecosystem,
            outcomes,
        });
    }
    Report {
        root: plans
            .first()
            .map(|plan| plan.root.clone())
            .unwrap_or_default(),
        parts,
    }
}

/// Prints the per-gate report and a single overall result.
pub fn print_report(report: &Report) {
    println!("verify: {}", report.root.display());
    for part in &report.parts {
        println!("  [{}] {}", part.label, part.ecosystem);
        for outcome in &part.outcomes {
            let detail = match outcome.state.detail() {
                Some(detail) if outcome.command.is_empty() => detail.to_string(),
                Some(detail) => format!("{} — {detail}", outcome.command),
                None => outcome.command.clone(),
            };
            println!(
                "    {:<14} {:<10} {}",
                outcome.state.name(),
                outcome.gate.name(),
                detail
            );
        }
    }
    println!("{}", summary(report));
}

pub fn render_json(report: &Report) -> Result<String, String> {
    let parts: Vec<_> = report
        .parts
        .iter()
        .map(|part| {
            let gates: Vec<_> = part
                .outcomes
                .iter()
                .map(|outcome| {
                    serde_json::json!({
                        "gate": outcome.gate.name(),
                        "status": outcome.state.name(),
                        "command": outcome.command,
                        "detail": outcome.state.detail(),
                    })
                })
                .collect();
            serde_json::json!({
                "label": part.label,
                "ecosystem": part.ecosystem,
                "gates": gates,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "root": report.root,
        "status": report.status().name(),
        "exit_code": exit_status(report),
        "parts": parts,
    }))
    .map_err(|error| error.to_string())
}

/// The overall result line.
pub fn summary(report: &Report) -> String {
    let failed = report.count("failed");
    let passed = report.count("passed");
    let not_applicable = report.count("not-applicable");
    let unavailable = report.count("unavailable");
    let blocked = report.count("blocked");
    let result = match report.status() {
        OverallStatus::Passed => "PASS",
        OverallStatus::Failed => "FAIL",
        OverallStatus::Incomplete => "INCOMPLETE",
        OverallStatus::NoExecution => "NOTHING EXECUTED",
    };
    format!(
        "result: {result} ({}) — {passed} passed, {failed} failed, {not_applicable} not applicable, {unavailable} unavailable, {blocked} blocked",
        report.status().name()
    )
}
