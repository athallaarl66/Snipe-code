# Tasks

## 1. Registry and renderer

- [x] 1.1 Add `templates/ai-tools.json` with one entry per supported tool (id, display name, artifact list of path + format + content template), covering the cross-tool file, Claude Code, Cursor, OpenCode, Antigravity, GitHub Copilot, Windsurf, Hermes Agent, Codex, and Gemini CLI; verify with a test that parses the registry and asserts every entry resolves
- [x] 1.2 Implement the format renderers — markdown, `mdc`, skill with YAML frontmatter, and command — keyed by format rather than by tool; verify with a unit test per format asserting the rendered output has the expected shape (frontmatter for skill and `mdc`)
- [x] 1.3 Make an unknown format in a registry entry an error rather than a silent skip; verify with a test using a fixture registry naming a format the renderer does not implement
- [x] 1.4 Collect artifacts by resolved destination path and write each path once, reporting overlaps instead of resolving them by ordering; verify with a test selecting two tools that claim `AGENTS.md` and asserting one file is written and the overlap is reported

## 2. Selection

- [x] 2.1 Add the AI tool multi-select step to the wizard in `src/commands/new.rs`, defaulting to none, and carry the selection into `GenerateConfig`; verify with a test that generation with no selection writes no agent artifact
- [x] 2.2 Add the non-interactive selection option so scripts and CI never prompt, accepting a comma-separated list and the explicit value for none; verify with a test asserting a non-interactive run writes exactly the selected tools' artifacts and no prompt is issued
- [x] 2.3 Confirm both producers resolve to the same selection value and that generation only consumes that value; verify with a test covering the same selection expressed both ways producing identical output

## 3. Per-tool artifacts

- [x] 3.1 Replace the two hardcoded rule functions in `src/ai_workspace.rs` with rendering from the registry, keeping the workspace directory's shared rules content for tools with no dedicated convention; verify with a test asserting the workspace directory still contains its rules files
- [x] 3.2 Generate the auto-loaded artifact for each selected tool at its documented path; verify with a test per tool asserting the path exists and that no other tool's artifact was written
- [x] 3.3 Generate the command artifact for the tools that expose a `/` concept (Claude Code, OpenCode, Antigravity), and do not generate one for tools without that concept; verify with tests for both cases
- [x] 3.4 Include the AI artifacts in the dry-run preview and the generation file list; verify with a test asserting a dry run with tools selected lists every artifact and creates no file
- [x] 3.5 Add a regression test asserting no file is created or modified under a home-directory agent configuration path when any tool is selected; verify with `cargo test`

## 4. Structured output

- [x] 4.1 Emit the generation result — selected tools, artifacts written, and outcome — as structured data when requested, with human-facing progress on stderr and the result on stdout; verify with a test asserting stdout parses as structured data and contains no progress text
- [x] 4.2 Document the new options in the help text in `src/main.rs` (the AI tool selection and the structured-output request); verify with a test asserting the help output documents both

## 5. Documentation

- [x] 5.1 Document the supported tool list, the artifact each tool receives, the selection options, and the note that project-local skill directories require that tool's own configuration to pick them up; verify by following the documented steps and confirming a generated project works with at least one selected tool
- [x] 5.2 Add a test asserting the README lists every tool present in the registry, so the two cannot drift apart; verify with `cargo test`

## 6. Integration verification

- [x] 6.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [x] 6.2 Generate one project with several tools selected, including two that claim the same path, and confirm the artifact set matches the registry and the preview
- [ ] 6.3 Confirm CI is green, and record the run URL

## Workflow follow-up

- Confirm `fix-ci-and-report-truth` has been archived before archiving this change — both write `openspec/specs/ai-workspace-generation/spec.md`.
- Archive, then confirm `openspec/specs/ai-tool-integration/` exists and the new requirements are appended under `openspec/specs/ai-workspace-generation/`.
