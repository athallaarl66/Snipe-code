# Spec Delta

## Purpose

Menjalankan quality gate dengan status hasil yang jujur dan pengamanan eksplisit sebelum command eksternal dapat mengubah proyek, memakai network, atau menjangkau database.

## ADDED Requirements

### Requirement: Target dan efek gate terlihat sebelum eksekusi
Sebelum menjalankan gate, sistem MUST menyatakan command, direktori kerja, dan efek yang diketahui; pengguna MUST dapat membatasi gate yang dipilih.

#### Scenario: Gate dipilih secara eksplisit
- **WHEN** pengguna menjalankan gate pada proyek yang dikenali
- **THEN** sistem menampilkan command dan direktori kerja sebelum menjalankan gate

#### Scenario: Preflight menolak target
- **WHEN** preflight keselamatan mengidentifikasi target database produksi atau ambigu
- **THEN** sistem tidak menjalankan command gate terkait dan mengembalikan status `blocked` beserta alasan tanpa membocorkan credential

### Requirement: Gate unavailable tidak dihitung sebagai lulus
Evaluator MUST membedakan `passed`, `failed`, `unavailable`, `blocked`, dan `not-applicable`; command keseluruhan MUST tidak menyatakan semua gate lulus jika gate wajib unavailable atau blocked.

#### Scenario: Satu gate lulus dan gate lain unavailable
- **WHEN** run mengeksekusi sekurangnya satu gate dengan sukses tetapi gate applicable lain berstatus unavailable
- **THEN** laporan mempertahankan status unavailable dan status keseluruhan tidak menyatakan coverage lengkap atau lulus penuh

#### Scenario: Tidak ada gate yang bisa dieksekusi
- **WHEN** semua gate applicable berstatus unavailable atau blocked
- **THEN** command mengembalikan status keseluruhan non-sukses dan menjelaskan gate yang tidak berjalan

### Requirement: Efek samping dan limitasi gate terdokumentasi
Setiap gate MUST melaporkan efek dependency, filesystem, atau network yang diketahui, dan dokumentasi MUST menjelaskan perubahan yang mungkin dibuat pada project tree.

#### Scenario: Gate menjalankan dependency installation
- **WHEN** gate menjalankan tool yang dapat mengunduh dependency atau mengubah manifest/lockfile
- **THEN** laporan dan help/docs menyatakan efek yang diketahui sebelum command dijalankan

### Requirement: Database bersama tidak disentuh tanpa verifikasi target dan izin
Gate yang dapat menjalankan test atau install hook MUST memblokir database produksi/remote/ambigu apabila konfigurasi targetnya tidak bisa dipastikan aman; nama DB yang mengandung kata `test` saja tidak membuktikan isolasi.

#### Scenario: Test menunjuk database produksi atau remote
- **WHEN** preflight menemukan target produksi atau remote
- **THEN** sistem menolak eksekusi sebelum proses child dimulai dan tidak mengubah database

#### Scenario: Target database tidak dapat ditentukan
- **WHEN** environment atau konfigurasi memberi target DB ambigu/tidak terbaca
- **THEN** sistem mengembalikan blocked dengan alasan tanpa mencoba test atau install hook
