## Why

Template registry sudah ada (US-001), tapi belum ada cara untuk generate struktur folder berdasarkan stack selection. US-002 (Init Tanpa Template) butuh stack definitions dan generator yang bisa bikin clean architecture folders tanpa template-specific files.

## What Changes

- Tambah `src/stack.rs` — stack definitions (frontend, backend, database options) beserta folder structures masing-masing
- Tambah `src/generator.rs` — engine yang create folders + security config files berdasarkan template + stack selection
- Update `src/main.rs` — wire stack selector + generator ke template registry

## Capabilities

### New Capabilities
- `stack-selector`: Definisi stack options (Next.js, React/Vite, Vue, Nuxt.js, .NET 8, NestJS, Go, Spring Boot, Laravel, PostgreSQL, MySQL) beserta folder structure masing-masing
- `structure-generator`: Engine yang generate folders + files berdasarkan template rules + stack selection

### Modified Capabilities
_(none)_

## Impact

- **Code**: `src/stack.rs` (baru), `src/generator.rs` (baru), `src/lib.rs` (update), `src/main.rs` (update)
- **Dependencies**: `std::fs` (folder creation)
- **Testing**: Integration tests di `tests/`
