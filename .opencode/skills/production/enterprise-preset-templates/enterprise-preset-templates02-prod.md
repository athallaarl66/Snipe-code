# US-002: Init Tanpa Template — Production Requirements

## User Acceptance
**As a** developer
**I want** skip template dan langsung milih stack
**So that** saya dapat struktur folder clean + secure tanpa template-specific files

## Business Value
Developer yang sudah punya workflow sendiri tetap dapat struktur standar tanpa file yang tidak perlu.

## User Persona
Senior developer yang sudah tahu stack-nya dan butuh clean architecture saja.

## User Journey
1. Run `snipe-code new`
2. Pilih "No Template — clean start"
3. Pilih Frontend, Backend, DB
4. Masukkan nama project
5. Clean structure di-generate

## Success Criteria
- [ ] "No Template" ada di urutan pertama
- [ ] Struktur clean architecture sesuai stack
- [ ] Security config tetap included
- [ ] Tidak ada template-specific files

## Edge Cases
- User pilih None untuk semua stack → warning: "Pilih minimal 1 stack"
- Docker dipilih tapi backend None → skip docker-compose
