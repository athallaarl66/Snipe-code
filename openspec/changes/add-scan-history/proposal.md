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

## Impact

Menyentuh model hasil audit/verify, modul penyimpanan baru, CLI JSON, konfigurasi lokal, dan test migrasi. Perubahan bergantung pada kontrak scan/export dari `harden-audit-cli`; terapkan dan arsipkan change itu lebih dulu. Database produk tidak boleh memakai koneksi atau kredensial target proyek. Tidak mencakup engine rule SAST/SCA, GUI, AI, atau konektor.
