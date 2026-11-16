fn run() {
    std::process::Command::new("sh")
        .arg("-c")
        .arg("echo fixed")
        .status()
        .unwrap();
}
