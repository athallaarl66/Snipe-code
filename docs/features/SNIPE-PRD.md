# Product Requirements Document (PRD)

## Document Information
- **Document Version**: 1.0
- **Created Date**: 2026-07-28
- **Last Updated**: 2026-07-28
- **Author**: SNIPE-CODE Team
- **Project**: SNIPE-CODE CLI
- **Feature**: Enterprise Preset Templates

---

## 1. Executive Summary

### 1.1 Purpose
Memberikan pengguna template preset siap pakai untuk berbagai kebutuhan (IoT Dashboard, Company Profile, Portfolio, Blog) serta opsi bebas template — cukup pilih stack, dapatkan struktur folder yang aman, clean architecture, dan sesuai standar industri.

### 1.2 Scope
**In Scope:**
- Template preset: IoT Dashboard, Company Profile, Portfolio, Blog/Posts
- Opsi "No Template" — hanya generate struktur folder sesuai stack
- Pilihan stack dinamis (Frontend + Backend + DB/ORM)
- Struktur folder standar Clean Architecture / DDD
- Security best practices baked into generated structure
- Git init + CI/CD pipeline auto-generated

**Out of Scope:**
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
- [ ] `snipe-code new` menampilkan daftar template: IoT Dashboard, Company Profile, Portfolio, Blog
- [ ] Setelah pilih template, bisa pilih stack (Frontend + Backend + DB/ORM)
- [ ] Template menentukan struktur folder tambahan (misal: dashboard/ ada layout khusus, blog/ ada posts/)
- [ ] Semua file yang digenerate punya komentar security header

**Priority:** Must Have

#### US-002: Init Tanpa Template
**As a** developer
**I want** skip template dan langsung milih stack
**So that** saya dapat struktur folder clean + secure tanpa template-specific files

**Acceptance Criteria:**
- [ ] Ada opsi "No Template — clean start" di daftar template
- [ ] Struktur yang digenerate tetap clean architecture sesuai stack
- [ ] Tetap ada security config (.env.example, helmet config, CORS setup)
- [ ] Git init tetap jalan

**Priority:** Must Have

#### US-003: Dry-Run Preview
**As a** developer
**I want** lihat struktur yang bakal digenerate sebelum bikin file
**So that** saya yakin sebelum eksekusi

**Acceptance Criteria:**
- [ ] `snipe-code new --dry-run` print simulasi folder tree
- [ ] Tidak ada file yang beneran dibuat
- [ ] Output jelas: "Would create: frontend/src/pages/, backend/src/controllers/..."

**Priority:** Should Have

#### US-004: Template IoT Dashboard
**As a** developer IoT
**I want** template dengan struktur dashboard monitoring
**So that** saya langsung bisa develop fitur IoT tanpa setup layout

**Acceptance Criteria:**
- [ ] Frontend: layout sidebar + header + real-time chart area
- [ ] Backend: struktur WebSocket handler, controller untuk data sensor
- [ ] Structure: `dashboard/` terisolasi dengan `components/`, `hooks/`, `pages/`
- [ ] Docker compose untuk DB + MQTT broker

**Priority:** Must Have

#### US-005: Template Company Profile
**As a** freelancer
**I want** template company profile yang siap di-custom
**So that** saya bisa deliver ke klien dalam hitungan jam

**Acceptance Criteria:**
- [ ] Struktur halaman: Home, About, Services, Contact, Team
- [ ] SEO meta tags sudah included
- [ ] Responsive layout out-of-the-box
- [ ] Backend opsional (bisa static site pake frontend aja)

**Priority:** Must Have

#### US-006: Template Portfolio
**As a** developer/designer
**I want** template portfolio untuk showcase project
**So that** saya punya landing page personal yang profesional

**Acceptance Criteria:**
- [ ] Halaman: Hero, Projects grid, Skills, Experience, Contact
- [ ] Dark/light mode toggle included
- [ ] Projects data dari JSON/yaml (mudah di-edit)
- [ ] Deploy-ready ke Vercel/Netlify

**Priority:** Should Have

#### US-007: Template Blog/Posts
**As a** content creator
**I want** template blog dengan CMS-ready structure
**So that** saya bisa nulis dan publish tanpa setup dari nol

**Acceptance Criteria:**
- [ ] Struktur: posts/, categories/, tags/, author page
- [ ] Markdown/MDX support untuk konten
- [ ] RSS feed auto-generated
- [ ] Backend opsional: bisa pake headless CMS atau static generation

**Priority:** Should Have

### 4.2 Functional Requirements by Module

#### Module 1: Template Selector
- **FR-TS-01:** CLI menampilkan daftar template interaktif (pake dialoguer / arrow keys)
- **FR-TS-02:** Setiap template punya deskripsi 1 baris
- **FR-TS-03:** Opsi "No Template" selalu ada di urutan pertama
- **FR-TS-04:** Setelah pilih template, lanjut ke stack selection flow

#### Module 2: Stack Selection
- **FR-SS-01:** Pilih Frontend: Next.js, React (Vite), Vue, Nuxt.js — atau none
- **FR-SS-02:** Pilih Backend: .NET 8, NestJS, Go (Gin/Fiber), Spring Boot, Laravel — atau none
- **FR-SS-03:** Pilih DB/ORM: PostgreSQL, MySQL — auto-integrasi dengan ORM sesuai stack
- **FR-SS-04:** Validasi minimal: setidaknya 1 stack dipilih (Frontend atau Backend)

#### Module 3: Structure Generator
- **FR-SG-01:** Generate folder tree sesuai template + stack
- **FR-SG-02:** Security bawaan: `.env.example`, `helmet`/CORS config, `.gitignore`, input sanitizer structure
- **FR-SG-03:** Clean Architecture: `controllers/`, `services/`, `repositories/`, `middleware/`, `routes/` (backend)
- **FR-SG-04:** Component isolation: `components/ui/`, `components/layout/`, `hooks/`, `pages/` (frontend)
- **FR-SG-05:** Docker compose untuk DB (opsional, sesuai stack)

#### Module 4: Git & CI/CD Automation
- **FR-GC-01:** Auto `git init` setelah generate
- **FR-GC-02:** Generate `.gitignore` optimal per stack
- **FR-GC-03:** Generate `.github/workflows/main.yml` dengan lint + test pipeline
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
- CORS, Helmet, rate limiter config auto-included di backend
- SQL injection protection structure (parameterized queries di repository layer)

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
  Industrial IoT Dashboard
  Company Profile
  Portfolio
  Blog / Posts

? Pilih Frontend: Next.js | React (Vite) | Vue | Nuxt.js | None
? Pilih Backend: .NET 8 | NestJS | Go | Spring Boot | Laravel | None
? Pilih Database: PostgreSQL | MySQL | None
? Project Name: my-project
? Use Docker? Yes / No

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
- `--quiet` flag untuk skip semua prompt — langsung pake default

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
- Template definitions: JSON file di `~/.snipe-code/templates/`
- Template files: embedded binary (compile-time) atau fetch from registry
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
| BR-002 | Folder exists | Target folder sudah ada | Prompt: overwrite? / cancel / merge |
| BR-003 | Docker skip | User pilih No Docker | Tidak generate docker-compose.yml |
| BR-004 | Backend-only | Frontend = None | Skip frontend folder dan package.json |
| BR-005 | Frontend-only | Backend = None | Skip backend folder dan Docker untuk backend |

---

## 10. Assumptions & Constraints

### 10.1 Assumptions
- User punya Git terinstall
- User tahu stack apa yang mau dipake
- Koneksi internet untuk `git init` (first commit)

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
| R-002 | Struktur folder bentrok dengan existing project | Low | High | Deteksi otomatis + prompt overwrite/merge/cancel |
| R-003 | Stack version mismatch | Low | Medium | Di template cuma generate struktur — user install dependency sendiri atau via `npm install` |

---

## 12. Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Time to init project | < 30 detik (end-to-end) | CLI timing |
| Template usage rate | > 60% dari total `snipe-code new` | CLI telemetry (opt-in) |
| User satisfaction | Rating > 4/5 | Survey setelah generate |

---

## 13. Approval

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