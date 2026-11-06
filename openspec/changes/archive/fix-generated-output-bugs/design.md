## Context

Lihat `proposal.md` untuk motivasi dan batas scope. Source saat ini menunjukkan keadaan berikut:

- `src/cicd.rs:41` dan `src/cicd.rs:64` memakai `npm install`, tetapi langkah `npm run lint` tetap unconditional untuk stack yang tidak selalu memiliki script `lint`.
- `src/stack.rs:293-297` menulis `nest-cli.json` dengan object JSON yang valid.
- `src/audit.rs:85-114` menggabungkan file extension `.ts`, `.tsx`, `.js`, `.jsx`, dan `.json` dengan `find_dotfiles` untuk nama `.env` serta `.env.local` pada `src/audit.rs:216-231`. Hidden directory, `node_modules`, dan `target` tetap dilewati.
- `src/stack.rs:369-387` menulis `backend/.env` dengan `APP_KEY` placeholder. Tidak ada scaffold `bootstrap/app.php`, routes, atau kernel/application structure yang membuat output tersebut dapat dibuktikan bootable.
- `src/audit.rs:312-320` menulis disclosure saat loop PDF melewati batas `y < 50.0`; exporter tetap menggunakan satu halaman dan tidak merepresentasikan finding yang terpotong.
- `src/generator.rs:233-277` menggunakan tanggal runtime untuk blog post dan RSS. `tests/bugfix_test.rs:106-126` memeriksa bahwa tanggal tidak sama dengan literal lama dan bentuknya valid, bukan bahwa nilainya sama dengan waktu test.
- `src/lib.rs` sudah tidak mendaftarkan `ui_components`, dan file tersebut sudah dihapus pada source tree.
- Tidak ada hasil `cargo test`, Clippy, release build, atau verifikasi native yang dapat dianggap sebagai bagian dari reconciliasi artifact ini.

## Goals / Non-Goals

**Goals:**
- Batasi perubahan pada defect output yang benar-benar terlihat di source.
- Pastikan object `nest-cli.json` valid, `.env` sample punya `APP_KEY` non-empty, dan `vendor/` hanya muncul sekali.
- Tambahkan scan rekursif untuk `.env` dan `.env.local` tanpa memperluas ke semua dotfile.
- Beri tahu ketika PDF satu halaman tidak mencetak seluruh finding.
- Gunakan tanggal runtime pada template blog/RSS tanpa mengklaim test sudah membuktikan tanggal saat ini.
- Hapus modul dead code yang tidak dipakai.

**Non-Goals / Deferred:**
- Tidak mengklaim Laravel scaffold bootable atau `php artisan test` lulus. `harden-generator-project-quality` menangani skeleton Laravel, Composer metadata, environment template yang aman, dan native verification.
- Tidak memperbaiki atau memverifikasi lint/test capability pada seluruh generated CI. `harden-generator-project-quality` menangani CI yang hanya memanggil script yang ada serta build/test matrix.
- Tidak membuat PDF multi-page atau menjamin seluruh finding direpresentasikan pada PDF/DOCX. Disclosure satu halaman adalah satu-satunya kontrak change ini; report completeness lengkap deferred ke hardening.
- Tidak memperluas scan ke `.env.example`, dotfile lain, atau seluruh secret syntax. Coverage tambahan dan scope reporting lebih luas deferred ke `feat-stack-aware-audit` dan `harden-generator-project-quality`.
- Tidak mengklaim tanggal blog/RSS sudah diverifikasi terhadap UTC clock pada test. Audit report date dan cross-change date deduplication tetap di luar scope.
- Tidak menjalankan atau memperbarui command verification sebagai bagian dari artifact-only change ini.

## Decisions

### D1: Ganti hanya install command pada generated Node workflow
Ganti literal `npm ci` menjadi `npm install` pada `src/cicd.rs` untuk frontend dan NestJS backend. Scaffold tidak menghasilkan lockfile, sehingga `npm install` kompatibel dengan output saat ini. Jangan menyatakan workflow lulus secara keseluruhan: `npm run lint` masih dapat gagal untuk manifest yang tidak mendeklarasikan script tersebut. Validasi lint/typecheck/test dilakukan pada `harden-generator-project-quality`.

### D2: Perbaiki object `nest-cli.json` saja
Content `nest-cli.json` ditulis sebagai object JSON lengkap dengan `$schema`, `collection`, dan `sourceRoot`. Requirement dibatasi pada parseability file tersebut; build atau startup NestJS penuh tidak diklaim sebagai hasil change ini.

### D3: Batasi dotfile scan pada dua nama eksplisit
Gunakan helper rekursif yang menerima nama file, lalu panggil dengan `[".env", ".env.local"]` dari `check_hardcoded_secrets`. Helper memakai aturan skip yang sama dengan scan extension: hidden directory, `node_modules`, dan `target`. Jangan menyebut behavior ini sebagai coverage semua dotfile. `.env.example` dan pola secret lanjutan tetap deferred.

### D4: Disclosure PDF bukan pagination
Saat `y < 50.0`, hitung `findings.len() - i` dan cetak `N more finding(s) omitted — page space exceeded`, lalu hentikan loop. Contract hanya menyatakan exporter memberi tahu truncation; tidak menyatakan exporter membuat halaman tambahan atau bahwa DOCX ikut dipaginasi.

### D5: Tanggal runtime untuk template, bukan klaim verifikasi clock
Helper tanggal lokal di `src/generator.rs` dipakai oleh `blog-sample-post` dan `blog-rss-feed`. Regression test yang tersedia memeriksa literal lama tidak muncul dan format tanggal valid. Test tidak mengunci timezone, tidak membandingkan output dengan clock, dan tidak membuktikan `chrono_placeholder()` pada audit sudah berubah; boundary itu tetap milik `feat-stack-aware-audit`.

### D6: Hapus modul dead code
Hapus `src/ui_components.rs` dan `pub mod ui_components;` dari `src/lib.rs` karena tidak ada call site. Jika fitur shadcn diperlukan, change terpisah harus mendefinisikan scope dan integrasinya.

## Risks / Trade-offs

- [Generated Node workflow masih dapat gagal pada lint/test yang tidak dikonfigurasi] → jangan memperluas claim; capability-aware CI dan native verification deferred ke hardening.
- [`.env` Laravel adalah sample, bukan environment aman atau bootable contract] → dokumentasikan batasnya dan defer skeleton/test verification.
- [PDF tetap memotong data pada satu halaman] → disclosure mencegah truncation diam-diam; pagination dan report completeness lengkap tetap requirement hardening.
- [Dotfile coverage sempit] → lakukan ekspansi hanya setelah scope, placeholder handling, dan test matrix disepakati.
- [Date helper dapat berbeda dari format audit locale/timezone] → verifikasi tanggal runtime dan shared helper ditunda ke change stack-aware/hardening.

## Migration Plan

1. Pertahankan source change yang sudah terlihat untuk install command, JSON NestJS, environment sample, duplicate gitignore, dotfile `.env`/`.env.local`, PDF disclosure, tanggal template, dan penghapusan dead code.
2. Gunakan regression tests yang benar-benar ada; jangan mengubahnya menjadi klaim bootability, lint/test success, atau current-date equality.
3. Tunda Laravel skeleton, capability-aware CI, native matrix, report pagination/completeness, dotfile breadth, dan full date verification ke `harden-generator-project-quality` atau `feat-stack-aware-audit` sesuai scope change tersebut.
4. Tidak ada migrasi data. Rollback dilakukan dengan revert perubahan source; generated output tidak dimigrasikan.
