# Design

## Context

See `proposal.md` – Why for motivation. Constraints that shape this design:

- `src/stack.rs` is 1449 lines. Its public shape is
  `StackOption { id, name, folders: &'static [&'static str], files: &'static [FileDef], init_command, install_command }`
  with `FileDef { path: &'static str, content: &'static str }`, exposed through
  `get_frontend_files(id) -> Option<&'static [FileDef]>` (same for backends).
  `src/generator.rs` iterates the slices and writes `file.path` / `file.content`;
  `src/commands/new.rs` reads `FRONTEND_STACKS` / `BACKEND_STACKS` /
  `DATABASE_STACKS` for display names.
- Existing tests couple to the current API shape:
  `tests/stack_generator_test.rs` asserts `FRONTEND_STACKS.len() == 5`,
  `BACKEND_STACKS.len() == 6`, `DATABASE_STACKS.len() == 3`, and that
  `get_frontend_folders("nextjs")` / `get_backend_folders("nestjs")` contain
  specific paths; `tests/bugfix_test.rs::test_laravel_scaffold_uses_supported_framework_line`
  calls `get_backend_files("laravel")` and inspects `file.path` / `file.content`.
- `tests/bugfix_test.rs::test_workflow_uses_npm_install_not_ci` asserts the
  generated workflow does **not** contain `npm run lint`, reasoning that the
  generated frontends declare no lint script. That invariant is what this change
  deliberately alters.
- Reference implementation for verification already exists in
  `src/bin/verify-generated.rs`: per stack it runs `npm install` + `npm run build`
  (frontend), `npm install` + `npm run build` (nestjs), `go mod …` + `go test` +
  `go build`, `mvn test package`, `dotnet restore|test|build`,
  `composer install` + `php artisan test`. It already wraps `npm`, `mvn`, and
  `composer` in `cmd.exe` on Windows. The new generated projects must satisfy
  these same commands, so its expectations are the compatibility target.
- CI verifies generation per stack by building the generated project with the
  real toolchain; that is the acceptance signal for this change too.
- The tool must stay offline and must remain a single portable binary.
- `fix-ci-and-report-truth` introduces `openspec/specs/project-scaffolding/` on
  archive. This change adds to that capability.

## Goals / Non-Goals

**Goals:**

- Template content is editable as files, with no Rust change required to alter
  or add a template file.
- One local directory can override built-in template content without recompiling.
- Every supported stack generates a project with a passing test entry point.
- Generated CI runs the test step, and never invokes a script that does not exist.

**Non-Goals:**

- Shipping the `verify` command, a security tier, or an analysis command.
- End-to-end, performance, accessibility, or coverage tooling.
- Changing the supported stack or template list.
- Making template content downloadable at runtime.

## Decisions

### Embed the template tree at compile time

Use a build-time directory-embedding crate (`include_dir`) to place `templates/`
inside the binary. Contents stay `&'static`, so the offline and single-binary
properties are preserved, and directory iteration is available for per-stack lookup.

*Alternatives:* one `include_str!` per file — zero dependencies, but ~100 manual
entries and every new template file still requires a Rust edit, defeating the
purpose; a `build.rs` code generator — zero dependency but a hand-rolled
generator to own and test; `rust-embed` — heavier feature surface than needed for
a read-only embed; shipping `templates/` next to the executable — breaks the
single-artifact property and the portable-exe install path.

### Override via an explicit environment variable

`SNIPE_CODE_TEMPLATES` names a directory laid out like `templates/`. Files it
provides win over the embedded version; anything absent falls back to the
default. Resolution is per file, not per stack, so a partial override works.

*Alternatives:* a convention directory under the user home — needs
platform-specific path discovery (`USERPROFILE` vs `HOME`) and is harder to test
hermetically; a project-local `.snipe-code/` directory — ambiguous for a tool
whose job is to create the project directory it would live in.

### Order the work as refactor first, feature second

Group the change so the extraction is behavior-preserving and lands with all
existing tests passing unchanged, and only then add test infrastructure. This
keeps the mechanical diff reviewable and gives a clean bisect point.

### Per-stack test infrastructure

- **Frontend (nextjs, react-vite, vue, nuxtjs)** — `vitest` with a config file and
  one example test asserting rendered output; `test` and `typecheck` scripts.
- **NestJS** — `jest` + `ts-jest` + `@nestjs/testing` + `supertest`, one
  controller spec; `test`, `typecheck` scripts.
- **Go (Gin)** — extract the router into a constructible function in the generated
  `main.go` so `main_test.go` can drive it through `httptest`; `go test ./...`
  needs no manifest entry.
- **.NET 8** — a separate xunit test project referencing the generated web project.
- **Spring Boot** — upgrade the existing placeholder `ApplicationTests` to a real
  `@SpringBootTest` + `MockMvc` test.
- **Laravel** — `phpunit.xml` and `tests/Feature/ExampleTest.php` already exist;
  they become meaningful once the dependency fix from `fix-ci-and-report-truth`
  lands.

### Lint and type-check scripts

Type-check scripts are cheap where a checker is already implied (`tsc` for
nextjs/react-vite, `vue-tsc` for vue/nuxtjs). Lint uses ESLint with a flat config
for the five manifest-based JavaScript/TypeScript stacks, and `go vet` for Go —
which needs no added dependency. Stacks with no linter in their toolchain declare
no lint script, satisfying the requirement by omission rather than by shipping a
linter nobody asked for.

### Generated CI steps are gated on the declared script

`src/cicd.rs` emits a test or lint step only when the corresponding script is
declared for that stack, preserving the existing invariant that generated CI never
calls a script the generated project does not have. The `test` step runs before
`build` so failures are attributed to the right phase.

## Risks / Trade-offs

- **[The refactor changes public accessor types, so ~92 existing tests may need
  mechanical updates]** → Land the extraction as its own task group and keep the
  whole suite green inside that group before adding any new content. Returning
  `Vec<FileDef>` (owned) rather than slices keeps most call sites
  (`iter().find(|f| f.path == …)`) compiling unchanged.
- **[New template files multiply quickly: test configs, example tests, and ESLint
  configs across nine stacks]** → This is exactly the cost the extraction pays
  down; without it `stack.rs` roughly doubles. Keep example tests minimal — one
  assertion each, exercising generated code.
- **[ESLint 9 flat config differs from the legacy `.eslintrc` format and from what
  training data suggests]** → Pin the flat-config form (`eslint.config.js`) and
  verify by running the generated `lint` script in CI; a wrong format fails loudly
  there rather than silently.
- **[Go test requires a constructible router, changing the generated `main.go`
  shape]** → Extract a `newRouter()`-style constructor and have `main()` call it;
  verify with `go test ./...` in CI, which is already wired.
- **[A larger generated CI adds wall-clock to every generated project's pipeline]** →
  Test and lint steps are cheap relative to install; keep them in the same job
  rather than adding new jobs.
- **[Existing test asserting `npm run lint` is absent will fail]** → Update that
  assertion in the same group that introduces the lint script, so the test and the
  behaviour change together.

## Migration Plan

No data migration. This change adds generated content and changes internal
accessor types; no previously generated project changes state.

Ordering and cross-change interaction:

1. `fix-ci-and-report-truth` must land and archive **first** — both changes write
   `openspec/specs/project-scaffolding/spec.md`, and this change's delta adds
   requirements to the capability that change introduces. Archiving this one first
   would create the capability spec from a partial set of requirements.
2. Within this change: extraction → override → frontend test infra → backend test
   infra → CI steps → docs.

Rollback: revert the commit. The embedded tree and the override lookup are
additive; reverting restores literal-based templates with no state to unwind.

Local pre-push gate: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, plus a
manual generation of one frontend and one backend verified with their real
toolchains.

## Open Questions

None. The per-stack tool choices are settled above; anything a reviewer disputes
is a content change inside `templates/`, which this design makes a file edit
rather than a code change.
