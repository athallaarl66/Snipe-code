# Tasks

## 1. Repair the Laravel scaffold (CI-blocking)

- [ ] 1.1 In `src/stack.rs`, change the generated `laravel/framework` constraint from `^11.0` to `^12.0` and align the dev requirements with the Laravel 12.x skeleton (`nunomaduro/collision` `^8.6`, `phpunit/phpunit` `^11.5.50`); update `tests/bugfix_test.rs::test_laravel_scaffold_uses_supported_framework_line` so it asserts `^12.0` and the absence of `^11.0`; verify with `cargo test`
- [ ] 1.2 Verify the generated Laravel project boots: generate a Laravel backend into a scratch directory, run `composer install --no-interaction --no-progress` and `php artisan test` using this machine's PHP 8.4 + Composer toolchain. If `php artisan test` fails, migrate `bootstrap/app.php` to `Application::configure()` and add the `routes/console.php` file that pattern expects, then re-run until the suite passes
- [ ] 1.3 Push the branch and confirm the `Backend (laravel)` CI job concludes `success` — this is the acceptance signal for the whole change, proving generate → `composer install` → `php artisan test` end to end

## 2. Audit report date

- [ ] 2.1 Move `real_date_yyyymmdd()`, `civil_from_days()`, `day_of_week()`, and `MONTHS` out of `src/generator.rs` into a shared module re-exported from `src/lib.rs`; update the blog sample-post caller; verify with `cargo test`
- [ ] 2.2 Replace `chrono_placeholder()` in `src/audit.rs` with the shared real-date helper at all three call sites (Markdown line 508, PDF line 575, DOCX line 721) and delete the placeholder function; verify the crate still builds with `cargo build`
- [ ] 2.3 Add a regression test asserting an exported Markdown report's `**Date:**` value equals the shared helper's output and parses as `YYYY-MM-DD`; verify with `cargo test`

## 3. AI agent instruction files

- [ ] 3.1 Emit `.cursorrules` and `AGENTS.md` at the generated project root from `src/ai_workspace.rs`, and add both to the generated file list so the generation report and dry-run preview include them; update `tests/dry_run_test.rs` expectations if any assert an exact root-file set; verify with a generation test asserting both files exist at the project root
- [ ] 3.2 Update the repository README's "AI Workspace" line so it describes the files actually produced; verify with a test asserting the README names `.cursorrules` and `AGENTS.md`
- [ ] 3.3 Add a regression test asserting every path in the `GenerateResult` file list resolves inside the destination project tree, so generation can never write to a home-directory agent configuration; verify with `cargo test`

## 4. CLI version

- [ ] 4.1 Add a `version` command and a `--version` flag arm to `src/main.rs` printing `env!("CARGO_PKG_VERSION")`, and document both in the help text; verify by running the built binary and asserting single-line `major.minor.patch` output with exit status 0
- [ ] 4.2 Add a test in `tests/cli_help_test.rs` asserting the help output documents `version` and `--version`, and that an unrecognized command or option token still exits with status 2; verify with `cargo test`

## 5. README to verified reality

- [ ] 5.1 Correct the README install section and the "Supported stack verification" table so each row matches the command `src/bin/verify-generated.rs` actually runs for that stack, and update the Laravel row to Laravel 12; keep the existing assertions in `tests/bugfix_test.rs::test_readme_uses_supported_quality_and_commands` passing
- [ ] 5.2 Add a README test asserting no documented feature references output the generator does not produce; verify with `cargo test`

## 6. Integration verification

- [ ] 6.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [ ] 6.2 Confirm the complete CI workflow concludes `success` with all 13 jobs green, and record the run URL as this change's evidence

## Workflow follow-up

- Archive the change once review requirements are satisfied, then confirm the synced specs appear under `openspec/specs/`.
