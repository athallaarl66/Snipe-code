# Design

## Context

See `proposal.md` – Why for motivation. The constraints that shape this design:

- The acceptance signal is CI, not a local build. `.github/workflows/ci.yml` runs
  `cargo run --bin verify-generated -- --backend laravel`, and
  `src/bin/verify-generated.rs` executes **two** commands for that stack:
  `composer install --no-interaction --no-progress`, then `php artisan test`.
  A constraint fix alone is therefore not sufficient — the generated app must
  also boot.
- CI installs PHP 8.2 (`shivammathur/setup-php`, `php-version: '8.2'`).
- Laravel's published skeletons require: `11.x` → php `^8.2`, `12.x` → php
  `^8.2`, `13.x` → php `^8.3`.
- The blocked line is Laravel 11 in full: the newest release, `v11.57.0`, is
  affected by four active advisories (`PKSA-d5tc-s1qs-h781`,
  `PKSA-m5cs-t1y6-qpcs`, `PKSA-3r5d-mb8f-1qw9`, `PKSA-mdq4-51ck-6kdq`). The
  lowest advisory-free version is `12.69.0`; the newest 12.x release is
  `12.70.0`.
- The generated Laravel skeleton is shaped like Laravel 10 —
  `bootstrap/app.php` constructs `Illuminate\Foundation\Application` directly
  and registers `Http\Kernel`, `Console\Kernel`, and `ExceptionHandler`
  singletons. The official 11+/12 skeleton instead uses
  `Application::configure()`. These legacy classes still exist in 12.x, so the
  skeleton's viability is unproven rather than known-broken. CI has never
  reached `php artisan test`, because `composer install` fails first.
- The tool is offline with no network dependencies; this change adds none.
- No specs exist in the project yet, so all four capabilities are new.

## Goals / Non-Goals

**Goals:**

- The `Backend (laravel)` CI job passes end to end (generate → install → test).
- Audit reports carry the real generation date in every export format.
- Generated projects contain the AI agent files the product documents.
- The CLI can report its own version.

**Non-Goals:**

- Multitool AI artifact generation (`.cursor/rules/*.mdc`, `.claude/skills/`,
  `.agents/workflows/`) — separate change.
- Generating test infrastructure for any stack — separate change.
- Raising the CI PHP version to reach Laravel 13.
- Changing the audit rule set or the report layout beyond the date field.

## Decisions

### Target Laravel 12, not 13

Laravel 12 satisfies the CI's PHP 8.2 and is advisory-free at the current patch
level. Laravel 13 requires PHP `^8.3` and would force a CI toolchain bump
touching unrelated jobs.

*Alternatives:* Laravel 13 + PHP 8.3 (deferred — wider blast radius for no
present benefit); keep 11 and add `policy.advisories.ignore-id` to the generated
`composer.json` (rejected — ships a scaffold that knowingly suppresses security
advisories).

*Detail:* align the sibling dev requirements with the 12.x skeleton
(`nunomaduro/collision` `^8.6`, `phpunit/phpunit` `^11.5.50`) rather than
leaving the current `^8.1` / `^11.0`.

### Make the README true instead of weakening the claim

The README and PRD `US-009` promise `.cursorrules` and `AGENTS.md`. The
generator writes neither; it writes `.ai-workspace/rules.md` and
`.ai-workspace/rules.mdc`. Emit the promised files.

*Alternative considered:* delete the claim from the README. Rejected — the AI
workspace is a product feature with a user story, so the defect is the missing
output, not the documentation.

### Emit both `.cursorrules` and `AGENTS.md`

`.cursorrules` is marked legacy by Cursor, but it is still read, and `AGENTS.md`
is now the cross-tool standard (Antigravity, Cursor, Claude Code, Codex,
Copilot, Windsurf, OpenCode). Emitting both keeps the documented claim true and
provides cross-tool coverage without committing to per-tool artifact layouts in
this change.

### Share one date helper

`real_date_yyyymmdd()` already exists in `src/generator.rs` alongside
`civil_from_days()`, `day_of_week()`, and `MONTHS`, and is used for the blog
sample post. Move the group to a shared module and have both `generator` and
`audit` call it.

*Alternative:* copy the helper into `audit.rs`. Rejected — two date
implementations drift, and `generator.rs` already carries a `ponytail:` note
asking for this dedupe.

### Version from the crate manifest

Report `env!("CARGO_PKG_VERSION")` so `Cargo.toml` stays the single source of
truth.

*Alternative:* a hand-maintained constant in `main.rs`. Rejected — it drifts
from the manifest the release workflow tags.

## Risks / Trade-offs

- **[The generated Laravel skeleton may not boot under Laravel 12, leaving the
  CI job red]** → The legacy classes still resolve in 12.x, so this is unproven,
  not confirmed. Mitigation: verify a generated Laravel project locally with the
  PHP 8.4 + Composer toolchain on this machine before pushing; if
  `php artisan test` fails, migrate `bootstrap/app.php` to
  `Application::configure()` and add the `routes/console.php` file that pattern
  expects. Tasks carry this verification step explicitly.
- **[A dependency constraint is a moving target as advisories are published]** →
  Use a caret constraint on the line (`^12.0`) so Composer resolves the newest
  advisory-free patch, and rely on the CI job as the detector: a future advisory
  blocking 12.x fails the job loudly instead of silently shipping.
- **[`.cursorrules` is legacy]** → `AGENTS.md` is emitted alongside it and is
  read by Cursor; the modern `.cursor/rules/*.mdc` layout is deferred to the
  multitool change.
- **[Windows shell resolution for `composer`/`npm`]** → Already handled by the
  existing `cmd.exe` wrapper in `verify-generated.rs`; no change needed.

## Migration Plan

No data or schema migration; the change is confined to generated content, report
metadata, and one CLI arm. Suggested order:

1. Laravel constraint + skeleton (the CI-blocking item — land first so CI can go
   green on its own signal).
2. Shared date helper + audit call sites.
3. AI agent files.
4. `version` command / `--version` flag.
5. README correction.

Rollback: revert the commit. Generated projects are produced fresh each run, so
no previously generated project changes state.

Local pre-push gate: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, plus a
manual Laravel generation verified with `composer install` and
`php artisan test`.

## Open Questions

None — the remaining unknown (whether the skeleton boots) is a risk with a
defined resolution path in the task list, not a decision that changes the specs
or the approach.
