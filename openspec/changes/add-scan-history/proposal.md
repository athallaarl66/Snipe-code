# Proposal

## Why

SNIPE-CODE kini mengembalikan hasil audit/gate per command tanpa model hasil bersama yang berversi atau riwayat lokal. Hasil kanonik dan penyimpanan lokal diperlukan agar temuan, cakupan, dan status `unknown` dapat dibandingkan tanpa menyimpan rahasia proyek.

## What Changes

- Tetapkan schema hasil berversi yang dapat dipakai CLI sekarang dan surface lain di masa depan.
- Simpan run, cakupan, temuan, gate, dan triage minimum pada SQLite lokal produk, bukan database proyek yang dianalisis.
- Redaksi data sensitif; sediakan migrasi transaksional dan kebijakan retensi/ekspor/penghapusan.
- Catat versi engine/rule serta asal dan keterbatasan hasil.

## Capabilities

### New Capabilities
- `scan-history`: Hasil scan kanonik dan riwayat lokal yang berversi serta menjaga privasi.

### Modified Capabilities

## CR alignment

See `../../CR-ALIGNMENT.md`. Existing scan/history code is partial, not absent. Complete evaluated rule states, reject contradictory summaries, and require deletion preview, dry-run, and scope-bound confirmation. Preserve OS-specific privacy limits; do not infer Windows ACL guarantees from Unix permissions.

## Impact

Menyentuh model hasil audit/verify, modul penyimpanan baru, CLI JSON, konfigurasi lokal, dan test migrasi. Perubahan bergantung pada kontrak scan/export dari `harden-verify-safety`; terapkan dan arsipkan change itu lebih dulu. Database produk tidak boleh memakai koneksi atau kredensial target proyek. Tidak mencakup engine rule SAST/SCA, GUI, AI, atau konektor.
