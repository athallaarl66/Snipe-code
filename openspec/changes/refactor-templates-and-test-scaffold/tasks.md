# Tasks

## 1. Extract template content to files (behaviour-preserving)

- [ ] 1.1 Add the template-embedding build dependency, create `templates/`, and move the frontend stack content out of `src/stack.rs` into `templates/frontends/<id>/` (folders, files, init/install commands); verify with `cargo test` — `tests/frontend_contracts_test.rs` asserts the generated output and must pass **unchanged**
- [ ] 1.2 Move backend stack content into `templates/backends/<id>/` and preset content into `templates/presets/<id>/`, including the portfolio `projects.json`, the blog sample post, and the RSS feed source; verify with `cargo test` — `tests/blog_test.rs`, `tests/portfolio_test.rs`, and `tests/company_profile_test.rs` must pass unchanged
- [ ] 1.3 Remove the now-unused literal definitions from `src/stack.rs` so it holds only the registry and loader; verify `cargo build` succeeds and `wc -l src/stack.rs` shows the file is a fraction of its previous 1449 lines
- [ ] 1.4 Change the public accessors to return owned `Vec<FileDef>` resolved from the embedded tree, update `src/generator.rs` and `src/commands/new.rs`, and adapt the assertions in `tests/stack_generator_test.rs` and `tests/bugfix_test.rs` that reference the changed types; verify `cargo test --all-targets --all-features` passes

## 2. Template override

- [ ] 2.1 Implement per-file resolution: when `SNIPE_CODE_TEMPLATES` names a readable directory laid out like `templates/`, a file it provides replaces the embedded default for the same relative path, and absent files fall back to the default; verify with a test that points the variable at a temp directory containing one overridden file and asserts the generated project uses the override. Guard the environment mutation with the process-wide test mutex in `tests/common/mod.rs` — environment variables are process-global and `set_var` is `unsafe` under edition 2024
- [ ] 2.2 Add a partial-override test asserting that files the override directory does not provide still come from the built-in defaults; verify with `cargo test`
- [ ] 2.3 Report whether built-in or local template content was used in the generation output; verify with a test asserting the reported source differs between the two cases

## 3. Frontend test infrastructure

- [ ] 3.1 Next.js: add `vitest`, a `vitest.config.ts`, one example test asserting rendered output, and `test` / `typecheck` scripts; verify with a test asserting the generated manifest declares both scripts and that the example test file exists
- [ ] 3.2 React (Vite) and Vue: same additions with the stack-appropriate renderer, using `vue-tsc` for the Vue type-check script; verify with a test asserting both scripts are declared for each stack
- [ ] 3.3 Nuxt.js: add `vitest` with the Nuxt test utilities, an example test, and `test` / `typecheck` scripts; verify with a test asserting both scripts are declared
- [ ] 3.4 Extend `tests/frontend_contracts_test.rs` so every frontend must additionally declare a `test` script; verify with `cargo test`

## 4. Backend test infrastructure

- [ ] 4.1 NestJS: add `jest` + `ts-jest` + `@nestjs/testing` + `supertest`, one controller spec, and `test` / `typecheck` scripts; verify with a test asserting the spec file and both scripts exist
- [ ] 4.2 Go (Gin): extract a constructible router from the generated `main.go` and add `main_test.go` that drives it through `httptest`; verify by generating the stack and running `go test ./...` in the generated backend
- [ ] 4.3 .NET 8: add an xunit test project referencing the generated web project, with one test that constructs and exercises the app; verify by generating the stack and running `dotnet test`
- [ ] 4.4 Spring Boot: replace the placeholder `ApplicationTests` with a `@SpringBootTest` + `MockMvc` test exercising a generated endpoint; verify by generating the stack and running `mvn test`
- [ ] 4.5 Laravel: confirm the existing `phpunit.xml` and feature test pass against the Laravel 12 constraint from `fix-ci-and-report-truth`; verify by generating the stack and running `php artisan test`

## 5. Lint and generated CI steps

- [ ] 5.1 Add an ESLint flat config (`eslint.config.js`) plus a `lint` script to the five manifest-based JavaScript/TypeScript stacks, and a `go vet ./...` lint step for Go; verify each generated `lint` command exits 0 against a freshly generated project
- [ ] 5.2 Extend `src/cicd.rs` to emit test and lint steps per stack, gated on the corresponding script being declared, with the test step ordered before build; verify with a test asserting a frontend workflow contains the test step and that a backend with no test script gets no test step
- [ ] 5.3 Update `tests/bugfix_test.rs::test_workflow_uses_npm_install_not_ci`, which currently asserts the workflow never contains `npm run lint`, so it asserts the replacement invariant — lint runs because the generated frontend now declares it; verify with `cargo test`

## 6. Template documentation

- [ ] 6.1 Document the `templates/` layout, the `SNIPE_CODE_TEMPLATES` override directory with its per-file precedence, and how to add or change a template file without touching Rust; verify by following the documented steps and confirming an edited template file appears in a generated project
- [ ] 6.2 Add a test asserting the README documents the override variable name; verify with `cargo test`

## 7. Integration verification

- [ ] 7.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [ ] 7.2 Confirm CI is green across every generated-project job, proving each stack's install, build, and test commands succeed in a clean environment, and record the run URL

## Workflow follow-up

- Confirm `fix-ci-and-report-truth` is already archived before archiving this change — both write `openspec/specs/project-scaffolding/spec.md`.
- Archive, then confirm the new requirements are appended under `openspec/specs/project-scaffolding/` and that `openspec/specs/template-sources/` is created.
