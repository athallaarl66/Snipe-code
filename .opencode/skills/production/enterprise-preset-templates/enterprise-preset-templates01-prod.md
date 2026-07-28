# US-001: Init dengan Template — Production Requirements

## User Acceptance
**As a** developer
**I want** memilih template preset saat `snipe-code new`
**So that** saya langsung dapat struktur proyek yang sesuai kebutuhan

## Business Value
Developer bisa langsung mulai develop tanpa setup manual. Mengurangi waktu inisialisasi dari jam ke < 30 detik.

## User Persona
Developer yang ingin cepat start proyek baru dengan struktur yang sudah benar.

## User Journey
1. Run `snipe-code new`
2. Lihat daftar template: IoT Dashboard, Company Profile, Portfolio, Blog
3. Pilih salah satu
4. Pilih stack (Frontend + Backend + DB)
5. Masukkan nama project
6. Template di-generate

## Success Criteria
- [ ] Template bisa dipilih via arrow key
- [ ] Struktur folder sesuai template ter-generate
- [ ] Security config otomatis included
- [ ] Selesai dalam < 5 detik

## Edge Cases
- User pilih "No Template" → skip ke stack selection langsung
- Target folder sudah ada → prompt: overwrite / cancel / merge
- User cancel di tengah → tidak ada perubahan di filesystem
