# Spec Delta

## Purpose

Menjalankan test yang telah direview dalam batas proses, filesystem, network, dan database yang dapat dibuktikan serta melaporkan hasil dan efeknya secara jujur.

## ADDED Requirements

### Requirement: Setiap run memerlukan consent dan ringkasan command
Runner MUST menampilkan command, working directory, endpoint, environment names, mount, network policy, dan output yang diminta sebelum pengguna mengizinkan eksekusi.

#### Scenario: Pengguna menyetujui run yang ditinjau
- **WHEN** pengguna menyetujui ringkasan run yang tidak berubah
- **THEN** runner menjalankan hanya command, target, dan permission yang ditinjau serta mencatat approval

#### Scenario: Run berubah setelah review
- **WHEN** command, target, environment, mount, atau network berubah setelah ringkasan ditampilkan
- **THEN** runner membatalkan approval lama dan meminta persetujuan baru sebelum eksekusi

### Requirement: Target database tidak dipercaya diblokir
Runner MUST memblokir test/install hook yang dapat mengakses produksi/remote/ambiguous DB kecuali target disposable terisolasi dapat diverifikasi; nama database saja tidak membuktikan isolasi.

#### Scenario: Target database produksi atau remote
- **WHEN** preflight mengidentifikasi DB produksi atau remote yang tidak diizinkan
- **THEN** command tidak dimulai, tak ada koneksi/penulisan DB dilakukan, dan laporan berstatus blocked

#### Scenario: Database target ambigu
- **WHEN** konfigurasi efektif tidak cukup untuk membuktikan target DB aman
- **THEN** runner fail-closed sebelum proses dibuat dan meminta target yang dapat diverifikasi

### Requirement: Eksekusi fail closed bila isolasi tidak tersedia
Runner MUST menyatakan boundary aktual untuk platform dan MUST NOT mengklaim sandboxed/isolated jika batas filesystem, proses, atau network tidak ditegakkan.

#### Scenario: Platform tidak mendukung boundary yang diwajibkan
- **WHEN** OS tidak dapat menerapkan isolasi sesuai izin yang ditinjau
- **THEN** run berstatus blocked tanpa menjalankan command

### Requirement: Hasil run mencatat status dan efek
Run MUST merekam exit status, timeout/cancel, gate yang skipped/unavailable/blocked, log yang sudah disanitasi, serta artifact yang dibuat.

#### Scenario: Tool tidak tersedia atau run timeout
- **WHEN** executable tidak tersedia atau batas waktu habis
- **THEN** hasil menyatakan unavailable/timeout dan tidak melaporkan test pass

#### Scenario: Run gagal atau dibatalkan
- **WHEN** child process gagal atau pengguna membatalkan run
- **THEN** runner merekam status dan artefak yang sudah dibuat tanpa menghapus file di luar workspace terisolasi

### Requirement: Dependency tidak di-install tanpa izin
Runner MUST tidak mengunduh atau memasang dependency/tool kecuali langkah tersebut tercantum dalam ringkasan yang disetujui pengguna.

#### Scenario: Plan memasukkan install gate
- **WHEN** daftar command mencakup langkah install atau fetch dependency
- **THEN** runner menyatakan side effect dan network access sebelum meminta consent; tanpa approval gate tidak berjalan
