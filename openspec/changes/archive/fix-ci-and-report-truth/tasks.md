# Tasks

## 1. Repair the Laravel scaffold (CI-blocking)

- [x] 1.1 In `src/stack.rs`, change the generated `laravel/framework` constraint from `^11.0` to `^12.0` and align the dev requirements with the Laravel 12.x skeleton (`nunomaduro/collision` `^8.6`, `phpunit/phpunit` `^11.5.50`); update `tests/bugfix_test.rs::test_laravel_scaffold_uses_supported_framework_line` so it asserts `^12.0` and the absence of `^11.0`; verify with `cargo test` — done (src/stack.rs:607-613; test passes in `cargo test --all-targets --all-features`)
- [x] 1.2 Verify the generated Laravel project boots: generate a Laravel backend into a scratch directory, run `composer install --no-interaction --no-progress` and `php artisan test` using this machine's PHP 8.4 + Composer toolchain. If `php artisan test` fails, migrate `bootstrap/app.php` to `Application::configure()` and add the `routes/console.php` file that pattern expects, then re-run until the suite passes — done: `cargo run --bin verify-generated -- --backend laravel` exit 0 on PHP 8.3.32 + Composer 2.7.2 (`1 passed, 1 warning`); legacy `bootstrap/app.php` still boots on 12.x, no `Application::configure()` migration needed
- [x] 1.3 Push the branch and confirm the `Backend (laravel)` CI job concludes `success` — this is the acceptance signal for the whole change, proving generate → `composer install` → `php artisan test` end to end — done: run https://github.com/athallaarl66/Snipe-code/actions/runs/38667676102 (main @ 1456d87) has `Backend (laravel) success`

## 2. Audit report date

- [x] 2.1 Move `real_date_yyyymmdd()`, `civil_from_days()`, `day_of_week()`, and `MONTHS` out of `src/generator.rs` into a shared module re-exported from `src/lib.rs`; update the blog sample-post caller; verify with `cargo test` — done (`src/date.rs`, `pub mod date` in `src/lib.rs`; generator calls `date::*`)
- [x] 2.2 Replace `chrono_placeholder()` in `src/audit.rs` with the shared real-date helper at all three call sites (Markdown line 508, PDF line 575, DOCX line 721) and delete the placeholder function; verify the crate still builds with `cargo build` — done (`src/audit.rs:510,577,723` call `date::real_date_yyyymmdd()`; no `chrono_placeholder` left in the tree)
- [x] 2.3 Add a regression test asserting an exported Markdown report's `**Date:**` value equals the shared helper's output and parses as `YYYY-MM-DD`; verify with `cargo test` — done (`tests/bugfix_test.rs::test_markdown_report_uses_real_system_date`, passes)

## 3. AI agent instruction files

- [x] 3.1 Emit `.cursorrules` and `AGENTS.md` at the generated project root from `src/ai_workspace.rs`, and add both to the generated file list so the generation report and dry-run preview include them; update `tests/dry_run_test.rs` expectations if any assert an exact root-file set; verify with a generation test asserting both files exist at the project root — done (`src/ai_workspace.rs:13-14`; `tests/bugfix_test.rs::test_generated_project_contains_advertised_ai_agent_files`; dry-run test asserts both names)
- [x] 3.2 Update the repository README's "AI Workspace" line so it describes the files actually produced; verify with a test asserting the README names `.cursorrules` and `AGENTS.md` — done (README.md:19; `tests/bugfix_test.rs::test_readme_documented_outputs_are_generated`)
- [x] 3.3 Add a regression test asserting every path in the `GenerateResult` file list resolves inside the destination project tree, so generation can never write to a home-directory agent configuration; verify with `cargo test` — done (`tests/bugfix_test.rs::test_generated_files_stay_inside_project_tree`, passes)

## 4. CLI version

- [x] 4.1 Add a `version` command and a `--version` flag arm to `src/main.rs` printing `env!("CARGO_PKG_VERSION")`, and document both in the help text; verify by running the built binary and asserting single-line `major.minor.patch` output with exit status 0 — done: `target/debug/snipe-code.exe version` and `--version` both print `0.1.0`, exit 0 (help documents both)
- [x] 4.2 Add a test in `tests/cli_help_test.rs` asserting the help output documents `version` and `--version`, and that an unrecognized command or option token still exits with status 2; verify with `cargo test` — done (`tests/cli_help_test.rs::test_version_command_and_flag_report_crate_version`, `::test_help_documents_version_and_unknown_tokens_exit_2`; binary exits 2 for an unknown command)

## 5. README to verified reality

- [x] 5.1 Correct the README install section and the "Supported stack verification" table so each row matches the command `src/bin/verify-generated.rs` actually runs for that stack, and update the Laravel row to Laravel 12; keep the existing assertions in `tests/bugfix_test.rs::test_readme_uses_supported_quality_and_commands` passing — done (README.md:118-128 matches `verify-generated.rs:75-98` command-by-command; Laravel row reads `Laravel 12`)
- [x] 5.2 Add a README test asserting no documented feature references output the generator does not produce; verify with `cargo test` — done (`tests/bugfix_test.rs::test_readme_documented_outputs_are_generated`, passes)

## 6. Integration verification

- [x] 6.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0 — done: FMT 0 / CLIPPY 0 / TEST 0. `cargo fmt --check` was failing on the merged tree (4 files); formatted here, so the previous CI `Rust quality` failure is reproduced and fixed.
- [x] 6.2 Confirm the complete CI workflow concludes `success` with all 13 jobs green, and record the run URL as this change's evidence — done: run https://github.com/athallaarl66/Snipe-code/actions/runs/38673027154 (branch `fix/ci-npm11-node24`, sha `e6d278d1`) reports `jobs 13 success 13`; the post-merge `main` run https://github.com/athallaarl66/Snipe-code/actions/runs/38673279223 (sha `1a7bf750`, PR #8 merge) is also 13/13. Both `Frontend (nuxtjs)` and `Rust quality` are green, which was the point of the change.

## Blocker found while verifying 6.2

`Frontend (nuxtjs)` fails on the merged tree. Root cause reproduced locally from the
generated `package.json` alone: `npm install` (npm 10.9.3, shipped with Node 20/22) exits 1 with
`Cannot read properties of null (reading 'edgesOut')` inside `@npmcli/arborist`
`#loadPeerSet` while resolving the Nuxt 3.21 peer graph. `npm install --legacy-peer-deps` and
npm 11 both install the same graph cleanly (604 packages), and `npm run build` then succeeds.
Fix: pin Node 24 (npm 11) in the CI frontend/nestjs jobs and in the workflow the generator
writes (`src/cicd.rs`), plus align the generated Laravel job's PHP with `composer.json`
(`8.1` → `8.2`). Guarded by `tests/bugfix_test.rs::test_workflow_pins_npm11_node_and_supported_php`.

Full `verify-generated` sweep with npm 11 on PATH (real binary, real installs):
`nextjs 0`, `react-vite 0`, `vue 0`, `nuxtjs 0`, `nestjs 0`, `laravel 0`.

Observed, not a defect: `php artisan test` in a generated Laravel project reports
`1 warning, 1 passed` — phpdotenv `file_get_contents(<project>/.env): Failed to open stream`
under PHPUnit's error handler. The generated project intentionally ships only `.env.example`
(`tests/bugfix_test.rs::test_laravel_env_example_has_app_key_placeholder` pins that), the
application still boots and the Feature test asserts 200, and the job exits 0. Do not "fix" it by
generating a real `.env`.

## Workflow follow-up

- Archive the change once review requirements are satisfied, then confirm the synced specs appear under `openspec/specs/`.
