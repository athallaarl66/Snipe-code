use snipe_code::commands::new;
use snipe_code::audit;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "audit" {
        // snipe-code audit [path] [--export md,pdf]
        let path = args.get(2).map(|s| s.as_str()).unwrap_or(".");
        let export_format = args.iter()
            .find(|a| a.starts_with("--export"))
            .and_then(|a| a.split('=').nth(1))
            .unwrap_or("md");

        match audit::run_audit(path, export_format) {
            Ok(report) => {
                println!("Audit complete! Report saved to {}/Audit_Report.md", path);
                println!("\n{}", report);
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
