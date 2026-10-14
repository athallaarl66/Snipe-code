# Tasks

## 1. Extract the verification engine (behaviour-preserving)

- [ ] 1.1 Create `src/verify/` and move the per-stack command knowledge and the command runner out of `src/bin/verify-generated.rs`, including the `cmd.exe` wrapper for `.cmd` shims, the temp-workspace lifecycle, and the unique-suffix helper; expose it as a library module from `src/lib.rs`; verify `cargo build` succeeds
- [ ] 1.2 Reduce `src/bin/verify-generated.rs` to a thin caller that generates a project and invokes the engine, keeping its `--frontend` / `--backend` / `--database` interface and its error message text; verify with `cargo test` and confirm the binary still exits non-zero when a stack is unknown
- [ ] 1.3 Confirm the CI workflow is untouched and its per-stack generated-project jobs remain green after the extraction — this is the evidence the refactor preserved behaviour

## 2. Manifest-based gate detection

- [ ] 2.1 Implement gate detection that reads project manifests (package manifest scripts, PHPUnit configuration, Go module, Maven and .NET project files) and derives the applicable gates, reusing `audit::detect_scopes` for recognition; verify with a test per ecosystem asserting the derived gate set for a fixture project
- [ ] 2.2 Implement the four gate states (`passed`, `failed`, `not-applicable`, `unavailable`) and the fixed gate order (install, lint, typecheck, test, build); verify with tests asserting a project that declares no test script reports the test gate as `not-applicable` and runs no test command
- [ ] 2.3 Implement detection for a project with more than one recognized part, running each part's gates against that part's own directory; verify with a fixture containing both a frontend and a backend
- [ ] 2.4 Handle an unrecognized directory and a missing toolchain: report no gates found, list unavailable gates, and confirm a run where nothing executed exits `3`; verify with `cargo test`

## 3. The verify command

- [ ] 3.1 Route `verify [PATH]` in `src/main.rs`, print each command before running it, and print the per-gate summary with an overall result; verify by running the binary against a fixture project and checking the summary lists gates and outcomes
- [ ] 3.2 Implement the exit status contract — `0` all applicable gates passed, `1` a gate failed naming the gate and command, `2` usage error, `3` nothing executed — and cover each status with a test
- [ ] 3.3 Implement `--only <gate>` to restrict the run, rejecting a gate that does not apply with a usage error; verify with tests for both the restricted run and the rejected selector
- [ ] 3.4 Add `verify` to the help text and per-command help in `src/main.rs`; verify with a test asserting the help output documents the command and its options

## 4. Security gate

- [ ] 4.1 Add the `--security` gate for ecosystems whose toolchain ships a scanner (`npm audit`, `composer audit`, `dotnet list package --vulnerable`); verify with a test asserting the scanner command is derived for each of those ecosystems
- [ ] 4.2 Report the security gate as `not-applicable` with a stated reason for ecosystems whose scanner is not bundled (Go, Java); verify with tests for both ecosystems
- [ ] 4.3 Confirm a failing scan produces a failed gate and exit status 1, and that the gate is absent from the report when `--security` is not requested; verify with `cargo test`

## 5. Documentation

- [ ] 5.1 Document `verify` in the README: usage, the gate states, the exit status contract, and the explicit split between `verify` (spawns the project's own tools, which may use the network) and `audit` (fully offline, read-only); verify by following the documented examples against a fixture project
- [ ] 5.2 Add a test asserting the README documents the `verify` command and its exit status contract; verify with `cargo test`

## 6. Integration verification

- [ ] 6.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [ ] 6.2 Run `verify` manually against one project per recognized ecosystem and confirm the reported gates match what each project actually declares
- [ ] 6.3 Confirm CI is green with the harness delegating to the shared engine, and record the run URL

## Workflow follow-up

- Archive the change once review requirements are satisfied, then confirm `openspec/specs/project-verification/` exists with the synced requirements.
