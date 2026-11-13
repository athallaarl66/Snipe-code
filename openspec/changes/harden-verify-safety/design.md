# Design

## Context

`verify` berbagi engine dengan `verify-generated`. CLI menerima path pilihan caller; CI binary membuat workspace sementara baru, generate project sendiri, verifikasi, lalu menghapus workspace. Approval harus membedakan caller ini tanpa menyediakan bypass untuk path arbitrer. Audit juga menggabungkan scan dan export; dotfile traversal punya jalur terpisah dan `audit --json` masih menulis report.

## Goals / Non-Goals

**Goals:** status agregat fail-closed untuk gate wajib yang unavailable/blocked; preflight side effect yang aman; laporan command dan target yang dapat diaudit.

**Non-Goals:** sandbox lintas OS penuh dalam change ini, tool install otomatis, menambah SAST/SCA rules, atau memakai database remote.

## Decisions

- Perluas model outcome dengan `blocked` serta status agregat yang memisahkan complete-pass, partial/unknown, failure, dan no execution. Pilih exit code partial melalui design CLI yang diuji terhadap consumer lama; jangan mengubah code diam-diam.
- Preflight membedakan tiga keadaan: konfigurasi DB eksplisit yang remote/production-like/ambigu selalu `blocked` sebelum spawn; konfigurasi DB lokal yang dapat diklasifikasi menampilkan target tersanitasi; tidak ada konfigurasi DB yang terdeteksi menghasilkan peringatan, bukan klaim `not-applicable` atau aman. CLI umum hanya melanjutkan keadaan terakhir setelah approval eksplisit pada preview yang menyatakan target tidak terdeteksi. `verify-generated` boleh memakai otorisasi internal hanya untuk project hasil generate di workspace buatannya sendiri, tetap melaporkan target tak terdeteksi, dan tidak menganggapnya aman. Tidak adanya setting bukan bukti aplikasi tidak memakai DB.
- Terapkan runner injection untuk menguji status/preflight tanpa spawn proses target. Klasifikasi target dan approval tidak membuktikan isolasi. Tidak ada sandbox lintas OS pada change ini.
- CLI `verify` menampilkan command, direktori, efek diketahui, serta target DB yang disamarkan. Minta approval eksplisit per gate; approval tidak dapat mengesampingkan preflight remote/production/ambigu dan batal jika scope berubah.
- `verify-generated` boleh menyetujui otomatis hanya untuk workspace sementara yang dibuat harness dan proyek yang baru di-generate di dalamnya; harness tidak menerima path arbitrer. Sebelum child process, hapus environment variable DB target/host yang dikenali (termasuk `DATABASE_URL`, `DB_URL`, `DB_CONNECTION`, `DB_HOST`, `DB_DATABASE`, `PGHOST`, `PGDATABASE`); konfigurasi eksplisit unsafe tetap diblokir. Workspace sementara bukan sandbox: child process tetap dapat memakai network, endpoint, atau target DB yang hardcoded / tidak terdeteksi; jangan klaim isolasi atau keamanan menyeluruh.
- Audit scan menghasilkan data terstruktur tanpa write. Export membaca hasil berversi, mensyaratkan format dan output eksplisit, memvalidasi sebelum menulis. Command `audit` legacy tetap kompatibel dengan warning side-effect.
- Root audit dikanonicalisasi; symlink/reparse point dilewati sebelum traversal dan setiap pembacaan diverifikasi berada dalam root.
- Parser menolak flag tak dikenal/duplikat, path ganda, dan nilai hilang sebelum I/O. PDF dipaginasi dan seluruh finding diverifikasi lewat ekstraksi teks.
- Scan history/SQLite tetap di luar scope.

## CR alignment boundaries

See `../../CR-ALIGNMENT.md`. Safety closure also covers template override containment, truthful PDF/parity evidence, generated Compose configuration and Go CI regressions. These are bounded corrections, not a new scaffolding product. Existing agent artifacts need immediate consent wording before the later workflow redesign. DB classification is not isolation. Explicit remote/production-like/ambiguous database configuration stays blocked and user approval cannot override it; a missing configuration is a distinct state: warning plus explicit approval, blocked without approval, and never reported as proof that no database is used. The generated harness is scoped to its own workspace with inherited DB environment variables cleared and is documented as not a sandbox (hardcoded host allowlist, real network access). Sandbox platform implementation remains owned by `add-qa-planning-runner`; no approval bypass substitutes for it.

## Risks / Trade-offs

- [Exit code baru merusak automation lama] Dokumentasikan perubahan dan sediakan mode compatibility bila diperlukan; update integration tests/CLI docs.
- [DB URL tidak selalu mengungkapkan target efektif] Konfigurasi eksplisit yang tidak terbaca atau credential opaque memicu blocked; jangan klaim aman jika hanya sebagian konfigurasi terlihat.
- [Approval pada target tidak terdeteksi] Pengguna dapat menyetujui gate padahal aplikasi memakai DB lewat konfigurasi hardcoded yang tidak diperiksa; mitigasi: peringatan eksplisit pada preview, larangan klaim aman/tidak ada, dan dokumentasi batas sumber konfigurasi yang dikenali.
- [Sandbox scope lintas platform besar] Tunda runner sandbox; laporan menamai batas aktual per OS dan menolak klaim isolated.
- [Preflight memiliki false positive] Sajikan alasan dan opsi konfigurasi eksplisit, tanpa bypass diam-diam.

## Migration Plan

1. Tambah status outcome baru secara additive untuk JSON/report dan unit tests runner palsu.
2. Perbaiki agregasi exit status, perbarui usage/help/README dan semver/release notes sebelum distribusi.
3. Tambah preflight tiga keadaan: konfigurasi eksplisit remote/production-like/ambigu diblokir; tanpa konfigurasi terdeteksi menghasilkan peringatan dan hanya berjalan dengan approval eksplisit; target lokal terklasifikasi ditampilkan tersanitasi.
4. Jika rilis bermasalah, revert policy preflight hanya melalui release rollback yang diumumkan; jangan menghapus outcome/status yang tersimpan dalam output terdahulu.
