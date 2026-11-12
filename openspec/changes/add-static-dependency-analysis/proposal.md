# Proposal

## Why

`audit` saat ini tidak memberi analisis aliran data lintas fungsi atau analisis dependency yang konsisten offline untuk semua ekosistem. Scanner bertahap dengan cakupan/version yang dinyatakan dan status `unknown` mengurangi false assurance.

## What Changes

- Tambahkan SAST berbasis rule/source-to-sink hanya untuk bahasa yang diumumkan; hasil kandidat mencantumkan bukti, confidence, dan keterbatasan.
- Tambahkan SCA dari manifest/lockfile dan snapshot advisory lokal dengan provenance/tanggal.
- Tandai ekosistem, file, rule, snapshot stale, dan kondisi yang tidak dievaluasi sebagai `unknown` atau status terpisah; tidak menganggapnya bersih.
- Uji tiap rule/adapter dengan fixture positif dan negatif; mode offline tidak membuat koneksi jaringan.

## Capabilities

### New Capabilities
- `static-analysis`: Temuan keamanan statis terukur dan dibatasi rule/language yang didukung.
- `dependency-analysis`: Evaluasi dependency terhadap snapshot advisory berversi dengan perilaku offline yang eksplisit.

### Modified Capabilities

## CR alignment

See `../../CR-ALIGNMENT.md` for requirement-to-task acceptance mapping and dependency boundaries. Current checkboxes are evidence claims, not completion inferred from this planning update.

## Impact

Menyentuh engine audit, parser manifest/lockfile, format hasil, dan fixture/test. Bergantung pada hasil kanonik `add-scan-history`; terapkan dan arsipkan terlebih dahulu. Sumber dan format advisory serta daftar awal bahasa/ekosistem harus dipastikan saat design sebelum menetapkan coverage. Tidak mencakup perbaikan otomatis, online AI, atau klaim parity SonarQube.
