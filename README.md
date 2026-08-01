# SNIPE-CODE CLI 🚀
**Enterprise-Grade Project Scaffolding & Security Audit Tool**

SNIPE-CODE adalah tool CLI berbasis Rust yang dirancang untuk mempercepat inisialisasi proyek dengan struktur folder standar industri, keamanan bawaan (*security-baked*), dan otomatisasi alur kerja (Git, CI/CD, Docker).

---

## ✨ Fitur Utama

- 🏗️ **Preset Templates**: 
  - `Company Profile` (SEO ready)
  - `Portfolio` (Showcase ready)
  - `Blog / Posts` (MDX & RSS support)
  - `No Template` (Clean start)
- 🛠️ **Multi-Stack Scaffolding**: Dukungan Next.js, React, Vue, Nuxt (Frontend) serta .NET 8, NestJS, Go, Spring Boot, Laravel (Backend).
- 🔐 **Security First**: Otomatis generate `.env.example`, konfigurasi CORS/Helmet, dan `.gitignore` yang aman.
- 📋 **Security Audit**: Scan folder proyek dan ekspor laporan ke format **Markdown**, **PDF**, atau **DOCX**.
- 🤖 **AI Workspace**: Otomatis generate `.cursorrules` dan `AGENTS.md` untuk pengalaman coding lebih baik dengan AI Agent.
- ⚙️ **Otomasi**: Auto `git init`, `initial commit`, GitHub Actions (CI/CD), dan `docker-compose.yml`.

---

## 🚀 Cara Instalasi

Ada 3 cara — pilih sesuai kebutuhan. Semua butuh binary hasil compile; bedanya cuma di mana binary itu ditaruh.

### Opsi A — Build dari Source (untuk developer / punya Rust)

```bash
git clone https://github.com/athallaarl66/snipe-code.git
cd snipe-code
cargo build --release
```

Binary ada di `target/release/snipe-code.exe`. Jalankan langsung dari sana, atau salin ke folder mana pun (portable — bisa dibawa ke device lain tanpa install apa pun).

**Catatan storage:** folder `target/` hasil compile bisa ratusan MB. Binary-nya sendiri kecil (~5MB). Setelah selesai, hapus artifact compile dengan:
```bash
cargo clean
```
(exe di `target/release/` ikut terhapus — salin dulu ke tempat permanen kalau mau dipakai terus.)

### Opsi B — Pakai Release Binary (untuk end user / tanpa Rust)

Tidak perlu install Rust. Terima saja file `snipe-code.exe` dari developer/rilis, lalu:
- Taruh di folder mana pun, jalankan: `snipe-code.exe audit . --export pdf,docx`
- Atau tambahkan folder-nya ke PATH biar bisa ketik `snipe-code` dari mana saja

### Opsi C — Install ke System (agar perintah `snipe-code` dikenali dari mana saja)

```bash
cargo install --path .
```

Menyalin binary ke `~/.cargo/bin` (sudah di PATH). Setelah ini `snipe-code` bisa dipanggil dari direktori mana pun.

> **⚠️ Opsional? Tidak.** Tanpa salah satu cara di atas, perintah `snipe-code` tidak akan dikenali (`'snipe-code' is not recognized`). Pilih minimal satu opsi — A, B, atau C.

### 🗑️ Uninstall

- **Via cargo** (Opsi C): `cargo uninstall snipe-code`
- **Portable exe** (Opsi A/B): cukup hapus file `snipe-code.exe` — tidak ada registry/system files yang disentuh

---

## 🛠️ Panduan Penggunaan

### 1. Inisialisasi Proyek Baru
Jalankan perintah berikut untuk memulai wizard interaktif:
```bash
snipe-code
```
*Atau jika belum di-install ke system:* `cargo run --`

**Alur Wizard:**
1. **Pilih Template**: Gunakan arrow keys untuk memilih preset.
2. **Pilih Stack**: Pilih Frontend, Backend, dan Database yang diinginkan.
3. **Project Name**: Masukkan nama folder tujuan.
4. **Collision Handling**: Jika folder sudah ada, Anda akan ditanya untuk *Overwrite* atau *Cancel*.
5. **Done!** Proyek siap dengan Git, CI/CD, dan AI Workspace.

### 2. Preview Tanpa Membuat File (Dry Run)
Ingin melihat struktur folder apa yang akan dibuat tanpa benar-benar membuatnya?
```bash
snipe-code --dry-run
```

### 3. Security Audit & Export
Scan folder proyek Anda untuk mengecek kelengkapan file standar dan ekspor laporannya:
```bash
# Scan folder saat ini dan ekspor ke PDF & DOCX
snipe-code audit . --export pdf,docx
```
Laporan akan muncul dengan nama `Audit_Report.pdf` dan `Audit_Report.docx`.

---

## 🧪 Testing

Tool ini dilengkapi dengan 50+ automated tests untuk menjamin kestabilan.
```bash
cargo test
```

---

## 📂 Struktur Dokumentasi
- `docs/features/`: PRD & Technical Design.
- `docs/production/`: Detail breakdown tugas per fitur.

---

Built with ❤️ by **SNIPE-CODE Team**
