# Design

## Context

See `proposal.md` – Why for motivation. Constraints and existing pieces:

- `src/ai_workspace.rs` is 72 lines: `generate_ai_workspace(project_path)` creates
  `.ai-workspace/`, writes `rules.md` from `windsurf_rules()` and `rules.mdc` from
  `cursor_rules()`, and creates `.ai-workspace/audit/.gitkeep`. The two rule
  functions are hardcoded string literals, and the files land in a directory no
  agent tool reads by default.
- `fix-ci-and-report-truth` adds `.cursorrules` and `AGENTS.md` at the generated
  project root, and states the constraint that generation never writes outside the
  destination tree.
- `refactor-templates-and-test-scaffold` introduces `templates/` with compile-time
  embedding and a local override directory, so template content is a file edit.
- `serde_json` is already a dependency (`Cargo.toml`); `dialoguer` is already a
  dependency and provides multi-select prompts, used by the existing wizard.
- `src/generator.rs` exposes `GenerateConfig { project_name, template_id, frontend,
  backend, database }` and a dry-run preview that lists the files to be written.
- `.hermes/skills/` and `.opencode/skills/` already exist in this repository as
  project-local artifacts created by a tool's init command, so project-local
  agent directories are an established convention rather than a novel risk.

Tool conventions, verified against each tool's documentation, are the registry's
content:

| Tool | Artifacts | Slash command |
|---|---|---|
| cross-tool | `AGENTS.md` | no |
| Claude Code | `CLAUDE.md`, `.claude/skills/<name>/SKILL.md`, `.claude/commands/<name>.md` | yes |
| Cursor | `.cursor/rules/<name>.mdc`, `.cursorrules` (legacy) | no |
| OpenCode | `.opencode/skills/<name>/SKILL.md`, `.opencode/commands/<name>.md` | yes |
| Antigravity | `GEMINI.md`, `.agents/rules/*.md`, `.agents/skills/*.md`, `.agents/workflows/*.md` | yes |
| GitHub Copilot | `.github/copilot-instructions.md` | no |
| Windsurf | `.windsurf/rules/*.md` | no |
| Hermes Agent | `.hermes/skills/<name>/SKILL.md` | no |
| Codex | `AGENTS.md` | no |
| Gemini CLI | `GEMINI.md` | no |

## Goals / Non-Goals

**Goals:**

- A developer picks the agent tools they use, and the generated project works with
  them without moving files.
- Supporting another tool is a registry entry, not a code change.
- An agent or script can drive generation and read the result without parsing prose.
- Nothing is written outside the project.

**Non-Goals:**

- An MCP server or any runtime agent protocol.
- Tailoring artifact content per framework beyond what the stack templates already
  provide.
- Structured output for commands other than generation.
- Persisting the tool selection between runs in a configuration file.

## Decisions

### Registry as JSON under `templates/`

`templates/ai-tools.json` holds one entry per tool: its id, display name, and an
artifact list where each artifact names a path, a format, and the template that
provides its content.

*Alternatives:* YAML — needs a parser dependency for no benefit, since
`serde_json` is already present; Rust constants — that is today's hardcoded
problem, where adding a tool means editing and recompiling the tool.

### Formats, not tools, drive rendering

Renderers are keyed by format (`markdown`, `mdc`, `skill` with YAML frontmatter,
`command`), not by tool. Tools that share a format share a renderer, so the
maintenance surface is the number of formats rather than the number of tools.

### Tools with a slash command get two artifacts

Claude Code, OpenCode, and Antigravity expose a `/` command concept in addition to
auto-loaded rules. For those, the registry lists both an auto-loaded artifact and a
command artifact. Tools without that concept get only the auto-loaded artifact, so
the generator does not invent files a tool ignores.

### Keep `.ai-workspace/` as the shared content

The workspace directory keeps the rules content for tools that read no dedicated
convention, rendered from the same source as the per-tool artifacts rather than
from a second copy of the text. The two hardcoded rule functions are removed, so
there is one place where the guidance lives.

### Deduplicate artifacts by resolved path

`AGENTS.md` is claimed by both the cross-tool entry and Codex, and `GEMINI.md` by
both Antigravity and Gemini CLI. Artifacts are collected by resolved destination
path and written once, so selecting overlapping tools cannot write the same file
twice with different content.

### Structured output on stdout, progress on stderr

Generation writes human-facing progress to stderr and, when structured output is
requested, the result to stdout. That is what keeps the two channels from
interleaving — a single stream cannot carry both safely.

### Selection has two producers and one type

The interactive multi-select and the non-interactive option both resolve to the
same selection value, and generation only ever sees that value. This avoids two
code paths that can disagree about what "selected" means.

## Risks / Trade-offs

- **[Tool conventions drift — Cursor already marks `.cursorrules` legacy, and
  agent tools reorganise their directories between releases]** → The conventions
  are data. A drift is a registry edit plus a test expectation, not a code change.
  Each entry cites the path its tool documents today, and the registry is the one
  place to update.
- **[Artifacts claimed by several tools could be written twice with different
  content]** → Deduplicate by resolved destination path; the second writer for a
  path is skipped and the overlap is reported rather than silently resolved by
  ordering.
- **[A registry entry could name an unimplemented format and silently produce
  nothing]** → Generation reports an unknown format as an error, per the capability
  spec, rather than skipping the artifact.
- **[Writing several agent directories into a project may surprise a user who
  expected none]** → The dry-run preview lists every artifact, `--ai-tools none`
  produces the previous behaviour exactly, and nothing is written unless selected.
- **[`.hermes/skills/` is inert until the Hermes configuration lists the directory
  under `skills.external_dirs`]** → Document this in the generated artifact's
  accompanying text and in the README instead of implying the skill is already
  active. The generator cannot configure another tool's global settings, and the
  project-local constraint forbids writing outside the project.
- **[Ten tools × up to four artifacts each is a large per-tool surface to keep
  correct]** → Formats are shared, so only path and format are per tool; tests
  assert each registry entry's artifacts land at the documented path, which is what
  breaks loudly when a convention moves.

## Migration Plan

No data migration. The change adds a registry, replaces two hardcoded rule
functions with rendered output, adds a selection step, and adds one output option.

Ordering and cross-change interaction:

1. `fix-ci-and-report-truth` must land and archive first — it creates
   `openspec/specs/ai-workspace-generation/spec.md`, which this change appends to,
   and it introduces the root instruction files this change generalises.
2. `refactor-templates-and-test-scaffold` must land first — the registry lives
   under `templates/`, which that change creates.
3. Within this change: registry and renderer → selection → per-tool artifacts →
   structured output → documentation.

Rollback: revert the commit. `--ai-tools none` already reproduces the behaviour of
a project generated before this change, so a user who dislikes the result has an
immediate workaround independent of the revert.

Local pre-push gate: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, plus a
manual generation with several tools selected, confirming each artifact's path and
that the dry-run preview lists them all.

## Open Questions

None. The registry's contents are settled by the per-tool table above; any entry a
reviewer disputes is a data edit with a test expectation, not a design change.
