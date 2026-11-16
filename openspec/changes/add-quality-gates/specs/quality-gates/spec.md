# Spec Delta

## Purpose

Menghitung dan membandingkan metrik kualitas yang didefinisikan, menyatakan sumber data serta ketidakpastian, dan menerapkan quality gate tanpa menyamakan metrik hilang dengan sukses.

## ADDED Requirements

### Requirement: Metrik kualitas memiliki definisi berversi
Setiap metrik MUST menyebut definisi, unit, threshold/config version, exclusions, dan sumber input yang dipakai.

#### Scenario: Metrik dihitung
- **WHEN** pengguna menjalankan quality analysis dengan rule/config yang valid
- **THEN** hasil menyatakan nilai metrik, definisi/version, threshold, exclusions, dan cakupan file

### Requirement: Coverage hanya berasal dari report pengujian
Sistem MUST memperoleh coverage dari test coverage report yang format dan provenance-nya didukung; scan statis MUST NOT mengklaim menghitung coverage.

#### Scenario: Tidak ada coverage report
- **WHEN** pengguna meminta gate coverage tetapi tidak menyediakan report yang dapat divalidasi
- **THEN** coverage berstatus unknown dan tidak membuat nilai persentase dari scan statis

#### Scenario: Coverage report tidak valid
- **WHEN** report coverage rusak, format tidak didukung, atau path tidak cocok dengan project scope
- **THEN** sistem menyatakan error/unknown beserta alasan tanpa menerbitkan gate pass

### Requirement: Baseline dan PR delta mempertahankan status ambiguity
Perbandingan MUST menggunakan identitas rule/fingerprint berversi dan menyatakan baseline/base reference; unresolved rename, merge conflict, atau base yang hilang MUST menghasilkan unknown.

#### Scenario: Base reference tidak tersedia
- **WHEN** pengguna meminta PR delta tanpa base commit/reference yang dapat dibaca
- **THEN** hasil delta unknown dan sistem tidak menyebut finding sebagai resolved atau baru

#### Scenario: Fingerprint berubah akibat rule version
- **WHEN** baseline dan scan memakai versi rule/fingerprint yang berbeda
- **THEN** sistem menandai matching sebagai perlu rekonsiliasi, bukan menyatakan finding selesai

### Requirement: Gate memisahkan pass, fail, unknown, dan not-applicable
Quality gate MUST mengevaluasi tiap metric terhadap threshold berversi dan tidak boleh menganggap unknown sebagai pass.

#### Scenario: Metric wajib unknown
- **WHEN** gate membutuhkan metric yang tidak tersedia atau tidak dapat divalidasi
- **THEN** gate berstatus unknown, kecuali kebijakan eksplisit menandainya fail; gate tidak pass secara default

#### Scenario: Semua metric wajib memenuhi threshold
- **WHEN** seluruh metric wajib berhasil dievaluasi dan memenuhi threshold aktif
- **THEN** gate berstatus pass dan melaporkan config version serta metric yang dievaluasi

### Requirement: Duplication dan maintainability transparan
Metrik maintainability dan duplication MUST menyatakan algoritma, threshold, dan exclusion agar hasil dapat diulang serta ditinjau.

#### Scenario: File dikecualikan dari duplication
- **WHEN** pengguna memberi exclusion untuk file/path
- **THEN** hasil menyebut exclusion dan metrik tidak menghitung file itu
