# 🚀 SNIPE-CODE CLI (Enterprise Dev-Forge)

## 1. Overview

**Snipe-Code CLI** adalah _command-line interface_ bertenaga tinggi berbasis Rust yang dirancang untuk mengotomatisasi inisialisasi proyek _Fullstack_ skala industri. Alat ini mengadopsi standar _Clean Architecture_ dan _Spec-Driven Development (SDD)_, sekaligus mengamankan aplikasi sejak menit pertama.

Lebih dari sekadar _scaffolder_, CLI ini menyuntikkan **AI Workspace Environment** yang dikonfigurasi khusus, bertindak sebagai **Local Security Auditor**, dan menangani manajemen _UI Component_ secara dinamis.

---

## 2. Fitur Utama (Core Features)

### A. Multi-Stack Scaffolding Engine & Git Automation

- **Pilihan Stack Dinamis:**
  - _Frontend:_ Next.js, React (Vite), Vue, Nuxt.js.
  - _Backend:_ .NET 8, NestJS, Go (Gin/Fiber), Spring Boot, Laravel.
  - _Database & ORM:_ PostgreSQL, MySQL (terintegrasi otomatis dengan EF Core, Prisma, GORM, dll).
- **Auto Git & CI/CD:** Otomatis menjalankan `git init`, men-generate `.gitignore` optimal, membuat _initial commit_, dan menyiapkan file `.github/workflows/main.yml` untuk _pipeline_ CI/CD.

### B. Enterprise Preset Templates

Sistem _scaffolding_ cerdas yang menyediakan arsitektur pra-konfigurasi untuk sistem berat, contoh:

- **"Industrial IoT Dashboard Preset":** Membangun _backend_ berkinerja tinggi (.NET/Go) yang siap untuk beban data masif, dipasangkan dengan _frontend_ interaktif, lengkap dengan penyusunan _layout global_ yang terisolasi secara spesifik di dalam direktori `dashboard/` untuk pemeliharaan jangka panjang.

### C. UI Component Auto-Installer (Opsional)

Pemasangan pustaka komponen UI (seperti `shadcn/ui`) secara otomatis langsung dari terminal.

- **Bulk Download Esensial:** Otomatis mengunduh komponen krusial (`Button`, `Card`, `Table`, `Sidebar`, `Input`) untuk mempercepat _prototyping_.
- **Barebone Option:** Pilihan untuk hanya menginisialisasi pustaka tanpa mengunduh komponen.

### D. AI Workspace & `/snipe-code` Guard

Otomatis membuat konfigurasi IDE agen AI (Windsurf, Claude, Cursor) di dalam root folder.

- **Security First:** Instruksi _system prompt_ statis yang melarang AI menulis kode rentan (SQL Injection, XSS, _hardcoded secrets_).
- **Token Optimizer:** Menyiapkan mode hemat token agar AI hanya fokus pada blok kode murni dan efisiensi logika.

### E. Local Security Audit

Pemindaian statis (SAST) bawaan yang tidak memerlukan _server_ eksternal. Laporan kerentanan keamanan dan _code smells_ akan diekspor otomatis menjadi `Audit_Report.md`, `Audit_Report.pdf`, atau `Audit_Report.docx`.

---

## 3. Arsitektur & Output Folder

```text
my-enterprise-app/
├── .git/                       <-- (Diinisialisasi otomatis dengan initial commit)
├── .github/workflows/          <-- (Pipeline CI/CD otomatis)
├── frontend/                   <-- (Aplikasi client, misal: Next.js)
│   └── components/ui/          <-- (Otomatis terisi Button, Card, Table jika opsional dipilih)
├── backend/                    <-- (API Server, terstruktur Clean Architecture / DDD)
├── docker-compose.yml          <-- (Instalasi DB lokal instan)
│
└── .ai-workspace/              <-- (Ruang kendali untuk AI Assistant)
    ├── .windsurf/rules.md      <-- (Instruksi sistem untuk IDE)
    ├── .cursor/rules.mdc
    └── audit/                  <-- (Hasil ekspor laporan Security Audit otomatis)
4. Cara Penggunaan (Dual-Execution Workflow)
CLI ini dapat beroperasi melalui terminal klasik maupun panel obrolan AI Assistant.

Mode 1: Terminal Commands
snipe-code new : Memulai menu interaktif (dialoguer) untuk membangun proyek, memilih UI components, dan konfigurasi database.

snipe-code new --dry-run : Mencetak simulasi struktur Clean Architecture dan eksekusi di layar terminal tanpa membuat file fisik apa pun (Mode aman).

snipe-code audit --export pdf,md : Memindai codebase secara lokal dan men-generate laporan.

Mode 2: AI Slash Commands (Dalam IDE)
Berkat injeksi di folder .ai-workspace, developer dapat langsung mengetik perintah berikut di obrolan agen AI:

/pentest : AI meninjau codebase dan mencari celah keamanan berdasarkan standar OWASP.

/snipe-code : Memaksa AI masuk ke mode efisiensi token ekstrem; AI berfungsi sebagai penembak jitu yang hanya mengeluarkan blok kode murni tanpa penjelasan.

/fix-smells : AI membaca laporan audit dan secara mandiri menulis perbaikan refactoring.

/gen-docs : AI membuat dokumentasi SDD (Spec-Driven Development) membaca dari struktur backend.

5. Distribusi & Instalasi
Dikompilasi menggunakan bahasa Rust untuk kecepatan maksimal.

Stand-alone Binary: Pengguna cukup mengunduh file .exe (Windows) atau biner (Mac/Linux) dari GitHub Releases dan dapat menjalankannya tanpa perlu menginstal Node.js atau Rust.

Developer Mode: git clone repositori dan jalankan cargo build --release.
```
