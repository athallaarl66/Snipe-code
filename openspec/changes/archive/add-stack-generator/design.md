## Context

Template registry (`src/template.rs`) sudah selesai. Sekarang butuh stack definitions dan generator untuk bisa generate struktur folder berdasarkan pilihan user. US-002 butuh clean architecture structure tanpa template-specific files.

Dari TDD, flow-nya:
```
Template Selector → Stack Selector → Config Prompt → Structure Generator → Git Automator
```

Kita di Stack Selector + Structure Generator.

## Goals / Non-Goals

**Goals:**
- Definisikan stack options (frontend, backend, database) beserta folder structure masing-masing
- Buat generator yang create folders + security config files
- Support business rules: skip frontend jika None, skip backend jika None, skip docker jika No Docker
- Generate `.env.example`, `.gitignore`, `README.md`

**Non-Goals:**
- CLI prompts / interactive flow (nanti di `commands/new.rs`)
- Git automation (nanti di `git.rs`)
- Template-specific folder additions (nanti ditambah di generator)

## Decisions

### D1: Stack definitions as static data

**Pilihan**: Static structs di `src/stack.rs` dengan `const` arrays

**Alternatives considered**:
- JSON files — lebih fleksibel tapi tambah I/O
- Database — overkill untuk CLI tool

**Rationale**: Stack options jarang berubah. Static data = zero runtime cost, zero dependency.

### D2: Generator creates actual filesystem

**Pilihan**: `std::fs::create_dir_all` + `std::fs::write` untuk buat folders + files

**Alternatives considered**:
- Dry-run only (return paths tanpa create) — useful tapi bukan primary mode
- Template engine (askama/tera) — overkill untuk simple file generation

**Rationale**: Direct `std::fs` paling simple. Dry-run bisa ditambah sebagai flag nanti.

### D3: Security config as embedded strings

**Pilihan**: Embed `.env.example` dan `.gitignore` content sebagai `const &str` di Rust

**Alternatives considered**:
- Separate files di `templates/` — tambah file management
- Runtime generation — lebih fleksibel tapi lebih kompleks

**Rationale**: Content statis, jarang berubah. Embedded = zero runtime I/O.

## Risks / Trade-offs

- **[Folder collision]** Kalau folder sudah ada, `create_dir_all` gak error (idempotent). → Safe.
- **[File overwrite]** Kalau file sudah ada, `write` akan overwrite. → Mitigate dengan check dulu atau flag `--force`.
- **[Stack version drift]** Generate structure only, gak install dependencies. → User install sendiri via package manager.
