## 1. Stack Definitions

- [x] 1.1 Create `src/stack.rs` with `StackOption` struct (id, name, folders)
- [x] 1.2 Define `FRONTEND_STACKS` constant with 5 options (Next.js, React/Vite, Vue, Nuxt.js, None)
- [x] 1.3 Define `BACKEND_STACKS` constant with 6 options (.NET 8, NestJS, Go, Spring Boot, Laravel, None)
- [x] 1.4 Define `DATABASE_STACKS` constant with 3 options (PostgreSQL, MySQL, None)
- [x] 1.5 Add `get_frontend_folders(id)` function
- [x] 1.6 Add `get_backend_folders(id)` function
- [x] 1.7 Add `validate_selection(frontend, backend)` function

## 2. Structure Generator

- [x] 2.1 Create `src/generator.rs` with `GenerateConfig` struct
- [x] 2.2 Implement `generate_folders()` — create all folders from stack + template
- [x] 2.3 Implement `generate_env_example()` — create `.env.example`
- [x] 2.4 Implement `generate_gitignore()` — create `.gitignore`
- [x] 2.5 Implement `generate_readme()` — create `README.md`
- [x] 2.6 Implement `generate()` — orchestrate all generation steps

## 3. Wire Modules

- [x] 3.1 Add `pub mod stack;` and `pub mod generator;` to `src/lib.rs`
- [x] 3.2 Update `src/main.rs` to use stack + generator

## 4. Integration Tests

- [x] 4.1 Test `get_frontend_folders("nextjs")` returns correct folders
- [x] 4.2 Test `get_backend_folders("nestjs")` returns correct folders
- [x] 4.3 Test `validate_selection("none", "none")` fails
- [x] 4.4 Test `validate_selection("nextjs", "none")` passes
- [x] 4.5 Test `generate()` creates expected folders
- [x] 4.6 Test `generate()` creates `.env.example` and `.gitignore`
- [x] 4.7 Test `generate()` with frontend only (no backend folders)
- [x] 4.8 Test `generate()` with backend only (no frontend folders)

## 5. Verify

- [x] 5.1 `cargo build` succeeds
- [x] 5.2 `cargo test` passes all tests
