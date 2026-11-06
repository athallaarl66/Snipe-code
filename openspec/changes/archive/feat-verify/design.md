# Design

## Context

See `proposal.md` – Why for motivation. What already exists and where it lives:

- `src/bin/verify-generated.rs` is the whole of today's verification, and it is a
  `.rs` binary target rather than a library module. It parses `--frontend`,
  `--backend`, `--database`; generates `generated-app` into
  `env::temp_dir()/snipe-verify-<pid>-<nanos>`; `set_current_dir`s into it;
  generates; then runs a per-stack table:
  frontend and `nestjs` → `npm install` + `npm run build`; `go-gin` →
  `go mod download`, `go mod tidy`, `go test ./...`, `go build ./...`;
  `spring-boot` → `mvn test package`; `dotnet8` → `dotnet restore`,
  `dotnet test --no-restore`, `dotnet build --no-restore`; `laravel` →
  `composer install --no-interaction --no-progress` + `php artisan test`.
  Any unknown backend is an error. The first non-zero status aborts and the
  temp directory is removed on the way out.
- `command()` in that file already solves the Windows problem: for `npm`, `mvn`,
  and `composer` it spawns `cmd.exe /C <program>` and passes the args, because
  those are `.cmd` shims that `CreateProcess` cannot start directly. Go, dotnet,
  and php are real executables and are spawned directly.
- `src/audit.rs::detect_scopes` already recognizes a project from files on disk
  (it returns scope names such as the detected languages and dotfiles) and is
  reused by the audit report. It does not recognize Python or Ruby, and it does
  not read manifests.
- `src/cicd.rs` already encodes per-stack command knowledge for generated
  projects. It is guidance, not a manifest reader, so verification cannot use it
  as the source of truth for a project the tool did not generate.
- Exit statuses are already a convention: `src/main.rs` uses `2` for usage errors
  and routes audit's `0`/`1`/`2`/`3` contract (`1` policy failure, `3` other
  audit errors).
- CI calls the harness with `cargo run --bin verify-generated -- --backend laravel`
  and similar, one job per stack.

## Goals / Non-Goals

**Goals:**

- A user can point the CLI at any project and learn whether its quality gates pass.
- Gates are derived from the project, so the command works on projects the tool
  did not generate.
- CI and users execute the same verification code path.
- The tool itself still performs no network access.

**Non-Goals:**

- Generating missing test infrastructure — that is
  `refactor-templates-and-test-scaffold`.
- Replacing the heuristic `audit` command, or merging the two.
- Recognizing ecosystems beyond what the project already detects.
- Gate timeouts, parallelism, retries, or caching.

## Decisions

### Verify runs the project's tools; the tool stays offline

The offline constraint means the tool makes no network calls. It does not mean the
commands it spawns avoid the network — `npm install` and `npm audit` inherently
use it, exactly as they do in the project's own CI today. The heuristic `audit`
command remains the fully offline, read-only analysis path. State this split in
the README so the distinction is not a surprise.

### Derive gates from manifests, not from the stack list

Read the project's own manifests (package manifest scripts, a PHPUnit
configuration, a Go module, a Maven or .NET project file) and derive gates from
what they declare. Consulting `src/cicd.rs` for a fixed stack list would only work
on projects this tool generated, which is the limitation being removed.

*Alternatives:* reuse the `verify-generated.rs` stack table — works only for
generated projects; add a `--stack` flag — pushes detection work onto the user,
which is what `verify` exists to eliminate.

### Reuse `detect_scopes` for recognition instead of writing a second detector

It already answers "is there a project here and what is in it", and the audit
report already depends on it. Its documented blind spot (no Python or Ruby
recognition) becomes verification's blind spot too; that is a known, stated limit
rather than a second detector to keep in sync.

### Deterministic gate order with named states

Run gates in a fixed order — install, lint, typecheck, test, build — so a failure
is attributed to the right phase and runs are comparable. Each gate reports one of
four states: `passed`, `failed`, `not-applicable` (the project declares nothing for
it), `unavailable` (the tool is not installed).

An `unavailable` gate does not by itself fail the run — a missing toolchain is an
environment fact, not a project defect — but it is listed in the summary, and if
that leaves no gate actually executed the run exits `3` so "nothing was verified"
is never reported as success.

### One engine, two callers

The engine moves into `src/verify/` with a library API. `src/main.rs` routes the
`verify` command to it; `src/bin/verify-generated.rs` becomes generate-then-call.
This is the point of the change: CI's green check and a user's local check must
exercise identical logic, so a fix for one is a fix for both.

*Alternative:* leave the harness alone and build a parallel user-facing runner —
two command tables encoding the same knowledge, guaranteed to drift.

### Security gate uses each ecosystem's bundled scanner

`npm audit`, `composer audit`, and `dotnet list package --vulnerable` ship with
their toolchains. Go's `govulncheck` and Java's OWASP dependency-check do not, so
those ecosystems report the gate as `not-applicable` with the reason stated rather
than silently passing. Only scanners that are genuinely available are run.

### Command execution is bounded by what the project declares

Verification runs the project's own declared scripts, which is the same trust model
as running the project's CI. It never accepts a command string from the user, never
builds a shell string, and always spawns through `Command` with an explicit
argument vector. Every command is printed before it runs.

## Risks / Trade-offs

- **[Verification spawns project-declared scripts, which is code execution]** → The
  boundary is that the commands come from the project's own manifest and argument
  vectors are explicit; a user-supplied command string is never accepted, and no
  shell string is constructed. Each command is echoed before it runs so the user
  sees what executed.
- **[`detect_scopes` does not recognize Python or Ruby, so those projects are not
  verifiable]** → Documented as a limit. Extending detection is a separate change;
  reusing the existing detector keeps one source of truth.
- **[Refactoring the harness could silently change what CI verifies, making a red
  project look green]** → Land the extraction as its own task group with the CI
  workflow untouched, and require the per-stack CI jobs to stay green across that
  group before the `verify` command is added.
- **[Gate runtime is unbounded; `npm install` or `mvn test package` can take
  minutes]** → Gates run sequentially with output streamed so progress is visible;
  no timeout is imposed, because a wrong timeout turns a slow machine into a false
  failure. A timeout option is a later addition.
- **[`unavailable` gates could let a run report success while verifying little]** →
  Unavailable gates are listed explicitly in the summary, and a run where nothing
  actually executed exits `3` instead of `0`.
- **[Windows resolution differs per tool]** → Reuse the proven `cmd.exe` wrapper
  for `.cmd` shims and spawn real executables directly; the existing CI jobs across
  Windows and Linux exercise both paths.

## Migration Plan

No data or schema migration. This change adds a module and a command and reduces a
binary target to a caller.

Suggested order:

1. Move the gate knowledge and command runner into `src/verify/` and reduce
   `verify-generated.rs` to a caller — behaviour-preserving, with the CI workflow
   and its per-stack jobs unchanged and green.
2. Add manifest-based gate detection.
3. Route the `verify` command, including the exit status contract and `--only`.
4. Add the `--security` gate.
5. Documentation.

Rollback: revert the commit. Nothing generated on disk persists, and the CI
workflow keeps calling the same entry point throughout.

Cross-change interaction:

- No hard dependency. `refactor-templates-and-test-scaffold` adds test scripts to
  generated projects, which makes verification's test gate applicable where it
  currently is not. That is a beneficial interaction, not a conflict, but a run of
  this change before that one verifies fewer gates on generated projects.
- This change does not touch `openspec/specs/project-scaffolding/` or
  `openspec/specs/ai-workspace-generation/`, so it has no archive-order constraint.

Local pre-push gate: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, plus a
manual `verify` run against one project of each recognized ecosystem.

## Open Questions

None. The scanner availability per ecosystem is settled above from what ships with
each toolchain; anything a reviewer disputes is a table entry in the security gate.
