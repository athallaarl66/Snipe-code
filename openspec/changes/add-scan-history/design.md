# Design

## Context

Lihat `proposal.md` dan `specs/scan-history/spec.md`. Kode saat ini sudah menyediakan `ScanEnvelope`, fingerprint berversi, SQLite opt-in, serta scan/export/history. Sisa pekerjaan adalah kejujuran status rule, validasi ringkasan, redaksi lintas consumer, dan keselamatan penghapusan; lihat `../../CR-ALIGNMENT.md`.

## Goals / Non-Goals

**Goals:** hasil JSON stabil dan berversi; riwayat lokal minimal; migrasi transaksional; redaksi sebelum persistence.

**Non-Goals:** menyimpan source penuh, encrypt-at-rest claim tanpa implementasi kunci teruji, multi-user sync, konektor, dan menyamakan database lokal dengan target project.

## Decisions

- Tambahkan envelope hasil v1 yang memuat `schema_version`, `run_id`, timestamp UTC, root identity yang direduksi, tool/rule version, scope/exclusions, rule status, findings, dan gate status. Setiap finding memakai `rule_id` stabil dan fingerprint versioned; jangan menjanjikan rename matching sampai diuji.
- SQLite berada di direktori data pengguna SNIPE-CODE; tidak pernah mengambil connection string proyek. Gunakan transaksi migrasi dan foreign key; pilih dependency/storage API setelah ukur build-size dan dukungan Windows/macOS/Linux.
- Simpan evidence minimum: relative path, line/range, deskripsi redacted dan hash fingerprint. Jangan menyimpan snippet source secara default. Filter nilai secret pada render/persistence dan uji canary yang tidak boleh muncul di DB, JSON, log, maupun export.
- Pisahkan operasi read result dari persistence/history. History opt-in atau sesuai consent yang jelas; sediakan list/show/export/delete lokal. Offline tidak memanggil jaringan.
- Tulis migrasi berurutan dan schema metadata. Backup sebelum upgrade bila diperlukan; migrasi gagal rollback atomik dan aplikasi tetap dapat membuka versi lama tanpa menulis data rusak.

## CR alignment

See `../../CR-ALIGNMENT.md`. Existing scan/history code is partial, not absent. Complete evaluated rule states, reject contradictory summaries, and require deletion preview, dry-run, and scope-bound confirmation. Preserve OS-specific privacy limits; do not infer Windows ACL guarantees from Unix permissions.

## Risks / Trade-offs

- [Fingerprint berubah saat rule berubah] Sertakan versi fingerprint dan rule; rekonsiliasi lintas versi jangan otomatis.
- [Path/timestamp bocorkan informasi host] Simpan path relatif secara default; redaksi path absolut pada share/export.
- [SQLite file tetap dapat dibaca user lain di host] Terapkan permission file OS dan dokumentasikan batas tanpa klaim enkripsi; evaluasi encryption terpisah.
- [Migrasi rusak history] Snapshot/backup terverifikasi dan test failure injection untuk tiap migrasi.

## Migration Plan

1. Perkenalkan envelope/result JSON tanpa mengubah output legacy, dengan consumer tests.
2. Tambahkan SQLite baru; file DB dibuat hanya setelah pengguna memilih history.
3. Migrasikan schema transaksional dan uji open/upgrade/failure pada fixture.
4. Tidak import report lama otomatis karena tidak punya fingerprint/provenance; jika import ditambahkan, labeli `legacy/unmapped`.
5. Rollback dapat menonaktifkan history tanpa menghapus DB; export/delete lokal tetap dapat diakses dengan versi yang bisa membacanya.
