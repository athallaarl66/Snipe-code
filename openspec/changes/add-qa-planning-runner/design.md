# Design

## Context

Lihat proposal dan delta `qa-planning` serta `qa-runner`. Saat ini `verify` menjalankan tool proyek secara langsung dan mengwarisi environment. Target OS SNIPE-CODE lintas platform; tidak ada sandbox, model QA, atau consent store sekarang.

## Goals / Non-Goals

**Goals:** pisahkan plan dari run; default read-only; setiap command run ditinjau dan diotorisasi; block bila target DB atau isolasi tidak bisa dibuktikan.

**Non-Goals:** GUI, model AI, automatic fix, live connector, atau menganggap `verify` lama sudah sandboxed. Tidak menjalankan target tak dikenal sebagai bagian perencanaan.

## Decisions

- `qa plan` hanya menghasilkan case dan draft script dari file yang dipilih, tanpa install, endpoint request, child process, atau menulis target. Export rancangan hanya ke destination eksplisit.
- Representasikan tiap case dengan ID, requirement/source refs, category, precondition, input, expected result, adapter/version dan status (`draft`, `ready-for-review`, `unsupported`). Validasi syntax tidak berarti sudah dijalankan.
- Runner memakai manifest command immutable untuk approval. Preview canonical command, cwd, target URL/DB identity redacted, environment names, mounts, filesystem/network capabilities, output path, timeout dan install effect. Hash manifest; approval berlaku hanya untuk hash yang sama dan satu run.
- Tampilkan dua tahap UI/CLI: review/approve run, kemudian hasil. Tanpa interaksi TTY/UI yang dapat menampilkan consent, run berstatus blocked. Tidak ada approval lewat default flag atau konfirmasi tersimpan global.
- Implement sandbox adapter per OS yang menyatakan capability filesystem/process/network. Jangan mengklaim isolasi hanya karena subprocess berjalan di temp directory. Bila platform adapter tidak dapat membatasi process/token, mounts, network dan secrets sesuai manifest, jangan spawn command; return blocked.
- DB gate: identifikasi dari konfigurasi efektif dengan redaksi. Deny produksi/remote/unknown secara default. Disposable DB hanya bila identity dan boundary/container/credentials dibuktikan; tidak pernah menjalankan reset/drop/cleanup di luar sandbox yang dibuat khusus run. Jangan percaya kata `test` pada nama.
- Semua dependency install/hook adalah command terpisah yang ikut preview dan approval, bukan implisit. Logs scrub token/secret sebelum persistence; cancellation tidak membersihkan file di luar workspace run.
- Perubahan dijalankan bertahap: case planning dahulu, lalu script generation/validation, kemudian pilot runner untuk satu OS/capability matrix yang ditentukan. Jangan membuka semua adapters di satu release.

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
