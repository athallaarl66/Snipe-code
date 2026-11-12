## 1. English intent launcher

- [ ] 1.1 Extract command routing so direct commands and launcher choices call the same `new`, `analyze`, `verify`, `audit`, help, and exit boundaries.
- [ ] 1.2 Add the English interactive no-argument menu in this order: Create a new project, Understand an existing project, Run tests and quality checks, Audit project security, View help, Exit.
- [ ] 1.3 Make a non-interactive no-argument invocation print concise usage, exit `2`, and prove it creates no `my-project` or other filesystem output.
- [ ] 1.4 Add launcher integration tests for every route, cancellation, safe exit, direct-command bypass, and no-action-before-selection behavior in the CLI test suite.

## 2. Project-generation wizard

- [ ] 2.1 Replace mixed-language wizard copy and `Enterprise Dev-Forge` with English product, prompt, navigation, status, warning, error, and completion text.
- [ ] 2.2 Implement explicit wizard state for forward, back, cancel, and summary transitions while preserving non-interactive flags and JSON output.
- [ ] 2.3 Rewrite preset metadata so No preset, Company Profile, Portfolio, and Blog describe only their generated files and distinguish presets from framework stacks.
- [ ] 2.4 Add step indicators, keyboard guidance, exact destination, and a final summary of preset, frontend, backend, database, AI integrations, and Git choices.
- [ ] 2.5 Add Preview, Create project, Back, and Cancel summary actions; reuse dry-run rendering for Preview and return to the summary without writes.
- [ ] 2.6 Require separate authorization for Git initialization and initial commit, retaining the generated project when either Git action is declined or unavailable.
- [ ] 2.7 Preserve overwrite safety: show the canonical destination, default to cancel, reject symlink or junction destinations, and leave existing content unchanged on back, cancel, input failure, or generation failure.
- [ ] 2.8 Extend generation, collision, dry-run, structured-output, and CLI tests for English copy, navigation, summary, preview, Git decisions, and unchanged non-interactive flags.

## 3. Verification reports

- [ ] 3.1 Add verification options for `json`, `junit`, and `md` report formats plus an explicit output path, validating both before any gate runs.
- [ ] 3.2 Serialize the existing verification report to JSON with root, project parts, gates, commands, exact states, details, counts, and overall result.
- [ ] 3.3 Render JUnit XML without a new dependency, mapping not-applicable and unavailable gates to skipped cases with distinct messages and preserving exact states as properties.
- [ ] 3.4 Render a Markdown test and quality-gate report with target, parts, gate commands, outcomes, failure details, and summary.
- [ ] 3.5 Refuse to replace an existing report unless the user passes an explicit overwrite option; report invalid format or missing path as exit `2` before gates, and write failures as exit `3` after execution.
- [ ] 3.6 Preserve gate-derived exit `0`, `1`, `3`, or `4` after a successful report write; usage errors remain `2`, keep errors on stderr, and print only the requested structured result or report path on stdout.
- [ ] 3.7 Add renderer, parser, CLI, malformed-output, report-collision, write-failure, and exit-status tests; retain fake runners for unit tests that must not spawn external toolchains.

## 4. AI-agent workflows

- [ ] 4.1 Extend the AI tool registry model and structured generation result to report each selected agent's rule, skill, command, and workflow artifact types explicitly.
- [ ] 4.2 Add English wizard guidance stating that AI selection writes project-local artifacts only and does not run a model, authenticate a provider, store credentials, or require an API key.
- [ ] 4.3 Replace the generic audit-only command template with separate audit, verification, and test-authoring templates for Claude Code, OpenCode, and Antigravity using each tool's documented invocation path.
- [ ] 4.4 Keep Hermes Agent, Codex, Cursor, Gemini CLI, GitHub Copilot, Windsurf, and cross-tool integrations limited to artifact types the registry actually generates; do not advertise slash commands where none are generated.
- [ ] 4.5 Make audit workflows run deterministic audit, review false positives, fix only confirmed authorized findings, run tests, and repeat audit.
- [ ] 4.6 Make verification workflows run deterministic verification, diagnose failures from code and output, apply only requested fixes, and verify again.
- [ ] 4.7 Make test-authoring workflows read source and requirements, ask when expected behavior is ambiguous, author or repair tests, and execute them through `snipe-code verify --only test`.
- [ ] 4.8 Update AI integration tests to assert exact per-agent files, three task-specific workflows where supported, no stale generic `/snipe-code` artifact, no unsupported slash-command claims, path deduplication, and no writes outside the project.

## 5. Documentation and command help

- [ ] 5.1 Update README to show the English launcher as the first-time flow and direct `new`, `analyze`, `audit`, and `verify` commands as launcher-free paths for experienced users and CI.
- [ ] 5.2 Document generation steps, destination and configuration review, preview, cancellation, Git authorization, and honest preset output.
- [ ] 5.3 Document executable versus AI-agent responsibilities: generated test scaffold, deterministic test execution and reporting, agent-assisted test authoring, and no embedded AI runtime.
- [ ] 5.4 Add an agent compatibility table that lists rules, skills, commands, and workflows separately for every registry entry.
- [ ] 5.5 Document verification JSON, JUnit XML, and Markdown separately from audit Markdown, PDF, DOCX, and JSON; include exit-code and CI examples.
- [ ] 5.6 Rewrite general help, per-command help, FAQ, and affected feature documentation in English, keeping commands, paths, report names, and current limitations exact.
- [ ] 5.7 Update README contract tests in `verify_cli_test.rs`, `release_contract_test.rs`, `bugfix_test.rs`, `ai_integration_test.rs`, and `test_infra_contracts_test.rs` without dropping unrelated pinned literals.
- [ ] 5.8 Run the documentation prose checker and remove new em dashes, decorative bold, title-case headings, mixed-language UI examples, and unsupported claims.

## 7. CR workflow consistency

- [ ] 7.1 Expose existing scan/export/history routes and explain legacy audit writes before routing to it; show future quality/QA operations only as unavailable until delivered. Keep no-action-before-selection tests.
- [ ] 7.2 Require separate, scope-bound preview and user approval in each generated workflow before source edits, installation, tests, endpoint access, data sharing, or report writes. Instructions are not a runtime permission boundary; preserve core preflight rejection.
- [ ] 7.3 Preserve blocked, unavailable, and not-applicable separately in JSON/Markdown/JUnit, including exact-state properties for skipped cases; test failure, no-execution, and incomplete results retain exits 1, 3, and 4 after successful export.
- [ ] 7.4 Document the four distinct paths: generated example tests, verify executing declared tests, deterministic report export, and reviewed agent-assisted test authoring. Do not claim planned QA/model/connector runtime already exists.

See `../../CR-ALIGNMENT.md`.

## 6. Verification and release readiness

- [ ] 6.1 Run `cargo fmt --check`.
- [ ] 6.2 Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 6.3 Run `cargo test --all-targets --all-features`.
- [ ] 6.4 Run available generated-project verification without touching user projects; report unavailable toolchains and Docker/CI runs as NOT RUN rather than pass. Docker requires separate approval.
- [ ] 6.5 Add automated coverage for launcher routes, navigation, report exports, and generated workflow artifacts; report manual Windows and agent exercises separately when available.
- [ ] 6.6 Run `git diff --check`, confirm only intended tracked files changed, and update OpenSpec task evidence without committing, pushing, tagging, or publishing.
