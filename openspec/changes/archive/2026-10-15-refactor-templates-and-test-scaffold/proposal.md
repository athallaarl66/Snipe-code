# Proposal

## Why

`src/stack.rs` is 1449 lines of embedded string literals: the `package.json`,
`tsconfig.json`, Laravel `config/*.php`, and Go source for nine stacks all live as
Rust raw-string literals. Adding template content means editing Rust, and a
contributor who only wants to correct a `tsconfig` has to work inside the crate.
The same file is about to grow much larger: generated projects currently ship
almost no test infrastructure — 7 of 9 stacks generate no test file at all, and no
stack declares a lint or type-check script — so making the scaffolder emit
runnable tests would roughly double the literal count in a single file.

This change separates content from engine, then uses that separation to ship the
test infrastructure that generated CI already implies.

## What Changes

- **Template content moves out of Rust into real files.** Stack, template, and
  preset content lives under `templates/` and is embedded at compile time, so the
  binary stays a single self-contained artifact and stays offline.
  `src/stack.rs` becomes the registry and loader instead of the content.
- **A local template directory overrides the built-in defaults.** Files a local
  template directory provides win over the embedded version; everything else
  falls back to the default. Teams can replace a template without recompiling.
- **Generated projects ship runnable test infrastructure.** Each stack gains a
  test entry point, at least one example test that passes and exercises generated
  code, and the lint and type-check commands its toolchain supports.
- **Generated CI runs the new steps and invokes only declared scripts.** The
  generated workflow gains test and lint steps, and never invokes a
  package-manager script the generated manifest does not declare.
- **README documents the template layout and the override path.**

## Capabilities

### New Capabilities

- `template-sources`: where stack, template, and preset content is resolved from,
  including local override precedence over the built-in defaults.

### Modified Capabilities

- `project-scaffolding`: generated projects additionally receive test
  infrastructure — a test entry point, an example test, and lint / type-check
  commands where the toolchain provides them. The delta adds requirements and
  leaves the requirements introduced by `fix-ci-and-report-truth` untouched.
  **Apply and archive this change after `fix-ci-and-report-truth`**, because both
  write the same capability file `openspec/specs/project-scaffolding/spec.md`.

## Impact

- `src/stack.rs` — reduced to registry and loader. Public accessors change from
  returning `&'static [FileDef]` slices to owned `Vec<FileDef>` whose content is
  resolved from embedded or overridden files.
- `templates/` — new tree holding every stack, template, and preset file.
- `src/generator.rs`, `src/commands/new.rs` — adapt to the new accessor
  signatures; new template files flow through generation and dry-run unchanged.
- `src/cicd.rs` — per-stack test and lint steps, emitted only when the
  corresponding script is declared.
- `Cargo.toml` — one new build-time dependency used to embed the template tree.
  No runtime or network dependency is added; the tool remains fully offline.
- `tests/stack_generator_test.rs` — stack counts and folder assertions adapt to
  the new accessor types.
- `tests/bugfix_test.rs::test_workflow_uses_npm_install_not_ci` — currently
  asserts the generated workflow never contains `npm run lint`; the workflow now
  does run lint, because generated frontends now declare it.
- `tests/frontend_contracts_test.rs` — extended to assert every frontend declares
  a test script.
- `README.md` — template layout and override documentation.

Out of scope: the `verify` command and the security tier (separate changes);
changing which stacks or templates are supported; generating end-to-end,
performance, or accessibility test scaffolding.
