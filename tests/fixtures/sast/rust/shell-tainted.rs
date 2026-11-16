fn run() {
    let input = std::env::var("SNIPE_INPUT").unwrap();
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("echo {input}"))
        .status()
        .unwrap();
}
