use snipe_code::commands::new;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());

    new::run(dry_run);
}
