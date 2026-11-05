# Tasks

## 1. Status contract

- [ ] 1.1 Adapt tests pinned to current `exit_status` behavior; run `cargo test --test verify_test` to prove any applicable unavailable gate yields unknown/non-pass, even when another gate passes.
- [ ] 1.2 Add `blocked` per-gate outcome and explicit aggregate status; verify JSON/text report tests distinguish passed, unavailable, blocked, failed, and not-applicable.
- [ ] 1.3 Update CLI help, README, and verify contract tests for changed status/exit codes; verify documented codes match actual CLI integration tests.

## 2. Preflight and side effects

- [ ] 2.1 Inventory gate commands by write/network/install effects and add preflight decisions; verify runner-fake tests assert blocked commands never spawn.
- [ ] 2.2 Add redacted DB target classification for known production/remote/ambiguous settings; verify fixtures cover effective config precedence without printing credential values.
- [ ] 2.3 Add negative tests for unknown DB target and deceptive database name containing `test`; verify both block before test/install command invocation.
- [ ] 2.4 Document per-gate filesystem/network effects and limits of preflight; verify examples and exit status against generated help/tests.
- [ ] 2.5 Require scope-bound preview and explicit user approval before install, test, endpoint access, or data transmission; use fake runners to verify missing, stale, or agent-only approval blocks each action.

## 3. Compatibility

- [ ] 3.1 Add CLI regression cases for all existing verify options and new blocked/partial outcomes; run `cargo test --test verify_cli_test`.
- [ ] 3.2 Run `cargo test --test verify_test --test verify_cli_test`; verify unavailable never contributes to complete-pass and blocked target yields no child command.

## Workflow follow-up

- Arsipkan `harden-verify-safety` sesudah review exit-code dan compatibility contract.
