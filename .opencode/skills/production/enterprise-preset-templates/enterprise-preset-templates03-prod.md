# US-003: Dry-Run Preview — Production Requirements

## User Acceptance
**As a** developer
**I want** lihat struktur yang bakal digenerate sebelum bikin file
**So that** saya yakin sebelum eksekusi

## Business Value
Mencegah mistake — user bisa review dulu sebelum file beneran dibuat.

## User Persona
Developer yang hati-hati, mau pastikan output sebelum commit.

## User Journey
1. Run `snipe-code new --dry-run`
2. Pilih template + stack seperti biasa
3. Lihat folder tree output
4. Tidak ada file yang dibuat

## Success Criteria
- [ ] Output folder tree jelas
- [ ] Tidak ada file/folder yang terbuat
- [ ] Selesai dalam < 1 detik
