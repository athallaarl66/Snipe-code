# Proposal

## Why

`src/ai_workspace.rs` writes two files — `.ai-workspace/rules.md` and
`.ai-workspace/rules.mdc` — into a directory that no AI coding agent reads by
default. Agent tools look in their own places: `AGENTS.md` at the project root
(read by Antigravity, Cursor, Claude Code, Codex, Copilot, Windsurf, and
OpenCode), `CLAUDE.md` plus `.claude/skills/<name>/SKILL.md`, `.cursor/rules/*.mdc`,
`.opencode/skills/` and `.opencode/commands/`, `GEMINI.md` plus `.agents/rules/`
and `.agents/workflows/`, `.github/copilot-instructions.md`, and
`.windsurf/rules/`. A generated project therefore ships an "AI workspace" that
nothing consumes unless the developer moves the files by hand.

`fix-ci-and-report-truth` makes the README's existing promise true by writing the
two files it already advertises, `.cursorrules` and `AGENTS.md`. This change turns
that one-off into a real feature: coverage for the agent tools a developer actually
uses, chosen at generation time, driven by data instead of code.

## What Changes

- **The wizard gains an AI tool selection step**, multi-select, and the selection
  can be given non-interactively so scripts and CI never prompt. Selecting none
  stays valid and writes no agent artifact.
- **Per-tool artifacts are generated at each tool's documented locations.** Tools
  with a dedicated directory get their files inside it; cross-tool tools get the
  root instruction file. Tools with a `/` command concept also get a command
  artifact, not just an auto-loaded rule.
- **Tool coverage becomes a data-driven registry.** Each entry names the tool, the
  artifacts to write, and each artifact's format — so adding a tool is adding a
  registry entry, not editing generation logic. The registry is JSON, because
  `serde_json` is already a dependency and a YAML registry would add a parser for
  no benefit.
- **Generation can report its result as structured data**, so an agent or script
  can drive the tool and read back what was written without parsing prose.
- **All AI artifacts stay project-local.** Nothing is written under a
  home-directory agent configuration.

## Capabilities

### New Capabilities

- `ai-tool-integration`: how AI agent tool coverage is selected, how each tool's
  instruction artifacts are described in data, where they are written, and how a
  machine reads back the result.

### Modified Capabilities

- `ai-workspace-generation`: a generated project can carry artifacts for more than
  one agent tool at once, each at that tool's own location, instead of only the
  single workspace directory. The delta adds requirements and leaves the
  requirements introduced by `fix-ci-and-report-truth` intact.
  **Apply and archive this change after `fix-ci-and-report-truth`**, because both
  write `openspec/specs/ai-workspace-generation/spec.md`.

## Impact

- `src/ai_workspace.rs` — becomes the renderer: reads the registry entry for each
  selected tool and writes that tool's artifacts, replacing the two hardcoded
  rule functions.
- `templates/ai-tools.json` — new registry; depends on `templates/` existing, which
  `refactor-templates-and-test-scaffold` introduces.
- `src/commands/new.rs` — the selection step, the non-interactive option, and
  passing the selection through to generation.
- `src/generator.rs` — carries the selected tools through `GenerateConfig` and
  includes the written artifacts in the generation result and dry-run preview.
- `src/main.rs` — document the new option in the help text.
- `.ai-workspace/` — the shared rules content for tools with no dedicated
  convention is preserved; the two hardcoded copies are replaced by rendered
  output from the registry.
- Tests — registry parsing, per-tool artifact paths and formats, the no-selection
  case, and the project-local guarantee.
- `README.md` — the supported tool list and the selection option.

Out of scope: MCP server support, per-tool artifact content tailored to a
specific framework beyond what the stack already generates, and machine-readable
output for commands other than generation.

## Dependencies

- `refactor-templates-and-test-scaffold` must land first — the registry lives under
  `templates/`, which that change creates.
- `fix-ci-and-report-truth` must land and archive first, for the shared
  `ai-workspace-generation` capability file and because it introduces the root
  instruction files this change generalises.
