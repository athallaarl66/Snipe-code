# Spec Delta

## Purpose

Membatasi pembacaan audit ke root proyek, memisahkan scan read-only dari export eksplisit, dan menjaga kompatibilitas command legacy.

## ADDED Requirements

### Requirement: Audit scan dibatasi root dan tidak menulis ke source
Audit MUST melewati symlink/reparse-point, memastikan file yang dibaca berada di root kanonik, dan memisahkan scan read-only dari export eksplisit berbasis hasil scan berversi.

#### Scenario: Link menunjuk file di luar root
- **WHEN** audit menemukan dotfile atau file lain berupa symlink/reparse-point yang menunjuk keluar root
- **THEN** audit melewati entry tanpa membaca target atau membocorkan kontennya lewat report, JSON, atau log

#### Scenario: Scan dan export
- **WHEN** pengguna menjalankan `scan [PATH] [--json]` atau `export <scan.json> --format <md,pdf,docx> --output <DIR>`
- **THEN** scan tidak menulis ke project; export memvalidasi schema versi dan menulis hanya format yang diminta ke direktori eksplisit, atau gagal tanpa klaim sukses

#### Scenario: Argumen audit ambigu
- **WHEN** argumen audit mengandung option tak dikenal/duplikat, nilai wajib hilang, atau lebih dari satu path target
- **THEN** CLI menolak sebelum membaca target atau menulis report

#### Scenario: Audit legacy dan PDF banyak halaman
- **WHEN** pengguna memakai command `audit` lama atau mengekspor banyak finding ke PDF
- **THEN** perilaku format/JSON/exit legacy tetap, side effect dilaporkan, dan PDF dipaginasi tanpa menghilangkan finding

### Requirement: Export preserves content inside page boundaries
Reports MUST preserve canonical finding identities and counts across JSON, Markdown, DOCX, and PDF. PDF text MUST remain within A4 margins with wrapping, pagination, and per-page footers; content extraction and visual acceptance MUST be recorded separately.

#### Scenario: Zero one and many findings
- **WHEN** reports contain zero, one, or many findings with long paths/descriptions
- **THEN** all PDF text coordinates remain inside page margins and extraction retains every finding; missing extraction or render evidence is not a pass

### Requirement: Histori audit tetap di luar scope
Change ini MUST NOT menambah penyimpanan histori audit atau schema database.

#### Scenario: Scan dijalankan tanpa permintaan histori
- **WHEN** pengguna menjalankan `audit`, `scan`, atau `export`
- **THEN** tool tidak membuat atau mengubah database histori
