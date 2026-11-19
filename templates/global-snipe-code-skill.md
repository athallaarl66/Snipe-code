---
name: snipe-code
description: Use when scaffolding, analyzing, auditing, verifying, or testing a project with SNIPE-CODE.
---

# SNIPE-CODE

This skill guides the host AI agent. `snipe-code` is a deterministic CLI, not an AI model or test author. Check `snipe-code help` and `snipe-code help <command>` for the installed version before relying on options. Never invent commands, flags, results, or coverage.

## Choose a task

Use the user's stated goal; otherwise ask which task and target path they want:

1. **Scaffold** a project with `new`.
2. **Analyze** a codebase with `analyze` and source tracing.
3. **Audit** security with `scan`, source review, and optional `audit` reports.
4. **Verify** existing declared gates with `verify`.
5. **Automate tests** by tracing requirements and source, authoring runnable tests when requested and approved, then running them with separate approval.

Resolve the target explicitly; `[PATH]` defaults to the current directory. Read project instructions and existing files before changes. Never expose secret values from environment files, command output, findings, or database URLs.

## Safety and approval

- `analyze` and default `scan` are read-only. `scan --save` writes local history. `--snapshot` selects an input, not a network refresh.
- `new --dry-run` previews generation. Actual `new` writes under the current directory. Check destination and collisions; obtain approval before generation or replacement. Git initialization and initial commit require separate choices.
- `audit` writes `Audit_Report` files in the target and may replace existing reports, including with `--json`. Obtain approval for formats and destination first.
- `verify` executes project-controlled commands that can install dependencies, change files, access networks or databases, or run arbitrary scripts. It is not a sandbox; `--no-install` does not make other gates safe. Inspect manifests and scripts, identify working directories, commands, database targets and possible effects, then obtain explicit approval for that scope. The CLI also requires interactive gate approval; do not bypass a block or infer approval from a preview. Unknown database targets are not proof of isolation. Stop on refusal or ambiguity. Never run against production or shared data without explicit authorization and an appropriate isolation plan.
- Ask separately before editing source or tests, overwriting reports, installing tools, accessing a network, or running commands with side effects. Preserve unrelated files. If approval, tools, or safe prerequisites are missing, report the blocker rather than claiming completion.

## Scaffold

Collect preset, frontend, backend, database, name, destination, and optional agent integration. Inspect `snipe-code help new` for IDs. Preview first:

```sh
snipe-code new --dry-run --template blog --frontend nextjs --backend none --database none --name site --ai-tools none
```

`--template`: `none`, `company-profile`, `portfolio`, `blog`. `--frontend`: `nextjs`, `react-vite`, `vue`, `nuxtjs`, `none`. `--backend`: `dotnet8`, `nestjs`, `go-gin`, `spring-boot`, `laravel`, `none`. `--database`: `postgresql`, `mysql`, `none`. `--ai-tools` selects comma-separated IDs or `none`; `--json` outputs the generation result. Presets add files to a stack; AI artifacts are project-local instructions, not model execution. After approval, create and inspect actual output; distinguish preview from creation and report Git choices separately.

## Analyze and trace

Run `snipe-code analyze <path> --json` or omit `--json` for human output. It reports structure, languages, line counts, entry points, dependencies and limits; hidden, dependency and build directories are excluded. Counts include blank lines and comments. A detected entry point or dependency does not prove runtime behavior.

For the requested behavior, read project instructions and manifests, locate entry points and relevant symbols, then follow callers, inputs, validation, data flow, side effects, errors and tests to their implementation. Inspect actual source at each step, including configuration and boundary conditions. Cite current `path:line` evidence and explain excluded, unavailable or untraced scope. Distinguish observed behavior from inference and assumptions.

## Audit

Start with `snipe-code scan <path> --json` for an offline scan without report export. Review each relevant finding against referenced source and execution context; identify input, sink, preconditions, impact and uncertainty. A heuristic finding is not proof of exploitability; no findings is not proof of security. Never reproduce secrets in citations.

If the user needs reports and approves writes, use `snipe-code audit <path> --export md,pdf,docx` with only requested comma-separated formats. Without `--export`, interactive mode asks for formats and non-interactive mode defaults to Markdown. `--json` prints structured output but still writes reports. Check actual exit status and resulting paths; distinguish this legacy report from verify reports.

## Verify

Inspect manifests and declared commands first. With approval, run a scoped command such as `snipe-code verify <path> --only test --no-install`. Gates: `install`, `lint`, `typecheck`, `test`, `build`, `security`. `--security` includes dependency checks; `--no-install --only install` is invalid. `--json` prints a structured result. To write a verification report, use `--report json|junit|md --output <file>`; `--overwrite` permits replacement and needs approval. Do not combine `--json` with a report option.

Report each gate's actual state and command, working directory, exit status, and limitations. Exit 0 means all applicable gates passed; 1 means a gate failed; 2 is usage error; 3 means no gate ran or report write failed; 4 means incomplete due to blocked or unavailable gates. Not-applicable is not a pass. `verify` executes existing project commands; it does not author test cases.

## Test automation

1. Agree on target behavior and acceptance criteria. Trace requirements and contracts to entry points, implementation, validation, side effects and existing tests; cite source. Identify positive, negative, boundary and error cases and select unit, integration or E2E based on actual repository evidence and requested scope. Do not assume an endpoint, expected status, framework or fixture exists.
2. Inspect manifests, test configuration, existing runnable tests, scripts, fixtures and available tooling. Reuse the project's framework and conventions. Identify database targets, destructive operations, external services, network access and credentials without disclosing secrets. A local or test-named DB is not proof of isolation.
3. When the user requests and approves test edits, write minimal runnable tests with meaningful assertions tied to the contract. Preserve unrelated files. If a framework or prerequisite is missing, report it; obtain separate approval before installing or changing setup. Scaffold examples are not proof that behavior is tested.
4. Before execution, preview the exact command, working directory, affected files, DB and network risks, and cleanup or isolation plan. Obtain separate execution approval. Stop if effects or DB target remain unsafe or unknown. Use the narrowest relevant project test command or approved `snipe-code verify <path> --only test`; do not bypass CLI approval. `verify` runs existing declared tests, including newly authored tests only if their project command includes them; it is not the AI test-writing workflow.
5. Capture actual command, exit status, test counts and failures from output. Inspect failures and report skipped, blocked and unrun cases explicitly. Do not claim coverage or success from test code alone. Cite changed tests and observed evidence; disclose missing environments and residual risks.

## Response

State target and scope, commands actually run, source references, findings with evidence and severity rationale, files changed or reports written, gate states and exit codes, and remaining risks or blockers as applicable. Separate facts, inference and recommendations. Mark anything not run, unavailable, blocked or outside scope; never claim full coverage or a clean security posture from a partial scan.
