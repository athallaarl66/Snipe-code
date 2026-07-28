## Why

SNIPE-CODE CLI belum punya kode sama sekali — masih fase dokumentasi. Template registry adalah foundation pertama yang dibutuhkan sebelum bisa build `snipe-code new` command. Tanpa ini, tidak ada cara untuk mendefinisikan dan memuat template preset (Company Profile, Portfolio, Blog) secara programatik.

## What Changes

- Tambah struct `Template`, `TemplateStructure`, `FolderDef`, `FileDef`, `GenerationRules` sebagai data model
- Tambah `TemplateRegistry` untuk memuat dan mengakses template definitions
- Embed 4 template definitions (none, company-profile, portfolio, blog) di compile-time via `include_str!`
- Tambah unit tests untuk validasi semua template ter-load dengan benar

## Capabilities

### New Capabilities
- `template-registry`: Core data structures dan registry untuk mendefinisikan, memuat, dan mengakses template preset definitions

### Modified Capabilities
_(none — project baru, belum ada existing specs)_

## Impact

- **Code**: `src/template.rs` (baru), `src/main.rs` (update)
- **Dependencies**: `serde` + `serde_json` (untuk deserialize template JSON)
- **Files**: `templates/*.json` (embedded at compile-time)
- **Testing**: Unit tests di `tests/template_test.rs`
