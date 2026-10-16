# SNIPE-CODE CLI

**Rust project scaffolding and heuristic security-audit tool**

SNIPE-CODE adalah tool CLI berbasis Rust yang dirancang untuk mempercepat inisialisasi proyek dengan struktur folder proyek, pemeriksaan keamanan heuristik, dan otomatisasi alur kerja (Git, CI/CD, Docker).

---

## ✨ Fitur Utama

- 🏗️ **Preset Templates**:
  - `Company Profile` (SEO ready)
  - `Portfolio` (Showcase ready)
  - `Blog / Posts` (MDX & RSS support)
  - `No Template` (Clean start)
- 🛠️ **Multi-Stack Scaffolding**: Dukungan Next.js, React, Vue, Nuxt (Frontend) serta .NET 8, NestJS, Go, Spring Boot, Laravel 12 (Backend).
- 🔐 **Safety Defaults**: Generate `.env.example`, konfigurasi stack, dan `.gitignore`; audit tetap heuristik dan tidak menggantikan security review.
- 📋 **Security Audit**: Scan folder proyek dan ekspor laporan ke format **Markdown**, **PDF**, atau **DOCX**.
- 🤖 **AI Workspace + AI Agent Tools**: `.ai-workspace/` (`.ai-workspace/rules.md`, `.ai-workspace/rules.mdc`) selalu dibuat; artifact per AI agent tool (`AGENTS.md`, `CLAUDE.md`, `.cursor/rules/`, `.opencode/`, `.agents/`, dst.) dibuat untuk tool yang **dipilih** saat generate.
- ⚙️ **Otomasi**: Auto `git init`, `initial commit`, GitHub Actions (CI/CD), dan `docker-compose.yml`.

---

## 🚀 Cara Instalasi

Ada 3 cara — pilih sesuai kebutuhan. Semua butuh binary hasil compile; bedanya cuma di mana binary itu ditaruh.

### Opsi A — Build dari Source (untuk developer / punya Rust)

```bash
git clone https://github.com/athallaarl66/Snipe-code.git
cd Snipe-code
cargo build --release
```

Binary ada di `target/release/snipe-code` (`target/release/snipe-code.exe` di Windows). Jalankan langsung dari sana, atau salin ke folder mana pun (portable — bisa dibawa ke device lain tanpa install apa pun).

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

_Atau jika belum di-install ke system:_ `cargo run --`

**Alur Wizard:**

1. **Pilih Template**: Gunakan arrow keys untuk memilih preset.
2. **Pilih Stack**: Pilih Frontend, Backend, dan Database yang diinginkan.
3. **Project Name**: Masukkan nama folder tujuan.
4. **AI Agent Tools** (opsional): multi-select tool AI yang dipakai; default tidak ada.
5. **Collision Handling**: Jika folder sudah ada, Anda akan ditanya untuk _Overwrite_ atau _Cancel_.
6. **Done!** Proyek siap dengan Git, CI/CD, dan AI Workspace.

### 2. Preview Tanpa Membuat File (Dry Run)

Ingin melihat struktur folder apa yang akan dibuat tanpa benar-benar membuatnya?

```bash
snipe-code --dry-run
```

### 3. Security Audit & Export

Scan folder proyek dan ekspor laporan:

```bash
snipe-code audit . --export md
snipe-code audit . --export pdf,docx
```

Format valid: `md`, `pdf`, `docx`. Duplikat dinormalisasi. Token kosong/unknown ditolak.

Exit code:

- `0`: audit selesai, tidak ada HIGH finding
- `1`: HIGH finding terdeteksi
- `2`: usage error, misalnya format export invalid
- `3`: runtime/export error, misalnya path tidak ada

Audit bersifat heuristik. Report mencantumkan detected context, rule groups, dan excluded paths. “No findings detected by evaluated rules” bukan jaminan proyek aman.

### 4. Verifikasi proyek yang sudah ada (`verify`)

Periksa apakah sebuah proyek benar-benar lulus gate kualitasnya:

```bash
snipe-code verify .                        # periksa semua gate yang berlaku
snipe-code verify ../proyek --only test    # ulangi satu gate saja
snipe-code verify . --security             # tambahkan gate kerentanan dependency
```

Gate diturunkan dari manifest proyek — bukan dari daftar stack yang di-generate tool ini — sehingga `verify` bekerja pada proyek mana pun yang dikenali:

| Manifest                        | Gate yang dijalankan                                                   |
| ------------------------------- | ---------------------------------------------------------------------- |
| `package.json`                  | `npm install`, lalu `npm run <script>` untuk `typecheck`, `lint`, `test`, `build` yang memang dideklarasikan |
| `composer.json` + `artisan`     | `composer install --no-interaction --no-progress`, `php artisan test`   |
| `composer.json` + `phpunit.xml` | `composer install --no-interaction --no-progress`, `php vendor/bin/phpunit` |
| `go.mod`                        | `go mod download`, `go mod tidy`, `go vet ./...`, `go test ./...`, `go build ./...` |
| `pom.xml`                       | `mvn test`, `mvn package -DskipTests`                                  |
| `*.csproj` / `*.sln`            | `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |

Urutan gate tetap: `install` → `lint` → `typecheck` → `test` → `build` (+ `security` bila diminta). Setiap gate melaporkan satu dari empat status:

- `passed` — perintah selesai dengan exit 0
- `failed` — perintah exit non-zero; nama gate dan perintahnya dicetak
- `not-applicable` — proyek tidak mendeklarasikan apa pun untuk gate itu, jadi tidak ada perintah yang dijalankan
- `unavailable` — tool-nya tidak terpasang di mesin ini; gate lain tetap lanjut

Exit code:

- `0`: semua gate yang berlaku lulus
- `1`: ada gate yang gagal
- `2`: usage error, misalnya `--only` menunjuk gate yang tidak berlaku atau opsi tidak dikenal
- `3`: tidak ada gate yang dijalankan, misalnya proyek tidak dikenali atau semua tool tidak tersedia

**`verify` vs `audit`.** `verify` menjalankan tool milik proyek (`npm`, `composer`, `mvn`, `dotnet`, `go`, `cargo`) sehingga **bisa memakai jaringan** — persis seperti CI proyek itu sendiri. Tool `snipe-code` tetap offline: ia tidak pernah memanggil jaringan, hanya men-spawn perintah yang sudah dideklarasikan proyek, dan setiap perintah dicetak sebelum dijalankan. Untuk analisis read-only yang sepenuhnya offline, pakai `snipe-code audit`.

`--security` memakai scanner bawaan ekosistem: `npm audit`, `composer audit`, `dotnet list package --vulnerable`. Go dan Java melaporkan gate ini sebagai `not-applicable` beserta alasannya, karena `govulncheck` dan OWASP dependency-check bukan bagian dari toolchain bawaannya.

Proyek dengan lebih dari satu bagian (misalnya `frontend/` dan `backend/`) diverifikasi per bagian, masing-masing di direktorinya sendiri. Proyek yang tidak dikenali dilaporkan `no gates found` dan keluar dengan status `3` tanpa menjalankan perintah apa pun. Batasan yang dikenal: deteksi memakai `audit::detect_scopes`, yang belum mengenali Python dan Ruby.

### Supported stack verification

| Capability                     | Native verification (commands run by CI)                              |
| ------------------------------ | --------------------------------------------------------------------- |
| Next.js, React Vite, Vue, Nuxt | `npm install`, `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build` |
| NestJS                         | `npm install`, `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build` |
| Go Gin                         | `go mod download`, `go mod tidy`, `go test ./...`, `go vet ./...`, `go build ./...` |
| Spring Boot                    | `mvn test package`                                                     |
| .NET 8                         | `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |
| Laravel 12                     | `composer install --no-interaction --no-progress`, `php artisan test`   |
| PostgreSQL/MySQL Compose       | `docker compose config`                                                |

Generated projects use one safe project-name destination. Existing destination requires explicit overwrite. Generation stages files first; failed generation cleans staging. Git init/commit is best effort.

### Template: struktur, override, dan cara mengubahnya

Isi setiap stack dan preset hidup sebagai file nyata di `templates/`, lalu di-embed ke binary saat build — tool tetap offline dan tetap satu file executable:

```text
templates/
├── frontends/<id>/          # nextjs, react-vite, vue, nuxtjs
│   ├── manifest.txt         # name=..., init=..., install=...
│   ├── folders.txt          # satu path folder per baris
│   └── files/<path>         # isi file yang ikut digenerate
├── backends/<id>/           # dotnet8, nestjs, go-gin, spring-boot, laravel
│   └── (manifest.txt, folders.txt, files/<path>)
└── presets/<id>/            # preset.json (folder + rules) dan content/<template_source>.txt
```

Menambah atau mengubah template **tidak perlu menyentuh Rust**: tambahkan atau edit file di `templates/`, lalu `cargo build`. Setiap file di `files/<path>` digenerate apa adanya ke path yang sama di dalam proyek hasil generate, `folders.txt` menentukan folder kosong yang dibuat, dan `manifest.txt` menentukan nama tampilan, perintah init, dan perintah install stack.

**Override lokal tanpa recompile.** Set `SNIPE_CODE_TEMPLATES` ke direktori dengan layout seperti `templates/`:

```bash
SNIPE_CODE_TEMPLATES=/path/ke/templates-kerja snipe-code new
```

Resolusi bersifat per file: file yang disediakan direktori override menang atas default built-in, sedangkan file yang tidak disediakan tetap memakai default. Jadi override sebagian tetap sah — misalnya hanya `frontends/nextjs/files/package.json` untuk menaikkan versi dependency. Output `new` mencantumkan sumbernya, `Templates: local templates from <dir>` atau `Templates: built-in templates`, supaya jelas isi mana yang dipakai pada generate terakhir.

### AI agent tools (opsional)

Kalau mau, proyek hasil generate bisa langsung membawa artifact instruksi untuk tool AI yang Anda pakai — masing-masing di path yang tool itu memang baca, bukan di satu folder bersama:

| Tool | Artifact | Command |
| --- | --- | --- |
| Cross-tool (AGENTS.md) | `AGENTS.md` | — |
| Claude Code | `CLAUDE.md`, `.claude/skills/snipe-code/SKILL.md` | `.claude/commands/snipe-code.md` |
| Cursor | `.cursor/rules/snipe-code.mdc`, `.cursorrules` | — |
| OpenCode | `.opencode/skills/snipe-code/SKILL.md` | `.opencode/commands/snipe-code.md` |
| Antigravity | `GEMINI.md`, `.agents/rules/snipe-code.md`, `.agents/skills/snipe-code.md` | `.agents/workflows/snipe-code.md` |
| GitHub Copilot | `.github/copilot-instructions.md` | — |
| Windsurf | `.windsurf/rules/snipe-code.md` | — |
| Hermes Agent | `.hermes/skills/snipe-code/SKILL.md` | — |
| Codex | `AGENTS.md` | — |
| Gemini CLI | `GEMINI.md` | — |

Daftar tool, path, dan formatnya hidup di `templates/ai-tools.json` (format: `markdown`, `mdc`, `skill`, `command`) — menambah tool = menambah entri registry, bukan mengubah kode generate. Format yang tidak dikenal = error, bukan artifact yang dilewati diam-diam.

```bash
snipe-code new                                    # wizard, multi-select AI tools (default: tidak ada)
snipe-code new --ai-tools cursor,claude-code      # pilih tool tanpa prompt
snipe-code new --ai-tools none                    # tanpa artifact agen
snipe-code new --ai-tools claude-code --name demo --json
                                                  # hasil generate sebagai JSON di stdout
```

- `--ai-tools <ids|none>` melewati prompt pemilihan. Tanpa terminal (script/CI) wizard memakai default dan tidak pernah prompt; beri `--name` supaya nama proyeknya jelas.
- `--json` menaruh hasil generate di stdout (`project`, `ai_tools`, `ai_artifacts`, `ai_overlaps`, `files`, `outcome`); progress ke stderr, jadi stdout tetap bisa dibaca mesin.
- Path yang diklaim lebih dari satu tool — misalnya `AGENTS.md` oleh cross-tool dan Codex — ditulis sekali, dan overlap-nya dilaporkan di output.
- `.ai-workspace/rules.md` dan `rules.mdc` selalu ada; isinya dirender dari sumber yang sama dengan artifact per tool, jadi panduannya tidak bisa saling bertentangan.

**Catatan:** direktori skill bersifat project-local (`<proyek>/.claude/skills/`, `<proyek>/.opencode/skills/`, `<proyek>/.hermes/skills/`). Tool baru memakai skill itu kalau konfigurasi tool tersebut memang menunjuk direktori proyek. `snipe-code` tidak menulis konfigurasi global di home directory — semua artifact ada di dalam folder proyek.

### Release verification

Release archives include executable, `LICENSE`, `README.md`, and `SOURCE_REVISION`. Verify checksum:

```bash
sha256sum --check snipe-code-<version>-<target>.*.sha256
```

Verify keyless Cosign bundle:

```bash
cosign verify-blob --bundle archive.bundle archive
```

Only use commands against artifacts actually published on GitHub Releases.

---

## 🧪 Testing

Run repository quality gates:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

---

## 📂 Struktur Dokumentasi

- `docs/features/`: PRD & Technical Design.
- `docs/production/`: Detail breakdown tugas per fitur.

---

Built with ❤️ by **SNIPE-CODE Team**
