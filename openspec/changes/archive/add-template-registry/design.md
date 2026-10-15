## Context

SNIPE-CODE CLI adalah project baru (Rust). Belum ada kode — baru `cargo init` dengan `Hello, world!`. Template registry adalah module pertama yang dibutuhkan sebagai foundation untuk `snipe-code new` command.

Dari TDD, architecture-nya:
```
main → commands::new → template::registry + stack::selector + generator::engine + git::automator
```

Template registry berdiri sendiri, gak depends ke module lain. Cocok jadi starting point.

## Goals / Non-Goals

**Goals:**
- Definisikan struct `Template`, `TemplateStructure`, `FolderDef`, `FileDef`, `GenerationRules`
- Buat `TemplateRegistry` yang bisa load semua template definitions
- Embed template JSON di compile-time (zero runtime I/O)
- Semua 4 template (none, company-profile, portfolio, blog) ter-load dengan benar

**Non-Goals:**
- Stack selection logic (nanti di `stack.rs`)
- Folder/file generation (nanti di `generator.rs`)
- CLI prompts / interactive flow (nanti di `commands/new.rs`)
- Git automation (nanti di `git.rs`)

## Decisions

### D1: Embed at compile-time vs runtime JSON

**Pilihan**: `include_str!()` + `serde_json::from_str()` at compile-time

**Alternatives considered**:
- Runtime JSON files di `~/.snipe-code/templates/` — lebih fleksibel tapi tambah I/O, path resolution complexity
- Hardcoded Rust structs — lebih cepat tapi gak bisa di-update tanpa recompile

**Rationale**: Compile-time embedding = zero runtime cost, zero file system dependency. Template berubah jarang, recompile acceptable. Untuk v1, ini paling simple dan performa.

### D2: Module structure

**Pilihan**: Single file `src/template.rs` untuk struct + registry

**Alternatives considered**:
- `src/template/mod.rs` + `src/template/registry.rs` — over-engineered untuk satu module
- Flat structs di `main.rs` — gak scalable

**Rationale**: Satu file cukup untuk struct definitions + registry. Split nanti kalau udah复杂.

### D3: Serde for deserialization

**Pilihan**: `serde` + `serde_json` dengan `#[derive(Deserialize)]`

**Alternatives considered**:
- Manual parsing — verbose, error-prone
- `toml` crate — gak cocok untuk nested structures

**Rationale**: Industry standard di Rust ecosystem. `serde` sudah mature, `derive` macro hemat boilerplate.

## Risks / Trade-offs

- **[Compile-time size]** Embed semua JSON di binary → binary lebih besar ~50KB. → Acceptable untuk CLI tool.
- **[Template changes require recompile]** Kalau mau update template, harus rebuild. → Fine untuk v1. Future: runtime loading.
- **[No validation at load time]** Kalau JSON malformed, error saat deserialize. → Mitigate dengan unit tests yang cover semua template.
