# Design

## Context

Lihat `proposal.md` untuk motivasi dan `specs/audit-cli/spec.md` untuk kontrak. `run_audit_report` saat ini memindai lalu langsung menulis format pilihan ke root; `audit --json` juga tetap menulis report. `walk_files` melewati symlink, tetapi pencarian dotfile punya jalur terpisah.

## Goals / Non-Goals

**Goals:** batasi pemindaian ke root, pisahkan hasil scan dari side effect export, validasi input, dan berikan report lengkap.

**Non-Goals:** menambah aturan SAST/SCA, histori persisten, atau mengubah heuristik yang sudah ada selain pencegahan symlink.

## Decisions

- Pertahankan `audit` sebagai compatibility path selama migrasi. Tambahkan warning side effect ke stderr; jangan mengubah default export maupun exit code dalam perubahan ini.
- Tambahkan `scan [PATH] [--json]` sebagai jalur read-only. JSON berisi hasil scan, tidak menulis file.
- Tambahkan `export <scan.json> --format <md,pdf,docx> --output <DIR>`; path output diwajibkan dan validasi dilakukan sebelum menulis. Format ini mengkonsumsi hasil scan yang tersimpan, bukan scan ulang diam-diam.
- Tolak `--export` dan `--json` yang duplikat, flag tak dikenal, positional target ganda, dan nilai flag hilang sebelum file dibaca/ditulis.
- Abaikan symlink/reparse point pada penemuan file; sebelum baca, pastikan path kanonik berada di dalam root. Perlu uji race-oriented/reparse semantics pada platform CI; bila implementasi tidak dapat membuktikan batas itu, fail closed pada entry tersebut.
- Gunakan data hasil scan yang sama untuk Markdown, PDF, DOCX, dan JSON. PDF memakai pagination; test ekstrak teks memeriksa semua finding ID/nomor tercantum tanpa omission.

## Risks / Trade-offs

- [Perubahan CLI menambah surface] Pertahankan `audit` lama, help jelas, dan integration tests untuk lama/baru.
- [Path canonicalization platform-specific] Fixture symlink/reparse point di Linux dan Windows; scan entry yang tidak dapat diverifikasi sebagai unknown/skipped, bukan diikuti.
- [File hasil scan dapat diubah sebelum export] Export memvalidasi versi/schema serta menolak JSON rusak; tampilkan bahwa hasil dapat stale dan export tidak mengulang scan.
- [PDF text extraction belum membuktikan layout visual] Uji ekstraksi otomatis plus inspeksi render pada fixture panjang sebelum mengklaim layout valid.

## Migration Plan

1. Rilis command `scan` dan `export`, warning pada `audit` legacy; dokumentasi menunjukkan command pengganti.
2. Pertahankan `audit` tanpa perubahan default selama periode migrasi; tanggal penghentian belum ditetapkan, jadi jangan hapus alias dalam change ini.
3. Perubahan lanjutan yang mengubah schema result harus memperbarui pembaca export dengan version field dan test kompatibilitas.
4. Rollback: revert command baru tanpa mengubah file report/API legacy yang sudah ada.