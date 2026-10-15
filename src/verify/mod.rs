//! Verification engine: one code path for the `verify` command and for the CI
//! harness that checks generated projects.

pub mod detect;
pub mod exec;

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
    /// The project declares nothing for this gate.
    NotApplicable(String),
    /// The tool is not installed on this machine.
    Unavailable(String),
}

impl GateState {
    pub fn name(&self) -> &'static str {
        match self {
            GateState::Passed => "passed",
            GateState::Failed(_) => "failed",
            GateState::NotApplicable(_) => "not-applicable",
            GateState::Unavailable(_) => "unavailable",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            GateState::Passed => None,
            GateState::Failed(message)
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

/// `0` every applicable gate passed, `1` a gate failed, `3` nothing executed.
/// Usage errors return `2` before a report exists.
pub fn exit_status(report: &Report) -> i32 {
    if report.parts.is_empty() {
        return 3;
    }
    if report.first_failure().is_some() {
        return 1;
    }
    if report.executed() == 0 {
        return 3;
    }
    0
}

/// How the `verify` command was invoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub path: PathBuf,
    pub only: Option<Gate>,
    pub security: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            only: None,
            security: false,
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
pub fn run(root: &Path, options: &Options, runner: Runner<'_>) -> Result<Report, String> {
    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }
    let plans = detect::plan(
        root,
        options.security || options.only == Some(Gate::Security),
    );
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
    let mut report = execute(&plans, options.only, runner);
    report.root = root.to_path_buf();
    Ok(report)
}

/// Runs each plan's gates in the fixed order, recording one outcome per gate.
pub fn execute(plans: &[PartPlan], only: Option<Gate>, runner: Runner<'_>) -> Report {
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

/// The overall result line.
pub fn summary(report: &Report) -> String {
    let failed = report.count("failed");
    let passed = report.count("passed");
    let not_applicable = report.count("not-applicable");
    let unavailable = report.count("unavailable");
    let result = match exit_status(report) {
        0 => "PASS",
        2 => "USAGE ERROR",
        3 => "NOTHING EXECUTED",
        _ => "FAIL",
    };
    format!(
        "result: {result} — {passed} passed, {failed} failed, {not_applicable} not applicable, {unavailable} unavailable"
    )
}
