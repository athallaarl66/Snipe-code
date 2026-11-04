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
Operasi perencanaan MUST hanya membaca scope yang dipilih dan MUST NOT menjalankan script, install tool, membuat request ke endpoint, atau mengubah file project.

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
