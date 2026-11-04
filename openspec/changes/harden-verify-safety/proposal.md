# Proposal

## Why

`verify` menjalankan tool proyek langsung di direktori target dengan environment host dan dapat meninggalkan perubahan; tidak ada preflight yang membatasi target DB. Hasil juga dapat exit 0 saat satu gate tidak tersedia, sehingga pemanggil mengira verifikasi lengkap.

## What Changes

- Tambahkan pemeriksaan keselamatan sebelum menjalankan gate yang bisa menulis atau mengakses network.
- Blokir target DB produksi atau ambigu; nyatakan alasan dan jangan spawn command jika preflight gagal.
- Pertahankan status `unavailable` sebagai hasil non-pass, termasuk dalam exit status/kontrak machine-readable.
- Dokumentasikan side effect, izin, perilaku `--only`, dan migrasi kompatibilitas.
- Uji dengan runner palsu; jangan gunakan DB bersama atau tool proyek nyata untuk fixture keselamatan.

## Capabilities

### New Capabilities
- `safe-verification`: Menjalankan quality gate dengan preflight, status hasil yang jujur, dan batas side effect yang terlihat.

### Modified Capabilities

## Impact

Menyentuh `src/verify/`, `src/main.rs`, README dan test verify/CLI. Perubahan exit code atau kebijakan blokir adalah kontrak publik dan harus terdokumentasi serta diuji. Belum membangun sandbox lintas OS; bila isolasi target tidak dapat dibuktikan, command berisiko harus ditolak atau memerlukan izin eksplisit sesuai desain. Tidak mencakup GUI, SQLite history, atau penambahan gate SAST/SCA.
