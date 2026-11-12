# Design

## Context

`verify` berbagi engine dengan `verify-generated`. CLI menerima path pilihan caller; CI binary membuat workspace sementara baru, generate project sendiri, verifikasi, lalu menghapus workspace. Approval harus membedakan caller ini tanpa menyediakan bypass untuk path arbitrer. Audit juga menggabungkan scan dan export; dotfile traversal punya jalur terpisah dan `audit --json` masih menulis report.

## Goals / Non-Goals

**Goals:** status agregat fail-closed untuk gate wajib yang unavailable/blocked; preflight side effect yang aman; laporan command dan target yang dapat diaudit.

**Non-Goals:** sandbox lintas OS penuh dalam change ini, tool install otomatis, menambah SAST/SCA rules, atau memakai database remote.

## Decisions

- Perluas model outcome dengan `blocked` serta status agregat yang memisahkan complete-pass, partial/unknown, failure, dan no execution. Pilih exit code partial melalui design CLI yang diuji terhadap consumer lama; jangan mengubah code diam-diam.
- Preflight hanya dapat memblokir fakta yang dapat diperiksa lokal; redaksi URL/credential. Jika konfigurasi DB unresolved, hasil `blocked`; jangan mengandalkan nama database atau variable tunggal.
- Gunakan runner injection yang telah ada untuk memastikan test status/preflight tidak spawn command nyata. Isolasi yang tidak disediakan platform tidak boleh disimulasikan sebagai sandbox.
- CLI `verify` menampilkan command, direktori, efek diketahui, serta target DB yang sudah disamarkan. Minta persetujuan eksplisit per gate dan blokir jika approval tidak ada atau scope berubah. Tidak ada flag umum untuk melewati approval.
- `verify-generated` memakai jalur approval internal yang tidak diekspos melalui CLI/library API umum. Otorisasi hanya berlaku pada workspace sementara yang baru dibuat dan project yang di-generate proses itu; database preflight tetap berlaku. Harness menolak path caller-arbitrary.
- Audit scan menghasilkan data terstruktur tanpa write. Export membaca hasil berversi, mensyaratkan format dan output eksplisit, memvalidasi sebelum menulis. Command `audit` legacy tetap kompatibel dengan warning side-effect.
- Root audit dikanonicalisasi; symlink/reparse point dilewati sebelum traversal dan setiap pembacaan diverifikasi berada dalam root.
- Parser menolak flag tak dikenal/duplikat, path ganda, dan nilai hilang sebelum I/O. PDF dipaginasi dan seluruh finding diverifikasi lewat ekstraksi teks.
- Scan history/SQLite tetap di luar scope.

## CR alignment boundaries

See `../../CR-ALIGNMENT.md`. Safety closure also covers template override containment, truthful PDF/parity evidence, generated Compose configuration and Go CI regressions. These are bounded corrections, not a new scaffolding product. Existing agent artifacts need immediate consent wording before the later workflow redesign. DB classification is not isolation. Missing or ambiguous database settings remain blocked by the existing preflight; no proof of database absence is required as a separate feature. Sandbox platform implementation remains owned by `add-qa-planning-runner`; no approval bypass substitutes for it.

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
