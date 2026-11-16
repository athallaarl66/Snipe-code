# Design — feat-cli-ux-help-audit

## Context

Lihat `proposal.md` untuk motivasi. Kondisi saat ini:

- `src/main.rs` hanya 42 baris — dispatch argumen manual: `args[1] == "audit"` → `audit::run_audit(path, &export_format)`, selain itu dianggap wizard `new::run(dry_run)`. Tidak ada parsing help, tidak ada subcommand `new`, tidak ada penanganan argumen tak dikenal.
- `src/audit.rs` sudah memisahkan check logic dari export (`run_audit` → `generate_markdown/pdf/docx`), jadi menambah prompt interaktif tidak perlu refactor besar.
- `dialoguer` 0.11 + `console` 0.15 sudah ada di `Cargo.toml` — **tidak ada dependency baru**.
- CLI berjalan di Windows (device user), bash (dev), dan lintas platform. Output harus plain-text friendly.

## Goals / Non-Goals

**Goals:**
- Dispatch command yang jelas: `audit`, `new` (eksplisit), `help`, `--help`, `-h`, tak dikenal → friendly error
- Audit tanpa `--export` → prompt interaktif format (Markdown/PDF/DOCX), default md
- FAQ + uninstall + troubleshooting di help
- README section instalasi jujur (build vs install vs portable exe)

**Non-Goals:**
- Bukan rewrite ke clap/structopt — parsing manual 42 baris masih cukup, YAGNI
- Bukan command parser penuh (tidak ada flag bersarang, `--export` cukup)
- Tidak menambah fitur audit baru (check logic tetap)

## Decisions

### D1: Tetap manual arg parsing, tanpa clap
`clap` adalah solusi kanonik Rust CLI, tapi kasus ini cuma 4-5 bentuk pemanggilan. Manual parsing = diff kecil, zero dependency. Kalau nanti command > 5 atau butuh flag bersarang, migrasi ke `clap` adalah upgrade path alami (`ponytail: clap`, add when command surface grows).

### D2: Refactor dispatch ke match di `main.rs`
```
args[1] == "help" | "--help" | "-h"  → print_help()
args[1] == "audit"                    → parse path + --export, run_audit
args[1] == "new"                      → new::run(dry_run)
args[1].starts_with('-')              → print_help() (treat as help)
else                                  → friendly "Unknown command" + hint help
```
Subcommand `new` dieksplisitkan (sebelumnya apa pun selain `audit` dianggap wizard — ini source of confusion). Backward compatible: `snipe-code` tanpa argumen tetap wizard.

### D3: Audit format selection — flag menang, prompt sebagai fallback
`--export` diberikan → langsung export (non-interactive, untuk skrip). Tidak diberikan → `MultiSelect` dari dialoguer dengan default `["Markdown"]`. Empty selection → warning + re-prompt (bukan silent exit). Konsisten dengan PRD 6.2 (semua prompt arrow-key) dan FR-US-008 (export md/pdf/docx).

### D4: Help/FAQ sebagai fungsi konstanta string
Print konstanta `&str` — tidak perlu engine template. FAQ & uninstall cukup statis. Konten: usage overview, per-command detail, FAQ (build/install/uninstall/troubleshoot).

### D5: README update sebagai bagian change
README adalah kontrak pengguna. Section Instalasi dipecah: "Build dari source (dev)" vs "Pakai release binary (end user)" vs "Install ke PATH (opsional)" + subsection Uninstall + FAQ pointer ke `snipe-code help`.

## Risks / Trade-offs

- [MultiSelect di terminal non-interactive (CI/pipe) bisa hang] → cek `console::Term::is_term()`; kalau stdin bukan terminal, default ke md tanpa prompt
- [Help output panjang jadi noise] → format section pendek, "..." untuk detail; `help <cmd>` untuk detail per command
- [Backward compat: dulu argumen aneh dianggap wizard] → sekarang error + hint; ini intended breaking, dokumentasikan di proposal
- [Windows console encoding untuk simbol ✓/⚠] → dialoguer/console sudah handle; fallback ASCII di help output

## Migration Plan

1. Implement dispatch + help di `main.rs` (D2, D4)
2. Implement MultiSelect di `audit.rs` (D3)
3. Update README (D5)
4. `cargo test` — tambah test untuk help output + dispatch (unit/integration)
5. `cargo build --release` → verifikasi `snipe-code.exe` jalan: `--help`, `audit` tanpa flag (interactive), `audit --export pdf,docx` (non-interactive)
6. Rollback: revert commit — tidak ada migrasi data/skema

## Open Questions

Tidak ada. Semua keputusan yang mengubah spec/approach sudah diputuskan di atas.
