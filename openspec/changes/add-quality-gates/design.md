# Design

## Context

Lihat proposal dan `specs/quality-gates/spec.md`. `audit` menghitung heuristik temuan tanpa maintainability/duplication model; hasil `verify` mencatat gate per ecosystem. Tidak ada baseline, coverage import, atau histori terstruktur sebelum `add-scan-history`.

## Goals / Non-Goals

**Goals:** definisi metrik berversi, coverage import tervalidasi, compare baseline dengan ambiguity eksplisit, gate yang fail/unknown tidak lulus diam-diam.

**Non-Goals:** klaim score universal, menghitung coverage dari source scan, remote PR API, atau aturan yang sama dengan semua edisi produk lain.

## Decisions

- Gunakan result envelope `add-scan-history`; simpan metric ID/version, config version, value/unit, input provenance, exclusions dan status. Verifikasi kompatibilitas kontrak yang dipakai tanpa menunggu archive.
- Implementasi maintainability/duplication hanya untuk metrik yang punya definisi deterministik dan corpus fixture terukur. Hindari satu score agregat yang menyembunyikan submetric/unknown.
- Coverage hanya diimpor dari format yang dipilih lewat design dengan fixture; validasi source path mapping ke project root, format/version, dan jumlah file. Tanpa report valid hasil `unknown`.
- Baseline merujuk run/schema/rule/fingerprint version dan base commit/ref opsional. Compare menghasilkan new/unchanged/resolved hanya untuk match yang jelas; rename atau rule-version mismatch menghasilkan ambiguous/unknown.
- Quality gate memakai config eksplisit berversi; setiap metric policy = required/optional, threshold, handling unknown. Default unknown non-pass, failure threshold = fail. CLI exit code baru harus disejajarkan dengan `harden-verify-safety`.

## CR alignment

## Approved first slice

- User selected Rust metrics and LCOV import. No additional dependency: use installed `syn`, `proc-macro2`, `sha2`, serde and stdlib. Source analysis is read-only; it does not run target commands or contact services.
- Maintainability proxy `rust-function-lines` v1: maximum inclusive physical line span of each free/impl/trait function body. Empty Rust corpus or unsupported source languages/parse budget failures are unknown, not zero. This is not a universal maintainability score.
- Duplication `rust-duplicate-blocks` v1: exact non-overlapping blocks of 50 Rust tokens, comments/whitespace discarded, identifiers/literals preserved, group delimiters included. Walk files in lexical order; greedily match each 50-token window against earlier non-overlapping accepted windows. Count additional duplicate blocks, including same-file duplicates. No alpha-renaming or semantic equivalence. Bound files, bytes, nesting and total tokens; unsupported input is unknown.
- Config v1 explicitly supplies enabled, metric ID/version/unit, required, comparison/threshold, unknown handling and relative path-prefix exclusions. No implicit threshold. Coverage unit is basis-points (10000 = 100%), rounded down; other units are lines/blocks/findings. Required unknown cannot pass; optional unknown is still visible and non-pass. Disabled policies are not-applicable.
- LCOV input v1 accepts complete `SF`, `DA`, `end_of_record` records plus recognized summary/function/branch fields, validates counts, maps existing files within canonical project root, rejects links/traversal/duplicate records and oversized input. Provenance is report SHA-256 plus mapped files and covered/total lines, not a claim that tests ran in this command or that coverage is fresh/current.
- `quality [PATH] --config <FILE> [--coverage <LCOV>] [--baseline <SCAN_JSON> --base-ref <REF>] [--json] [--save]` performs current scan and attaches versioned quality report to optional scan-envelope field, preserving old v2 inputs. Save uses existing opt-in history. Baseline metadata stores run/project/result schema, envelope and per-finding rule/fingerprint versions, supplied ref and resolved Git commit. Baseline file is immutable input, not rewritten.
- Compare fingerprints only after project/schema/rule/fingerprint and evaluated-scope compatibility, accessible base Git ref, no merge-conflicted entries and no detected rename. Git checks are read-only. Ambiguity means unknown and empty new/resolved/unchanged lists; never auto-resolve. Baseline scan-to-commit binding is caller-supplied provenance, not cryptographically established; there is no remote PR lookup.
- Quality outcomes remain separate from verify execution states. CLI maps fail to 1, invalid args/config to 2, runtime errors or no evaluated metric to 3, mixed evaluated/unknown to 4, complete pass/not-applicable to 0. Fail takes precedence over unknown. No changes to verify approval or generated-project automation.

See `../../CR-ALIGNMENT.md` for requirement-to-task acceptance mapping and dependency boundaries. Current checkboxes are evidence claims, not completion inferred from this planning update.

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
