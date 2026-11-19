# SNIPE-CODE CLI

[![CI](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml/badge.svg)](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml)[![XERJ Corpus Hub contributor](https://hub.xerj.org/contributors/athallaarl66/badge.svg)](https://hub.xerj.org/contributors/athallaarl66)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)


A Rust CLI that scaffolds projects, reads an existing codebase, and audits it for
security risks. Core scaffolding and analysis do not require an AI service or API key.
Verification runs external project commands, which may access the network.

This checkout is version `0.6.0`. Changes are listed in [CHANGELOG.md](CHANGELOG.md).

Distribution status checked 2026-11-19: GitHub release `v0.6.0` is published but
has no binary assets. Its Windows and Linux archive URLs return 404, so the default
latest-release installers cannot install it yet. Check the
[release assets](https://github.com/athallaarl66/Snipe-code/releases/tag/v0.6.0)
again, or use the local build below. The newest crate on crates.io is `0.5.3`, not
`0.6.0`; the container's `v0.5.3` tag was checked on 2026-10-21.

## Install

### Shell installer

The no Rust toolchain channel requires a release with the matching archive and
checksum. By default it reads the latest release; while `v0.6.0` has no assets, use
[Build from source](#build-from-source) instead. When assets are available, download
the script into the current directory, inspect it, then run that same file.

Linux and macOS:

```bash
curl -fsSL -o install.sh https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.sh
less install.sh
bash ./install.sh
```

Windows PowerShell:

```powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.ps1 -OutFile .\install.ps1
Get-Content .\install.ps1
.\install.ps1
```

If PowerShell blocks scripts under its execution policy, review the policy and script;
do not bypass it blindly. The source-build EXE below does not require running the installer.

The installer downloads the release archive to a temporary directory, checks the
published SHA-256 checksum, checks the cosign bundle when `cosign` is installed, and
only then moves the binary into place. Nothing is piped into a shell. If the download
fails or a check does not pass, it exits non-zero, says why, and leaves no partial
binary behind. Default destination is `~/.local/bin` on POSIX and
`%LOCALAPPDATA%\Programs\snipe-code` on Windows; both print a PATH hint. Override with
`SNIPE_INSTALL_VERSION`, `SNIPE_INSTALL_DIR`, and `SNIPE_RELEASE_BASE_URL`.

### cargo install

The published crate is version 0.5.3:

```bash
cargo install snipe-code --locked
```

The `--locked` flag is required here too. A plain `cargo install snipe-code` re-resolves
dependencies and stops on the yanked `zip` release that `docx-rs` needs.

### Container image

Published at GitHub Container Registry, with tags `v0.5.3` and `latest`:

```bash
docker run --rm -v "$PWD:/work" -w /work ghcr.io/athallaarl66/snipe-code:latest audit . --export md
```

The image has not been run on this machine. Build from source if you want to test the
binary from your own checkout.

### Build from source

```bash
git clone https://github.com/athallaarl66/Snipe-code.git
cd Snipe-code
cargo build --release
```

For a release asset 404, build the desired checkout locally instead; the installer
does not fall back automatically. If the repository is already cloned and
`cargo build --release` succeeded, do not clone or rebuild again. Building requires a
Rust toolchain and may need network access for dependencies.

The binary lands in `target/release/snipe-code` (`.exe` on Windows). Copy it if
needed, but the destination must support the binary's platform and runtime dependencies.
`target/` can reach hundreds of megabytes; after copying the binary, `cargo clean`
removes build artifacts.

Cargo does not put that binary on `PATH`, so typing `snipe-code` in a fresh shell
still fails with `'snipe-code' is not recognized as an internal or external command`.
Run it by path from the checkout:

```powershell
./target/release/snipe-code.exe --version
./target/release/snipe-code.exe faq
```

PowerShell needs an explicit path to run an executable outside PATH. For a path with
spaces, use the call operator: `& "C:/path with spaces/snipe-code.exe"`. The relative
path above only works from the checkout root.

```bash
./target/release/snipe-code faq
```

`cargo run --release -- faq` does the same thing without leaving the source tree. To
type `snipe-code` from any directory, install the local build:

```bash
cargo install --path . --locked
```

That copies the binary into `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows),
which the Rust installer already adds to `PATH`, so a new shell resolves `snipe-code`.
If it still is not recognized, add that directory to `PATH` yourself. The `--locked`
flag is required: a plain `cargo install --path .` re-resolves dependencies, finds the
`zip` releases its `docx-rs` dependency needs yanked upstream, and stops with
`failed to select a version for the requirement`. `--locked` resolves from the
repository's `Cargo.lock` instead and succeeds. Like any `cargo install`, it takes a snapshot: re-run it after changing
the code, or keep using `cargo run`.

### First manual check on a disposable project

In PowerShell, from the checkout root, point the built EXE at a separate test
project. Replace the example path with an existing disposable project directory;
these commands do not create it:

```powershell
$exe = (Resolve-Path ./target/release/snipe-code.exe).Path
$project = "C:/path/to/disposable-project"
& $exe --version
& $exe analyze $project
& $exe scan $project
```

Confirm the version prints, analysis includes project structure and Limits, and the
scan finishes; review findings and coverage limits rather than treating no findings as
proof of safety. `analyze` is read-only; `scan` does not save history without
`--save`. To run from inside the test project, use `Set-Location $project` and the
same `& $exe analyze .` or `& $exe scan .` commands. Do not start with `audit` or
`verify`: audit writes reports, while verify executes project commands that may write
files, reach the network, or access databases.

### Uninstall

Delete the binary the installer placed, or run `cargo uninstall snipe-code` if you
installed through cargo. No registry keys or system files are touched.

## Commands

Run `snipe-code` in an interactive terminal to see the ASCII SNIPE-CODE banner,
compiled version, author Athalla, and quick commands: `new`, `analyze`, `scan`,
`verify`, and `help`. The launcher then lets you choose a goal: Create a new
project, Understand an existing project, Run tests and quality checks, Audit project
security, View help, Install global AI skill, or Exit. Security offers scan, export, history, and legacy audit;
legacy audit writes reports into the target. With no interactive terminal, a bare
`snipe-code` prints usage and exits `2` without starting an operation.

Direct commands bypass the launcher, including `new`, `analyze`, `audit`, and
`verify`. For scripts, specify `snipe-code new` rather than a bare invocation.

| Command | What it does |
| --- | --- |
| `snipe-code` | Interactive goal launcher |
| `snipe-code new` | Project-generation wizard |
| `snipe-code analyze [PATH]` | Read-only report: languages and size, structure, entry points, dependencies, limits |
| `snipe-code audit [PATH]` | Legacy heuristic audit with report export |
| `snipe-code scan [PATH] [--json] [--save]` | Read-only audit; `--save` stores versioned JSON locally |
| `snipe-code export <scan.json> --format <md,pdf,docx> --output <DIR>` | Export saved scan result to explicit directory |
| `snipe-code history` | Manage locally saved scan runs; export writes JSON |
| `snipe-code skill [AGENT] [--install]` | Preview or install global AI agent instructions |
| `snipe-code verify [PATH]` | Runs the quality gates the project declares |
| `snipe-code quality [PATH] --config <FILE>` | Versioned quality gates; see quality gate configuration below |
| `snipe-code version` | The compiled version (alias `--version`) |
| `snipe-code help [command]` | General help and per-command help |
| `snipe-code faq` | Install, uninstall, troubleshooting |

### Where each command runs and what it writes

| Command | Run it from | Reads | Writes | Exit codes |
| --- | --- | --- | --- | --- |
| `new` | the directory that should hold the new project. The project is created at `<current directory>/<name>`, so `cd` to its parent first. A path argument does not move it. | the embedded templates, or `SNIPE_CODE_TEMPLATES` | `<current directory>/<name>/`; Git actions depend on the invocation, as described below | `0` generated, `1` generation failed, `2` usage error, `3` runtime error |
| `analyze [PATH]` | anywhere; `PATH` defaults to the current directory | project files | nothing | `0`, `2`, `3` |
| `audit [PATH]` | anywhere; legacy defaults to current directory | project files | legacy report files inside `PATH` | `0`, `1`, `2`, `3` |
| `scan [PATH] [--json] [--save]` | anywhere | project files | JSON on stdout; history DB only with `--save` | `0`, `2`, `3` |
| `export <scan.json> --format <formats> --output <DIR>` | anywhere | versioned scan result | selected report files inside `DIR` | `0`, `2`, `3` |
| `history` | anywhere | local history DB | JSON file for export; DB for delete | `0`, `2`, `3` |
| `skill [AGENT] [--install]` | anywhere | embedded skill instructions and home environment | nothing on preview; skill directory and file on install | `0`, `2`, `3` |
| `verify [PATH]` | the project directory, or point `PATH` at it | manifests | an explicit report when requested; gates may write into the project | `0`, `1`, `2`, `3`, `4` |
| `help`, `version`, `faq` | anywhere | nothing | nothing | `0` |

Commands with side effects:

- `new` creates a project. Interactive Git initialization and commit require separate
  choices; non-interactive `new` attempts both when Git is available.
- `scan` is read-only unless passed `--save`, which writes to the local history database.
- Legacy `audit` writes reports into the scanned directory. Without a terminal it skips
  the format prompt and exports Markdown, printing
  `Terminal non-interaktif — default export: markdown.` first.
- `verify` runs the project's commands. For example, `npm install` leaves `node_modules/`
  and `package-lock.json`, `go mod tidy` rewrites `go.mod` and adds `go.sum`, `dotnet
  restore` leaves `obj/`, and Maven leaves `target/`. Pass `--no-install` to skip the
  install gate and use the dependencies already on disk.

## new: scaffold a project

```bash
snipe-code                                    # interactive goal launcher
snipe-code new --dry-run                      # preview the structure, write nothing
snipe-code new --ai-tools cursor,claude-code  # pick AI artifacts without prompting
snipe-code new --ai-tools none --name demo --json
snipe-code new --template blog --frontend nextjs --backend none --database none --name site
```

The interactive wizard selects preset, frontend, backend, database, project name,
AI integrations, Git initialization, and initial commit. It shows the destination
and selections before offering Preview, Create project, Back, or Cancel. Preview
shows planned files without creating the project. An existing destination prompts
for cancellation or overwrite. Git initialization and commit are separate choices;
a declined or unavailable Git action leaves the generated project in place.

Presets add structure to the selected framework stack. No preset adds no
preset-specific files; Company Profile adds page and layout structure; Portfolio
adds portfolio structure; Blog adds MDX, a sample post, and static RSS. Docker,
CI, and AI workspace files are written during generation.

The project directory is created at `<current directory>/<name>`, so run the wizard
from the folder that should hold it. The name comes from the prompt or `--name`; no
argument redirects the output somewhere else.

| Flag | Effect |
| --- | --- |
| `--dry-run` | Print the structure without creating a file |
| `--template <id>` | `none`, `company-profile`, `portfolio`, or `blog`; skips the template prompt |
| `--frontend <id>` | `nextjs`, `react-vite`, `vue`, `nuxtjs`, or `none`; skips the frontend prompt |
| `--backend <id>` | `dotnet8`, `nestjs`, `go-gin`, `spring-boot`, `laravel`, or `none`; skips the backend prompt |
| `--database <id>` | `postgresql`, `mysql`, or `none`; skips the database prompt |
| `--ai-tools <ids\|none>` | AI agent artifacts to generate, comma separated; `none` skips the prompt and generates none |
| `--name <n>` | Skip the project-name prompt |
| `--json` | Generate result (`project`, `ai_tools`, `ai_artifacts`, `ai_artifact_types`, `ai_overlaps`, `files`, `outcome`) as JSON on stdout, progress on stderr |

With no terminal, explicit `snipe-code new` uses defaults and does not prompt: preset
none, frontend Next.js, backend none, database none, project name `my-project`. Pass
`--name` to make the project name explicit. After generation it attempts Git init and
an initial commit when Git is available; failures are warnings.

Supported stacks. Frontend: Next.js, React (Vite), Vue, Nuxt. Backend: .NET 8,
NestJS, Go (Gin), Spring Boot, Laravel 12. Database: PostgreSQL, MySQL. A generated
project carries `.env.example`, `.gitignore`, `docker-compose.yml` for backend and
database combinations, CI/CD workflows, and test infrastructure.

## analyze: what a project is made of

```bash
snipe-code analyze .                 # text report
snipe-code analyze ../other --json    # the same report as JSON on stdout
```

Read-only and offline: no file in the target project is created, modified, or deleted,
and nothing is fetched from the network. The report has five sections:

- Languages: each language found, with its file and line totals.
- Structure: top-level entries and the project parts the audit detector recognizes.
- Entry points: where execution starts for each part, for example
  `frontend/src/main.tsx`, `backend/main.go`, `src/main.rs`, `*Application.java`,
  `Program.cs`, `artisan`.
- Dependencies: what each manifest declares (`package.json`, `composer.json`,
  `go.mod`, `Cargo.toml`, `pom.xml`, `*.csproj`).
- Limits: what it could not analyze.

Traversal reuses the audit walkers, so dependency and build output directories
(`node_modules`, `vendor`, `target`, `dist`, `build`, `bin`, `obj`, `out`, `coverage`,
`reports`) and hidden directories are left out of the numbers. One exception: `src/bin`
holds real source in a Rust crate, so it still counts.

What the Limits section says:

- The figures are line counts, not lines of code. Blank lines and comments are
  included: there is no code-versus-comment split, which would need a parser per
  language.
- `pom.xml` and `*.csproj` are scanned for dependency element names rather than
  parsed, because this tool ships no XML parser. The other four manifests are parsed.
- Files whose extension is outside the language table are reported as not analyzed
  with their file count, instead of being dropped.

Exit status: `0` report printed, `2` usage error, `3` the path is missing or is not a
directory. The `--json` output carries the same findings as the text report.

## audit: security scan

```bash
snipe-code scan . --json > scan.json
snipe-code export scan.json --format md,pdf --output ./reports
snipe-code audit . --export md --json     # legacy; writes report file and JSON
```

`scan` performs read-only heuristic checks and Rust candidate analysis, and emits
JSON envelope with `schema_version: 2`. It does not create history unless called
with `--save`.

Rust rules cover straight-line local bindings from `std::env::var`/`args` to
`sh`/`bash -c` command chains, SQL methods (`execute`, `query`, `query_row`,
`prepare`), and `axum::response::Html`/`warp::reply::html`. SQL receiver types are
not resolved. Bound parameters remain separate from SQL text; `ammonia::clean`
and `html_escape::encode_safe` protect the modeled HTML output, not shell or SQL.
Findings carry rule ID/version, confidence, candidate label and line evidence,
without source text. The paired corpus is `tests/fixtures/sast/rust`.

The adapter does not resolve aliases, types, cross-function flows or macro
expansion. Unsupported control flow/transforms and parser input budgets report
`unknown`; invalid Rust reports `error`. Other source languages remain
`unknown`, not clean. Parsing is bounded by source size, delimiter depth and an
operator budget, not a wall-clock timeout. Legacy heuristic identities stay
separate from Rust rule versions.

SCA reads the scan root's `package-lock.json` version 3, including pinned direct
and transitive npm packages. Other ecosystems, workspace/link/git dependencies,
aliases, missing lockfiles and incomplete graphs produce `unknown`. Results live
in `result.sca` with package/version, installed path, advisory ID, severity and
snapshot provenance. They do not change legacy heuristic severity totals.

```bash
snipe-code advisory refresh ./app --cache ./advisories --yes
snipe-code scan ./app --snapshot ./advisories --json --save
```

Refresh requires `curl` with HTTPS and sends package names and exact versions to
`https://api.osv.dev/v1/query`, not source or credentials. It accepts GHSA records
from GitHub Advisory Database (CC-BY-4.0); no live advisory data ships with the
binary. Snapshots contain format version, retrieval/expiry timestamps and SHA-256.
They expire after seven days. SHA-256 detects corruption, not authenticity;
refresh trusts HTTPS. Invalid updates retain the previous active snapshot.

Scan performs no refresh or network request. Without `--snapshot`, advisory
status is `unknown`. The option accepts a snapshot file or cache directory.
Missing, malformed or corrupt snapshots stay `unknown`; expired snapshots stay
`stale`, not clean. A clean package result covers only its cached version query,
not all vulnerabilities or ecosystems. Tests use synthetic responses, not live OSV.

`audit --json` keeps its legacy `AuditResult` format. `scan --json` emits versioned
envelope. `export` reads either format and writes selected `md`, `pdf`, or `docx` reports
into an existing directory. It never rescans source.

```bash
snipe-code scan . --json --save
snipe-code history list
snipe-code history show <run-id>
snipe-code history export <run-id> --output ./scan.json
snipe-code history delete <project-id>
```

`history export` writes the canonical envelope as JSON. Use `export scan.json --format
md,pdf,docx --output DIR` to render reports.

History is opt-in. SQLite lives under `%LOCALAPPDATA%/snipe-code` on Windows,
`~/Library/Application Support/snipe-code` on macOS, or `$XDG_DATA_HOME/snipe-code`
(default `~/.local/share/snipe-code`) on Linux. Set `SNIPE_CODE_DATA_DIR` to select
another location. History has no automatic expiry; delete a project's runs with its
`project_id` from `history list`. Finding paths are relative, descriptions are redacted,
and Unix files use owner-only permissions. SQLite is not encrypted; protect the user
account and backups accordingly. Saving and reading history use no network or project
database credentials.

Audit traversal skips symbolic links and Windows reparse points, including junctions,
before opening their targets. Canonical-path checks also reject reads outside the
selected root. This does not make audit a sandbox for concurrent filesystem changes.

Legacy `audit` remains compatible during migration and writes `Audit_Report.*` into
the scanned directory. It prints a side-effect notice to stderr. Legacy `--json` keeps
valid JSON on stdout while report files are still written.

Exit code:

- `0`: audit finished, no HIGH finding
- `1`: a HIGH finding was found
- `2`: usage error, for example an invalid export format
- `3`: runtime or export error, for example a missing path

The audit is heuristic. The report names the detected contexts, the rule groups that
ran, and the excluded paths. "No findings detected by evaluated rules" does not prove
a project is safe.

## Quality gate configuration

The static scan does not compute coverage. Supply an LCOV report with `--coverage` to import it.

<!-- quality-config-example -->
```json
{
  "version": 1,
  "enabled": true,
  "metrics": [
    {
      "id": "rust-function-lines",
      "version": 1,
      "unit": "lines",
      "required": true,
      "comparison": "maximum",
      "threshold": 100,
      "unknown": "unknown",
      "exclusions": ["src/generated"]
    },
    {
      "id": "rust-duplicate-blocks",
      "version": 1,
      "unit": "blocks",
      "required": true,
      "comparison": "maximum",
      "threshold": 20,
      "unknown": "fail",
      "exclusions": []
    },
    {
      "id": "lcov-line-coverage",
      "version": 1,
      "unit": "basis-points",
      "required": false,
      "comparison": "minimum",
      "threshold": 6000,
      "unknown": "unknown",
      "exclusions": []
    },
    {
      "id": "new-findings",
      "version": 1,
      "unit": "findings",
      "required": false,
      "comparison": "maximum",
      "threshold": 0,
      "unknown": "unknown",
      "exclusions": []
    }
  ]
}
```

## verify: run the project's own gates

```bash
snipe-code verify .                        # every declared gate
snipe-code verify ../project --only test    # re-run one gate
snipe-code verify . --security              # add the dependency vulnerability gate
snipe-code verify . --no-install            # skip the install gate
```

Gates come from the project's manifests, not from this tool's stack list, so
`snipe-code verify` works on any project it recognizes:

| Manifest | Gates run |
| --- | --- |
| `package.json` | `npm install`, then `npm run <script>` for the declared `typecheck`, `lint`, `test`, `build` |
| `composer.json` + `artisan` | `composer install --no-interaction --no-progress`, `php artisan test` |
| `composer.json` + `phpunit.xml` | `composer install --no-interaction --no-progress`, `php vendor/bin/phpunit` |
| `go.mod` | `go mod download`, `go mod tidy`, `go vet ./...`, `go test ./...`, `go build ./...` |
| `pom.xml` | `mvn test`, `mvn package -DskipTests` |
| `*.csproj` / `*.sln` | `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |

Gate order is fixed: `install`, `lint`, `typecheck`, `test`, `build`, plus `security`
when requested. Each gate reports `passed`, `failed`, `unavailable`, `blocked`, or
`not-applicable`. A missing tool or blocked action does not count as a pass. A run
with mixed passed and unavailable/blocked gates has aggregate status `incomplete`.

Before each applicable gate, `verify` previews commands, working directories, and
known effects, then asks for approval tied to that gate scope. Without a terminal or
approval, the gate is `blocked`. For install and test gates, detected remote,
production-like, and ambiguous database targets are blocked before execution. If no
target is detected, `verify` says the target is unknown and requires approval; that does
not prove the project avoids a database. Preflight omits database credentials from its
target summary. `--only` limits preview and approval to the selected gate.

Preflight checks recognized DB environment variables and `.env`, `.env.local`,
`.env.test`, `.env.test.local`, `.env.testing`, and `.env.testing.local` in each
gate's directory. It does not inspect every framework configuration or hardcoded target.
Inherited environment values take precedence for the same key; conflicting unoverridden
dotenv values are blocked.
Child output is filtered for recognized DB usernames and passwords from the process
environment and six `.env` variants in the gate directory, including URL userinfo,
percent-decoded forms, uniformly upper- or lowercase percent escapes, and JSON string
escaping. Mixed-case escapes, unknown credential sources,
other encodings, child-created or transformed secrets are not guaranteed to be masked.
Do not place secrets in command arguments or paths; review output before sharing.

Localhost and a test-like database name do not prove isolation from shared data.

Install may download dependencies, write files, and run install hooks. Tests may write
files, access databases, or use the network. Lint, typecheck, and build commands may
write files or use the network; security commands may query vulnerability data. Child
command output can contain secrets, even when the preflight target summary omits them.

`verify-generated` creates and checks a project in its own temporary workspace. It
removes recognized inherited database target variables before starting gate commands.
The workspace is not a sandbox: commands can still access network, hardcoded targets,
and host files. The harness does not accept a caller-supplied project path.

Paths within a template override must be relative. Generation checks for traversal,
symlinks, and Windows reparse points before staging, but concurrent changes to the
override tree can race with these checks.

Exit code:

- `0`: every applicable gate passed
- `1`: a gate failed
- `2`: usage error, for example `--only` naming a gate that does not apply
- `3`: no gate executed or no project gate was found (`tidak ada gate yang dijalankan`)
- `4`: partial result; an applicable gate was unavailable or blocked after another
  gate executed

Use `--json` for a machine-readable report containing per-gate and aggregate status.

To save verification results, choose one format and an explicit destination:

```bash
snipe-code verify . --report json --output verify.json
snipe-code verify . --report junit --output verify.xml
snipe-code verify . --report md --output verify.md
```

These are test and quality-gate reports, not audit security reports. Existing files
are kept unless `--overwrite` is supplied. A successful write keeps the gate's
exit code; a write error exits `3`. File reports and stdout `--json` are separate
options. Project commands still need approval and can write files or use the network.
In CI without approval, applicable gates are blocked rather than marked as passed.

Preflight does not sandbox project commands or prove all their side effects. Approved
commands run with host permissions and may access network or files beyond the project.
They can write `node_modules/` and `package-lock.json` (`npm install`), update `go.sum`
and `go.mod` (`go mod tidy`), and create `target/`, `obj/`, or `bin/`. Nothing is
removed. `--no-install` skips install and reports `not-applicable`; combining it with
`--only install` is a usage error. Use `snipe-code analyze` or `snipe-code scan` for
read-only checks. Legacy `audit` writes reports into the scanned directory.

`--security` uses the scanners each ecosystem ships: `npm audit`, `composer audit`,
`dotnet list package --vulnerable`. Go and Java report this gate as `not-applicable`
with a reason, since `govulncheck` and OWASP dependency-check are not part of their
default toolchains.

A project with more than one part, such as `frontend/` and `backend/`, is verified
part by part in its own directory. An unrecognized project reports `no gates found`.
Known limitation: detection uses `audit::detect_scopes`, which does not recognize
Python or Ruby yet.

## Templates

Each stack and preset body is a real file under `templates/`, embedded into the binary
at build time, so scaffolding does not need to download templates:

```text
templates/
├── frontends/<id>/          # nextjs, react-vite, vue, nuxtjs
│   ├── manifest.txt         # name=..., init=..., install=...
│   ├── folders.txt          # one folder path per line
│   └── files/<path>         # content generated at that same path
├── backends/<id>/           # dotnet8, nestjs, go-gin, spring-boot, laravel
│   └── (manifest.txt, folders.txt, files/<path>)
└── presets/<id>/            # preset.json and content/<template_source>.txt
```

Adding or changing a template needs no Rust edit: change files under `templates/` and
run `cargo build`.

To override locally without recompiling, point `SNIPE_CODE_TEMPLATES` at a directory
laid out like `templates/`:

```bash
SNIPE_CODE_TEMPLATES=/path/to/working-templates snipe-code new
```

Resolution is per file. A file the override directory provides wins over the built-in
default, and files it does not provide keep the default, so a partial override is
valid. Example: only `frontends/nextjs/files/package.json` to bump a dependency
version. The `new` output names the source it used, `Templates: local templates from
<dir>` or `Templates: built-in templates`.

## Global AI skill

The global skill is separate from project-local `new --ai-tools` artifacts. It installs
instructions for a host AI agent; `snipe-code` itself is a deterministic CLI, not a
model or test author. Installation neither starts an agent nor guarantees discovery;
use the agent's skill discovery or invocation mechanism and ask it to use the
`snipe-code` skill. Check the agent's documentation for its exact invocation syntax.
The skill asks the agent to check CLI help for the installed version.

After the agent discovers the skill, invocation depends on the host:

- Claude Code: start a prompt with `/snipe-code` ([skills documentation](https://code.claude.com/docs/en/skills)).
- Codex CLI or IDE extension: mention `$snipe-code`, or select it with `/skills`
  ([skills documentation](https://developers.openai.com/codex/skills)).
- OpenCode: ask the agent to load the `snipe-code` skill through its skill tool
  ([skills documentation](https://opencode.ai/docs/skills/)).
- Hermes: ask the agent to load the `snipe-code` skill in the selected profile;
  discovery depends on that profile's configuration.
- Cursor, GitHub Copilot, Windsurf, Antigravity, and Gemini CLI: ask the agent to
  use the `snipe-code` skill; consult its documentation for discovery and invocation.

These are agent prompts, not `snipe-code` shell commands. Names can conflict with
other skills; verify the selected skill and its location before use.

In an interactive terminal, run `snipe-code` and select **Install global AI skill**,
or run `snipe-code skill`. Choose an agent, review the displayed destination, then
confirm installation; confirmation defaults to no. Without a terminal, `skill` with
no agent prints usage and exits `2`. Direct commands work without prompts:

```bash
snipe-code skill claude             # preview destination only; no writes
snipe-code skill claude --install   # install after reviewing destination
snipe-code help skill
```

Replace `claude` with the agent ID below. Paths show the installed `SKILL.md`;
`<home>` is `USERPROFILE` on Windows or `HOME` elsewhere. The relevant home must be
nonempty and absolute, including for preview.

| Agent ID | Destination |
| --- | --- |
| `claude` | `<home>/.claude/skills/snipe-code/SKILL.md` |
| `opencode` | `<home>/.config/opencode/skills/snipe-code/SKILL.md` |
| `codex` | `<home>/.agents/skills/snipe-code/SKILL.md` |
| `hermes` | `<HERMES_HOME>/skills/snipe-code/SKILL.md` |
| `cursor` | `<home>/.cursor/skills/snipe-code/SKILL.md` |
| `github-copilot` | `<home>/.copilot/skills/snipe-code/SKILL.md` |
| `windsurf` | `<home>/.codeium/windsurf/skills/snipe-code/SKILL.md` |
| `antigravity` | `<home>/.gemini/config/skills/snipe-code/SKILL.md` |
| `gemini-cli` | `<home>/.gemini/skills/snipe-code/SKILL.md` |

For Hermes, set `HERMES_HOME` to the absolute home directory of the intended profile
before preview or install; it does not fall back to `HOME` or `USERPROFILE`. For
example, in PowerShell:

```powershell
$env:HERMES_HOME = 'C:/path/to/hermes/profile'
snipe-code skill hermes
snipe-code skill hermes --install
```

Install creates parent directories and a new `snipe-code/SKILL.md`. An existing
`snipe-code` directory causes failure, even if empty; it is never overwritten. A
failed write can leave a partial file or parent directories: inspect them before retrying.
Invalid arguments exit `2`; home resolution, selection, or installation errors exit `3`.

The skill guides the agent through scaffolding, source tracing, security review,
verification, and test automation. With requested and approved test edits, the AI
agent may author tests using the project's existing framework; `verify` only runs
existing project commands and does not write tests. Test execution requires separate
approval and review of commands, database targets, isolation, and network effects.
`analyze` and default `scan` are read-only; `scan --save` writes history, `new` writes
a project, and `audit` writes reports that may replace existing reports. `verify`
executes project-controlled commands that may change files or access networks or
databases; `--no-install` does not make other gates safe. Instructions are not a
permission boundary, and outcomes depend on the host agent.

## AI agent tools

In interactive `snipe-code new`, at `[6/8] AI integrations`, use Space to select
zero or more tools and Enter to continue. Choose Continue, Back, or Cancel on the
next prompt. Alternatively, use `--ai-tools` with comma-separated registry IDs, or
`--ai-tools none` to select none. Without a terminal or `--ai-tools`, no integrations
are selected.

Selected integrations add files inside the generated project, not to global agent
configuration. Each tool receives different artifact types and paths as listed below;
selecting one does not install the tool. They do not start a model, authenticate a
provider, store credentials, or require an API key. Agent behavior and discovery
depend on the agent; generated instructions are not a runtime permission boundary.
In particular, generating Hermes skill files does not guarantee Hermes will discover or
load them automatically; check the Hermes configuration and discovery rules for your
installation.

| Tool | Rules | Skills | Commands | Workflows |
| --- | --- | --- | --- | --- |
| Cross-tool (AGENTS.md) | `AGENTS.md` | - | - | - |
| Claude Code | `CLAUDE.md` | `.claude/skills/snipe-code/SKILL.md`, `.claude/skills/snipe-qa-planning/SKILL.md` | `.claude/commands/snipe-{audit,verify,test}.md` | - |
| Cursor | `.cursor/rules/snipe-code.mdc`, `.cursorrules` | - | - | - |
| OpenCode | - | `.opencode/skills/snipe-code/SKILL.md`, `.opencode/skills/snipe-qa-planning/SKILL.md` | `.opencode/commands/snipe-{audit,verify,test}.md` | - |
| Antigravity | `GEMINI.md`, `.agents/rules/antigravity.md` | `.agents/skills/snipe-code/SKILL.md`, `.agents/skills/snipe-qa-planning/SKILL.md` | - | `.agents/workflows/snipe-{audit,verify,test}.md` |
| GitHub Copilot | `.github/copilot-instructions.md` | - | - | - |
| Windsurf | `.windsurf/rules/snipe-code.md` | - | - | - |
| Hermes Agent | - | `.hermes/skills/snipe-code/SKILL.md`, `.hermes/skills/snipe-qa-planning/SKILL.md` | - | - |
| Codex | `AGENTS.md` | - | - | - |
| Gemini CLI | `GEMINI.md` | - | - | - |

The three named artifacts guide audit review, verification, and test authoring.
They request separate approval for effects; the agent remains responsible for
following those instructions, and CLI preflight still applies. Generated projects
include example test infrastructure. `verify` runs existing declared tests, not
new tests inferred from source. An agent may author or repair tests from source
and requirements, subject to review and approval.

```bash
snipe-code new --ai-tools cursor,claude-code
snipe-code new --ai-tools none --name demo
snipe-code new --dry-run
```

`.ai-workspace/rules.md` and `.ai-workspace/rules.mdc` are always written.
Selected artifacts are project-local. Paths claimed by multiple integrations, such
as `AGENTS.md`, are written once and overlaps are reported. Registry entries and
artifact formats are in `templates/ai-tools.json`.

## Supported platforms

The target list lives in `.github/release-targets.json`. Both the release matrix and
the release validation read that file, and this table follows it.

| Platform | Target | Archive |
| --- | --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.zip` |
| macOS x86_64 (Intel) | `x86_64-apple-darwin` | `.tar.gz` |
| macOS arm64 (Apple silicon) | `aarch64-apple-darwin` | `.tar.gz` |

Adding a target means editing that one file: the `package` matrix and the archive count
in the `publish` job read the same list, so there is no second edit.

### Checking a release

Every archive contains the executable, `LICENSE`, `README.md`, and `SOURCE_REVISION`,
alongside a checksum and a keyless signature bundle:

```bash
sha256sum --check snipe-code-<version>-<target>.tar.gz.sha256
cosign verify-blob --bundle snipe-code-<version>-<target>.tar.gz.bundle \
  --certificate-identity-regexp 'https://github.com/athallaarl66/Snipe-code/.github/workflows/release.yml@.*' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  snipe-code-<version>-<target>.tar.gz
```

Run those only against artifacts that are actually on the GitHub Releases page. The
same release run publishes the crate, so `cargo install snipe-code --locked` installs
the same version that was released.

## CI

This repository's CI (`.github/workflows/ci.yml`) runs the Rust gate plus a matrix that
exercises generated projects:

| Job | What it proves |
| --- | --- |
| `rust` | `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` |
| `generated-frontend` | Next.js, React Vite, Vue, Nuxt: `npm install`, `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build` |
| `generated-backend` | NestJS, Go Gin, Spring Boot, .NET 8, Laravel 12: the install, lint, test, and build commands each declares |
| `compose` | `docker compose config` for PostgreSQL, MySQL, and the NestJS + PostgreSQL combination |

The local gate:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Documentation

- `docs/README.md`: index of every document in this directory.
- `docs/features/`: PRD and technical design.
- `docs/production/`: scenario breakdowns per user story.
- `docs/uat/`: user acceptance cases and their evidence status.

## Author

Athalla ([@athallaarl66](https://github.com/athallaarl66))

## License

MIT, see [LICENSE](LICENSE).
