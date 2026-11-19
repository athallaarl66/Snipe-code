//! Verification engine: one code path for the `verify` command and for the CI
//! harness that checks generated projects.

pub mod detect;
pub mod exec;
pub mod report;

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
type Approval<'a> = &'a dyn Fn(&GatePlan, &PartPlan, Option<&str>) -> bool;

/// Run with interactive preview on stderr for machine-readable stdout.
pub fn run_stderr(root: &Path, options: &Options, runner: Runner<'_>) -> Result<Report, String> {
    run_with_approval(root, options, runner, &approve_interactively_stderr, false)
}

pub fn run(root: &Path, options: &Options, runner: Runner<'_>) -> Result<Report, String> {
    run_with_approval(root, options, runner, &approve_interactively, false)
}

fn run_with_approval(
    root: &Path,
    options: &Options,
    runner: Runner<'_>,
    approval: Approval<'_>,
    generated: bool,
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
    let mut report = execute(&plans, options.only, runner, approval, generated);
    report.root = root.to_path_buf();
    Ok(report)
}

/// Generate and verify only a fresh harness-owned project using built-in templates.
pub fn run_generated_project(frontend: &str, backend: &str, database: &str) -> Result<(), String> {
    if std::env::var_os(crate::template::OVERRIDE_ENV).is_some() {
        return Err("verify-generated refuses caller-controlled template overrides".to_string());
    }
    if !crate::stack::FRONTEND_IDS.contains(&frontend)
        || !crate::stack::BACKEND_IDS.contains(&backend)
        || !crate::stack::DATABASE_STACKS
            .iter()
            .any(|db| db.id == database)
        || (frontend == "none" && backend == "none")
    {
        return Err("invalid generated project stack selection".to_string());
    }
    let workspace = exec::temp_workspace("snipe-verify")?;
    let result = (|| {
        let config = crate::generator::GenerateConfig {
            project_name: "generated-app".to_string(),
            template_id: "none".to_string(),
            frontend: frontend.to_string(),
            backend: backend.to_string(),
            database: database.to_string(),
            ai_tools: Vec::new(),
        };
        crate::generator::generate_in(&workspace, &config, false)?;
        let root = workspace.join("generated-app");
        let options = Options {
            path: root.clone(),
            ..Options::default()
        };
        let report = run_with_approval(
            &root,
            &options,
            &exec::run_generated,
            &|gate, part, target| {
                println!(
                    "generated workspace is not a sandbox; network, hardcoded targets and filesystem remain accessible"
                );
                println!("  gate: {} in {}", gate.gate.name(), part.dir.display());
                if let Some(target) = target {
                    println!("  database target: {target}");
                }
                true
            },
            true,
        )?;
        let stack = if backend == "none" { frontend } else { backend };
        if report.parts.is_empty() {
            return Err(format!(
                "unsupported stack {stack}: no verification gates found in {}",
                root.display()
            ));
        }
        print_report(&report);
        match report.first_failure() {
            Some((_, _, _, message)) => Err(format!("{stack}: {message}")),
            None => match exit_status(&report) {
                0 => Ok(()),
                status => Err(format!("{stack}: no gate executed ({status})")),
            },
        }
    })();
    let cleanup = std::fs::remove_dir_all(&workspace).map_err(|e| e.to_string());
    result.and(cleanup)
}

fn approve_interactively(plan: &GatePlan, part: &PartPlan, database_target: Option<&str>) -> bool {
    approve_interactively_to(plan, part, database_target, false)
}

/// CLI approval callback for invocations whose stdout must remain machine-readable.
fn approve_interactively_stderr(
    plan: &GatePlan,
    part: &PartPlan,
    database_target: Option<&str>,
) -> bool {
    approve_interactively_to(plan, part, database_target, true)
}

fn approve_interactively_to(
    plan: &GatePlan,
    part: &PartPlan,
    database_target: Option<&str>,
    stderr: bool,
) -> bool {
    let secrets = exec::database_credentials(&part.dir);
    let line = |text: String| {
        let text = exec::redact(&text, &secrets);
        if stderr {
            eprintln!("{text}");
        } else {
            println!("{text}");
        }
    };
    line(format!(
        "Preflight: {} gate in {}",
        plan.gate.name(),
        part.dir.display()
    ));
    for command in &plan.commands {
        line(format!("  command: {}", command.display()));
        line(format!("  cwd: {}", command.dir.display()));
    }
    line(format!("  effects: {}", gate_effects(plan.gate)));
    if let Some(target) = database_target {
        line(format!("  database target: {target}"));
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

fn classify_database_target(
    directory: &Path,
    _gate: Gate,
    generated: bool,
) -> Result<(String, BTreeMap<String, String>), String> {
    let environment: BTreeMap<_, _> = std::env::vars_os()
        .filter_map(|(key, value)| {
            let key = key.to_str()?.to_string();
            (is_database_setting(&key)
                && !(generated && exec::GENERATED_CLEARED_DB_KEYS.contains(&key.as_str())))
            .then(|| (key, value.to_string_lossy().into_owned()))
        })
        .collect();
    let mut settings = BTreeMap::new();
    for name in [
        ".env",
        ".env.local",
        ".env.test",
        ".env.test.local",
        ".env.testing",
        ".env.testing.local",
    ] {
        match fs::read_to_string(directory.join(name)) {
            Ok(text) => {
                for (key, value) in parse_env(&text)? {
                    if environment.contains_key(&key) {
                        continue;
                    }
                    if settings
                        .get(&key)
                        .is_some_and(|existing| existing != &value)
                    {
                        return Err("conflicting database settings; gate blocked".to_string());
                    }
                    settings.insert(key, value);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err("database configuration could not be read; gate blocked".to_string());
            }
        }
    }
    // Inherited process values override the same dotenv key.
    settings.extend(environment);
    let target = classify_settings(directory, &settings)?;
    Ok((target, settings))
}

fn parse_env(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut settings = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (key, value) = match line.split_once('=') {
            Some(pair) => pair,
            None if is_database_setting(line.trim()) => {
                return Err("invalid database configuration; gate blocked".to_string());
            }
            None => continue,
        };
        let key = key.trim();
        if !is_database_setting(key) {
            continue;
        }
        let value = value.trim();
        let value = if value.starts_with('"') || value.starts_with('\'') {
            let quote = value.chars().next().unwrap();
            if value.len() < 2 || !value.ends_with(quote) {
                return Err("invalid database configuration; gate blocked".to_string());
            }
            &value[1..value.len() - 1]
        } else {
            value
        };
        if settings
            .insert(key.to_string(), value.to_string())
            .is_some()
        {
            return Err("duplicate database settings; gate blocked".to_string());
        }
    }
    Ok(settings)
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
            | "PGHOSTADDR"
            | "PGSERVICE"
            | "PGSERVICEFILE"
    )
}

fn classify_settings(
    directory: &Path,
    settings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if settings.is_empty() {
        return Ok("not detected; database use and target remain unknown".to_string());
    }
    if settings.values().any(|value| value.contains('$')) {
        return Err("database configuration is ambiguous; gate blocked".to_string());
    }
    if ["PGHOSTADDR", "PGSERVICE", "PGSERVICEFILE"]
        .iter()
        .any(|key| settings.contains_key(*key))
    {
        return Err("database routing configuration is ambiguous; gate blocked".to_string());
    }
    if settings.contains_key("DATABASE_URL") && settings.len() > 1
        || settings.contains_key("DB_URL") && settings.len() > 1
        || settings.contains_key("DB_HOST") && settings.contains_key("PGHOST")
        || settings.contains_key("DB_DATABASE") && settings.contains_key("PGDATABASE")
    {
        return Err("conflicting database settings; gate blocked".to_string());
    }
    if let Some(url) = settings
        .get("DATABASE_URL")
        .or_else(|| settings.get("DB_URL"))
    {
        if let Some(target) = url.strip_prefix("sqlite:") {
            if !valid_sqlite_target(target)
                || looks_production(target)
                || looks_production(&database_target_label(directory, target))
            {
                return Err(
                    "SQLite target is ambiguous or production-like; gate blocked".to_string(),
                );
            }
            return Ok("local SQLite target [redacted]".to_string());
        }
        if !["postgres://", "postgresql://", "mysql://"]
            .iter()
            .any(|scheme| url.starts_with(scheme))
            || url.contains(['?', '#', '%', '$'])
        {
            return Err("database URL target is ambiguous; gate blocked".to_string());
        }
        let Some((host, database)) = parse_database_url(url) else {
            return Err("database URL target is ambiguous; gate blocked".to_string());
        };
        return classify_host_database(&host, &database);
    }

    if settings.get("DB_CONNECTION").is_some_and(|value| {
        ![
            "sqlite",
            "mysql",
            "mariadb",
            "pgsql",
            "postgres",
            "postgresql",
        ]
        .iter()
        .any(|supported| value.eq_ignore_ascii_case(supported))
    }) {
        return Err("database connection is ambiguous; gate blocked".to_string());
    }
    if settings
        .get("DB_CONNECTION")
        .is_some_and(|value| value.eq_ignore_ascii_case("sqlite"))
    {
        if settings.contains_key("DB_HOST")
            || settings.contains_key("PGHOST")
            || settings.contains_key("PGDATABASE")
        {
            return Err("conflicting database settings; gate blocked".to_string());
        }
        let database = settings
            .get("DB_DATABASE")
            .map(String::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "SQLite target unknown; gate blocked".to_string())?;
        if !valid_sqlite_target(database)
            || looks_production(database)
            || looks_production(&database_target_label(directory, database))
        {
            return Err("SQLite target is ambiguous or production-like; gate blocked".to_string());
        }
        return Ok("local SQLite target [redacted]".to_string());
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
    if database.trim().is_empty() {
        return Err("database target unknown; gate blocked".to_string());
    }
    if looks_production(host) || looks_production(database) {
        return Err("production-like database target; gate blocked".to_string());
    }
    if !matches!(
        host.to_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    ) {
        return Err("remote or ambiguous database host; gate blocked".to_string());
    }
    Ok(format!("local database [redacted] at {host}"))
}

fn valid_sqlite_target(target: &str) -> bool {
    if target == ":memory:" {
        return true;
    }
    if target.trim().is_empty()
        || target.starts_with("//")
        || target.starts_with("\\\\")
        || target.contains(['?', '#', '%', '$'])
    {
        return false;
    }
    let rest = if target.as_bytes().get(1) == Some(&b':')
        && target.as_bytes()[0].is_ascii_alphabetic()
        && matches!(target.as_bytes().get(2), Some(b'/' | b'\\'))
    {
        &target[2..]
    } else {
        target
    };
    !rest.contains(':')
}

fn database_target_label(directory: &Path, target: &str) -> String {
    if target == ":memory:" {
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
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']')?;
        let port = if suffix.is_empty() {
            None
        } else {
            Some(suffix.strip_prefix(':')?)
        };
        (host, port)
    } else {
        match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if host.is_empty()
        || path.is_empty()
        || path.contains('/')
        || port.is_some_and(|value| value.parse::<u16>().ok().filter(|port| *port > 0).is_none())
    {
        return None;
    }
    Some((host.to_string(), path.to_string()))
}

fn looks_production(value: &str) -> bool {
    let value = value.to_lowercase();
    value.contains("prod")
        || value
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|token| matches!(token, "live" | "main"))
}

/// Runs each plan's gates in the fixed order, recording one outcome per gate.
fn execute(
    plans: &[PartPlan],
    only: Option<Gate>,
    runner: Runner<'_>,
    approval: Approval<'_>,
    generated: bool,
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
                match classify_database_target(&plan.dir, gate, generated) {
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
            // npm scripts are indirect commands: the displayed `npm run test`
            // does not reveal when the underlying script changes.
            let npm_manifest = if plan.ecosystem == "npm" {
                match fs::read(plan.dir.join("package.json")) {
                    Ok(bytes) => Some(bytes),
                    Err(_) => {
                        outcomes.push(GateOutcome {
                            gate,
                            state: GateState::Blocked(
                                "command scope could not be read".to_string(),
                            ),
                            command,
                        });
                        continue;
                    }
                }
            } else {
                None
            };
            if !approval(
                planned,
                plan,
                database_target.as_ref().map(|(label, _)| label.as_str()),
            ) {
                outcomes.push(GateOutcome {
                    gate,
                    state: GateState::Blocked(
                        if database_target
                            .as_ref()
                            .is_some_and(|(target, _)| target.contains("not detected"))
                        {
                            "database target unknown; explicit approval was not granted".to_string()
                        } else {
                            "explicit approval for this exact scope was not granted".to_string()
                        },
                    ),
                    command,
                });
                continue;
            }
            if let Some((_, approved_settings)) = &database_target {
                match classify_database_target(&plan.dir, gate, generated) {
                    Ok((_, current_settings)) if current_settings == *approved_settings => {}
                    _ => {
                        outcomes.push(GateOutcome {
                            gate,
                            state: GateState::Blocked(
                                "database scope changed after approval; gate blocked".to_string(),
                            ),
                            command,
                        });
                        continue;
                    }
                }
            }
            let mut state = GateState::Passed;
            for spec in &planned.commands {
                let current = detect::plan(&plan.root, gate == Gate::Security);
                let scope_matches = current.iter().any(|part| {
                    part.dir == plan.dir
                        && part.ecosystem == plan.ecosystem
                        && part.gates.iter().any(|candidate| candidate == planned)
                });
                let database_matches = database_target.as_ref().is_none_or(|(_, settings)| {
                    classify_database_target(&plan.dir, gate, generated)
                        .is_ok_and(|(_, current)| current == *settings)
                });
                let manifest_matches = npm_manifest.as_ref().is_none_or(|approved| {
                    fs::read(plan.dir.join("package.json"))
                        .is_ok_and(|current| current == *approved)
                });
                if !scope_matches || !database_matches || !manifest_matches {
                    state = GateState::Blocked(
                        "scope changed after approval; gate blocked".to_string(),
                    );
                    break;
                }
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

fn display_report(report: &Report) -> Report {
    let mut secrets = exec::database_credentials(&report.root);
    for part in &report.parts {
        secrets.extend(exec::database_credentials(&report.root.join(&part.label)));
    }
    secrets.sort_by_key(|value| std::cmp::Reverse(value.len()));
    secrets.dedup();
    let clean = |value: &str| exec::redact(value, &secrets);
    let mut result = report.clone();
    result.root = PathBuf::from(clean(&report.root.to_string_lossy()));
    for part in &mut result.parts {
        part.label = clean(&part.label);
        for outcome in &mut part.outcomes {
            outcome.command = clean(&outcome.command);
            match &mut outcome.state {
                GateState::Failed(value)
                | GateState::Blocked(value)
                | GateState::NotApplicable(value)
                | GateState::Unavailable(value) => *value = clean(value),
                GateState::Passed => {}
            }
        }
    }
    result
}

/// Prints the per-gate report and a single overall result.
pub fn print_report(report: &Report) {
    let report = display_report(report);
    let report = &report;
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
    let report = display_report(report);
    let report = &report;
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
        "counts": {
            "passed": report.count("passed"),
            "failed": report.count("failed"),
            "blocked": report.count("blocked"),
            "unavailable": report.count("unavailable"),
            "not-applicable": report.count("not-applicable"),
            "executed": report.executed(),
        },
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

#[cfg(test)]
mod database_safety_tests {
    use super::*;

    #[test]
    fn absent_settings_are_distinct_from_conflicting_settings() {
        let absent = classify_settings(Path::new("."), &BTreeMap::new()).unwrap();
        assert!(absent.contains("not detected"));

        let conflicting = BTreeMap::from([
            (
                "DATABASE_URL".to_string(),
                "postgres://user:secret@localhost/test".to_string(),
            ),
            ("DB_HOST".to_string(), "remote.example".to_string()),
        ]);
        let reason = classify_settings(Path::new("."), &conflicting).unwrap_err();
        assert!(reason.contains("conflicting"));
        assert!(!reason.contains("secret"));
    }

    #[test]
    fn unreadable_database_settings_are_blocked() {
        let root = std::env::temp_dir().join(format!("snipe-env-invalid-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".env"), [0xff]).unwrap();
        let error = classify_database_target(&root, Gate::Test, false).unwrap_err();
        assert!(error.contains("could not be read"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn conflicting_env_files_and_libpq_routing_are_blocked() {
        let root = std::env::temp_dir().join(format!("snipe-db-conflict-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join(".env.test"),
            "DB_HOST=remote.example\nDB_DATABASE=test\n",
        )
        .unwrap();
        fs::write(
            root.join(".env.testing"),
            "DB_HOST=localhost\nDB_DATABASE=test\n",
        )
        .unwrap();
        assert!(classify_database_target(&root, Gate::Test, false).is_err());
        fs::remove_dir_all(root).unwrap();
        for key in ["PGHOSTADDR", "PGSERVICE", "PGSERVICEFILE"] {
            let settings = BTreeMap::from([
                ("PGHOST".to_string(), "localhost".to_string()),
                ("PGDATABASE".to_string(), "test".to_string()),
                (key.to_string(), "remote".to_string()),
            ]);
            assert!(classify_settings(Path::new("."), &settings).is_err());
        }
    }

    #[test]
    fn unsupported_database_url_scheme_is_blocked() {
        let settings = BTreeMap::from([(
            "DATABASE_URL".to_string(),
            "unsupported://localhost/test".to_string(),
        )]);
        assert!(classify_settings(Path::new("."), &settings).is_err());
    }

    #[test]
    fn database_url_with_host_override_is_blocked() {
        let settings = BTreeMap::from([(
            "DATABASE_URL".to_string(),
            "postgres://localhost/test?host=remote.example".to_string(),
        )]);
        assert!(classify_settings(Path::new("."), &settings).is_err());
    }

    #[test]
    fn production_names_without_delimiters_are_blocked() {
        let settings = BTreeMap::from([
            ("DB_HOST".to_string(), "localhost".to_string()),
            ("DB_DATABASE".to_string(), "productiontest".to_string()),
        ]);
        assert!(classify_settings(Path::new("."), &settings).is_err());
    }

    #[test]
    fn sqlite_production_path_is_blocked_after_resolution() {
        let root = std::env::temp_dir().join(format!("snipe-sqlite-scope-{}", std::process::id()));
        let production = root.join("production");
        fs::create_dir_all(&production).unwrap();
        fs::write(production.join("test.db"), b"").unwrap();
        let settings = BTreeMap::from([
            ("DB_CONNECTION".to_string(), "sqlite".to_string()),
            ("DB_DATABASE".to_string(), "test.db".to_string()),
        ]);
        assert!(classify_settings(&production, &settings).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sqlite_without_explicit_database_is_blocked() {
        let settings = BTreeMap::from([("DB_CONNECTION".to_string(), "sqlite".to_string())]);
        assert!(classify_settings(Path::new("."), &settings).is_err());
    }

    #[test]
    fn conflicting_database_settings_are_blocked() {
        let settings = BTreeMap::from([
            ("DB_CONNECTION".to_string(), "sqlite".to_string()),
            ("DB_DATABASE".to_string(), ":memory:".to_string()),
            ("DB_HOST".to_string(), "remote.example".to_string()),
        ]);
        assert!(classify_settings(Path::new("."), &settings).is_err());
    }

    #[test]
    fn sqlite_urls_with_remote_or_production_targets_are_blocked() {
        for url in ["sqlite://remote.example/app", "sqlite:production.db"] {
            let settings = BTreeMap::from([("DATABASE_URL".to_string(), url.to_string())]);
            assert!(
                classify_settings(Path::new("."), &settings).is_err(),
                "{url}"
            );
        }
    }
}

#[cfg(test)]
mod tests;
