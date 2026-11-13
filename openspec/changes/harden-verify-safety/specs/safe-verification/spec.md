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

#### Scenario: Generated-project harness scope
- **WHEN** binary `verify-generated` membuat project dalam workspace sementara buatannya sendiri
- **THEN** harness boleh menyetujui gate otomatis hanya untuk project itu, membersihkan environment variable DB host/target yang dikenali sebelum child process, tidak menerima path arbitrer, dan tetap menjalankan preflight atas konfigurasi eksplisit yang ditemukan

#### Scenario: Workspace sementara bukan sandbox
- **WHEN** generated-project harness menjalankan command di workspace sementaranya
- **THEN** dokumentasi dan laporan menyatakan workspace membatasi lokasi project buatan saja; ia tidak membatasi network, endpoint, atau konfigurasi DB hardcoded/tidak terdeteksi dan tidak boleh disebut sandbox atau bukti isolasi

### Requirement: Gate unavailable bukan lulus
Evaluator MUST membedakan `passed`, `failed`, `unavailable`, `blocked`, dan `not-applicable`; hasil keseluruhan MUST tidak menyatakan lulus penuh jika gate applicable unavailable atau blocked.

#### Scenario: Satu gate lulus dan gate lain unavailable
- **WHEN** sekurangnya satu gate berhasil tetapi gate applicable lain unavailable
- **THEN** report mempertahankan status unavailable dan aggregate tidak menyatakan complete-pass

#### Scenario: Semua gate unavailable atau blocked
- **WHEN** tidak ada gate yang berhasil dijalankan
- **THEN** command keluar non-sukses dan menjelaskan gate yang tidak berjalan

#### Scenario: Preflight menolak target
- **WHEN** preflight keselamatan mengidentifikasi target database produksi, remote, atau ambigu
- **THEN** sistem tidak menjalankan command gate terkait dan mengembalikan status `blocked` beserta alasan tanpa membocorkan credential

### Requirement: Database bersama tidak disentuh tanpa verifikasi target dan izin
Gate yang dapat menjalankan test atau install hook MUST memblokir database remote/production-like atau konfigurasi DB eksplisit yang ambigu sebelum spawn. Nama DB yang mengandung kata `test` saja tidak membuktikan isolasi. Preflight MUST membedakan konfigurasi ambigu dari kondisi tanpa konfigurasi database terdeteksi: kondisi tanpa konfigurasi menghasilkan peringatan target tidak diketahui dan memerlukan persetujuan eksplisit pengguna untuk CLI umum; tanpa persetujuan gate tetap `blocked`. Pengecualian hanya untuk binary `verify-generated`: otorisasi internal terbatas pada workspace yang dibuat harness dan project yang baru digenerate olehnya. Otorisasi ini bukan approval CLI umum dan tidak membuktikan DB tidak ada atau aman; preview/laporan tetap menyatakan target tidak terdeteksi.

#### Scenario: Test menunjuk database produksi atau remote
- **WHEN** preflight menemukan target produksi atau remote
- **THEN** sistem menolak eksekusi sebelum proses child dimulai dan tidak mengubah database

#### Scenario: Konfigurasi database ambigu atau bertentangan
- **WHEN** environment atau konfigurasi memberi target DB ambigu, tidak terbaca, atau bertentangan
- **THEN** sistem mengembalikan blocked dengan alasan tanpa mencoba test atau install hook, dan tidak menawarkan approval untuk melewatinya

#### Scenario: Tidak ada konfigurasi database terdeteksi
- **WHEN** preflight tidak menemukan konfigurasi database apa pun pada proyek
- **THEN** CLI umum memperingatkan target tidak terdeteksi, tidak menyatakan database tidak ada atau aman, dan hanya melanjutkan setelah approval eksplisit atas preview yang memuat peringatan itu; `verify-generated` boleh menggunakan otorisasi internal hanya pada project hasil generate di workspace miliknya sendiri dan tetap menampilkan status target tidak terdeteksi
