# CLI command contract: where each command runs and what it writes

## Why

The CLI documents what each command does but not where it must run, what it writes, or how its exit codes map to outcomes, so correct builds still look broken. A release binary in `target/release/` is not on `PATH`; `new` writes into the current directory regardless of arguments; `audit` drops `Audit_Report.*` inside the scanned project; `verify` mutates the tree it checks because its gates are the project's own install commands. Those are documented behavior gaps, and one of them is expensive: verifying this repository, whose sources contain a dozen template projects, triggers an `npm install` in each of them and leaves tens of thousands of untracked files.

## What Changes

- Add `--no-install` to `snipe-code verify`. The install gate is skipped, reported as `not-applicable` with the reason `skipped by --no-install`, and every other applicable gate still runs. Combining `--no-install` with `--only install` is a usage error (exit `2`).
- Document the run contract of every command in both READMEs: the directory it must be run from, what it reads, what it writes, and its exit codes.
- State the `verify` mutation instead of only warning about network use: the gates write `node_modules/`, `package-lock.json`, `go.sum`, a tidied `go.mod`, `target/`, `obj/`, and `bin/` inside the target project, and nothing is deleted.
- State that `audit` writes `Audit_Report.md`, `Audit_Report.pdf`, and `Audit_Report.docx` into the scanned directory, and that without a terminal it exports Markdown without prompting.
- Extend the `.gitignore` of a generated project with `bin/`, `obj/`, and the three `Audit_Report.*` names, so build output and audit reports are not committed by accident.
- Document the non-interactive wizard defaults by value (template none, frontend Next.js, backend none, database none, project `my-project`) instead of describing them as "defaults".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `cli-process-semantics`: `verify` gains an option that skips the install gate, with its own usage error and its own exit-code outcome when nothing is left to run.
- `product-documentation-accuracy`: command documentation must state where a command runs and what it writes, mutation must be disclosed, and the documented cargo install command must work as written.
- `generated-project-contracts`: the generated `.gitignore` must cover the artificial artifacts this tool's own commands create inside a generated project.

## Impact

- Code: `src/verify/mod.rs` (`Options::skip_install`, `SKIP_INSTALL_REASON`, gate rewriting in `run`), `src/main.rs` (`HELP_TEXT`, `VERIFY_HELP`, `FAQ_TEXT`), `src/generator.rs` (`gitignore_content`), `src/bin/verify-generated.rs` (struct literal).
- Tests: `tests/verify_test.rs`, `tests/verify_cli_test.rs`, `tests/bugfix_test.rs`.
- Docs: `README.md`, `README.id.md`, `docs/features/technical.md`.
- No breaking change: `verify` without the flag behaves exactly as before, and no existing exit code changes meaning.
