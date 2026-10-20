# SNIPE-CODE CLI (README Bahasa Indonesia)

> Versi utama berbahasa Inggris: [README.md](README.md).

[![CI](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml/badge.svg)](https://github.com/athallaarl66/Snipe-code/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

CLI Rust untuk scaffold proyek, membaca isi codebase yang sudah ada, dan mengauditnya
untuk risiko keamanan. Semua berjalan lokal: tanpa akun, tanpa API key, tanpa layanan
AI. README versi Inggris: [README.md](README.md).

## Install

### Installer shell (tanpa Rust toolchain)

Linux dan macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.sh | less
bash install.sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.ps1 | more
.\install.ps1
```

Urutan kerja installer: unduh archive rilis ke direktori sementara, cek SHA-256 yang
dipublikasikan, cek bundle cosign kalau `cosign` terpasang, baru pindahkan binary-nya.
Tidak ada yang di-pipe ke shell. Kalau unduhan gagal atau cek integritas tidak lolos,
installer keluar dengan status non-zero, menyebut sebabnya, dan tidak meninggalkan
binary parsial. Direktori default `~/.local/bin` (POSIX) dan
`%LOCALAPPDATA%\Programs\snipe-code` (Windows); keduanya mencetak petunjuk PATH.
Bisa diubah lewat `SNIPE_INSTALL_VERSION`, `SNIPE_INSTALL_DIR`, dan
`SNIPE_RELEASE_BASE_URL`.

### cargo install (punya Rust toolchain)

```bash
cargo install snipe-code --locked
```

Memasang crate versi rilis dari crates.io ke `~/.cargo/bin`. Butuh Rust toolchain,
jadi ini bukan jalur "no Rust toolchain" di atas. Flag `--locked` tetap wajib di sini:
paket yang dipublikasikan membawa `Cargo.lock`-nya, dan tanpa flag itu Cargo
menyelesaikan dependency graph ulang, bertemu rilis `zip` yang dibutuhkan `docx-rs`
(semuanya sudah yanked di upstream), lalu berhenti tanpa memilih versi.

### Image container (Linux)

```bash
docker run --rm -v "$PWD:/work" -w /work ghcr.io/athallaarl66/snipe-code:latest audit . --export md
```

Image berisi binary Linux dari rilis di atas base distroless, jadi tidak ada shell,
package manager, atau apa pun selain binary itu. Tag-nya mengikuti tag rilis, dan
`latest` mengikuti rilis terbaru. Mount proyek yang mau diperiksa; container hanya
melihat yang di-mount.

### Build dari source

```bash
git clone https://github.com/athallaarl66/Snipe-code.git
cd Snipe-code
cargo build --release
```

Binary ada di `target/release/snipe-code` (`.exe` di Windows) dan portable, bisa
disalin ke mana saja. Folder `target/` bisa ratusan MB; setelah binary disalin keluar,
bersihkan dengan `cargo clean`.

Cargo tidak menaruh binary itu di `PATH`, jadi mengetik `snipe-code` di shell baru
tetap gagal dengan `'snipe-code' is not recognized as an internal or external command`.
Jalankan lewat path-nya dari folder checkout:

```powershell
.\target\release\snipe-code.exe faq
```

```bash
./target/release/snipe-code faq
```

`cargo run --release -- faq` melakukan hal yang sama tanpa keluar dari source. Kalau
mau `snipe-code` bisa dipanggil dari direktori mana pun, install hasil build lokal:

```bash
cargo install --path . --locked
```

Perintah itu menyalin binary ke `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` di
Windows), yang sudah ditambahkan ke `PATH` oleh installer Rust, jadi shell baru
langsung mengenali `snipe-code`. Kalau masih tidak dikenali, tambahkan sendiri
direktori itu ke `PATH`. Flag `--locked` wajib di sini: `cargo install --path .` tanpa
flag itu menyelesaikan dependency ulang, menemukan rilis `zip` yang dibutuhkan
`docx-rs` sudah yanked di upstream, lalu berhenti dengan
`failed to select a version for the requirement`. Dengan `--locked`, resolusi memakai
`Cargo.lock` repo dan install berhasil. Sama seperti `cargo install` lain, ini cuma menyalin sekali: jalankan ulang
setelah mengubah kode, atau tetap pakai `cargo run`.

### Uninstall

Hapus binary yang dipasang installer, atau `cargo uninstall snipe-code` kalau dipasang
lewat cargo. Tidak ada registry key atau file sistem yang disentuh.

## Command

| Command | Fungsi |
| --- | --- |
| `snipe-code` atau `snipe-code new` | Wizard: template, stack, database, nama proyek, AI tools, generate |
| `snipe-code analyze [PATH]` | Laporan read-only: bahasa dan ukurannya, struktur, entry point, dependency, limits |
| `snipe-code audit [PATH]` | Audit keamanan heuristik, export Markdown/PDF/DOCX |
| `snipe-code verify [PATH]` | Jalankan gate kualitas yang dideklarasikan proyek |
| `snipe-code version` | Versi binary (alias `--version`) |
| `snipe-code help [command]` | Bantuan umum dan per perintah |
| `snipe-code faq` | Install, uninstall, troubleshooting |

### Setiap command dijalankan dari mana dan menulis apa

| Command | Dijalankan dari | Dibaca | Ditulis | Exit code |
| --- | --- | --- | --- | --- |
| `new` (atau tanpa argumen) | direktori yang nanti memuat proyek baru. Proyeknya dibuat di `<direktori saat ini>/<nama>`, jadi `cd` dulu ke folder induknya. Argumen path tidak memindahkannya. | template yang tertanam di binary, atau `SNIPE_CODE_TEMPLATES` | `<direktori saat ini>/<nama>/`, lalu `git init` dan initial commit di dalamnya | `0` berhasil, `1` generate gagal, `2` usage error, `3` error runtime |
| `analyze [PATH]` | mana saja; `PATH` default direktori saat ini | file proyek | tidak ada | `0`, `2`, `3` |
| `audit [PATH]` | mana saja; `PATH` default direktori saat ini | file proyek | `Audit_Report.md`, `.pdf`, `.docx` di dalam `PATH` | `0`, `1`, `2`, `3` |
| `verify [PATH]` | direktori proyeknya, atau arahkan `PATH` ke sana | manifest | tidak ada dari snipe-code sendiri; gate yang dijalankannya menulis ke dalam proyek | `0`, `1`, `2`, `3` |
| `help`, `version`, `faq` | mana saja | tidak ada | tidak ada | `0` |

Dua command berikut mengubah file di disk:

- `audit` menulis laporannya ke dalam direktori yang di-scan, bersebelahan dengan kode.
  Tanpa terminal, prompt format dilewati dan langsung export Markdown, dengan pesan
  `Terminal non-interaktif — default export: markdown.` dicetak lebih dulu, jadi
  perintahnya tetap jalan apa adanya di script.
- `verify` bukan read-only. Gate-nya adalah perintah milik proyek itu sendiri, jadi
  `npm install` meninggalkan `node_modules/` dan `package-lock.json`, `go mod tidy`
  menulis ulang `go.mod` dan menambah `go.sum`, `dotnet restore` meninggalkan `obj/`,
  dan Maven meninggalkan `target/`. Tidak ada yang dihapus, tapi isi foldernya tidak
  sama lagi sesudahnya. Pakai `--no-install` untuk melewati gate install dan memakai
  dependency yang sudah ada di disk.

## new: scaffold proyek

```bash
snipe-code                                    # wizard
snipe-code --dry-run                          # preview struktur, tanpa menulis file
snipe-code new --ai-tools cursor,claude-code  # pilih artifact AI tanpa prompt
snipe-code new --ai-tools none --name demo --json
```

Alur wizard: template, frontend, backend, database, nama proyek, prompt tabrakan
folder (Overwrite atau Cancel), generate, CI/CD, Docker, AI workspace, lalu
`git init` dan initial commit.

Direktori proyeknya dibuat di `<direktori saat ini>/<nama>`, jadi jalankan wizard dari
folder yang memang harus memuatnya. Namanya datang dari prompt atau `--name`; tidak ada
argumen yang memindahkan outputnya.

| Flag | Efek |
| --- | --- |
| `--dry-run` | Cetak struktur tanpa membuat file |
| `--ai-tools <ids\|none>` | Artifact agen AI yang digenerate, dipisah koma; `none` melewati prompt dan tidak membuat artifact |
| `--name <n>` | Lewati prompt nama proyek |
| `--json` | Hasil generate (`project`, `ai_tools`, `ai_artifacts`, `ai_overlaps`, `files`, `outcome`) sebagai JSON di stdout, progress ke stderr |

Tanpa terminal (script atau CI) wizard memakai nilai default dan tidak pernah
bertanya: template none, frontend Next.js, backend none, database none, nama proyek
`my-project`. Pakai `--name` supaya nama proyeknya jelas.

Stack yang didukung, frontend: Next.js, React (Vite), Vue, Nuxt. Backend: .NET 8,
NestJS, Go (Gin), Spring Boot, Laravel 12. Database: PostgreSQL, MySQL. Proyek hasil
generate membawa `.env.example`, `.gitignore`, `docker-compose.yml` untuk kombinasi
backend dan database, workflow CI/CD, serta infrastruktur test.

## analyze: proyek ini isinya apa

```bash
snipe-code analyze .                 # laporan teks
snipe-code analyze ../lain --json    # laporan yang sama sebagai JSON di stdout
```

Read-only dan offline: tidak ada file di proyek target yang dibuat, diubah, atau
dihapus, dan tidak butuh jaringan. Lima bagian laporannya:

- Languages: setiap bahasa yang terdeteksi dengan jumlah file dan jumlah baris.
- Structure: entri tingkat atas dan bagian proyek yang dikenali detektor audit.
- Entry points: titik mulai eksekusi per bagian, misal `frontend/src/main.tsx`,
  `backend/main.go`, `src/main.rs`, `*Application.java`, `Program.cs`, `artisan`.
- Dependencies: isi manifest yang dibaca (`package.json`, `composer.json`, `go.mod`,
  `Cargo.toml`, `pom.xml`, `*.csproj`).
- Limits: apa yang tidak bisa dianalisis.

Traversal memakai walker yang sama dengan audit, jadi direktori dependency dan hasil
build (`node_modules`, `vendor`, `target`, `dist`, `build`, `bin`, `obj`, `out`,
`coverage`, `reports`) dan direktori hidden tidak ikut dihitung. Satu pengecualian:
`src/bin` berisi source betulan di crate Rust, jadi tetap dihitung.

Isi bagian Limits:

- Angka yang dilaporkan adalah jumlah baris, bukan baris kode. Baris kosong dan
  komentar ikut dihitung, dan tidak ada pemisahan code versus comment, karena itu
  butuh parser per bahasa.
- `pom.xml` dan `*.csproj` dipindai untuk nama elemen dependency, bukan di-parse,
  karena tool ini tidak membawa XML parser. Empat manifest lain di-parse.
- File dengan ekstensi di luar tabel bahasa dilaporkan sebagai not analyzed beserta
  jumlah file-nya, bukan dibuang diam-diam.

Exit status: `0` laporan tercetak, `2` salah pemakaian, `3` path tidak ada atau bukan
folder. Output `--json` memuat temuan yang sama dengan laporan teks.

## audit: security scan

```bash
snipe-code audit .                        # prompt pilih format export
snipe-code audit . --export md            # hanya Markdown
snipe-code audit . --export pdf,docx      # PDF dan DOCX
snipe-code audit . --export md --json     # file laporan plus JSON di stdout
```

Format yang valid: `md`, `pdf`, `docx`. Duplikat dinormalisasi, token kosong atau
tidak dikenal jadi usage error. Dengan `--json` file laporan tetap ditulis, sedangkan
ringkasan "exported files" pindah ke stderr supaya stdout bisa di-parse.

Tiap format ditulis ke dalam direktori yang di-scan sebagai `Audit_Report.md`,
`Audit_Report.pdf`, dan `Audit_Report.docx`. Ketiga nama itu ada di `.gitignore` proyek
hasil generate, jadi laporannya tidak ikut ke-commit tanpa sengaja.

Exit code:

- `0`: audit selesai, tidak ada temuan HIGH
- `1`: ada temuan HIGH
- `2`: usage error, misal format export tidak valid
- `3`: error runtime atau export, misal path tidak ada

Audit ini heuristik. Laporan menyebut context yang terdeteksi, rule group yang
dievaluasi, dan path yang dikecualikan. "No findings detected by evaluated rules"
bukan bukti proyek aman.

## verify: jalankan gate proyek

```bash
snipe-code verify .                        # semua gate yang dideklarasikan
snipe-code verify ../proyek --only test     # ulangi satu gate
snipe-code verify . --security              # tambah gate kerentanan dependency
snipe-code verify . --no-install            # lewati gate install
```

Gate diturunkan dari manifest proyek, bukan dari daftar stack tool ini, jadi
`snipe-code verify` jalan di proyek apa pun yang dikenali:

| Manifest | Gate yang dijalankan |
| --- | --- |
| `package.json` | `npm install`, lalu `npm run <script>` untuk `typecheck`, `lint`, `test`, `build` yang dideklarasikan |
| `composer.json` + `artisan` | `composer install --no-interaction --no-progress`, `php artisan test` |
| `composer.json` + `phpunit.xml` | `composer install --no-interaction --no-progress`, `php vendor/bin/phpunit` |
| `go.mod` | `go mod download`, `go mod tidy`, `go vet ./...`, `go test ./...`, `go build ./...` |
| `pom.xml` | `mvn test`, `mvn package -DskipTests` |
| `*.csproj` / `*.sln` | `dotnet restore`, `dotnet test --no-restore`, `dotnet build --no-restore` |

Urutan gate tetap: `install`, `lint`, `typecheck`, `test`, `build`, ditambah
`security` kalau diminta. Setiap gate melaporkan satu dari empat status:

- `passed`: perintah selesai dengan exit 0
- `failed`: perintah exit non-zero, nama gate dan perintahnya dicetak
- `not-applicable`: proyek tidak mendeklarasikan apa pun untuk gate itu
- `unavailable`: tool-nya tidak terpasang di mesin ini, gate lain tetap lanjut

Exit code:

- `0`: semua gate yang berlaku lulus
- `1`: ada gate yang gagal
- `2`: usage error, misal `--only` menunjuk gate yang tidak berlaku
- `3`: tidak ada yang dijalankan, proyeknya tidak dikenali atau semua tool tidak ada.
  Run seperti itu mencetak `tidak ada gate yang dijalankan` lalu keluar tanpa
  menjalankan satu perintah pun.

Perbedaan `verify` dan `audit`: `verify` menjalankan tool milik proyek (`npm`,
`composer`, `mvn`, `dotnet`, `go`, `cargo`) sehingga bisa memakai jaringan, sama
seperti CI proyek itu. Tool `snipe-code` sendiri tidak pernah memanggil jaringan, ia
hanya menjalankan perintah yang sudah dideklarasikan proyek dan mencetak tiap
perintah sebelum dijalankan. Untuk analisis read-only yang sepenuhnya offline, pakai
`snipe-code analyze` atau `snipe-code audit`.

Perintah-perintah itu juga menulis di dalam proyek: `node_modules/` dan
`package-lock.json` dari `npm install`, `go.sum` plus `go.mod` yang dirapikan
`go mod tidy`, `obj/` dari `dotnet restore`, `target/` dari Maven. Jadi satu kali
`verify` meninggalkan folder yang isinya tidak sama lagi, dan tidak ada yang dihapus.
`--no-install` melewati gate install, melaporkannya sebagai `not-applicable` dengan
alasan `skipped by --no-install`, dan tetap menjalankan gate lain. Itu yang kamu mau
kalau dependency sudah terpasang. Menggabungkan `--no-install` dengan
`--only install` adalah usage error, karena meminta gate itu sekaligus melewatinya.

`--security` memakai scanner bawaan ekosistem: `npm audit`, `composer audit`,
`dotnet list package --vulnerable`. Go dan Java melaporkan gate ini `not-applicable`
beserta alasannya, karena `govulncheck` dan OWASP dependency-check bukan bagian dari
toolchain bawaannya.

Proyek dengan lebih dari satu bagian, misal `frontend/` dan `backend/`, diverifikasi
per bagian di direktorinya masing-masing. Proyek yang tidak dikenali melaporkan
`no gates found`. Batas yang diketahui: deteksinya memakai `audit::detect_scopes`,
yang belum mengenali Python dan Ruby.

## Template

Isi setiap stack dan preset ada sebagai file nyata di `templates/`, lalu di-embed ke
binary saat build, jadi tool tetap offline dan tetap satu file executable:

```text
templates/
├── frontends/<id>/          # nextjs, react-vite, vue, nuxtjs
│   ├── manifest.txt         # name=..., init=..., install=...
│   ├── folders.txt          # satu path folder per baris
│   └── files/<path>         # isi file yang digenerate di path yang sama
├── backends/<id>/           # dotnet8, nestjs, go-gin, spring-boot, laravel
│   └── (manifest.txt, folders.txt, files/<path>)
└── presets/<id>/            # preset.json dan content/<template_source>.txt
```

Menambah atau mengubah template tidak perlu menyentuh Rust: edit file di `templates/`
lalu `cargo build`.

Override lokal tanpa recompile: arahkan `SNIPE_CODE_TEMPLATES` ke direktori dengan
layout seperti `templates/`.

```bash
SNIPE_CODE_TEMPLATES=/path/ke/templates-kerja snipe-code new
```

Resolusinya per file. File yang disediakan direktori override menang atas default
built-in, file yang tidak disediakan tetap memakai default, jadi override sebagian
tetap sah. Contoh: hanya `frontends/nextjs/files/package.json` untuk menaikkan versi
dependency. Output `new` menyebut sumbernya, `Templates: local templates from <dir>`
atau `Templates: built-in templates`.

## AI agent tools

Proyek hasil generate bisa membawa artifact instruksi untuk tool AI yang dipakai,
masing-masing di path yang tool itu baca. Ada tiga jenis file, dan cara tiap jenis sampai
ke agen berbeda.

| Jenis | File | Kapan jalan |
| --- | --- | --- |
| Rules | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/snipe-code.mdc`, `.windsurf/rules/snipe-code.md`, `.github/copilot-instructions.md`, `.agents/rules/antigravity.md` | dibaca tool di awal sesi, tidak perlu diketik apa pun |
| Skill | `.claude/skills/snipe-code/SKILL.md`, `.opencode/skills/snipe-code/SKILL.md`, `.hermes/skills/snipe-code/SKILL.md`, `.agents/skills/snipe-code.md` | muncul di daftar skill tool, dan agen memuatnya saat deskripsinya cocok dengan pekerjaan |
| Command | `.claude/commands/snipe-code.md`, `.opencode/commands/snipe-code.md`, `.agents/workflows/snipe-code.md` | kamu ketik `/snipe-code` di tool itu |

### Isi rules dan skill

Keduanya membawa panduan yang sama: baca `.ai-workspace/rules.md` sebelum mengubah kode,
jaga perubahan tetap sebatas permintaan, jalankan perintah build dan test proyek sebelum
mengklaim perubahan bekerja, dan jalankan ulang `snipe-code audit . --export md` kalau
perubahan menyentuh input handling, secret, atau query. Body-nya identik untuk semua
tool; yang berbeda cuma frontmatter, karena tiap format mendeklarasikan metadatanya
dengan caranya sendiri. `skill` memakai `name` dan `description`, rule `mdc` memakai
`globs` dan `alwaysApply`, `command` memakai `description`.

Command `/snipe-code` adalah audit yang sama, dijalankan manual: menjalankan
`snipe-code audit . --export md`, memperbaiki setiap finding HIGH, menjalankan perintah
test proyek, lalu mengulang auditnya. Ini satu-satunya command yang didapat proyek hasil
generate, dan tidak menerima argumen: `analyze`, `verify`, dan `new` tidak punya padanan
command.

`snipe-code` harus ada di `PATH` supaya langkah audit di skill dan command bisa jalan.

### Tool mana dapat apa

| Tool | File | Cara pakai |
| --- | --- | --- |
| Cross-tool (AGENTS.md) | `AGENTS.md` | dibaca agen apa pun yang mengikuti konvensi AGENTS.md |
| Claude Code | `CLAUDE.md`, `.claude/skills/snipe-code/SKILL.md`, `.claude/commands/snipe-code.md` | skill-nya memuat sendiri; `/snipe-code` memulai audit |
| Cursor | `.cursor/rules/snipe-code.mdc`, `.cursorrules` | menempel di setiap edit, karena rule-nya menyetel `alwaysApply: true` |
| OpenCode | `.opencode/skills/snipe-code/SKILL.md`, `.opencode/commands/snipe-code.md` | skill-nya memuat sendiri; `/snipe-code` memulai audit |
| Antigravity | `GEMINI.md`, `.agents/rules/antigravity.md`, `.agents/skills/snipe-code.md`, `.agents/workflows/snipe-code.md` | rule dan skill-nya memuat sendiri; workflow-nya memulai audit |
| GitHub Copilot | `.github/copilot-instructions.md` | dibaca sebagai instruksi repositori |
| Windsurf | `.windsurf/rules/snipe-code.md` | dibaca sebagai rule proyek |
| Hermes Agent | `.hermes/skills/snipe-code/SKILL.md` | skill-nya memuat sendiri |
| Codex | `AGENTS.md` | file yang sama dengan entri cross-tool |
| Gemini CLI | `GEMINI.md` | file yang sama dengan entri Antigravity |

Codex dan Gemini CLI menunjuk file yang sudah ditulis entri lain. Kalau keduanya dipilih,
file-nya ditulis sekali dan overlap-nya dilaporkan, jadi outputnya berbunyi `AGENTS.md:
kept agents-md, skipped codex`.

### Cara generate

```bash
snipe-code                                     # wizard langkah 6, multi-select, opsional
snipe-code new --ai-tools cursor,claude-code   # lewati prompt pemilihan
snipe-code new --ai-tools none --name demo     # tanpa artifact agen
snipe-code new --dry-run                       # tampilkan struktur, tidak menulis file
```

Semua path relatif terhadap root proyek, dan tidak ada konfigurasi agen yang masuk ke
home directory.

`.ai-workspace/rules.md` dan `.ai-workspace/rules.mdc` selalu ditulis, dipilih atau tidak
dipilih tool agen. Keduanya membawa panduan yang sama dengan yang dipakai artifact per
tool, jadi tool tanpa konvensi khusus tetap menemukan rules di satu tempat yang jelas.

Daftar tool, path, dan formatnya ada di `templates/ai-tools.json` (format:
`markdown`, `mdc`, `skill`, `command`). Menambah tool berarti menambah entri registry,
bukan mengubah kode. Format yang tidak dikenal menghasilkan error, bukan artifact yang
dilewati diam-diam.

- `--ai-tools <ids|none>` melewati prompt pemilihan; `none` berarti tanpa artifact agen.
- `--json` menaruh hasil generate di stdout, progress ke stderr.
- Path yang diklaim lebih dari satu tool, misal `AGENTS.md` oleh cross-tool dan Codex,
  ditulis sekali dan overlap-nya dilaporkan di output.

## Platform yang didukung

Daftar target ada di `.github/release-targets.json`. Matrix rilis dan validasi rilis
sama-sama membaca file itu, dan tabel di bawah ini mengikutinya.

| Platform | Target | Archive |
| --- | --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `.zip` |
| macOS x86_64 (Intel) | `x86_64-apple-darwin` | `.tar.gz` |
| macOS arm64 (Apple silicon) | `aarch64-apple-darwin` | `.tar.gz` |

Menambah target cukup mengedit file target itu: matrix `package` dan hitungan archive
di job `publish` membaca daftar yang sama, jadi tidak ada edit kedua.

### Cek rilis

Tiap archive berisi executable, `LICENSE`, `README.md`, dan `SOURCE_REVISION`, plus
checksum dan bundle signature keyless:

```bash
sha256sum --check snipe-code-<version>-<target>.tar.gz.sha256
cosign verify-blob --bundle snipe-code-<version>-<target>.tar.gz.bundle \
  --certificate-identity-regexp 'https://github.com/athallaarl66/Snipe-code/.github/workflows/release.yml@.*' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  snipe-code-<version>-<target>.tar.gz
```

Jalankan itu hanya untuk artifact yang benar-benar ada di halaman GitHub Releases.
Job rilis yang sama juga mempublikasikan crate-nya, jadi `cargo install snipe-code --locked`
memasang versi yang sama dengan yang dirilis.

## CI

CI repo ini (`.github/workflows/ci.yml`) menjalankan gate Rust plus matrix yang
menguji proyek hasil generate:

| Job | Yang diuji |
| --- | --- |
| `rust` | `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` |
| `generated-frontend` | Next.js, React Vite, Vue, Nuxt: `npm install`, `npm run typecheck`, `npm run lint`, `npm run test`, `npm run build` |
| `generated-backend` | NestJS, Go Gin, Spring Boot, .NET 8, Laravel 12: perintah install, lint, test, dan build yang mereka deklarasikan |
| `compose` | `docker compose config` untuk PostgreSQL, MySQL, dan kombinasi NestJS + PostgreSQL |

Gate lokal:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Dokumentasi

- `docs/features/`: PRD dan technical design.
- `docs/production/`: breakdown tugas per fitur.

## Author

Athalla ([@athallaarl66](https://github.com/athallaarl66))

## Lisensi

MIT, lihat [LICENSE](LICENSE).
