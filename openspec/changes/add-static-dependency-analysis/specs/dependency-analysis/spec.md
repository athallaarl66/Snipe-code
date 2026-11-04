# Spec Delta

## Purpose

Menganalisis dependency yang dideklarasikan dan mencocokkan versi terhadap advisory snapshot yang dapat diidentifikasi, tanpa menyamakan data yang hilang atau usang dengan bebas kerentanan.

## ADDED Requirements

### Requirement: SCA membaca dependency input yang dinyatakan
SCA MUST menyatakan ecosystem, file manifest/lockfile yang dibaca, resolusi direct/transitive yang berhasil, dan keterbatasan format.

#### Scenario: Lockfile tidak tersedia
- **WHEN** ecosystem memerlukan lockfile untuk resolusi versi tetapi file itu tidak ada
- **THEN** hasil menandai resolusi sebagai unknown dan tidak mengklaim dependency bebas advisory

### Requirement: Advisory snapshot memiliki provenance dan waktu
Setiap hasil SCA MUST mencantumkan sumber dan versi data advisory serta waktu snapshot; data hilang, tidak tervalidasi, atau kedaluwarsa MUST menghasilkan unknown/stale.

#### Scenario: Snapshot tidak tersedia dalam mode offline
- **WHEN** pengguna menjalankan SCA offline tanpa snapshot yang cocok
- **THEN** sistem tidak membuat koneksi dan melaporkan hasil advisory sebagai unknown

#### Scenario: Snapshot kedaluwarsa
- **WHEN** timestamp snapshot melewati batas kesegaran yang dikonfigurasi
- **THEN** hasil menandai snapshot stale dan tidak menyatakan tidak ada vulnerability

### Requirement: Update advisory online opt-in
Sistem MUST tidak mengunduh/update advisory secara default; update online memerlukan persetujuan eksplisit dan validasi provenance/integrity sebelum snapshot menggantikan data aktif.

#### Scenario: Pengguna menolak update
- **WHEN** pengguna menolak consent update advisory
- **THEN** tidak ada request jaringan dan snapshot lokal tidak berubah

#### Scenario: Snapshot baru gagal validasi
- **WHEN** update menghasilkan snapshot dengan signature/schema/integrity yang gagal divalidasi
- **THEN** sistem menolak snapshot baru dan mempertahankan snapshot sebelumnya

### Requirement: Hasil dependency terhubung ke advisory
Temuan dependency MUST memuat identitas dependency, versi terdeteksi, advisory ID/sumber, severity, dan status evaluasi.

#### Scenario: Versi dependency cocok advisory
- **WHEN** versi resolved cocok rentang affected dari advisory snapshot valid
- **THEN** hasil menunjukkan dependency dan advisory terkait dengan provenance snapshot
