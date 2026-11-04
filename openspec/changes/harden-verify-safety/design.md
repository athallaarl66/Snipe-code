# Design

## Context

Lihat `proposal.md` untuk motivasi dan `specs/safe-verification/spec.md` untuk kontrak. `verify` membuat gate plan dari manifest, menjalankan command melalui runner di direktori project, dan mewarisi environment proses. Exit status saat ini `0` ketika ada gate dieksekusi dan tak ada gate gagal, walaupun gate lain unavailable.

## Goals / Non-Goals

**Goals:** status agregat fail-closed untuk gate wajib yang unavailable/blocked; preflight side effect yang aman; laporan command dan target yang dapat diaudit.

**Non-Goals:** sandbox lintas OS penuh dalam change ini, tool install otomatis, menambah SAST/SCA rules, atau memakai database remote.

## Decisions

- Perluas model outcome dengan `blocked` serta status agregat yang memisahkan complete-pass, partial/unknown, failure, dan no execution. Pilih exit code partial melalui design CLI yang diuji terhadap consumer lama; jangan mengubah code diam-diam.
- Preflight hanya dapat memblokir fakta yang dapat diperiksa lokal; redaksi URL/credential. Jika konfigurasi DB unresolved, hasil `blocked`; jangan mengandalkan nama database atau variable tunggal.
- Gunakan runner injection yang telah ada untuk memastikan test status/preflight tidak spawn command nyata. Isolasi yang tidak disediakan platform tidak boleh disimulasikan sebagai sandbox.
- Sebelum menambahkan dependency install/hook, modelkan effects dan minta izin untuk mode yang dapat menulis/mengakses network. Pertahankan flag opt-out yang ada.

## Risks / Trade-offs

- [Exit code baru merusak automation lama] Dokumentasikan perubahan dan sediakan mode compatibility bila diperlukan; update integration tests/CLI docs.
- [DB URL tidak selalu mengungkapkan target efektif] Unknown/credential opaque memicu blocked; jangan klaim aman jika hanya sebagian konfigurasi terlihat.
- [Sandbox scope lintas platform besar] Tunda runner sandbox; laporan menamai batas aktual per OS dan menolak klaim isolated.
- [Preflight memiliki false positive] Sajikan alasan dan opsi konfigurasi eksplisit, tanpa bypass diam-diam.

## Migration Plan

1. Tambah status outcome baru secara additive untuk JSON/report dan unit tests runner palsu.
2. Perbaiki agregasi exit status, perbarui usage/help/README dan semver/release notes sebelum distribusi.
3. Tambah preflight untuk DB/target yang diketahui; unknown fail-closed.
4. Jika rilis bermasalah, revert policy preflight hanya melalui release rollback yang diumumkan; jangan menghapus outcome/status yang tersimpan dalam output terdahulu.
