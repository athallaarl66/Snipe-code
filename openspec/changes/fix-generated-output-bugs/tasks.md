## 1. CI Generated Workflow

- [x] 1.1 Ganti `npm ci` -> `npm install` di `src/cicd.rs` (job frontend + backend NestJS)
- [x] 1.2 Tambahkan regression coverage untuk workflow Node: tidak mengandung `npm ci` dan mengandung `npm install`; test ini tidak menyatakan lint/test step lulus
- [ ] 1.3 Rekonsiliasi `npm run lint` dengan script yang benar-benar ada di setiap manifest dan verifikasi workflow secara native; defer ke `harden-generator-project-quality`

## 2. Scaffold File Integrity

- [x] 2.1 Perbaiki content `nest-cli.json` di `src/stack.rs` menjadi object JSON valid
- [x] 2.2 Tulis `backend/.env` untuk stack Laravel dengan `APP_KEY` placeholder non-empty; task ini tidak mengklaim Laravel bootable
- [x] 2.3 Hapus duplikat `vendor/` di `gitignore_content()` (`src/generator.rs`)
- [x] 2.4 Regression coverage untuk parse `nest-cli.json`, `APP_KEY` non-empty, dan satu entry `vendor/`
- [ ] 2.5 Lengkapi skeleton/Composer metadata Laravel dan jalankan `php artisan test`; defer ke `harden-generator-project-quality`

## 3. Audit Dotfile Scan

- [x] 3.1 Tambahkan `find_dotfiles(path, names)` di `src/audit.rs` untuk nama `.env` dan `.env.local`, dengan skip directory yang sama
- [x] 3.2 Panggil helper dari `check_hardcoded_secrets` untuk kedua nama tersebut
- [x] 3.3 Regression coverage untuk secret nyata dan placeholder `your-` pada `.env`
- [ ] 3.4 Tambahkan assertion line number dan excluded-directory, lalu perluas ke dotfile lain bila scope disepakati; defer coverage penuh ke stack-aware/hardening

## 4. Report Truncation Disclosure

- [x] 4.1 Di `generate_pdf`, catat jumlah finding yang terpotong dan cetak `N more finding(s) omitted - page space exceeded`
- [ ] 4.2 Tambahkan test overflow/fits dan paging atau representasi lengkap untuk semua finding; change ini hanya menjamin disclosure pada PDF satu halaman, report completeness lengkap deferred ke hardening

## 5. Blog Date & Dead Code

- [x] 5.1 Tambahkan helper tanggal runtime tanpa dependency baru di `src/generator.rs`
- [x] 5.2 Pakai tanggal runtime pada template `blog-sample-post` dan `blog-rss-feed`
- [x] 5.3 Hapus `src/ui_components.rs` dan `pub mod ui_components;` di `src/lib.rs`
- [ ] 5.4 Verifikasi nilai tanggal template sama dengan UTC date saat test; test saat ini hanya memeriksa literal lama tidak muncul dan format valid, date report ditunda ke `feat-stack-aware-audit`

## 6. Verification Status

- [ ] 6.1 Jalankan `cargo test` sebagai verifikasi implementasi; command tidak dijalankan dalam reconciliasi artifact-only ini
- [ ] 6.2 Jalankan `cargo clippy --all-targets` sebagai verifikasi implementasi; command tidak dijalankan dalam reconciliasi artifact-only ini
- [ ] 6.3 Jalankan release build dan verifikasi manual `new --dry-run`, NestJS, Laravel, workflow, serta audit `.env`; Laravel/CI/native contracts deferred ke `harden-generator-project-quality` dan tidak diklaim selesai di sini

> Tidak ada item verification yang ditandai selesai hanya dari inspeksi source. Native stack verification dan current-date equality harus punya bukti terpisah.
