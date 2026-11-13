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

### Requirement: Status konfigurasi database dan batas isolasi dinyatakan jujur
Runner MUST memblokir sebelum spawn bila konfigurasi DB eksplisit remote/production-like atau ambigu/tidak dapat diurai; approval MUST NOT melewati blokir ini. Bila konfigurasi DB tidak terdeteksi, manifest MUST menyatakan target unknown, memberi peringatan, dan meminta approval eksplisit CLI; tanpa approval status `blocked`. Approval tidak membuktikan database tidak ada atau aman. Bersihkan environment DB host/target yang dikenali sebelum child process. Temp workspace bukan sandbox: runner MUST NOT mengklaim isolasi filesystem, proses, network, endpoint, atau terhadap DB hardcoded/tidak terdeteksi tanpa boundary yang benar-benar ditegakkan. Nama `test` tidak membuktikan isolasi.

#### Scenario: Target database produksi atau remote
- **WHEN** preflight mengidentifikasi DB produksi atau remote yang tidak diizinkan
- **THEN** command tidak dimulai, tak ada koneksi/penulisan DB dilakukan, dan laporan berstatus blocked

#### Scenario: Database target ambigu
- **WHEN** konfigurasi efektif tidak cukup untuk membuktikan target DB aman
- **THEN** runner fail-closed sebelum proses dibuat dan meminta target yang dapat diverifikasi

#### Scenario: Tidak ada konfigurasi database terdeteksi
- **WHEN** runner tidak menemukan konfigurasi database pada target
- **THEN** manifest menyatakan target tidak terdeteksi, run hanya dimulai setelah persetujuan eksplisit pengguna, dan laporan tidak mengklaim database aman atau tidak ada

### Requirement: Batas eksekusi dan isolasi dilaporkan akurat
Runner MUST membedakan host execution dengan sandbox yang boundary filesystem/process/network-nya benar-benar ditegakkan. Temporary workspace saja MUST NOT disebut sandbox. Host execution dapat berjalan hanya dengan approval scope-bound, preflight DB, dan pembersihan environment target DB yang dikenali; laporan MUST menyebutnya host execution dan mengungkapkan keterbatasan yang diketahui. Mode yang secara eksplisit meminta boundary isolasi yang tidak tersedia MUST berstatus `blocked`.

#### Scenario: Run disetujui pada host tanpa sandbox
- **WHEN** pengguna menyetujui manifest yang secara jelas menyebut host execution dan preflight DB tidak menemukan konfigurasi eksplisit yang harus diblokir
- **THEN** runner menjalankan hanya command dalam scope, membersihkan environment DB host/target yang dikenali, dan tidak mengklaim isolasi dari network, filesystem host, endpoint, atau konfigurasi DB hardcoded

#### Scenario: Mode sandbox diminta tetapi boundary tidak tersedia
- **WHEN** run meminta boundary filesystem/process/network yang platform tidak dapat tegakkan
- **THEN** mode sandbox berstatus `blocked` dan tidak diam-diam beralih ke host execution

#### Scenario: Workspace sementara
- **WHEN** command dijalankan dari workspace sementara yang dibuat runner
- **THEN** workspace membatasi lokasi file yang dikelola runner saja dan tidak membuktikan isolasi command dari host atau network.

### Requirement: Hasil run mencatat status dan efek
Run MUST merekam exit status, timeout/cancel, gate yang skipped/unavailable/blocked, log yang sudah disanitasi, serta artifact yang dibuat.

#### Scenario: Tool tidak tersedia atau run timeout
- **WHEN** executable tidak tersedia atau batas waktu habis
- **THEN** hasil menyatakan unavailable/timeout dan tidak melaporkan test pass

#### Scenario: Run gagal atau dibatalkan
- **WHEN** child process gagal atau pengguna membatalkan run
- **THEN** runner merekam status dan artefak yang sudah dibuat tanpa menghapus file di luar output/workspace yang secara eksplisit dikelolanya; workspace sementara bukan jaminan isolasi

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
Sistem MUST menguji kontrak CLI dan agent untuk planning, preview, approval, perubahan scope, hasil, dan boundary yang tidak tersedia. UAT manusia dicatat terpisah bila tersedia; OS atau boundary yang belum diverifikasi tidak diklaim didukung dan eksekusi yang memerlukannya tetap `blocked`.

#### Scenario: Boundary OS belum diverifikasi
- **WHEN** boundary isolasi OS yang diperlukan belum terbukti
- **THEN** sistem tidak mengklaim dukungan atau parity dan memblokir eksekusi yang memerlukan boundary tersebut
