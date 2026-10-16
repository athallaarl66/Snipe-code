use snipe_code::audit;
use snipe_code::commands::new;
use snipe_code::verify;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = std::process::exit;

    match args.get(1).map(|s| s.as_str()) {
        // Tanpa command → wizard (backward compatible)
        None => run_new_cmd(&args),
        Some("help") | Some("--help") | Some("-h") => match args.get(2) {
            Some(cmd) => print_command_help(cmd),
            None => print_help(),
        },
        Some("faq") => print_faq(),
        Some("version") | Some("--version") => println!("{}", env!("CARGO_PKG_VERSION")),
        Some("new") => run_new_cmd(&args),
        Some("audit") => run_audit_cmd(&args),
        Some("verify") => run_verify_cmd(&args),
        // Backward compatible: snipe-code --dry-run
        Some("--dry-run") => run_new_cmd(&args),
        Some(other) if other.starts_with('-') => {
            println!("Unknown option: {}", other);
            println!("Run 'snipe-code help' untuk daftar perintah.");
            code(2);
        }
        Some(other) => {
            println!("Unknown command: {}", other);
            println!("Run 'snipe-code help' untuk daftar perintah.");
            code(2);
        }
    }
}

/// Parses the `new` options. The wizard itself is interactive; these options let
/// a script or CI run it without a terminal.
fn run_new_cmd(args: &[String]) {
    let mut options = new::Options {
        dry_run: args.iter().any(|arg| arg == "--dry-run"),
        json: args.iter().any(|arg| arg == "--json"),
        ..Default::default()
    };

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--ai-tools" => match args.get(i + 1) {
                Some(value) => {
                    options.ai_tools = Some(value.clone());
                    i += 2;
                }
                None => {
                    eprintln!(
                        "Usage error: --ai-tools membutuhkan nilai (id dipisah koma, atau 'none')."
                    );
                    std::process::exit(2);
                }
            },
            "--name" => match args.get(i + 1) {
                Some(value) => {
                    options.name = Some(value.clone());
                    i += 2;
                }
                None => {
                    eprintln!("Usage error: --name membutuhkan nilai.");
                    std::process::exit(2);
                }
            },
            _ => i += 1,
        }
    }

    if let Err(e) = new::run(options) {
        eprintln!("snipe-code: {}", e);
        std::process::exit(1);
    }
}

fn run_audit_cmd(args: &[String]) {
    let mut path = ".".to_string();
    let mut export_opt: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--export" {
            if let Some(val) = args.get(i + 1) {
                export_opt = Some(val.clone());
                i += 2;
            } else {
                eprintln!("Usage error: --export membutuhkan nilai.");
                std::process::exit(2);
            }
        } else {
            path = args[i].clone();
            i += 1;
        }
    }

    let export_format = match export_opt {
        Some(f) if !f.trim().is_empty() => f,
        Some(_) => {
            eprintln!("Usage error: --export tidak boleh kosong.");
            std::process::exit(2);
        }
        None => {
            if console::Term::stdout().is_term() {
                audit::select_export_formats().join(",")
            } else {
                println!("Terminal non-interaktif — default export: markdown.");
                "md".to_string()
            }
        }
    };

    match audit::run_audit_report(&path, &export_format) {
        Ok(result) => {
            println!("Audit complete! Exported files:");
            for file in &result.exported_files {
                println!("  - {}/{}", path, file);
            }
            if result.policy_failed {
                eprintln!("Audit policy failed: {} HIGH finding(s)", result.high);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Audit failed: {}", e);
            let code = if e.starts_with("unsupported export format")
                || e.contains("empty token")
                || e.contains("at least one export")
            {
                2
            } else {
                3
            };
            std::process::exit(code);
        }
    }
}

fn run_verify_cmd(args: &[String]) {
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    if rest.iter().any(|token| token == "--help" || token == "-h") {
        println!("{}", VERIFY_HELP);
        return;
    }
    let options = match verify::Options::parse(&rest) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Usage error: {}", error);
            std::process::exit(2);
        }
    };
    let report = match verify::run(&options.path, &options, &verify::exec::run) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Usage error: {}", error);
            std::process::exit(2);
        }
    };
    if report.parts.is_empty() {
        eprintln!("verify: no gates found in {}", options.path.display());
    }
    verify::print_report(&report);
    std::process::exit(verify::exit_status(&report));
}

fn print_help() {
    println!("{}", HELP_TEXT);
    println!();
    print_faq();
}

fn print_faq() {
    println!("{}", FAQ_TEXT);
}

fn print_command_help(cmd: &str) {
    match cmd {
        "audit" => println!("{}", AUDIT_HELP),
        "verify" => println!("{}", VERIFY_HELP),
        "new" => println!("{}", NEW_HELP),
        "help" => print_help(),
        _ => {
            println!("Unknown topic: {}", cmd);
            println!("Run 'snipe-code help' untuk daftar perintah.");
        }
    }
}

const HELP_TEXT: &str = r#"SNIPE-CODE CLI — Rust scaffolding and heuristic audit tool

USAGE
    snipe-code [COMMAND] [OPTIONS]

COMMANDS
    new        Jalankan wizard interaktif untuk generate proyek baru
               (default jika tidak ada command)
    audit      Scan folder proyek dan export laporan security audit
    verify     Periksa gate kualitas proyek yang sudah ada
    help       Tampilkan bantuan ini (alias: --help, -h)
    version    Tampilkan versi CLI yang di-compile (alias: --version)
    faq        Tampilkan FAQ (install, uninstall, troubleshooting)

OPTIONS
    --dry-run      Preview struktur folder tanpa membuat file (dengan new)
    --ai-tools <l> (new) AI agent tools yang di-generate, id dipisah koma
                   (mis. cursor,claude-code) atau 'none' untuk tanpa artifact
    --name <n>     (new) Nama proyek; melewati prompt nama
    --json         (new) Hasil generate sebagai JSON di stdout; progress ke stderr
    --export <f>   Format export audit: md, pdf, docx (koma, contoh: pdf,docx)
    --only <g>     (verify) Batasi ke satu gate: install, lint, typecheck, test,
                   build, security
    --security     (verify) Tambahkan gate kerentanan dependency
    --version      Tampilkan versi CLI (sama dengan 'snipe-code version')

CONTOH
    snipe-code                          Wizard interaktif
    snipe-code new                      Wizard interaktif
    snipe-code new --dry-run            Preview struktur tanpa membuat file
    snipe-code new --ai-tools cursor,claude-code
                                        Generate dengan artifact Cursor + Claude Code
    snipe-code new --ai-tools none --name demo --json
                                        Tanpa artifact agen, hasil JSON di stdout
    snipe-code audit .                  Scan folder ini (pilih format interaktif)
    snipe-code audit . --export pdf,docx
                                        Scan dan export PDF + DOCX langsung
    snipe-code verify .                 Periksa gate proyek di direktori ini
    snipe-code verify ../app --only test
                                        Jalankan hanya gate test

DETAIL PER COMMAND
    snipe-code help new
    snipe-code help audit
    snipe-code help verify
"#;

const FAQ_TEXT: &str = r#"FAQ

Q: 'snipe-code' is not recognized as an internal or external command
A: Binary belum di-build/di-install. Solusi:
   1) Build dari source (butuh Rust):  cargo build --release
      Binary ada di target/release/snipe-code.exe — jalankan langsung
      atau salin ke folder mana pun (portable).
   2) Install ke PATH:  cargo install --path .
      Setelah ini 'snipe-code' bisa dipanggil dari direktori mana pun.

Q: Bagaimana cara uninstall?
A: - Jika di-install via cargo:  cargo uninstall snipe-code
   - Jika memakai exe portable: cukup hapus file snipe-code.exe
     (tidak ada registry/system files yang disentuh)

Q: Apa perbedaan build --release dan install --path?
A: build --release menghasilkan file exe di target/release/ (bisa disalin,
   portable). install --path . menyalinnya ke ~/.cargo/bin supaya perintah
   'snipe-code' dikenali dari mana saja. Install menambah ~5MB; folder
   target/ (hasil compile) yang besar — hapus dengan:  cargo clean

Q: Audit tidak bisa pilih format?
A: Jalankan 'snipe-code audit .' tanpa --export — muncul menu pilihan
   format (Markdown/PDF/DOCX). Atau beri flag langsung:
   snipe-code audit . --export pdf
"#;

const NEW_HELP: &str = r#"snipe-code new — Generate proyek baru

USAGE
    snipe-code new [--dry-run] [--ai-tools <ids|none>] [--name <n>] [--json]

ALUR WIZARD
    1. Pilih template (No Template / Company Profile / Portfolio / Blog)
    2. Pilih Frontend (Next.js, React, Vue, Nuxt, atau None)
    3. Pilih Backend (.NET 8, NestJS, Go, Spring Boot, Laravel, atau None)
    4. Pilih Database (PostgreSQL, MySQL, atau None)
    5. Masukkan nama proyek
    6. Pilih AI agent tools (multi-select, opsional — default: tidak ada)
    7. Jika folder sudah ada: Overwrite atau Cancel

OPTIONS
    --dry-run            Tampilkan struktur yang akan dibuat tanpa membuat file
    --ai-tools <ids>     AI agent tools yang di-generate, id dipisah koma:
                         agents-md, claude-code, cursor, opencode, antigravity,
                         github-copilot, windsurf, hermes-agent, codex, gemini-cli
                         Nilai 'none' berarti tidak ada artifact agen. Opsi ini juga
                         melewati prompt pemilihan AI tools.
    --name <n>           Nama proyek; melewati prompt nama
    --json               Hasil generate (tools, artifact, outcome) sebagai JSON
                         di stdout; progress dan peringatan ke stderr

CONTOH
    snipe-code new
    snipe-code new --dry-run
    snipe-code new --ai-tools cursor,claude-code
    snipe-code new --ai-tools none --name demo
    snipe-code new --ai-tools claude-code --name demo --json
"#;

const AUDIT_HELP: &str = r#"snipe-code audit — Security audit folder proyek

USAGE
    snipe-code audit [PATH] [--export <md,pdf,docx>]

DESKRIPSI
    Scan folder proyek untuk memeriksa kelengkapan file standar dan
    risiko keamanan, lalu export laporan. Tanpa --export, muncul menu
    pilihan format interaktif (Markdown/PDF/DOCX).

ARGUMEN
    PATH         Folder yang di-scan (default: direktori saat ini)

OPTIONS
    --export <md,pdf,docx>   Format export langsung tanpa prompt
                             (bisa kombinasi, pisahkan dengan koma)

CONTOH
    snipe-code audit .                        Pilih format interaktif
    snipe-code audit . --export pdf,docx      Export PDF + DOCX langsung
    snipe-code audit ../proyek --export md    Export markdown
"#;

const VERIFY_HELP: &str = r#"snipe-code verify — Periksa gate kualitas proyek

USAGE
    snipe-code verify [PATH] [--only <gate>] [--security]

DESKRIPSI
    Membaca manifest proyek lalu menjalankan gate yang memang dideklarasikan:
    install, lint, typecheck, test, build. Gate yang tidak dideklarasikan
    dilaporkan sebagai not-applicable, bukan dijalankan.

    Perintah ini menjalankan tool milik proyek (npm, composer, mvn, dotnet, go,
    cargo) sehingga bisa memakai jaringan. Untuk analisis read-only yang
    sepenuhnya offline, pakai 'snipe-code audit'.

ARGUMEN
    PATH            Folder proyek (default: direktori saat ini)

OPTIONS
    --only <gate>   Jalankan hanya gate ini (harus berlaku di proyek)
    --security      Tambahkan gate kerentanan dependency: npm audit,
                    composer audit, dotnet list package --vulnerable
    -h, --help      Bantuan perintah ini

EXIT STATUS
    0   semua gate yang berlaku lulus
    1   ada gate yang gagal (nama gate dan perintahnya dicetak)
    2   kesalahan pemakaian (opsi atau gate tidak dikenal)
    3   tidak ada gate yang dijalankan (proyek tidak dikenali / tool tidak ada)

CONTOH
    snipe-code verify .
    snipe-code verify ../proyek --only test
    snipe-code verify . --security
"#;
