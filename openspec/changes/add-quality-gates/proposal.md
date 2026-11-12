# Proposal

## Why

Hasil scan belum menghitung metrik maintainability/duplication, memakai coverage report dari luar, atau membandingkan baseline/PR dengan gate berversi. Gate tri-state dan bukti delta mencegah data yang hilang atau base yang ambigu dilaporkan sebagai lulus.

## What Changes

- Tambahkan metrik maintainability dan duplication dengan definisi, threshold, dan exclusions eksplisit.
- Impor coverage hanya dari report terdukung dengan provenance; scanner statis tidak mengklaim menghitung coverage.
- Bandingkan baseline/PR menggunakan fingerprint dan rule version; tandai base hilang, rename/merge ambigu sebagai `unknown`.
- Evaluasi quality gate sebagai `pass`, `fail`, `unknown`, atau `not-applicable`, dengan kebijakan missing metric eksplisit.

## Capabilities

### New Capabilities
- `quality-gates`: Metrik, baseline/delta, dan keputusan gate yang menyatakan ketidakpastian.

### Modified Capabilities

## CR alignment

See `../../CR-ALIGNMENT.md` for requirement-to-task acceptance mapping and dependency boundaries. Current checkboxes are evidence claims, not completion inferred from this planning update.

## Impact

Menyentuh hasil kanonik, fingerprint, parser coverage report, konfigurasi threshold, output CLI, dan test fixture. Bergantung pada model hasil dari `add-scan-history`; verifikasi kompatibilitas kontrak yang dipakai tanpa menunggu archive. Definisi metrik dan format report awal perlu dikunci melalui design dan corpus uji; belum mencakup GUI atau sinkronisasi PR remote.
