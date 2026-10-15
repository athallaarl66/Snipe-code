## Why

CLI saat ini tidak user-friendly untuk pemakaian nyata: `audit` memaksa user menghafal flag `--export pdf,docx` padahal PRD (section 6.2) mensyaratkan semua prompt interaktif arrow-key; tidak ada `--help`/FAQ sama sekali di `main.rs` (hanya cek `args[1] == "audit"`); dan README tidak menjelaskan cara install/distribusi yang benar — label "(Opsional)" pada `cargo install` membuat user di device lain gagal dengan "'snipe-code' is not recognized".

## What Changes

- Tambah perintah `help` / flag `--help` di `src/main.rs`: daftar command (`new` wizard, `audit`, `--dry-run`), opsi per command, contoh penggunaan, dan section FAQ singkat untuk user yang bingung (termasuk "kenapa 'snipe-code is not recognized'?" dan "bagaimana uninstall?")
- Ubah alur `audit`: jika `--export` tidak diberikan, tampilkan **MultiSelect interaktif** (dialoguer) untuk memilih format Markdown/PDF/DOCX — default markdown; `--export` tetap didukung untuk non-interactive/skrip
- Dokumentasi install yang jujur di `README.md`: beda `cargo build --release` / `cargo install --path .` / salin `.exe` portable; tips storage (`target/` ratusan MB, `cargo clean`); cara uninstall (`cargo uninstall snipe-code` untuk install via cargo, hapus file untuk portable exe)
- Build & verifikasi release binary `target/release/snipe-code.exe` sebagai deliverable distribusi (Windows x64, portable, tanpa Rust di device user)

## Capabilities

### New Capabilities
- `cli-help`: Perintah help/FAQ di CLI — `snipe-code --help`, `snipe-code help`, `snipe-code help audit` — menampilkan daftar command, opsi, contoh, dan FAQ (install, uninstall, troubleshooting)
- `audit-export-selector`: Pemilihan format export audit secara interaktif (Markdown/PDF/DOCX, multi-select) saat `--export` tidak diberikan; flag `--export` tetap berfungsi untuk mode non-interactive

### Modified Capabilities
_(none)_

## Impact

- **Code**: `src/main.rs` (arg parsing + help dispatch), `src/audit.rs` (interactive format selection), mungkin `src/lib.rs` (export fungsi untuk test)
- **Dependencies**: `dialoguer` (sudah ada — MultiSelect), `console` (sudah ada)
- **Docs**: `README.md` (section Instalasi + section FAQ/Uninstall baru)
- **Testing**: Integration tests di `tests/` untuk help output dan audit format selection
- **Deliverable**: `target/release/snipe-code.exe` (build release, bukan perubahan kode)
