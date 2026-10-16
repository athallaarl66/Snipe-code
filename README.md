# SNIPE-CODE CLI

[![CI](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml/badge.svg)](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust CLI that scaffolds projects, reads an existing codebase, and audits it for
security risks. Everything runs locally: no account, no API key, no AI service.
README in Indonesian: [README.id.md](README.id.md).

## Install

### Shell installer (no Rust toolchain)

Linux and macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.sh | less
bash install.sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.ps1 | more
.\install.ps1
```

The installer downloads the release archive to a temporary directory, checks the
published SHA-256 checksum, checks the cosign bundle when `cosign` is installed, and
only then moves the binary into place. Nothing is piped into a shell. If the download
fails or a check does not pass, it exits non-zero, says why, and leaves no partial
binary behind. Default destination is `~/.local/bin` on POSIX and
`%LOCALAPPDATA%\Programs\snipe-code` on Windows; both print a PATH hint. Override with
`SNIPE_INSTALL_VERSION`, `SNIPE_INSTALL_DIR`, and `SNIPE_RELEASE_BASE_URL`.

### cargo install (with a Rust toolchain)

```bash
cargo install snipe-code
```

Installs the released crate from crates.io into `~/.cargo/bin`. It needs a Rust
toolchain, so it is not the no-toolchain path above.

### Container image (Linux)

```bash
docker run --rm -v "$PWD:/work" -w /work ghcr.io/athallaarl66/snipe-code:latest audit . --export md
```

The image holds the released Linux binary on a distroless base, so there is no shell,
no package manager, and nothing else in it. Its tags follow the release tags, and
`latest` follows the newest release. Mount the project you want inspected; the
container sees only what you mount.

### Build from source

```bash
git clone https://github.com/athallaarl66/Snipe-code.git
cd Snipe-code
cargo build --release
```

The binary lands in `target/release/snipe-code` (`.exe` on Windows) and is portable,
so you can copy it anywhere. `target/` can reach hundreds of megabytes; once the
binary is copied out, `cargo clean` reclaims it.

### Uninstall

Delete the binary the installer placed, or run `cargo uninstall snipe-code` if you
installed through cargo. No registry keys or system files are touched.

## Commands

| Command | What it does |
| --- | --- |
| `snipe-code` or `snipe-code new` | Wizard: template, stacks, database, project name, AI tools, generate |
| `snipe-code analyze [PATH]` | Read-only report: languages and size, structure, entry points, dependencies, limits |
| `snipe-code audit [PATH]` | Heuristic security audit with Markdown/PDF/DOCX export |
| `snipe-code verify [PATH]` | Runs the quality gates the project declares |
| `snipe-code version` | The compiled version (alias `--version`) |
| `snipe-code help [command]` | General help and per-command help |
| `snipe-code faq` | Install, uninstall, troubleshooting |

## new: scaffold a project

```bash
snipe-code                                    # wizard
snipe-code --dry-run                          # preview the structure, write nothing
snipe-code new --ai-tools cursor,claude-code  # pick AI artifacts without prompting
snipe-code new --ai-tools none --name demo --json
```

Wizard order: template, frontend, backend, database, project name, collision prompt
(Overwrite or Cancel), generate, CI/CD, Docker, AI workspace, then `git init` and an
initial commit.

| Flag | Effect |
| --- | --- |
| `--dry-run` | Print the structure without creating a file |
| `--ai-tools <ids\|none>` | AI agent artifacts to generate, comma separated; `none` skips the prompt and generates none |
| `--name <n>` | Skip the project-name prompt |
| `--json` | Generate result (`project`, `ai_tools`, `ai_artifacts`, `ai_overlaps`, `files`, `outcome`) as JSON on stdout, progress on stderr |

With no terminal (a script or CI) the wizard uses defaults and never prompts, so pass
`--name` to make the project name explicit.

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
  with their file count, instead of being dropped quietly.

Exit status: `0` report printed, `2` usage error, `3` the path is missing or is not a
directory. The `--json` output carries the same findings as the text report.

## audit: security scan

```bash
snipe-code audit .                        # prompt for export format
snipe-code audit . --export md            # Markdown only
snipe-code audit . --export pdf,docx      # PDF and DOCX
snipe-code audit . --export md --json     # report file plus JSON on stdout
```

Valid formats are `md`, `pdf`, and `docx`. Duplicates are normalized; an empty or
unknown token is a usage error. With `--json` the report files are still written and
the "exported files" summary moves to stderr, which keeps stdout parseable.

Exit code:

- `0`: audit finished, no HIGH finding
- `1`: a HIGH finding was found
- `2`: usage error, for example an invalid export format
- `3`: runtime or export error, for example a missing path

The audit is heuristic. The report names the detected contexts, the rule groups that
ran, and the excluded paths. "No findings detected by evaluated rules" does not prove
a project is safe.

## verify: run the project's own gates

```bash
snipe-code verify .                        # every declared gate
snipe-code verify ../project --only test    # re-run one gate
snipe-code verify . --security              # add the dependency vulnerability gate
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
when requested. Each gate reports one of four statuses:

- `passed`: the command exited 0
- `failed`: the command exited non-zero, and the gate name and command are printed
- `not-applicable`: the project declares nothing for that gate
- `unavailable`: the tool is not installed here, and the remaining gates still run

Exit code:

- `0`: every applicable gate passed
- `1`: a gate failed
- `2`: usage error, for example `--only` naming a gate that does not apply
- `3`: nothing ran, because the project was not recognized or no tool was available.
  Such a run prints `tidak ada gate yang dijalankan` and exits without running a
  single command.

`verify` runs the project's own tools (`npm`, `composer`, `mvn`, `dotnet`, `go`,
`cargo`), so it can use the network, exactly like that project's CI. `snipe-code`
itself never touches the network; it only spawns commands the project already
declares, and prints each command before running it. For a fully offline read-only
analysis, use `snipe-code analyze` or `snipe-code audit`.

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
at build time, so the tool stays offline and stays a single executable:

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

## AI agent tools

A generated project can carry instruction artifacts for the AI tools you use, each at
the path that tool reads. The files fall into three groups, and how each group reaches
the agent differs.

| Job | Files | When it runs |
| --- | --- | --- |
| Rules | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/snipe-code.mdc`, `.windsurf/rules/snipe-code.md`, `.github/copilot-instructions.md`, `.agents/rules/antigravity.md` | the tool reads the file at the start of a session, nothing to type |
| Skill | `.claude/skills/snipe-code/SKILL.md`, `.opencode/skills/snipe-code/SKILL.md`, `.hermes/skills/snipe-code/SKILL.md`, `.agents/skills/snipe-code.md` | it shows up in the tool's skill list, and the agent loads it when its description matches the work |
| Command | `.claude/commands/snipe-code.md`, `.opencode/commands/snipe-code.md`, `.agents/workflows/snipe-code.md` | you type `/snipe-code` in the tool |

### What the rules and the skill tell the agent

Both carry the same guidance: read `.ai-workspace/rules.md` before editing code, keep the
change scoped to the request, run the project's build and test command before claiming
the change works, and re-run `snipe-code audit . --export md` when the change touches
input handling, secrets, or queries. The body is identical for every tool; only the
frontmatter differs, because each format declares its metadata its own way. A `skill`
gets `name` and `description`, an `mdc` rule gets `globs` and `alwaysApply`, a `command`
gets `description`.

The `/snipe-code` command is that audit run, started by hand: it runs
`snipe-code audit . --export md`, fixes every HIGH finding, runs the project's test
command, then re-runs the audit. It is the only command a generated project gets, and it
takes no arguments: `analyze`, `verify`, and `new` have no command counterpart.

`snipe-code` needs to be on `PATH` for the audit step in the skill and in the command to
work.

### Which tool gets what

| Tool | Files | How you use it |
| --- | --- | --- |
| Cross-tool (AGENTS.md) | `AGENTS.md` | read by any agent that follows the AGENTS.md convention |
| Claude Code | `CLAUDE.md`, `.claude/skills/snipe-code/SKILL.md`, `.claude/commands/snipe-code.md` | the skill loads itself; `/snipe-code` starts the audit |
| Cursor | `.cursor/rules/snipe-code.mdc`, `.cursorrules` | attached to every edit, because the rule sets `alwaysApply: true` |
| OpenCode | `.opencode/skills/snipe-code/SKILL.md`, `.opencode/commands/snipe-code.md` | the skill loads itself; `/snipe-code` starts the audit |
| Antigravity | `GEMINI.md`, `.agents/rules/antigravity.md`, `.agents/skills/snipe-code.md`, `.agents/workflows/snipe-code.md` | the rule and the skill load themselves; the workflow starts the audit |
| GitHub Copilot | `.github/copilot-instructions.md` | read as repository instructions |
| Windsurf | `.windsurf/rules/snipe-code.md` | read as a project rule |
| Hermes Agent | `.hermes/skills/snipe-code/SKILL.md` | the skill loads itself |
| Codex | `AGENTS.md` | same file the cross-tool entry writes |
| Gemini CLI | `GEMINI.md` | same file the Antigravity entry writes |

Codex and Gemini CLI point at files another entry already writes. Selecting both writes
the file once and reports the overlap, so the output reads `AGENTS.md: kept agents-md,
skipped codex`.

### Generating them

```bash
snipe-code                                     # wizard step 6, multi-select, optional
snipe-code new --ai-tools cursor,claude-code   # skip the selection prompt
snipe-code new --ai-tools none --name demo     # no agent artifacts at all
snipe-code new --dry-run                       # print the structure, write nothing
```

Every path is relative to the project root, and no agent configuration lands in your home
directory.

`.ai-workspace/rules.md` and `.ai-workspace/rules.mdc` are always written, whether or not
an agent tool was selected. They carry the guidance the per-tool artifacts render from,
so a tool with no dedicated convention still finds the rules in one known place.

The tool list, paths, and formats live in `templates/ai-tools.json` (formats:
`markdown`, `mdc`, `skill`, `command`). Adding a tool is a registry entry, not a code
change. An unknown format produces an error instead of being dropped.

- `--ai-tools <ids|none>` skips the selection prompt; `none` means no agent artifacts.
- `--json` puts the generate result on stdout and progress on stderr.
- A path claimed by more than one tool, such as `AGENTS.md` by the cross-tool entry and
  Codex, is written once and the overlap is reported.

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
same release run publishes the crate, so `cargo install snipe-code` installs the same
version that was released.

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

- `docs/features/`: PRD and technical design.
- `docs/production/`: task breakdowns per feature.

## Author

Athalla ([@athallaarl66](https://github.com/athallaarl66))

## License

MIT, see [LICENSE](LICENSE).
