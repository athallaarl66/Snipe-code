# Design

## Context

Lihat proposal dan `specs/quality-gates/spec.md`. `audit` menghitung heuristik temuan tanpa maintainability/duplication model; hasil `verify` mencatat gate per ecosystem. Tidak ada baseline, coverage import, atau histori terstruktur sebelum `add-scan-history`.

## Goals / Non-Goals

**Goals:** definisi metrik berversi, coverage import tervalidasi, compare baseline dengan ambiguity eksplisit, gate yang fail/unknown tidak lulus diam-diam.

**Non-Goals:** klaim score universal, menghitung coverage dari source scan, remote PR API, atau aturan yang sama dengan semua edisi produk lain.

## Decisions

- Gunakan result envelope `add-scan-history`; simpan metric ID/version, config version, value/unit, input provenance, exclusions dan status. Terapkan setelah `add-scan-history` diarsipkan.
- Implementasi maintainability/duplication hanya untuk metrik yang punya definisi deterministik dan corpus fixture terukur. Hindari satu score agregat yang menyembunyikan submetric/unknown.
- Coverage hanya diimpor dari format yang dipilih lewat design dengan fixture; validasi source path mapping ke project root, format/version, dan jumlah file. Tanpa report valid hasil `unknown`.
- Baseline merujuk run/schema/rule/fingerprint version dan base commit/ref opsional. Compare menghasilkan new/unchanged/resolved hanya untuk match yang jelas; rename atau rule-version mismatch menghasilkan ambiguous/unknown.
- Quality gate memakai config eksplisit berversi; setiap metric policy = required/optional, threshold, handling unknown. Default unknown non-pass, failure threshold = fail. CLI exit code baru harus disejajarkan dengan `harden-verify-safety`.

## Risks / Trade-offs

- [Metric definition berubah] Versi metric/config; jangan bandingkan angka lintas definisi tanpa konversi eksplisit.
- [Coverage report malicious/oversized] Batasi ukuran dan path, parse terisolasi, reject invalid schema.
- [Fingerprint mismatch membuat delta bising] Tandai ambiguous, jangan auto-close finding.
- [Unknown terlalu sering] Tampilkan alasan dan input yang dibutuhkan tanpa merendahkan policy menjadi pass.

## Migration Plan

1. Terapkan setelah `add-scan-history`; jangan menambah kolom ad hoc ke JSON legacy.
2. Rilis satu metric/coverage format per slice setelah fixture. Simpan versi config/metric pada hasil.
3. Gate default non-blocking hanya sampai pengguna mengaktifkan config; pada config yang enabled, required unknown tidak lulus.
4. Rollback dengan menonaktifkan metric pack/config baru; jangan rewrite baseline/historic run.
