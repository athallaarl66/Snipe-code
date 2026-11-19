use std::fs;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_snipe-code");
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(std::path::PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "snipe-launcher-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn no_arguments_without_terminal_is_usage_and_does_not_write() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("canary"), "unchanged").unwrap();
    let output = Command::new(BIN)
        .current_dir(&fixture.0)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    let entries: Vec<_> = fs::read_dir(&fixture.0).unwrap().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        fs::read_to_string(fixture.0.join("canary")).unwrap(),
        "unchanged"
    );
}

#[test]
fn direct_commands_bypass_launcher_and_keep_status_and_json() {
    let fixture = Fixture::new();
    for args in [&["analyze", "--json"][..], &["verify", "--json"][..]] {
        let output = Command::new(BIN)
            .current_dir(&fixture.0)
            .stdin(Stdio::null())
            .args(args)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(if args[0] == "verify" { 3 } else { 0 })
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(value.is_object());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("What would you like to do?"));
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}

// Windows runtime PTY coverage is opt-in because it requires Python and pywinpty.
#[cfg(windows)]
#[test]
#[ignore = "requires Python with pywinpty"]
fn windows_pty_launcher_routes() {
    let fixture = Fixture::new();
    let script = r#"
import os, queue, sys, threading, time
from winpty import PtyProcess
binary, root = sys.argv[1:]
env = os.environ.copy()
for key in ('HOME', 'USERPROFILE', 'APPDATA', 'LOCALAPPDATA'):
    env[key] = root
env['SNIPE_CODE_TEMPLATES'] = os.path.join(root, 'no-templates')

def run(steps, exit_code=0):
    proc = PtyProcess.spawn([binary], cwd=root, env=env, dimensions=(40, 120))
    chunks = queue.Queue()
    def reader():
        try:
            while True: chunks.put(proc.read(4096))
        except EOFError: pass
    threading.Thread(target=reader, daemon=True).start()
    text = ''
    try:
        for expected, keys in steps:
            deadline = time.monotonic() + 10
            while expected not in text:
                remaining = deadline - time.monotonic()
                if remaining <= 0: raise AssertionError('missing %r in %r' % (expected, text))
                text += chunks.get(timeout=remaining)
            text = text.split(expected, 1)[1]
            if keys is not None: proc.write(keys)
        deadline = time.monotonic() + 10
        while proc.isalive() and time.monotonic() < deadline: time.sleep(0.05)
        assert not proc.isalive(), 'process did not exit'
        assert proc.exitstatus == exit_code, (proc.exitstatus, text)
    finally:
        if proc.isalive(): proc.terminate(force=True)

menu = 'What would you like to do?'
down = '\x1b[B'
run([('SNIPE-CODE v', None), ('Author: Athalla', None), ('Quick commands:', None),
     (menu, '\x1b')])
assert os.listdir(root) == [], os.listdir(root)
run([(menu, down * 6 + '\r')])
assert os.listdir(root) == [], os.listdir(root)
run([(menu, down * 5 + '\r'), ('Choose AI for global skill', '\x1b'),
     (menu, down * 6 + '\r')])
assert os.listdir(root) == [], os.listdir(root)
run([(menu, '\r'), ('[1/8]', '\x1b'), ('Cancelled', None), (menu, down * 6 + '\r')])
run([(menu, down * 2 + '\r'), ('not a sandbox', None), ('Project path', '\r')], 3)
run([(menu, down * 4 + '\r'), ('USAGE', None), (menu, down * 6 + '\r')])
run([(menu, down * 3 + '\r'), ('Security operation', down * 4 + '\r'),
     (menu, down * 6 + '\r')])
run([(menu, down * 3 + '\r'), ('Security operation', down * 3 + '\r'),
     ('Project path', '\r'), ('Select report formats', '\x1b'),
     ('Audit cancelled', None), (menu, down * 6 + '\r')])
run([(menu, down + '\r'), ('Project path', '\r'), ('Target: .', None),
     (menu, down * 6 + '\r')])
run([(menu, down * 3 + '\r'), ('Security operation', '\r'),
     ('Project path', '\r'), ('Target: .', None), (menu, down * 6 + '\r')])
assert os.listdir(root) == [], os.listdir(root)
"#;
    let output = Command::new("python")
        .args(["-c", script, BIN])
        .arg(&fixture.0)
        .output()
        .expect("Python with pywinpty is required");
    assert!(
        output.status.success(),
        "PTY harness failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn direct_scan_and_help_bypass_launcher() {
    let fixture = Fixture::new();
    for args in [&["scan"][..], &["help"][..]] {
        let output = Command::new(BIN)
            .current_dir(&fixture.0)
            .stdin(Stdio::null())
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("What would you like to do?"));
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}
