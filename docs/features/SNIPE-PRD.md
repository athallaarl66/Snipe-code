# Product Requirements Document (PRD)

## Document Information
- **Document Version**: 1.0
- **Created Date**: 2026-07-28
- **Last Updated**: 2026-10-21
- **Author**: SNIPE-CODE Team
- **Project**: SNIPE-CODE CLI
- **Feature**: Enterprise Preset Templates

---

## 1. Executive Summary

### 1.1 Purpose
Memberikan pengguna template preset siap pakai (Company Profile, Portfolio, Blog) serta opsi tanpa template — cukup pilih stack, dapatkan struktur folder yang aman, clean architecture, dan sesuai standar industri. Preset IoT Dashboard masih rencana dan belum ada di CLI.

### 1.2 Scope
**In Scope:**
- Template preset: Company Profile, Portfolio, Blog/Posts
- Opsi "No Template" — hanya generate struktur folder sesuai stack
- Command `analyze` (laporan read-only) dan `verify` (menjalankan gate proyek)
- Pilihan stack dinamis (Frontend + Backend + DB/ORM)
- Struktur folder standar Clean Architecture / DDD
- Security best practices baked into generated structure
- Git init + CI/CD pipeline auto-generated

**Out of Scope:**
- Template IoT Dashboard (deferred)
- Template marketplace / community templates (future)
- UI component auto-installer (fitur terpisah)
- Deployment scripts ke cloud provider

### 1.3 Business Objectives
- Mempercepat inisialisasi proyek dari jam ke menit
- Menjamin struktur folder aman & standar sejak awal
- Memberikan fleksibilitas — template atau tidak, tetap dapet best practice

---

## 2. Background & Context

### 2.1 Current Situation
Developer masih setup proyek manual: bikin folder, install framework, config database, setup struktur. Hasilnya tidak konsisten, rawan typo, dan sering lupa security best practice.

### 2.2 Problem Statement
Tanpa template standar, setiap proyek punya struktur beda-beda. Codebase sulit dimaintain, security terlewat, dan onboarding developer baru lambat.

### 2.3 Business Case
Satu command `snipe-code new` bisa generate proyek siap pakai dalam <30 detik. Mengurangi human error, enforce standar, dan bikin developer happy.

---

## 3. Stakeholder Analysis

| Stakeholder | Role | Interest | Influence |
|-------------|------|----------|-----------|
| Developer | End user | High | High |
| Tech Lead | Standarisasi tim | High | High |
| Freelancer | Bikin portfolio/company profile cepet | High | Medium |
| Product Owner | Feature decision | Medium | High |

---

## 4. Functional Requirements

### 4.1 User Stories

#### US-001: Init dengan Template
**As a** developer
**I want** memilih template preset saat `snipe-code new`
**So that** saya langsung dapat struktur proyek yang sesuai kebutuhan

**Acceptance Criteria:**
- [x] `snipe-code new` menampilkan daftar template: No Template, Company Profile, Portfolio, Blog
- [x] Setelah pilih template, bisa pilih stack (Frontend + Backend + Database)
- [x] Template menentukan struktur folder tambahan (blog/ punya posts/, portfolio/ punya sections/)
- [ ] Semua file yang digenerate punya komentar security header (belum ada; yang digenerate `.env.example` dan `.gitignore`)

**Priority:** Must Have

#### US-002: Init Tanpa Template
**As a** developer
**I want** skip template dan langsung milih stack
**So that** saya dapat struktur folder clean + secure tanpa template-specific files

**Acceptance Criteria:**
- [ ] Ada opsi "No Template — clean start" di daftar template
- [x] Struktur yang digenerate tetap clean architecture sesuai stack
- [x] Tetap ada security config dasar (`.env.example`, `.gitignore`; config helmet/CORS per framework belum digenerate)
- [x] Git init tetap jalan

**Priority:** Must Have

#### US-003: Dry-Run Preview
**As a** developer
**I want** lihat struktur yang bakal digenerate sebelum bikin file
**So that** saya yakin sebelum eksekusi

**Acceptance Criteria:**
- [x] `snipe-code new --dry-run` print simulasi folder tree
- [x] Tidak ada file yang beneran dibuat
- [x] Output jelas: `[DRY RUN] Would create:` diikuti tree struktur

**Priority:** Should Have

#### US-005: Template Company Profile
**As a** freelancer
**I want** template company profile yang siap di-custom
**So that** saya bisa deliver ke klien dalam hitungan jam

**Acceptance Criteria:**
- [x] Struktur folder halaman: Home, About, Services, Contact, Team (folder saja, halaman belum terisi)
- [ ] SEO meta tags sudah included (rencana)
- [ ] Responsive layout out-of-the-box (rencana)
- [x] Backend opsional (bisa static site pake frontend aja)

**Priority:** Must Have

#### US-006: Template Portfolio
**As a** developer/designer
**I want** template portfolio untuk showcase project
**So that** saya punya landing page personal yang profesional

**Acceptance Criteria:**
- [x] Struktur folder: Hero, Projects grid, Skills, Experience, Contact (folder saja)
- [ ] Dark/light mode toggle included (rencana)
- [x] Projects data dari JSON (`frontend/src/data/projects.json`)
- [ ] Deploy-ready ke Vercel/Netlify (butuh setup user)

**Priority:** Should Have

#### US-007: Template Blog/Posts
**As a** content creator
**I want** template blog dengan CMS-ready structure
**So that** saya bisa nulis dan publish tanpa setup dari nol

**Acceptance Criteria:**
- [x] Struktur: posts/, categories/, tags/, author page
- [x] Markdown/MDX support untuk konten (`hello-world.mdx`)
- [ ] RSS feed auto-generated (yang digenerate hanya sampel statis `frontend/public/rss.xml`, tidak ikut update saat post ditambah)
- [x] Backend opsional: bisa pake headless CMS atau static generation

**Priority:** Should Have

#### US-008: Security Audit & Export
**As a** security conscious developer
**I want** melakukan audit cepat pada folder proyek
**So that** saya tahu apakah best-practice dasar sudah terpenuhi

**Acceptance Criteria:**
- [x] `snipe-code audit` men-scan keberadaan .env.example, .gitignore, dll.
- [x] Bisa mengekspor laporan ke format Markdown, PDF, dan DOCX.
- [x] Output file audit tersimpan di direktori target sebagai `Audit_Report.*`

**Priority:** Must Have

#### US-009: AI Workspace Generation
**As a** developer who uses AI
**I want** struktur proyek yang di-generate sudah "AI-Ready"
**So that** saya bisa langsung menggunakan Cursor/Windsurf dengan context yang tepat.

**Acceptance Criteria:**
- [x] Membuat `.cursorrules` dengan instruksi stack yang dipilih, saat tool Cursor dipilih (opt-in).
- [x] Membuat `AGENTS.md` untuk panduan AI Agent, saat tool terkait dipilih (opt-in).
- [x] `.ai-workspace/rules.md` dan `rules.mdc` selalu ditulis.

**Priority:** Must Have


### 4.2 Functional Requirements per Module

#### Module 1: Template Selector
- **FR-TS-01:** CLI menampilkan daftar template interaktif (pake dialoguer / arrow keys)
- **FR-TS-02:** Setiap template punya deskripsi 1 baris
- **FR-TS-03:** Opsi "No Template" tersedia sebagai default/clean start
- **FR-TS-04:** Setelah pilih template, lanjut ke stack selection flow

#### Module 2: Stack Selection
- **FR-SS-01:** Pilih Frontend: Next.js, React (Vite), Vue, Nuxt.js — atau none
- **FR-SS-02:** Pilih Backend: .NET 8, NestJS, Go (Gin), Spring Boot, Laravel — atau none
- **FR-SS-03:** Pilih Database: PostgreSQL, MySQL — atau none. Integrasi ORM belum digenerate; yang ditulis hanya `DATABASE_URL` di `.env.example`
- **FR-SS-04:** Validasi minimal: setidaknya 1 stack dipilih (Frontend atau Backend)

#### Module 3: Structure Generator
- **FR-SG-01:** Generate folder tree sesuai template + stack
- **FR-SG-02:** Security bawaan: `.env.example` dan `.gitignore`. Config helmet/CORS per framework dan input sanitizer belum digenerate
- **FR-SG-03:** Clean Architecture: `controllers/`, `services/`, `repositories/`, `middleware/`, `routes/` (backend)
- **FR-SG-04:** Component isolation: `components/ui/`, `components/layout/`, `hooks/`, `pages/` (frontend)
- **FR-SG-05:** Docker compose untuk DB, digenerate saat backend dan database sama-sama dipilih

#### Module 4: Git & CI/CD Automation
- **FR-GC-01:** Auto `git init` setelah generate
- **FR-GC-02:** Generate satu `.gitignore` yang berlaku untuk semua stack
- **FR-GC-03:** Generate `.github/workflows/main.yml` dengan job per bagian, memakai script yang benar-benar dideklarasikan proyek
- **FR-GC-04:** Auto `git add . && git commit -m "Initial commit: [template] + [stack]"`

---

## 5. Non-Functional Requirements

### 5.1 Performance Requirements
- `snipe-code new` (dengan template) selesai dalam < 5 detik
- Dry-run output dalam < 1 detik
- Ukuran template files: < 100KB per preset

### 5.2 Security Requirements
- Tidak ada hardcoded secrets di file template
- `.env.example` untuk semua konfigurasi
- Tidak ada secret nyata di file template; `.env.example` hanya berisi placeholder
- Config CORS, Helmet, rate limiter per framework dan contoh parameterized query belum digenerate (rencana)

### 5.3 Usability Requirements
- Semua prompt pake arrow key navigation — no typing needed
- Setiap langkah bisa di-cancel (Ctrl+C)
- Output folder tree yang di-generate biar user tau apa aja yang kebikin
- Error message jelas: "Folder already exists" bukan "Error 1"

---

## 6. User Interface Requirements

### 6.1 Prompt Flow
```
$ snipe-code new

? Pilih Template:
  No Template — clean start (recommended)
  Company Profile
  Portfolio
  Blog / Posts

? Pilih Frontend: Next.js | React (Vite) | Vue | Nuxt.js | None
? Pilih Backend: .NET 8 | NestJS | Go | Spring Boot | Laravel | None
? Pilih Database: PostgreSQL | MySQL | None
? Project Name: my-project
? Pilih AI Agent Tools (opsional, multi-select)

✔ Generating project structure...

my-project/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   └── ...
│   └── ...
├── backend/
│   ├── src/
│   │   ├── controllers/
│   │   ├── services/
│   │   ├── repositories/
│   │   └── ...
│   └── ...
├── .env.example
├── .gitignore
├── docker-compose.yml
└── README.md

✔ Project created!
✔ Git initialized
✔ Initial commit made
```

### 6.2 UX Guidelines
- Pilihan default ada di opsi yang paling umum (recommended)
- Warna: hijau untuk success, kuning untuk info, merah untuk error
- Spinner selama generate
- Tanpa terminal, semua prompt dilewati dan nilai default dipakai (template none, frontend nextjs). Tidak ada flag `--quiet`

---

## 7. Data Requirements

### 7.1 Data Flow
1. User run `snipe-code new`
2. CLI tampilkan daftar template → pilih
3. CLI tampilkan stack selection → pilih
4. CLI tampilkan konfigurasi tambahan (Docker, project name)
5. CLI generate folder tree based on template + stack
6. CLI init git + commit

### 7.2 Template Storage
- Template definitions: file di dalam repo `templates/` (frontends, backends, presets), di-embed ke binary saat compile dengan `include_dir!`
- Override lokal tanpa recompile lewat env var `SNIPE_CODE_TEMPLATES`; resolusi per file, file yang tidak di-override tetap memakai versi bawaan
- Tidak ada fetch dari registry
- Struktur per template: `{ name, description, structure: { folders[], files[], rules{} } }`

---

## 8. Integration Requirements

### 8.1 Internal Integration
- **Stack Engine:** Ambil stack config dari module Multi-Stack Scaffolding
- **Git Engine:** Auto-init via module Git Automation

---

## 9. Business Rules

| Rule ID | Rule Description | Condition | Action |
|---------|-----------------|-----------|--------|
| BR-001 | No empty project | No stack selected | Warning: "Pilih minimal Frontend atau Backend" |
| BR-002 | Folder exists | Target folder sudah ada | Prompt: Overwrite atau Cancel; tanpa terminal berhenti dengan error |
| BR-003 | Docker skip | User pilih No Docker | Tidak generate docker-compose.yml |
| BR-004 | Backend-only | Frontend = None | Skip frontend folder dan package.json |
| BR-005 | Frontend-only | Backend = None | Skip backend folder dan Docker untuk backend |

---

## 10. Assumptions & Constraints

### 10.1 Assumptions
- User punya Git terinstall
- User tahu stack apa yang mau dipake

### 10.2 Constraints
- Template hanya yang built-in di v1
- Generasi struktur, bukan kode fungsional (controller masih template/stub)

### 10.3 Dependencies
- Rust `dialoguer` crate untuk interactive prompts
- Rust `serde` + `serde_json` untuk template definitions

---

## 11. Risks & Mitigation

| Risk ID | Risk Description | Probability | Impact | Mitigation Strategy |
|---------|-----------------|-------------|--------|---------------------|
| R-001 | Template terlalu generic jadi gak berguna | Medium | Medium | Validasi dengan developer real sebelum rilis; iterasi berdasarkan feedback |
| R-002 | Struktur folder bentrok dengan existing project | Low | High | Deteksi otomatis, prompt Overwrite atau Cancel, publish bertahap dengan backup dan rollback |
| R-003 | Stack version mismatch | Low | Medium | Di template cuma generate struktur — user install dependency sendiri atau via `npm install` |

---

## 12. Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Time to init project | < 30 detik (end-to-end) | CLI timing |
| Template usage rate | > 60% dari total `snipe-code new` | Belum diukur: CLI tidak mengirim telemetry |
| User satisfaction | Rating > 4/5 | Survey setelah generate |

---

## Current implementation status

This status section overrides older unchecked planning examples above.

Implemented:
- `new` mendukung `--template`, `--frontend`, `--backend`, `--database`, `--name`, `--ai-tools`, `--json`, dan `--dry-run`.
- Presets: `none`, `company-profile`, `portfolio`, and `blog`.
- Frontends: `nextjs`, `react-vite`, `vue`, `nuxtjs`, and `none`.
- Backends: `dotnet8`, `nestjs`, `go-gin`, `spring-boot`, `laravel`, and `none`.
- `analyze` read-only. `audit` menulis laporan ke folder yang di-scan. `verify` menjalankan tool proyek dan bisa menulis output dependency/build.
- Temuan audit bersifat heuristik dan perlu review kode sebelum diperbaiki.

Limitations and deferred work:
- IoT Dashboard, live RSS generation, template marketplace, cloud deployment, dan auto-install komponen UI belum diimplementasikan.
- RSS blog hanya sampel statis dengan URL localhost; post baru tidak memperbarui file itu.
- Release `v0.5.3` sudah punya empat archive dengan checksum dan cosign bundle, crate `0.5.3` ada di crates.io, dan image ada di ghcr.io. Installer belum diuji di setiap platform terhadap rilis ini, dan belum ada UAT manual (lihat `docs/uat/UAT-MATRIX.md`).

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Product Owner | | | |
| Technical Lead | | | |
| Developer Representative | | | |

---

## 14. Change History

| Version | Date | Author | Description of Changes |
|---------|------|--------|----------------------|
| 1.0 | 2026-07-28 | SNIPE-CODE Team | Initial version — Enterprise Preset Templates + No Template option |
| 1.1 | 2026-10-21 | SNIPE-CODE Team | Status per acceptance criterion disamakan dengan kode 0.5.3: tanpa `--quiet`, tanpa opsi merge, RSS statis, IoT masih rencana, template dari `templates/` + `SNIPE_CODE_TEMPLATES` |