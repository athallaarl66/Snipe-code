# Spec Delta

## Purpose

Membuat rancangan test case dan script yang dapat ditinjau dari requirement serta project scope tanpa mengeksekusi command atau target aplikasi.

## ADDED Requirements

### Requirement: Rencana QA menjaga traceability
Setiap test case MUST memiliki ID stabil, tujuan, precondition, input/aksi, hasil yang diharapkan, kategori, dan referensi requirement atau temuan sumbernya bila tersedia.

#### Scenario: Buat rencana dari requirement
- **WHEN** pengguna memilih requirement untuk dibuatkan test plan
- **THEN** sistem menghasilkan case positif/negatif yang menautkan requirement ke tujuan dan expected result

### Requirement: Perencanaan tidak menjalankan project
Operasi perencanaan dasar MUST hanya membaca scope yang dipilih dan MUST NOT menjalankan script, install tool, membuat request ke endpoint, atau mengubah file project. Integrasi AI/MCP opsional adalah tahap terpisah yang memerlukan preview dan approval eksplisit, bukan bagian dari planning dasar; tahap tersebut MUST NOT menjalankan test atau mengubah file project.

#### Scenario: Pengguna meminta test plan
- **WHEN** sistem membuat test case/script dari project
- **THEN** sistem hanya membaca input yang dibutuhkan dan project target tetap tidak berubah

### Requirement: Generated script berstatus draft sampai divalidasi
Script hasil generator MUST dilabeli draft, menyatakan adapter/version dan kebutuhan environment, serta tidak boleh disebut pass atau executed sebelum dijalankan oleh runner yang disetujui.

#### Scenario: Adapter tidak didukung
- **WHEN** pengguna meminta format script yang tidak memiliki adapter tervalidasi
- **THEN** sistem menolak generation untuk adapter tersebut dan tidak menghasilkan script seolah siap dipakai

#### Scenario: Script berhasil divalidasi secara statis
- **WHEN** generated script lulus pemeriksaan syntax/struktur yang didukung
- **THEN** sistem menandainya ready-for-review tanpa menyatakan test telah dijalankan atau lulus

### Requirement: Simpan atau export memerlukan tujuan eksplisit
Sistem MUST tidak menulis artefak QA ke project target saat merencanakan; export yang diminta MUST memakai tujuan yang disetujui pengguna.

#### Scenario: Export rencana QA
- **WHEN** pengguna memilih format dan lokasi export secara eksplisit
- **THEN** sistem hanya membuat artefak rencana pada lokasi tersebut dan melaporkan kegagalannya secara akurat

### Requirement: CLI dan agent berbagi kemampuan planning
Sistem MUST menyediakan CLI tanpa GUI dan agent dengan skills, termasuk testing skill untuk proyek pengguna, dengan kontrak planning dan policy yang sama.

#### Scenario: Planning melalui CLI dan agent
- **WHEN** pengguna memberikan input dan scope setara melalui CLI atau agent
- **THEN** keduanya menyediakan kemampuan planning dan batas read-only yang setara tanpa memerlukan GUI

### Requirement: Integrasi AI dan MCP bersifat opsional dan disetujui
AI dan MCP lokal maupun online MUST opsional; operasi dasar MUST tersedia tanpa integrasi tersebut. Sebelum akses endpoint atau pengiriman data, sistem MUST menampilkan tujuan, data, dan efek lalu memperoleh persetujuan eksplisit pengguna untuk scope yang tepat; perubahan scope MUST memerlukan persetujuan baru. Secret MUST diredaksi dan scope yang tidak diketahui MUST diblokir.

#### Scenario: Integrasi meminta data project
- **WHEN** agent atau CLI hendak mengakses endpoint AI/MCP atau mengirim data project, termasuk ke layanan lokal
- **THEN** sistem menampilkan preview tujuan dan data serta menunggu persetujuan eksplisit pengguna sebelum akses atau pengiriman; tanpa persetujuan aksi diblokir

#### Scenario: Planning tanpa integrasi
- **WHEN** pengguna tidak mengaktifkan AI atau MCP
- **THEN** planning dasar tetap tersedia tanpa koneksi atau transmisi data ke integrasi tersebut

### Requirement: Adapter QA dikembangkan bertahap
Sistem MUST merencanakan dukungan unit/regression, Playwright, Postman melalui Postman CLI, Bruno, dan Maestro secara bertahap. Setiap adapter MUST menyatakan versi dan status dukungan; adapter yang belum tervalidasi MUST tetap `unsupported`.

#### Scenario: Adapter yang belum siap diminta
- **WHEN** pengguna meminta adapter roadmap yang belum tervalidasi
- **THEN** sistem melaporkan `unsupported` dan tidak mengklaim script siap atau test lulus
