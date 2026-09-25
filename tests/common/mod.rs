//! Shared test utilities: unique temporary workspaces for generation tests.
//!
//! `Workspace` creates a unique empty directory under the OS temp dir, chdirs
//! into it, and restores + cleans up on drop. A process-wide mutex serializes
//! the chdir so parallel tests cannot stomp each other's working directory.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static SEQ: AtomicU64 = AtomicU64::new(0);

pub struct Workspace {
    pub dir: PathBuf,
    original: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Workspace {
    pub fn new(label: &str) -> Self {
        let guard = CWD_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let original = std::env::current_dir().expect("read cwd");
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("snipe-ws-{}-{}-{}", label, std::process::id(), n));
        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        fs::create_dir_all(&dir).expect("create workspace");
        std::env::set_current_dir(&dir).expect("chdir to workspace");
        Workspace {
            dir,
            original,
            _guard: guard,
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
        let _ = fs::remove_dir_all(&self.dir);
    }
}
