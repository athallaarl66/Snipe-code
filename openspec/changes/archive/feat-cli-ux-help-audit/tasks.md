## 1. Command Dispatch & Help

- [x] 1.1 Refactor `src/main.rs`: ganti `if args.len() > 1 && args[1] == "audit"` menjadi `match` yang menangani `help`/`--help`/`-h`, `audit`, `new`, argumen tak dikenal (friendly error + hint), dan tanpa argumen (wizard default)
- [x] 1.2 Tambah subcommand eksplisit `new` (`snipe-code new [--dry-run]`) — backward compatible dengan `snipe-code` tanpa argumen yang tetap masuk wizard
- [x] 1.3 Implement fungsi `print_help()` — usage overview: daftar command (new, audit), opsi (`--dry-run`, `--export`), contoh pemakaian, pointer ke `help <command>`
- [x] 1.4 Implement `print_command_help(cmd)` untuk `audit` dan `new` — opsi per command + contoh; topic tak dikenal → pesan + hint `snipe-code help`
- [x] 1.5 Implement section FAQ di help — install (`cargo install --path .` / release binary), uninstall (`cargo uninstall snipe-code` atau hapus file), troubleshooting "'snipe-code' is not recognized"

## 2. Interactive Audit Export Format

- [x] 2.1 Di `src/audit.rs` (atau `main.rs`): jika `--export` tidak diberikan dan terminal interaktif, tampilkan `MultiSelect` format (Markdown default, PDF, DOCX) via dialoguer
- [x] 2.2 Empty selection → warning + re-prompt; non-interactive terminal (CI/pipe) → default ke markdown tanpa prompt
- [x] 2.3 Pertahankan `--export md,pdf,docx` non-interactive path (tanpa prompt)

## 3. Documentation

- [x] 3.1 Update `README.md` section Instalasi: pecah jadi "Build from source", "Pakai release binary (portable, tanpa install)", "Install ke PATH (opsional)" + catatan storage (`target/` besar, `cargo clean`) + Uninstall (`cargo uninstall snipe-code` / hapus file)

## 4. Tests & Verification

- [x] 4.1 Tambah integration tests: `--help` / `help` output memuat command list & FAQ; `help audit` memuat opsi export; command tak dikenal memunculkan friendly error
- [x] 4.2 Tambah test audit format selection: tanpa `--export` + non-interactive → default md (tanpa hang)
- [x] 4.3 Jalankan `cargo test` — semua test (existing 57 + baru) pass
- [x] 4.4 `cargo build --release` — verifikasi manual: `snipe-code.exe --help`, `audit` tanpa flag (interactive prompt), `audit --export pdf,docx` (non-interactive), wizard `new` masih jalan

## 5. Release Deliverable

- [x] 5.1 Konfirmasi `target/release/snipe-code.exe` siap distribusi (Windows x64, portable, tanpa Rust di device user)
