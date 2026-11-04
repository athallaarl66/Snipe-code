# Design

## Context

Lihat proposal dan delta `static-analysis` serta `dependency-analysis`. `audit` saat ini memakai enam kelompok heuristik berbasis file/baris; `verify --security` menjalankan `npm audit`, `composer audit`, atau `dotnet list package --vulnerable`, dan menandai adapter lain not-applicable. Tidak ada advisory DB offline.

## Goals / Non-Goals

**Goals:** adapter rule bertahap dengan status cakupan eksplisit; fixture vulnerable/safe; SCA offline berprovenance.

**Non-Goals:** menjanjikan analisis lengkap, exploit proof, semua ecosystem, advisory live, AI, atau auto-fix.

## Decisions

- Jadikan rule pack data berversi dengan stable `rule_id`, ecosystem/language scope, sumber/sink/sanitizer yang didukung, severity/confidence dan referensi. Implementasi pertama hanya untuk bahasa yang dipilih dan disetujui melalui corpus representatif; jangan menulis daftar dukungan belum teruji ke user docs.
- Mulai dengan rule berbasis AST/library yang telah tersedia atau parser sederhana hanya untuk subset yang dapat divalidasi. Jangan mengklaim interprocedural flow jika engine hanya per-file/per-function. Unsupported construct, timeout, parse error dan excluded path menjadi status terpisah.
- Pertahankan heuristik lama sebagai rule legacy/versioned untuk kompatibilitas. Jangan gabungkan fingerprint rule lama dan baru tanpa provenance; saran fix disimpan terpisah dan tidak diaplikasikan.
- SCA adapter mengonsumsi lockfile/manifest yang terpilih dan menghasilkan graph yang resolusinya nyata. Advisory source/cache format dipilih setelah evaluasi lisensi, stabilitas, offline distribution, freshness, signature dan ukuran; tidak mengunduh tool proyek.
- Simpan snapshot atomik dengan source, format/version, checksum/signature bila tersedia, fetch time dan expiry policy. Periksa integritas sebelum aktivasi; snapshot absent/stale/schema mismatch = unknown.
- Uji positive/negative fixtures per rule dan ecosystem: safe parameterized query, escaped/sanitized sink, fake secret, vulnerable dependency version, unaffected version, missing/invalid lockfile, stale snapshot dan offline no-egress.

## Risks / Trade-offs

- [False positive/negative] Nyatakan confidence/evidence; rule tanpa pair fixture tidak dirilis.
- [Parser subset memberi rasa coverage luas] Hanya nyatakan bahasa/construct yang diuji; unsupported menghasilkan unknown.
- [Advisory distribution/licensing berubah] Abstraksikan format dan pin provenance; perubahan source perlu security/legal review.
- [Snapshot supply-chain compromise] Validasi checksum/signature dan atomic replacement; fail closed tanpa merusak snapshot lama.
- [Heuristik lama berubah baseline] Versioned rule IDs dan label legacy.

## Migration Plan

1. Gunakan envelope hasil `scan-history`; perubahan menerapkannya setelah `add-scan-history` diarsipkan.
2. Tambahkan satu ecosystem/rule slice dengan fixture dan docs aktual; tambah ekosistem hanya setelah gate fixtures lulus.
3. Tambahkan import/refresh snapshot sebagai operasi terpisah opt-in; default SCA offline.
4. Jika rule pack gagal, matikan versi pack terkait; simpan history lama bersama rule version, jangan rewrite hasil historis.
5. Terapkan/archive `add-scan-history` dahulu; terapkan/archive `add-static-dependency-analysis` sebelum `add-quality-gates` supaya quality gate dapat mengonsumsi status SAST/SCA.
