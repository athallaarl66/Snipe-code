# Proposal

## Why

`audit` saat ini mencampur scan dengan export ke direktori proyek, dan pemeriksaan symlink dotfile dapat membaca `.env` di luar root. Kontrak eksplisit dan batas root diperlukan agar scan aman tanpa merusak perilaku CLI yang sudah digunakan.

## What Changes

- Tolak symlink sebelum pemrosesan file dotfile, dan cegah pembacaan di luar root.
- Pisahkan operasi scan read-only dari export yang menulis ke tujuan eksplisit.
- Validasi argumen `audit` agar flag tak dikenal dan path ambigu ditolak.
- Pertahankan kompatibilitas command, format report, dan exit code lama selama migrasi; dokumentasikan side effect yang masih berlaku.
- Perbaiki klaim dokumentasi tentang `audit`, `--json`, dan perilaku export.

## Capabilities

### New Capabilities
- `audit-cli`: Pemindaian audit berbatas root, ekspor eksplisit, dan kontrak CLI yang kompatibel.

### Modified Capabilities

## Impact

Menyentuh `src/audit.rs`, `src/main.rs`, dokumentasi CLI, serta test audit/CLI. Perubahan perilaku security harus punya fixture negatif symlink termasuk Windows; tidak menjalankan project target. Perubahan ini fondasi pertama bagi `add-scan-history`; spesifikasi tidak mencakup SQLite atau engine SAST/SCA baru.
