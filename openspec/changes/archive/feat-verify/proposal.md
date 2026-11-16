# Proposal

## Why

The tool can generate a project but cannot check that an existing one works.
The only verification in the repository is `src/bin/verify-generated.rs`, a
CI-internal harness: it always generates a fresh project into a temp directory
before running anything, its per-stack command list is a `match` arm inside a
`src/bin/` file, and it cannot be pointed at a project a user already has. A
developer who received a generated project — or who has any project at all — has
no way to ask "does this actually build and pass its tests?" short of remembering
and typing each ecosystem's command.

Everything needed already exists in pieces: stack detection in
`src/audit.rs::detect_scopes`, command running and Windows tool resolution in
`verify-generated.rs` (`command()` wraps `npm`, `mvn`, and `composer` in
`cmd.exe`), and per-stack command knowledge in `src/cicd.rs`. They just are not
reachable as a command.

This change turns that harness into a user-facing `verify` command and puts CI
and users on the same engine, so the code path CI proves is the code path a user
runs.

## What Changes

- **`snipe-code verify [PATH]`** inspects a project, derives the quality gates
  that apply to it from what the project itself declares, runs them, and reports
  a per-gate outcome plus an overall result.
- **Gates are derived, never assumed.** Verify runs a test, build, lint, or
  type-check gate only when the project declares or provides it, and reports
  "not applicable" instead of inventing a command.
- **`--security` adds a dependency vulnerability gate** using the scanner the
  project's ecosystem ships (`npm audit`, `composer audit`, `govulncheck`,
  `dotnet list package --vulnerable`), keeping the tool's own heuristic `audit`
  separate and fully offline.
- **`--only <gate>`** runs a subset, so a developer can iterate on one failure.
- **The CI harness becomes a thin caller of the same engine**, replacing the
  bespoke command table in `src/bin/verify-generated.rs`.
- **A clear exit status contract**: `0` all applicable gates passed, `1` a gate
  failed, `2` usage error, `3` environment error (nothing recognizable to verify).

## Capabilities

### New Capabilities

- `project-verification`: how an existing project's applicable quality gates are
  detected, run, reported, and mapped to an exit status — including the optional
  dependency vulnerability gate.

## Impact

- `src/verify/` — new module: gate detection, the gate runner, per-ecosystem
  command knowledge, and the result summary.
- `src/bin/verify-generated.rs` — reduced to generate-then-verify through the
  shared engine; its `match` command table and `command()` helper move into the
  engine.
- `src/audit.rs` — `detect_scopes` is reused for recognition; it is not moved, so
  the heuristic audit keeps working unchanged.
- `src/main.rs` — route `verify`, and document it in the help and FAQ text.
- `src/lib.rs` — expose the new module.
- `.github/workflows/ci.yml` — unchanged behaviour; it keeps calling the harness,
  which now delegates.
- Tests — a new suite covering gate detection per ecosystem, gating on declared
  scripts, per-gate reporting, and the exit status contract.
- `README.md` — a `verify` section.

Out of scope: generating test infrastructure (`refactor-templates-and-test-scaffold`),
changing the heuristic security audit, end-to-end or performance testing, and any
network access performed by the tool itself — `verify` only spawns the commands
the project already uses.

## Dependencies

- No hard dependency on the earlier changes. Independent of
  `fix-ci-and-report-truth` and `refactor-templates-and-test-scaffold`; it can
  land before or after either.
