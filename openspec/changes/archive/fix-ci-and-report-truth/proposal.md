# Proposal

## Why

Every CI run on this repository has failed — 16 of 16 workflow runs concluded
`failure` — while 12 of the 13 jobs inside the most recent run passed. The only
failing job is `Backend (laravel)`: the generated `composer.json` pins
`laravel/framework ^11.0`, and Composer's security-advisory policy now refuses
to install any Laravel 11 release, so `composer install` exits 2 before the job
can reach `php artisan test`. A permanently red badge, next to a "Supported
stack verification" table that has never been green, is the wrong starting point
for a public tool.

Three smaller defects compound the credibility problem: audit reports are
stamped with a hardcoded date instead of the real one, the README advertises AI
agent files the generator never writes, and the CLI cannot report its own
version.

## What Changes

- **Make the generated Laravel project installable.** Move the
  `laravel/framework` constraint from `^11.0` to `^12.0` so Composer resolves an
  advisory-free release instead of the blocked v11.57.0. Acceptance is the
  `Backend (laravel)` CI job passing end to end, which also runs
  `php artisan test`; if the generated skeleton does not boot under Laravel 12,
  migrating `bootstrap/app.php` to the current `Application::configure()` pattern
  is in scope.
- **Audit reports carry the real generation date.** Replace the hardcoded
  `chrono_placeholder()` value with a real system-date helper. The helper is
  shared with the blog template generator rather than duplicated.
- **AI workspace files match what the product advertises.** Write `.cursorrules`
  and `AGENTS.md` into the generated project root, in addition to the existing
  `.ai-workspace/` output, so README and PRD `US-009` become true instead of
  aspirational.
- **The CLI reports its own version.** Add a `version` command and a `--version`
  flag sourced from the crate version.
- **README corrected to verified reality.** Install/usage text and the stack
  verification table reflect only what CI actually proves.

## Capabilities

### New Capabilities

- `cli-interface`: command routing, flags, and what the CLI reports about itself
  — version, help, usage errors, and exit codes.
- `project-scaffolding`: the content a generated project receives for each stack
  and template, including dependency constraints that must resolve in a clean
  environment.
- `security-audit`: the heuristic scan and the reports it exports, including
  report metadata and the disclosure of what was and was not checked.
- `ai-workspace-generation`: the AI-agent instruction artifacts a generated
  project receives.

### Modified Capabilities

None. The project has no existing specs; this change introduces the four
capabilities above.

## Impact

- `src/stack.rs` — Laravel `composer.json` template, `laravel/framework`
  constraint (currently line 607), plus further skeleton files if the Laravel 12
  migration turns out to be required.
- `src/audit.rs` — `chrono_placeholder()` (line 831) and its three call sites
  (lines 508, 575, 721).
- `src/generator.rs` — the real-date helper (`real_date_yyyymmdd`, line 604)
  moves to a shared location so the audit and blog callers use one copy.
- `src/ai_workspace.rs` — additional project-root writes for `.cursorrules` and
  `AGENTS.md`.
- `src/main.rs` — a new `version` / `--version` arm.
- `README.md` — AI workspace claim and stack verification table.
- `.github/workflows/ci.yml` — no workflow change expected; the
  `Backend (laravel)` job going green is the acceptance signal.
- No new crates and no network calls added; the tool stays fully offline.

Out of scope: the architecture refactor and test-infrastructure generation
(tracked as separate changes), and any change to the audit rule set itself.
