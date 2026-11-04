# Tasks

## 1. Cegah pembacaan symlink di luar root

- [ ] 1.1 Ubah discovery dotfile supaya menolak symlink/reparse point sebelum cabang file/direktori; verifikasi unit test untuk symlink `.env` dan `.env.local` di POSIX.
- [ ] 1.2 Tegakkan pemeriksaan root kanonik sebelum pembacaan; verifikasi fixture yang menunjuk file di luar root tidak menghasilkan konten atau temuan pada report/JSON.
- [ ] 1.3 Tambah test Windows untuk junction/reparse point jika CI runner mendukung; bila tidak, pastikan behavior fail-closed terdokumentasi dan test gated tidak memberi false pass.
- [ ] 1.4 Perbarui dokumentasi audit tentang cakupan symlink dan batas hasil; verifikasi seluruh klaim cocok dengan test dan source.

## 2. Pisahkan scan dari export

- [ ] 2.1 Tambah operasi `scan [PATH] [--json]` tanpa file write; verifikasi CLI integration test memastikan JSON valid dan target proyek tidak berubah.
- [ ] 2.2 Tambah operasi export hasil scan berversi ke `--output` eksplisit; verifikasi hanya format/path yang diminta tertulis dan input schema rusak ditolak.
- [ ] 2.3 Tambah test error untuk tujuan tidak writable, format tidak dikenal, serta memastikan kegagalan tidak dilaporkan sebagai export sukses.
- [ ] 2.4 Dokumentasikan command baru, schema hasil sementara, dan perbedaan scan/export; verifikasi contoh command dapat diparse dari help/docs.

## 3. Kunci kompatibilitas dan kontrak report

- [ ] 3.1 Tegakkan parser audit ketat untuk option tak dikenal, positional ganda, option duplikat, dan nilai hilang; verifikasi usage exit serta tidak ada read/write sebelum gagal.
- [ ] 3.2 Pertahankan bentuk `audit` lama sementara dengan warning side effect di stderr; verifikasi test legacy untuk format, JSON stdout, dan exit code lama.
- [ ] 3.3 Tambah pagination PDF dan pemeriksaan ekstraksi teks pada fixture banyak temuan; verifikasi jumlah/ID finding sama dengan hasil JSON kanonik dan lakukan inspeksi render sebelum klaim layout selesai.
- [ ] 3.4 Koreksi README dan feature docs tentang side effect legacy, `--json`, scope, serta jalur pengganti; verifikasi pencarian tidak menemukan klaim audit legacy sebagai read-only.

## Workflow follow-up

- Arsipkan `harden-audit-cli` sebelum `add-scan-history`; change berikutnya memakai kontrak hasil dan scan/export dari capability ini.
