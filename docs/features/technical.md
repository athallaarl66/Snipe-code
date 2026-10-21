# Technical Design Document

## Document Information

- Version: 2.0
- Created: 2026-07-28
- Last updated: 2026-10-21
- Applies to: `snipe-code` 0.5.3
- Product requirements: `docs/features/SNIPE-PRD.md`

This document describes the code that exists. Where version 1.0 described a planned
structure, the sections below replace it with the current modules, template layout, and
workflow. Anything still unimplemented is listed in section 13 instead of being written
as if it shipped.

---

## 1. Modules

`src/` is a library (`snipe_code`) plus a binary, so tests call the generator, audit, and
verifier directly instead of shelling out.

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | Manual argument dispatch, `HELP_TEXT`, `FAQ_TEXT`, and per-command help constants, exit codes. No CLI framework. |
| `src/commands/new.rs` | Wizard: template, frontend, backend, database, project name, AI tools, collision prompt, dry run, JSON output. |
| `src/template.rs` | Embeds `templates/` with `include_dir!`, resolves per-file overrides, parses `presets/<id>/preset.json`. |
| `src/stack.rs` | Frontend and backend registries read from `templates/*/manifest.txt`, database list, database URL placeholders, selection validation. |
| `src/generator.rs` | Staging directory, validation, publish with a backup swap, `.env.example` / `.gitignore` / project `README.md` bodies, dry-run tree, `declared_scripts`. |
| `src/cicd.rs` | Generated `.github/workflows/main.yml`. |
| `src/docker.rs` | Generated `docker-compose.yml`. |
| `src/git.rs` | Best-effort `git init` and initial commit. |
| `src/audit.rs` | Shared file traversal, six heuristic checks, Markdown / PDF / DOCX export. |
| `src/analyze/mod.rs` | Read-only language, structure, entry point, and dependency report. |
| `src/verify/mod.rs`, `detect.rs`, `exec.rs` | Gate model, manifest-driven gate detection, command runner. |
| `src/ai_tools.rs` | AI tool registry (`templates/ai-tools.json`), artifact planning, format renderers. |
| `src/ai_workspace.rs` | Writes the planned AI artifacts into a generated project. |
| `src/date.rs` | Report date helper. |

Dependencies stay small on purpose: `serde`, `serde_json`, `dialoguer`, `console`,
`printpdf`, `docx-rs`, `include_dir`. No async runtime, no network client, no template
engine.

`src/bin/verify-generated.rs` and `src/bin/verify-compose.rs` are CI binaries. They
generate a project in a temporary directory and drive the same code paths as the CLI, so
CI exercises the real generator rather than a copy of it.

---

## 2. Template sources

Every stack and preset is data on disk, embedded into the binary at build time:

```
templates/
├── frontends/<id>/          nextjs, react-vite, vue, nuxtjs
│   ├── manifest.txt         name=..., init=..., install=...
│   ├── folders.txt          one folder path per line
│   └── files/<path>         file body written at the same relative path
├── backends/<id>/           dotnet8, nestjs, go-gin, spring-boot, laravel
│   └── manifest.txt, folders.txt, files/<path>
├── presets/<id>/            none, company-profile, portfolio, blog
│   ├── preset.json          folders, files, rules
│   └── content/<source>.txt preset file bodies, one `{}` placeholder for the date
└── ai-tools.json            AI tool registry
```

`manifest.txt` supplies the display name, an optional init command, and the install
command the verifier and generated CI use. `folders.txt` is a flat list: no per-folder
conditions, because the generator writes every folder listed for the selected stack.

Adding a stack is a directory plus an id in `FRONTEND_IDS` or `BACKEND_IDS` in
`src/stack.rs`. Adding a preset is a `preset.json` with its content files;
`TemplateRegistry::load_all()` picks it up with no other Rust change.

Resolution is per file. `SNIPE_CODE_TEMPLATES` points at an override directory laid out
like `templates/`; a file present there wins, a file absent falls back to the embedded
copy, so a partial override is valid. The `new` output prints the source it used as
`Templates: built-in templates` or `Templates: local templates from <dir>`.

---

## 3. Presets

| id | Name | Folders | Files |
| --- | --- | --- | --- |
| `none` | No Template, clean start | none | none |
| `company-profile` | Company Profile | 7 page and layout folders | none |
| `portfolio` | Portfolio | 6 section folders | `frontend/src/data/projects.json` |
| `blog` | Blog / Posts | 6 post, category, tag, and component folders | `frontend/src/posts/hello-world.mdx`, `frontend/public/rss.xml` |

`preset.json` carries `rules` (`require_backend`, `require_database`,
`docker_compose`, `security_configs`). The blog `rss.xml` is a fixed sample that points
at `http://localhost:3000`; nothing regenerates it when a post is added.

The IoT dashboard preset appears in the PRD as a plan. It has no `preset.json` and the
wizard never offers it.

---

## 4. CLI surface

```
snipe-code                                  wizard
snipe-code new [--dry-run] [--json]
               [--template <id>] [--frontend <id>] [--backend <id>]
               [--database <id>] [--name <n>] [--ai-tools <ids|none>]
snipe-code analyze [PATH] [--json]
snipe-code audit [PATH] [--export <md,pdf,docx>] [--json]
snipe-code verify [PATH] [--only <gate>] [--security] [--no-install]
snipe-code help [command]
snipe-code version
snipe-code faq
```

Exit codes: `0` success, `1` a HIGH audit finding or a failed gate, `2` usage error,
`3` runtime or export failure.

A supplied selection value skips only its own prompt. Without a terminal the wizard
prompts nothing and uses template `none`, frontend `nextjs`, backend `none`, database
`none`, no AI tools, and project name `my-project`. Asking for `--frontend none
--backend none` is a usage error, because a project needs at least one part.

The frontend, backend, database, and template values are validated against the
registries before anything is written, so an unknown id exits 2 and creates no
directory. A positional path argument is rejected as an unknown argument rather than
silently ignored.

The project is always created at `<current directory>/<name>`.

---

## 5. Generation flow

1. Resolve the selection and validate it.
2. Resolve the destination and check for an existing folder.
3. Build the AI artifact plan, so an unknown tool id fails before any write.
4. Generate into a staging directory beside the destination.
5. Validate the staged tree.
6. Publish. A new destination is renamed into place. An existing destination is moved to
   a unique sibling backup first and removed only after the swap succeeds; a failed
   publish restores the backup and reports where it is.
7. Write `docker-compose.yml` when both a backend and a database were selected, then the
   CI workflow, `.ai-workspace/rules.md` plus `rules.mdc`, the selected AI artifacts, and
   finally `git init` with an initial commit when git is available.

Step 7 sits outside the atomic publish on purpose. Git is best effort, so a missing git
binary prints a warning instead of failing a project that was generated successfully.

Collision handling has two outcomes. Interactively the prompt offers
`Overwrite (replace existing)` and `Cancel`, with Cancel preselected. Without a terminal
the run fails with `Error: Folder "<path>" already exists`, because a scripted run has
nobody to answer the prompt.

Generated files every project receives:

- `.env.example`, with the database URL placeholder for the selected database and no real
  secret. The body is `DATABASE_URL` plus `API_PORT`, `API_SECRET`, `CORS_ORIGIN`,
  `RATE_LIMIT_WINDOW`, and `RATE_LIMIT_MAX`.
- `.gitignore`, covering dependency, environment, IDE, OS, and build output directories,
  plus the three `Audit_Report.*` names.
- `README.md`, naming the selected stacks with their install and run commands.
- `.github/workflows/main.yml`, described in section 6.
- `docker-compose.yml`, with a PostgreSQL or MySQL service for the selected database.
- `.ai-workspace/rules.md` and `.ai-workspace/rules.mdc`, always.

---

## 6. Generated CI workflow

The workflow runs one job per part:

| Part | Jobs and commands |
| --- | --- |
| npm frontend | `npm install` on Node 24, then the declared `typecheck`, `lint`, `test`, `build` scripts |
| NestJS backend | `npm install` on Node 24, then the declared scripts |
| Go backend | Go 1.21: `go mod tidy`, `go test ./...`, `go vet ./...`, `go build ./...` |
| Spring Boot backend | Java 17: `mvn test package` |
| .NET backend | .NET `8.0.x`: `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |
| Laravel backend | PHP 8.2: `composer install`, `php artisan test` |

Only scripts a part actually declares are emitted, so a frontend without a `test` script
gets no test step instead of a failing one. `npm install` is used rather than `npm ci`
because a generated project ships no lockfile.

---

## 7. Audit

Six checks run over the scanned tree. Each finding carries a severity and a location:

| Check | What it looks for |
| --- | --- |
| Hardcoded secrets | Assignment-shaped literals inside source files |
| `.env` in git | A present `.env` beside version control metadata |
| SQL injection | Query strings built by concatenation |
| XSS risks | Unescaped HTML injection sinks |
| Security configs | Missing baseline configuration files |
| Stack source risks | Patterns tied to the recognized ecosystems |

Traversal skips VCS metadata, dependency directories, build output, and report output,
and the report states what it excluded. Export formats are `md`, `pdf`, and `docx`,
written as `Audit_Report.*` inside the scanned directory. Duplicate formats normalize;
an empty or unknown token is a usage error. With `--json` the files are still written and
the export summary moves to stderr so stdout stays parseable.

The report is heuristic. It states the detected scopes and the rule groups that ran, and
"no findings detected by evaluated rules" is not a claim that the project is safe.

---

## 8. Analyze

`analyze` walks the tree once and reports languages with file and line totals, top-level
structure, entry points per part, declared dependencies per manifest, and a Limits
section. Language detection is an extension table covering Rust, TypeScript, JavaScript,
Vue, PHP, Go, Java, C#, Python, Ruby, Shell, PowerShell, SQL, CSS, Sass, HTML, Markdown,
JSON, YAML, TOML, and XML.

Entry point candidates are checked per part in a fixed order, for example
`frontend/src/main.tsx` before `index.html`, and `backend/artisan` for Laravel.

`package.json`, `composer.json`, `go.mod`, and `Cargo.toml` are read as JSON or TOML
structures. `pom.xml` and `*.csproj` are scanned for dependency element names, because
the binary ships no XML parser. Line totals include blank lines and comments: there is no
code-versus-comment split.

---

## 9. Verify

Gates come from manifests in the target tree, never from the generator's stack list, so
`verify` works on projects this tool did not create.

| Manifest | Gates |
| --- | --- |
| `package.json` | `npm install`, then the declared `typecheck`, `lint`, `test`, `build` scripts |
| `composer.json` + `artisan` | `composer install --no-interaction --no-progress`, `php artisan test` |
| `composer.json` + `phpunit.xml` | `composer install --no-interaction --no-progress`, `php vendor/bin/phpunit` |
| `go.mod` | `go mod download`, `go mod tidy`, `go vet ./...`, `go test ./...`, `go build ./...` |
| `pom.xml` | `mvn test`, `mvn package -DskipTests` |
| `*.csproj` / `*.sln` | `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |

Gate order is fixed: install, lint, typecheck, test, build, then security when requested.
A gate reports `passed`, `failed`, `not-applicable` with a reason, or `unavailable` when
its tool is missing. `unavailable` does not fail the run.

`Report` is returned to both callers. `verify::run` takes a `Runner` function pointer, so
tests inject a fake runner and never spawn `npm` or `go`.

`--no-install` clears the install gate commands and records the reason
`skipped by --no-install`, so the gate still appears in the report.

---

## 10. AI agent artifacts

`templates/ai-tools.json` holds one entry per tool: `id`, `name`, and `artifacts[]` with
`path`, `format`, and `template`. Ten tools are registered: `agents-md`, `claude-code`,
`cursor`, `opencode`, `antigravity`, `github-copilot`, `windsurf`, `hermes-agent`,
`codex`, and `gemini-cli`.

Renderers are keyed by format: `markdown`, `mdc`, `skill`, `command`. An unknown format
or tool id is an error, never a silent skip. `AGENTS.md` is claimed by both the
cross-tool entry and Codex, and `GEMINI.md` by Antigravity and Gemini CLI; a claimed path
is written once and the overlap is reported in `GenerateResult.ai_overlaps`.

Artifacts are opt-in. Selecting none leaves only the shared `.ai-workspace/rules.md` and
`.ai-workspace/rules.mdc`. The command artifacts are instructions for an external agent
to run; `snipe-code` itself never calls a model.

---

## 11. Release and distribution

### 11.1 Build targets

The list is data in `.github/release-targets.json`. The `package` matrix and the expected
counts in `publish` both read it, so a platform change is one edit.

| Target | Runner | Platform | Archive |
| --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Linux x86_64 | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | Windows x86_64 | `.zip` |
| `x86_64-apple-darwin` | `macos-15-intel` | macOS x86_64 (Intel) | `.tar.gz` |
| `aarch64-apple-darwin` | `macos-latest` | macOS arm64 (Apple silicon) | `.tar.gz` |

### 11.2 Published channels

State observed on 2026-10-21 from the GitHub, crates.io, and ghcr.io APIs:

| Channel | State |
| --- | --- |
| GitHub release `v0.5.3` | 12 assets: four archives, four `.sha256` manifests, four cosign bundles |
| crates.io | `snipe-code` 0.5.3, with 0.5.1 and 0.5.2 before it |
| Container image | `ghcr.io/athallaarl66/snipe-code`, tags `v0.5.2`, `v0.5.3`, `latest` |
| `install.sh` / `install.ps1` | In the repository and inside the archives; not run against v0.5.3 on each platform |

Each archive contains the binary, `LICENSE`, `README.md`, and `SOURCE_REVISION`. The
installers download an archive, verify the published checksum, verify the cosign bundle
when `cosign` is installed, and only then place the binary.

`cargo install snipe-code --locked` installs the published version. The flag is required:
the packaged `Cargo.lock` pins `zip 0.6.6`, crates.io marks that release yanked, and a
lockfile pin overrides yank status while a fresh resolve does not.

### 11.3 Release workflow

`.github/workflows/release.yml` has five jobs:

| Job | Role |
| --- | --- |
| `quality` | Asserts the tag equals the crate version, then runs `cargo fmt --check`, Clippy `-D warnings`, `cargo test`, and `cargo publish --dry-run` |
| `targets` | Reads `.github/release-targets.json` and exposes the matrix and the target count |
| `package` | One job per target: build, stage the archive (binary, `LICENSE`, `README.md`, `SOURCE_REVISION`), checksum it, upload it |
| `publish` | Derives expected archive and checksum counts from the target list, asserts every target has an archive, runs `sha256sum --check`, signs each archive with keyless cosign, publishes the GitHub release |
| `crate` | Runs after `quality`, skips a version the registry already holds, then runs `cargo publish` |

`cargo publish` needs the `CARGO_REGISTRY_TOKEN` repository secret. Without it the
`crate` job fails loudly instead of skipping the publish.

---

## 12. Risks

| Risk | Mitigation |
| --- | --- |
| Heuristic audit produces false positives | The report states detected scopes and rule groups; generated agent guidance says to review findings before editing |
| `verify` mutates the project it checks | Every gate is the project's own command; the README lists what each writes; `--no-install` skips the install gate |
| Audit reports land inside the scanned project | Generated `.gitignore` ignores the three `Audit_Report.*` names |
| `zip` 0.6.x yanked upstream | Builds resolve from the committed `Cargo.lock`; install docs pass `--locked` |
| Overwrite destroys an existing directory | Staged publish with a sibling backup and rollback; the backup path is reported when recovery fails |
| Stack version drift | Templates are plain files, so a version bump is a template edit and a rebuild |
| Generated project needs a toolchain the user lacks | `verify` reports the gate as `unavailable` instead of failing the run |

---

## 13. Limits and unimplemented features

- Audit findings carry severity, location, and message strings rather than typed rule
  ids, so cross-run deduplication is not implemented.
- PDF export is page-limited; a long finding list reports an omission count instead of
  rendering every entry.
- `verify` recognizes the npm, Composer, Go, Maven, .NET, and Cargo ecosystems. Python and
  Ruby projects report no gates.
- Generated Laravel projects ship `.env.example` only, so `php artisan test` prints one
  phpdotenv warning while exiting 0.
- The blog RSS file is a static sample with a localhost URL.
- The IoT dashboard preset, template marketplace, template versioning, and
  `~/.snipe-code/stacks.json` user-defined stacks are plans in the PRD, not features.
- The installer scripts have not been run against a published release on each platform.
- The AI layer (`audit --explain`) is deferred. If it is added, it uses a user-supplied
  key or a local model, never a key embedded in the binary.

---

## 14. Change history

| Version | Date | Description |
| --- | --- | --- |
| 1.0 | 2026-07-28 | Initial design for enterprise preset templates |
| 2.0 | 2026-10-21 | Rewritten against the 0.5.3 code base: modules, template layout, presets, generated output, audit, analyze, verify, AI artifacts, release channels, limits |
