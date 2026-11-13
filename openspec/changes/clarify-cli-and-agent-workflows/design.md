## Context

The binary currently dispatches a no-argument invocation directly to `new`. Interactive generation then starts at preset selection, uses mixed Indonesian and English text, performs generation after the last selection without a configuration confirmation, and initializes Git on a best-effort basis. Direct commands already provide stable automation entry points.

The AI registry is data-driven and can render rules, skills, and commands, but the current `command` template is one generic audit-only `/snipe-code` workflow. Agent entries do not all expose the same artifact types. Verification already produces one in-memory `Report`, prints it, and maps it to a process status, but has no file renderer.

## Goals / Non-Goals

Goals:
- Give first-time executable users a goal-oriented English launcher.
- Keep direct command behavior available for scripts and experienced users.
- Add review points before project and Git writes.
- Make agent artifact capabilities explicit and add task-specific workflows.
- Render reusable verification reports from the existing verification result.

Non-goals:
- Replace the terminal UI with a desktop GUI or web application.
- Embed an LLM, provider SDK, API key, or model selection in the executable.
- Make every agent support slash commands when its native convention does not.
- Generate complete behavioral tests deterministically from arbitrary source code.
- Add PDF, DOCX, or HTML verification reports.
- Rename the existing direct commands.

## Decisions

### 1. No arguments select intent; explicit commands remain the automation API

Interactive no-argument dispatch opens a six-item launcher. Each action calls the same command boundary used by direct invocation. Non-interactive no-argument dispatch prints concise usage and returns `2` without entering `new`.

This keeps one implementation per operation and removes the unsafe default `my-project` path. Replacing direct commands with only a menu was rejected because scripts and CI require stable non-interactive entry points.

### 2. Interactive text is English-only

All launcher and wizard copy moves to English. Product IDs, command names, paths, and emitted machine data remain unchanged. `Enterprise Dev-Forge` is removed because it does not communicate behavior.

Adding localization was rejected for this release because it adds a translation system while the immediate issue is mixed language and unclear flow.

### 3. Generation becomes a small state machine

The wizard stores selections separately from execution and supports forward, back, preview, cancel, and confirm transitions. The final screen shows destination, preset, stacks, database, selected agent artifacts, and Git choices. Preview reuses `dry_run_preview` and returns to the summary.

Generation and Git authorization are separate decisions. Existing staging and overwrite safety remain authoritative. A single linear prompt chain was rejected because it cannot provide reliable back navigation or final review.

### 4. Preset display copy comes from truthful template metadata

Preset metadata remains data-driven, but descriptions are rewritten to match files generated today. The UI explains that presets add feature-oriented structure while stacks select frameworks. Emoji can remain decorative, but names and focus indicators must remain usable without them.

### 5. Verification renderers consume the existing report

JSON, JUnit XML, and Markdown renderers serialize `verify::Report`; they do not parse terminal text. New CLI options select one format and explicit output path. Validation occurs before running gates. Report write errors map to runtime status `3`, while a successful report write preserves the gate-derived status.

JUnit XML is included for CI ingestion. Markdown is included for human review. PDF, DOCX, and HTML were rejected because they add dependencies or complexity without improving the primary CI workflow.

### 6. Agent artifacts expose capabilities, not a false common denominator

The registry continues to own tool coverage. Artifact records and user-facing output distinguish rule, skill, command, and workflow. The generic command artifact is replaced by task-specific audit, verify, and test templates for agents with a native invocation convention.

Agents with rules or skills only keep those supported artifacts. Generated guidance can describe equivalent manual prompts but cannot claim a slash command exists.

### 7. Test authoring belongs to agents; execution and reporting belong to the executable

The test-authoring workflow receives user intent, reads code and requirements, resolves ambiguity, edits tests, and invokes verification. The executable continues to run declared tests without AI. This keeps test results reproducible while allowing coding agents to do reasoning-heavy test creation.

## CR alignment

See `../../CR-ALIGNMENT.md`. Use the M0 safety/status contract and shared QA policy before workflow execution. Existing scan/export/history remain direct commands; disclose legacy audit writes before invocation and do not expose future QA/gate commands as available. Successful report writes preserve exits 0/1/3/4 and exact blocked/unavailable states. Every generated workflow requests separate scoped approval for each risky effect; remote/production/ambiguous database preflight cannot be overridden, and an undetected database target follows the same explicit user-approval channel as the CLI.

## Risks / Trade-offs

- Launcher and back navigation increase interactive state complexity. Mitigation: model transitions as pure selection state and test routing separately from terminal rendering.
- Existing users may expect no arguments to start generation. Mitigation: retain `snipe-code new`, document the behavior change, and show `Create a new project` first.
- Agent command conventions differ and can change. Mitigation: keep artifact paths and types in the registry and test each generated path, without claiming unsupported discovery.
- JUnit mappings for unavailable and not-applicable gates can be ambiguous. Mitigation: map both to skipped test cases with distinct messages while preserving exact states in properties.
- Verification report output can overwrite a user file. Mitigation: require an explicit path and define collision behavior before implementation; default to refusing replacement unless explicitly authorized.
- README and help are asserted by tests. Mitigation: update named documentation-contract tests with the behavior change and retain unrelated required literals.

## Migration Plan

1. Add report renderers and tests without changing default verification output.
2. Expand agent registry data and templates, then update artifact contract tests.
3. Add the English generation state flow and final confirmation while retaining direct `new` flags.
4. Switch interactive no-argument dispatch to the launcher and non-interactive no-argument dispatch to safe usage output.
5. Update README, command help, feature docs, and release notes together.
6. Run formatting, Clippy, all Rust tests, generated-project verification, and documentation prose checks.

Rollback restores no-argument dispatch to direct generation and removes new templates and report flags. Existing direct commands and generated project formats remain compatible throughout.
