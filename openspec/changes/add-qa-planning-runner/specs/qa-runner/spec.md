# Spec Delta

## Purpose

Menjalankan test yang telah direview dalam batas proses, filesystem, network, dan database yang dapat dibuktikan serta melaporkan hasil dan efeknya secara jujur.

## ADDED Requirements

### Requirement: Setiap run memerlukan consent dan ringkasan command
Runner MUST menampilkan manifest command, working directory, endpoint, environment names, mount, network policy, output, timeout, dan efek instalasi sebelum pengguna mengizinkan eksekusi. Approval MUST terikat hash manifest yang ditinjau dan berlaku untuk satu run saja; perubahan manifest MUST membatalkan approval.

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

### Requirement: Postman setup is separately reviewed
Postman CLI setup MUST disclose detected version/configuration, install source/destination, collection, environment, endpoint, and data before explicit setup approval. Missing tools MUST be unavailable and denied approval blocked; credentials MUST NOT be stored plaintext.

#### Scenario: Setup is unavailable or declined
- **WHEN** Postman CLI is absent or the user declines installation/configuration
- **THEN** no implicit install or endpoint request occurs and no test is reported passed

### Requirement: Dependency tidak di-install tanpa izin
Runner MUST tidak mengunduh atau memasang dependency/tool kecuali langkah tersebut tercantum dalam ringkasan yang disetujui pengguna.

#### Scenario: Plan memasukkan install gate
- **WHEN** daftar command mencakup langkah install atau fetch dependency
- **THEN** runner menyatakan side effect dan network access sebelum meminta consent; tanpa approval gate tidak berjalan

### Requirement: Preview dan approval berlaku untuk semua aksi berisiko
CLI dan agent MUST meminta persetujuan eksplisit pengguna setelah preview untuk install, test, akses endpoint, dan transmisi data sebelum masing-masing aksi dijalankan. Preview MUST mencakup command atau aksi, target, data, efek, dan izin yang relevan; approval MUST terikat pada scope tersebut dan tidak boleh diberikan agent atas nama pengguna. Scope berubah atau tidak diketahui MUST membatalkan approval dan memblokir aksi sampai ditinjau ulang.

#### Scenario: Agent meminta menjalankan test atau mengirim data
- **WHEN** agent mengusulkan test, akses endpoint, install, atau transmisi data
- **THEN** sistem menampilkan preview kepada pengguna dan tidak melakukan aksi sampai pengguna menyetujui scope yang tepat

#### Scenario: Tidak ada kanal persetujuan
- **WHEN** preview atau persetujuan eksplisit pengguna tidak tersedia
- **THEN** aksi berstatus `blocked` tanpa install, test, akses endpoint, atau transmisi data

### Requirement: UAT parity CLI dan agent per OS
Setiap OS yang dinyatakan didukung MUST memiliki bukti UAT parity CLI dan agent untuk planning, preview, approval dan penolakan, perubahan scope, hasil, dan boundary yang tidak tersedia. OS atau boundary yang belum diverifikasi MUST tidak diklaim didukung dan eksekusi yang memerlukannya MUST `blocked`.

#### Scenario: Boundary OS belum diverifikasi
- **WHEN** UAT atau boundary isolasi OS yang diperlukan belum terbukti
- **THEN** sistem tidak mengklaim dukungan atau parity dan memblokir eksekusi yang memerlukan boundary tersebut
