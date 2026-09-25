use snipe_code::audit;
use snipe_code::commands::new;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let code = std::process::exit;

    match args.get(1).map(|s| s.as_str()) {
        // Tanpa command → wizard (backward compatible)
        None => run_new(false),
        Some("help") | Some("--help") | Some("-h") => match args.get(2) {
            Some(cmd) => print_command_help(cmd),
            None => print_help(),
        },
        Some("faq") => print_faq(),
        Some("new") => run_new(dry_run),
        Some("audit") => run_audit_cmd(&args),
        // Backward compatible: snipe-code --dry-run
        Some("--dry-run") => run_new(true),
        Some(other) if other.starts_with('-') => {
            println!("Unknown option: {}", other);
            println!("Run 'snipe-code help' untuk daftar perintah.");
            code(1);
        }
        Some(other) => {
            println!("Unknown command: {}", other);
            println!("Run 'snipe-code help' untuk daftar perintah.");
            code(1);
        }
    }
}

fn run_new(dry_run: bool) {
    if let Err(e) = new::run(dry_run) {
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
                i += 1;
            }
        } else {
            path = args[i].clone();
            i += 1;
        }
    }

    let export_format = match export_opt {
        Some(f) if !f.trim().is_empty() => f,
        Some(_) => {
            println!("Warning: --export tidak boleh kosong — default ke markdown.");
            "md".to_string()
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

    match audit::run_audit(&path, &export_format) {
        Ok(files) => {
            println!("Audit complete! Exported files:");
            for file in &files {
                println!("  - {}/{}", path, file);
            }
        }
        Err(e) => {
            println!("Audit failed: {}", e);
        }
    }
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
        "new" => println!("{}", NEW_HELP),
        "help" => print_help(),
        _ => {
            println!("Unknown topic: {}", cmd);
            println!("Run 'snipe-code help' untuk daftar perintah.");
        }
    }
}

const HELP_TEXT: &str = r#"SNIPE-CODE CLI — Enterprise Dev-Forge

USAGE
    snipe-code [COMMAND] [OPTIONS]

COMMANDS
    new        Jalankan wizard interaktif untuk generate proyek baru
               (default jika tidak ada command)
    audit      Scan folder proyek dan export laporan security audit
    help       Tampilkan bantuan ini (alias: --help, -h)
    faq        Tampilkan FAQ (install, uninstall, troubleshooting)

OPTIONS
    --dry-run      Preview struktur folder tanpa membuat file (dengan new)
    --export <f>   Format export audit: md, pdf, docx (koma, contoh: pdf,docx)

CONTOH
    snipe-code                          Wizard interaktif
    snipe-code new                      Wizard interaktif
    snipe-code new --dry-run            Preview struktur tanpa membuat file
    snipe-code audit .                  Scan folder ini (pilih format interaktif)
    snipe-code audit . --export pdf,docx
                                        Scan dan export PDF + DOCX langsung

DETAIL PER COMMAND
    snipe-code help new
    snipe-code help audit
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
    snipe-code new [--dry-run]

ALUR WIZARD
    1. Pilih template (No Template / Company Profile / Portfolio / Blog)
    2. Pilih Frontend (Next.js, React, Vue, Nuxt, atau None)
    3. Pilih Backend (.NET 8, NestJS, Go, Spring Boot, Laravel, atau None)
    4. Pilih Database (PostgreSQL, MySQL, atau None)
    5. Masukkan nama proyek
    6. Jika folder sudah ada: Overwrite atau Cancel

OPTIONS
    --dry-run    Tampilkan struktur yang akan dibuat tanpa membuat file

CONTOH
    snipe-code new
    snipe-code new --dry-run
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
