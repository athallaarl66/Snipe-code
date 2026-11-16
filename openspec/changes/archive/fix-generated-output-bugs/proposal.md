## Why

Source review menunjukkan beberapa perbaikan sudah ada, tetapi kontrak yang lebih luas belum terpenuhi. Workflow generated sudah memakai `npm install`, tetapi masih memaksa `npm run lint` yang tidak semua `package.json` deklarasikan. `nest-cli.json` sudah valid JSON. Scan secret hanya menangani `.env` dan `.env.local`, bukan semua dotfile. Generator Laravel menulis `.env` dengan `APP_KEY` placeholder, tetapi belum memiliki bootstrap Laravel yang diperlukan untuk boot atau menjalankan `php artisan test`. PDF sekarang memberi tahu jumlah finding yang terpotong, tetapi tetap hanya memakai satu halaman. Tanggal blog/RSS dihitung saat generation, tetapi test yang ada hanya menolak nilai hardcoded dan tidak membandingkan nilai dengan tanggal saat ini.

Klaim lama tentang seluruh workflow CI lulus, Laravel bootable, laporan PDF lengkap, dotfile coverage luas, serta verifikasi `cargo test`/Clippy/build/manual tidak dapat dipertahankan dari bukti saat ini. Requirement yang belum terpenuhi dicatat sebagai deferred di `harden-generator-project-quality`, bukan ditandai selesai di change ini.

## What Changes

- **CI install command**: Ganti `npm ci` menjadi `npm install` pada job Node karena scaffold tidak menulis lockfile. Pergantian ini tidak menjamin job lint/test lulus; capability-aware CI tetap deferred ke `harden-generator-project-quality`.
- **JSON NestJS**: Tulis `nest-cli.json` sebagai object JSON yang valid.
- **Audit dotfile**: Tambahkan pemindaian rekursif untuk nama file `.env` dan `.env.local` pada direktori yang tidak dikecualikan. Dotfile lain dan cakupan secret yang lebih luas deferred ke hardening.
- **Laravel environment**: Tetap tulis `backend/.env` dengan `APP_KEY` placeholder non-empty. Ini bukan bukti project dapat boot atau `php artisan test` dapat berjalan; skeleton dan verifikasi Laravel lengkap deferred ke hardening.
- **PDF truncation disclosure**: Saat exporter PDF kehabisan ruang pada halaman, cetak jumlah finding yang tidak tercetak. Ini bukan pagination atau jaminan seluruh finding direpresentasikan di semua format.
- **Blog/RSS date**: Gunakan tanggal runtime untuk sample post dan RSS. Test yang ada hanya memeriksa tanggal tidak hardcoded serta bentuknya; pembuktian tanggal UTC saat ini dan tanggal laporan tetap menjadi pekerjaan change lain.
- **Dead code**: Hapus modul `ui_components` yang tidak dipakai.
- **Gitignore**: Hapus duplikasi `vendor/`.

## Capabilities

### New Capabilities
- `ci-generated-workflow`: Generated Node workflow memilih `npm install` saat scaffold tidak menghasilkan lockfile; validasi lint/test tetap di luar scope.
- `scaffold-file-integrity`: Memastikan `nest-cli.json` yang dipilih valid, environment sample Laravel memiliki placeholder, dan `.gitignore` tidak menduplikasi `vendor/`. Bootability seluruh stack tidak diklaim.
- `audit-dotfile-scan`: Scan secret untuk `.env` dan `.env.local` secara rekursif di luar path yang dikecualikan.
- `report-complete`: Dibatasi pada disclosure jumlah finding yang terpotong oleh exporter PDF satu halaman; kelengkapan report dan pagination deferred ke hardening.

### Modified Capabilities
_(none)_

## Impact

- **Code**: Perubahan source yang sudah ada berada di `src/cicd.rs`, `src/stack.rs`, `src/audit.rs`, `src/generator.rs`, dan `src/lib.rs`; contract artifact ini hanya mengukur behavior yang terlihat.
- **Tests**: `tests/bugfix_test.rs` memuat regression check untuk install command, `nest-cli.json`, `APP_KEY`, duplicate `vendor/`, `.env` secret/placeholder, serta date blog/RSS. Test tidak membuktikan bootability atau seluruh workflow.
- **Verification**: Tidak ada klaim baru bahwa `cargo test`, Clippy, release build, atau verifikasi manual native sudah dijalankan dalam reconciliasi artifact ini.
- **Dependencies**: Tidak ada dependency baru yang diklaim.
- **Breaking**: Modul internal `ui_components` dihapus; tidak ada jaminan kompatibilitas API baru untuk modul tersebut.
