# Proposal

## Why

`verify` menjalankan tool proyek langsung di direktori target dengan environment host dan dapat meninggalkan perubahan; tidak ada preflight yang membatasi target DB. Hasil juga dapat exit 0 saat satu gate tidak tersedia, sehingga pemanggil mengira verifikasi lengkap. Sementara itu, `audit` mencampur scan dengan export ke direktori proyek dan pemeriksaan symlink dotfile berisiko membaca `.env` di luar root.

## What Changes

- Tambahkan pemeriksaan keselamatan sebelum menjalankan gate yang bisa menulis atau mengakses network.
- Wajibkan preview dan persetujuan eksplisit pengguna sebelum install, test, akses endpoint, atau transmisi data; scope yang berubah memerlukan approval baru.
- Blokir konfigurasi DB eksplisit yang remote/production-like/ambigu sebelum spawn; jika tidak ada konfigurasi DB yang terdeteksi, beri peringatan dan minta approval CLI eksplisit, tanpa menyebut target aman.
- Batasi approval otomatis `verify-generated` pada workspace sementara yang dibuat dan diisi harness; bersihkan environment DB target/host yang diwarisi sebelum child process. Harness bukan sandbox dan tidak membatasi endpoint/network atau konfigurasi DB hardcoded.
- Pertahankan status `unavailable` sebagai hasil non-pass, termasuk dalam exit status/kontrak machine-readable.
- Dokumentasikan side effect, izin, perilaku `--only`, dan migrasi kompatibilitas.
- Uji dengan runner palsu; jangan gunakan DB bersama atau tool proyek nyata untuk fixture keselamatan.
- Tolak symlink/reparse point sebelum membuka konten dan batasi pembacaan audit pada root kanonik.
- Tambahkan `scan [PATH] [--json]` read-only dan `export <scan.json> --format <md,pdf,docx> --output <DIR>` berbasis hasil scan berversi.
- Validasi opsi audit ketat; pertahankan command legacy, format, JSON stdout, exit code, dan beri warning side effect.
- Paginasikan PDF agar seluruh finding tetap tersedia.
- Histori scan/SQLite tetap di luar scope.

Scope audit CLI sebelumnya di `harden-audit-cli` digabung ke proposal ini agar lifecycle audit dan verify ditinjau dalam satu change.

## Capabilities

### New Capabilities
- `safe-verification`: Menjalankan quality gate dengan preflight, status hasil yang jujur, dan batas side effect yang terlihat.
- `audit-cli`: Pemindaian audit berbatas root, scan read-only, export eksplisit, dan kontrak CLI yang kompatibel.

- `generation-safety-regressions`: Containment of template overrides and generated Compose/Go CI contracts.

### Modified Capabilities

## Impact

Menyentuh `src/verify/`, `src/audit.rs`, `src/main.rs`, README dan test verify/CLI/audit. Perubahan exit code atau kebijakan blokir adalah kontrak publik dan harus terdokumentasi serta diuji. Tidak membangun sandbox lintas OS. CLI umum menjalankan command host hanya setelah preview dan approval eksplisit; batas non-isolated harus dinyatakan, dan approval tidak melewati blokir DB eksplisit remote/production/ambigu. Sandbox-mode yang boundary-nya tidak tersedia tetap blocked. `verify-generated` hanya mendapat otorisasi otomatis untuk workspace/project hasil buatannya sendiri dan bukan bukti isolasi. Tidak mencakup GUI, SQLite history, atau penambahan gate SAST/SCA.
