# Spec Delta

## Purpose

Mendeteksi pola keamanan statis pada source yang cakupannya dinyatakan, dengan bukti dan ketidakpastian agar temuan kandidat tidak disalahartikan sebagai bukti exploit atau jaminan keamanan.

## ADDED Requirements

### Requirement: Rule SAST menyatakan dukungan dan evaluasi
Setiap rule SAST MUST memiliki ID/version dan MUST melaporkan status evaluated, not-applicable, unknown, atau error untuk scope yang diproses.

#### Scenario: Source memakai bahasa tidak didukung
- **WHEN** scan menemukan source dalam bahasa yang belum didukung oleh rule pack aktif
- **THEN** hasil menandai scope tersebut unknown/not evaluated dan tidak menyatakan clean

### Requirement: Temuan kandidat memuat bukti terbatas
Temuan SAST MUST menyatakan rule ID/version, severity, confidence, lokasi, bukti yang cukup untuk ditinjau, dan caveat bahwa temuan adalah kandidat.

#### Scenario: Rule menemukan jalur source-to-sink yang didukung
- **WHEN** input tak tepercaya mencapai sink yang dimodelkan dan rule mengonfirmasi aliran dalam scope
- **THEN** hasil memuat rule, lokasi source/sink yang tersedia, confidence, dan label kandidat

### Requirement: Sanitization dan konteks dievaluasi oleh fixture
Rule SAST MUST membedakan contoh berisiko dari contoh aman yang relevan; rekomendasi MUST tidak mengubah source secara otomatis.

#### Scenario: Query memakai parameterisasi
- **WHEN** fixture menjalankan query dengan parameter terikat, bukan konkatenasi input
- **THEN** rule SQLi tidak melaporkannya sebagai temuan terkonfirmasi

#### Scenario: HTML output sudah di-escape
- **WHEN** fixture memasukkan nilai setelah escaping yang dikenal rule
- **THEN** rule XSS tidak menandai jalur itu sebagai sink tanpa perlindungan

### Requirement: Saran fix hanya rekomendasi
Saran perbaikan MUST ditampilkan sebagai usulan yang dapat ditinjau dan sistem MUST tidak menulis source saat scan atau saat menampilkan saran.

#### Scenario: Temuan memiliki saran perbaikan
- **WHEN** pengguna meminta saran untuk temuan
- **THEN** sistem menampilkan saran tanpa mengubah file target
