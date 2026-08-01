use snipe_code::commands::new;
use snipe_code::audit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "audit" {
        // snipe-code audit [path] [--export md,pdf,docx]
        let mut path = ".";
        let mut export_format = "md".to_string();

        let mut i = 2;
        while i < args.len() {
            if args[i] == "--export" {
                if let Some(val) = args.get(i + 1) {
                    export_format = val.clone();
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                path = &args[i];
                i += 1;
            }
        }

        match audit::run_audit(path, &export_format) {
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
    } else {
        let dry_run = args.contains(&"--dry-run".to_string());
        new::run(dry_run);
    }
}
