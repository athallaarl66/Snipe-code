## Why

Opening `snipe-code.exe` currently starts project generation at the template prompt, before the user chooses a goal or sees what the product can do. The CLI also mixes Indonesian and English, presents executable commands and AI-agent artifacts as if they were equivalent interfaces, and does not save verification results as reusable reports.

## What Changes

- Replace the no-argument generation shortcut with an English interactive launcher organized by user intent:
  - Create a new project
  - Understand an existing project
  - Run tests and quality checks
  - Audit project security
  - View help
  - Exit
- Keep `new`, `analyze`, `verify`, and `audit` as direct commands for experienced users, scripts, and CI.
- Make the entire interactive UI English, including prompts, descriptions, navigation hints, status messages, errors, summaries, and cancellation text.
- Rework the project-generation wizard into labeled steps with keyboard guidance, honest preset descriptions, visible destination, a configuration summary, preview, confirmation, back, and cancel paths before any write.
- Remove ambiguous product text such as `Enterprise Dev-Forge` and replace it with a concrete description of the CLI.
- Explain AI-agent integration before selection: it writes project-local instructions, skills, rules, and supported shortcuts; it does not run a model, authenticate a provider, or require an API key.
- Report each selected agent's actual integration type instead of implying that every agent receives the same skill or slash command.
- Replace the generic audit-only `/snipe-code` workflow with explicit agent workflows for audit, verification, and test authoring where the target agent supports invocable workflows. Keep rules or skills for agents that do not support slash commands.
- Define the AI test-authoring workflow as agent-assisted source and requirement analysis followed by execution through the deterministic `snipe-code verify` engine. The executable does not claim to infer complete test cases by itself.
- Add verification report export for automation and human review, with JSON, JUnit XML, and Markdown output plus an explicit destination. Verification remains deterministic and does not use AI.
- Keep security audit exports distinct from verification reports in CLI text, generated agent guidance, help, and README.
- Make a no-argument non-interactive invocation print usage and exit without creating `my-project` or changing the filesystem.
- Update README, command help, FAQ, and automated documentation contracts to describe the launcher, direct commands, report formats, AI boundaries, agent compatibility, side effects, and validation flow accurately.

## Capabilities

### New Capabilities

- `interactive-command-launcher`: English intent-based entry flow, safe non-interactive behavior, navigation, and routing to existing commands.
- `verification-report-export`: Machine-readable and human-readable verification reports with explicit formats and output paths.
- `ai-agent-workflows`: Explicit audit, verification, and test-authoring agent workflows with per-agent compatibility and clear separation from executable behavior.

### Modified Capabilities

- `safe-project-generation`: Generation gains step context, destination disclosure, summary, preview, confirmation, back, and cancel behavior before writes or Git side effects.
- `cli-process-semantics`: No-argument behavior, cancellation, launcher routing, verification report failures, and output-channel contracts gain stable exit semantics.
- `product-documentation-accuracy`: Documentation must distinguish direct CLI commands, deterministic reports, generated AI artifacts, invocable workflows, and unsupported agent behavior.

## Impact

- CLI dispatch and help in `src/main.rs`.
- Interactive generation flow in `src/commands/new.rs` and preset display metadata under `templates/presets/`.
- Verification result rendering and argument parsing in `src/verify/`.
- AI tool registry and artifact templates in `templates/ai-tools.json` and `templates/ai-tools/content/`.
- README, feature documentation, generated guidance, and tests that pin command text or artifact paths.
- Existing direct command names remain available. The no-argument behavior and generic `/snipe-code` agent workflow change, so release notes must call out those user-visible changes.
- No embedded model, provider SDK, API key, or new AI runtime dependency is introduced.
