# Spec Delta

## Purpose

Menentukan kontrak audit CLI yang membatasi pembacaan pada root yang dipilih, memisahkan scan dari penulisan report, dan mempertahankan perilaku lama selama migrasi.

## ADDED Requirements

### Requirement: Dotfile symlink di luar root tidak dibaca
Audit MUST menolak symlink/reparse-point dotfile sebelum membuka kontennya, dan MUST memastikan setiap file yang dibaca berada di dalam root kanonik yang dipilih.

#### Scenario: Dotfile symlink menunjuk file di luar root
- **WHEN** audit menemukan `.env` atau `.env.local` yang berupa symlink ke file di luar root
- **THEN** audit tidak membuka target dan tidak memasukkan kontennya atau temuan darinya ke report, JSON, atau log

#### Scenario: Traversal menemukan link yang keluar dari root
- **WHEN** proses traversal menemukan symlink atau reparse point yang dapat mengarah ke luar root
- **THEN** audit melewati entry tersebut dan mencatat cakupan yang dilewati tanpa mengikuti target

### Requirement: Scan tidak menulis ke proyek yang dipindai
Operasi scan MUST mengembalikan hasil terstruktur tanpa membuat atau menimpa file di root proyek.

#### Scenario: Scan tanpa export
- **WHEN** pengguna menjalankan operasi scan pada proyek
- **THEN** hasil tersedia pada output yang diminta dan file proyek tetap tidak berubah

### Requirement: Export memakai tujuan eksplisit
Operasi export MUST menulis hanya format yang diminta ke tujuan yang dipilih pengguna dan MUST melaporkan kegagalan tulis.

#### Scenario: Pengguna meminta export
- **WHEN** pengguna memilih format dan tujuan export yang valid
- **THEN** hanya artifact yang diminta dibuat pada tujuan tersebut dan hasil menyebutkan lokasi aktualnya

#### Scenario: Tujuan export tidak dapat ditulis
- **WHEN** tujuan export tidak ada atau tidak dapat ditulis
- **THEN** command mengembalikan error non-sukses dan tidak melaporkan export berhasil

### Requirement: Argumen audit ambigu ditolak
CLI MUST menolak option tidak dikenal, lebih dari satu path target, dan nilai option wajib yang hilang sebelum membaca target atau menulis file.

#### Scenario: Option audit tidak dikenal
- **WHEN** pengguna memberikan option yang tidak didukung
- **THEN** CLI menampilkan usage error, keluar dengan status usage, dan tidak membaca atau mengubah target

#### Scenario: Dua path target diberikan
- **WHEN** pengguna memberikan lebih dari satu path positional
- **THEN** CLI menampilkan usage error sebelum memulai audit

### Requirement: Perilaku audit lama tetap terdokumentasi selama migrasi
Perubahan kontrak MUST mempertahankan command, format report, dan exit behavior lama sampai migrasi/deprecation diumumkan secara eksplisit; dokumentasi MUST menyebutkan side effect legacy yang berlaku.

#### Scenario: Pengguna memakai bentuk command lama
- **WHEN** pengguna menjalankan bentuk `audit` lama selama masa kompatibilitas
- **THEN** command berperilaku sesuai kontrak lama dan secara jelas memperingatkan bahwa export menulis file ke target

#### Scenario: Pengguna memakai JSON lama
- **WHEN** pengguna menjalankan `audit --json` selama masa kompatibilitas
- **THEN** stdout tetap JSON valid dan dokumentasi menyatakan apakah report file tetap dibuat
