# Spec Delta

## Purpose

Menjalankan quality gate dengan status hasil yang jujur dan pengamanan eksplisit sebelum command eksternal dapat mengubah proyek, memakai network, atau menjangkau database.

## ADDED Requirements

### Requirement: Target dan efek gate terlihat sebelum eksekusi
Sebelum menjalankan gate, sistem MUST menampilkan preview command, direktori kerja, target, data, dan efek yang diketahui; pengguna MUST dapat membatasi gate yang dipilih. Install, test, akses endpoint, dan transmisi data MUST memerlukan persetujuan eksplisit pengguna atas scope yang ditampilkan sebelum aksi dilakukan; scope berubah atau tidak diketahui MUST membatalkan approval dan memblokir aksi sampai ditinjau ulang.

#### Scenario: Gate dipilih secara eksplisit
- **WHEN** pengguna menjalankan gate pada proyek yang dikenali
- **THEN** sistem menampilkan preview dan memperoleh persetujuan eksplisit pengguna sebelum menjalankan gate yang melakukan install, test, akses endpoint, atau transmisi data

#### Scenario: Approval tidak tersedia atau scope berubah
- **WHEN** pengguna belum menyetujui preview atau scope aksi berubah setelah persetujuan
- **THEN** gate terkait berstatus `blocked` sampai preview dan persetujuan eksplisit untuk scope aktual tersedia

#### Scenario: Generated-project CI menjalankan fixture terisolasi
- **WHEN** binary `verify-generated` membuat project di workspace sementara baru, hanya menjalankan manifest hasil generate, lalu menghapus workspace tersebut
- **THEN** harness boleh menyetujui scope otomatis tanpa approval pengguna, tetapi tidak boleh menerima path project arbitrer atau melewati preflight target remote/production yang terdeteksi

#### Scenario: Database test palsu
- **WHEN** nama DB mengandung `test`, tetapi target tidak terbukti lokal dan aman
- **THEN** sistem mengklasifikasikan target unknown/remote dan memblokir sebelum child process berjalan

### Requirement: Database absence and isolation are distinct
Preflight MUST distinguish proven absence of database use from missing configuration. Missing or ambiguous configuration MUST remain blocked. A local host, test-like name, approval, or temporary project folder MUST NOT be treated as proof of database isolation.

#### Scenario: No database use is established
- **WHEN** supported project inspection establishes and records absence of DB use for the selected gate
- **THEN** only DB preflight is not-applicable; gate execution still requires its normal approval and safety checks

#### Scenario: Local shared database or missing settings
- **WHEN** DB settings are missing or a local DB lacks isolation evidence
- **THEN** risky execution remains blocked before spawn, even if approval is offered

### Requirement: Gate unavailable bukan lulus
Evaluator MUST membedakan `passed`, `failed`, `unavailable`, `blocked`, dan `not-applicable`; hasil keseluruhan MUST tidak menyatakan lulus penuh jika gate applicable unavailable atau blocked.

#### Scenario: Satu gate lulus dan gate lain unavailable
- **WHEN** sekurangnya satu gate berhasil tetapi gate applicable lain unavailable
- **THEN** report mempertahankan status unavailable dan aggregate tidak menyatakan complete-pass

#### Scenario: Semua gate unavailable atau blocked
- **WHEN** tidak ada gate yang berhasil dijalankan
- **THEN** command keluar non-sukses dan menjelaskan gate yang tidak berjalan

#### Scenario: Preflight menolak target
- **WHEN** preflight keselamatan mengidentifikasi target database produksi atau ambigu
- **THEN** sistem tidak menjalankan command gate terkait dan mengembalikan status `blocked` beserta alasan tanpa membocorkan credential

### Requirement: Database bersama tidak disentuh tanpa verifikasi target dan izin
Gate yang dapat menjalankan test atau install hook MUST memblokir database produksi/remote/ambigu apabila konfigurasi targetnya tidak bisa dipastikan aman; nama DB yang mengandung kata `test` saja tidak membuktikan isolasi.

#### Scenario: Test menunjuk database produksi atau remote
- **WHEN** preflight menemukan target produksi atau remote
- **THEN** sistem menolak eksekusi sebelum proses child dimulai dan tidak mengubah database

#### Scenario: Target database tidak dapat ditentukan
- **WHEN** environment atau konfigurasi memberi target DB ambigu/tidak terbaca
- **THEN** sistem mengembalikan blocked dengan alasan tanpa mencoba test atau install hook
