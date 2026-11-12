# Design

## Context

Lihat proposal dan delta `qa-planning` serta `qa-runner`. Saat ini `verify` menjalankan tool proyek secara langsung dan mengwarisi environment. Target OS SNIPE-CODE lintas platform; tidak ada sandbox, model QA, atau consent store sekarang.

## Goals / Non-Goals

**Goals:** pisahkan plan dari run; default read-only; setiap command run ditinjau dan diotorisasi; block bila target DB atau isolasi tidak bisa dibuktikan.

**Non-Goals:** GUI, AI/MCP wajib, automatic fix, atau menganggap `verify` lama sudah sandboxed. Tidak menjalankan target tak dikenal sebagai bagian perencanaan.

## Decisions

- `qa plan` dasar hanya menghasilkan case dan draft script dari file yang dipilih, tanpa install, endpoint request, child process, atau menulis target. Integrasi AI/MCP hanya sebagai tahap opt-in terpisah setelah preview dan approval, bukan perilaku planning dasar; tahap ini tidak menjalankan test atau menulis target. Export rancangan hanya ke destination eksplisit.
- Representasikan tiap case dengan ID, requirement/source refs, category, precondition, input, expected result, adapter/version dan status (`draft`, `ready-for-review`, `unsupported`). Validasi syntax tidak berarti sudah dijalankan.
- Runner memakai manifest command immutable untuk approval. Preview canonical command, cwd, target URL/DB identity redacted, environment names, mounts, filesystem/network capabilities, output path, timeout dan install effect. Hash manifest; approval berlaku hanya untuk hash yang sama dan satu run.
- CLI dan agent/skills memakai policy serta manifest yang sama; agent tidak boleh menyetujui atas nama pengguna. Tampilkan preview lalu minta approval eksplisit dari pengguna sebelum setiap aksi berisiko. Tanpa kanal CLI/agent yang dapat menampilkan preview dan menerima consent pengguna, aksi berstatus blocked. Tidak ada approval lewat default flag atau konfirmasi tersimpan global.
- AI dan MCP lokal/online opsional, dinonaktifkan sampai dipilih pengguna; operasi dasar tetap tersedia tanpa keduanya. Sebelum koneksi endpoint atau transmisi data apa pun, preview penyedia, tujuan, data yang akan dikirim, efek dan batas akses; minta persetujuan eksplisit untuk manifest tersebut. Perubahan tujuan/data memerlukan preview dan approval baru; redaksi secret dan fail closed bila scope tidak diketahui. Local MCP/AI tidak dikecualikan dari aturan ini.
- Implement sandbox adapter per OS yang menyatakan capability filesystem/process/network. Jangan mengklaim isolasi hanya karena subprocess berjalan di temp directory. Bila platform adapter tidak dapat membatasi process/token, mounts, network dan secrets sesuai manifest, jangan spawn command; return blocked.
- DB gate: identifikasi dari konfigurasi efektif dengan redaksi. Deny produksi/remote/unknown secara default. Disposable DB hanya bila identity dan boundary/container/credentials dibuktikan; tidak pernah menjalankan reset/drop/cleanup di luar sandbox yang dibuat khusus run. Jangan percaya kata `test` pada nama.
- Semua dependency install/hook adalah command terpisah yang ikut preview dan approval, bukan implisit. Logs scrub token/secret sebelum persistence; cancellation tidak membersihkan file di luar workspace run.
- Perubahan dijalankan bertahap: case planning dahulu, lalu script generation/validation, kemudian pilot runner untuk satu OS/capability matrix yang ditentukan. Jangan membuka semua adapters di satu release.
- Roadmap adapter: mulai Postman CLI untuk proyek pengguna, lalu Playwright untuk web, Bruno untuk API, Maestro untuk mobile, serta unit/regression sesuai framework dan versi yang dipilih, masing-masing setelah kontrak, fixture, izin dan boundary OS siap. Urutan setelah pilot dapat disesuaikan berdasarkan kesiapan; format yang belum siap tetap `unsupported`. Testing skill bekerja pada proyek pengguna dan menggunakan adapter/policy yang sama, bukan instalasi atau eksekusi terselubung.
- Uji kontrak CLI dan agent untuk planning, preview, approval, perubahan manifest, hasil, dan boundary yang tidak tersedia. UAT manusia per OS dicatat terpisah bila tersedia; jangan mengklaim parity yang belum diuji.

## CR alignment

See `../../CR-ALIGNMENT.md`. Postman CLI is the first adapter pilot; the unit/regression task defines its subsequent framework slice, not an earlier competing pilot. Setup preview and approval are separate from run approval. Full M5 provider and service-specific connector runtime is deferred pending explicit scope and security contracts; optional planning integration requirements do not claim those services exist.

## Risks / Trade-offs

- [Sandbox universal mahal dan beda OS] Capability matrix per OS; unsupported = blocked.
- [DB preflight tidak dapat membuktikan target] Fail closed; minta disposable DB eksplisit.
- [Approval stale setelah plan berubah] Hash semua input/permission; setiap perubahan minta consent baru.
- [Script generator menghasilkan test salah] Label draft; adapter-specific fixtures dan review manusia.
- [Log membocorkan secret] Redaksi sebelum display/persistence dan canary tests.

## Migration Plan

1. Rilis `qa plan` tanpa runner; tidak memengaruhi command existing.
2. Definisikan format manifest/approval; uji policy dengan runner fake tanpa spawn process.
3. Implement satu sandbox adapter setelah threat model dan OS capability assessment disetujui; tidak ada implicit fallback.
4. `verify` legacy tetap didokumentasikan sebagai command yang dapat mengubah project; terapkan guard perubahan sebagai bagian dari change safety terpisah.
5. Jika isolasi gagal, fallback hanya ke `blocked`/planning-only; tidak ke execution host tanpa sandbox.
