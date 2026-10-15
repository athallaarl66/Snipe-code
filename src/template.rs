use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// Environment variable naming a directory laid out like `templates/`. Files it
/// provides replace the built-in default for the same relative path.
pub const OVERRIDE_ENV: &str = "SNIPE_CODE_TEMPLATES";

static TEMPLATES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/templates");

thread_local! {
    /// Override directory that supplied a file to this generation, for reporting.
    static LOCAL_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

fn override_root() -> Option<PathBuf> {
    let dir = std::env::var_os(OVERRIDE_ENV)?;
    let path = PathBuf::from(dir);
    path.is_dir().then_some(path)
}

fn remember(path: &Path) {
    LOCAL_ROOT.with(|slot| *slot.borrow_mut() = Some(path.to_path_buf()));
}

/// Reads `templates/<rel>`: the override directory wins per file, the embedded
/// default is the fallback for anything the override does not provide.
pub fn read(rel: &str) -> Option<String> {
    let local = override_root().and_then(|root| {
        std::fs::read_to_string(root.join(rel))
            .ok()
            .map(|text| (root, text))
    });
    if let Some((root, text)) = local {
        remember(&root);
        return Some(text);
    }
    TEMPLATES
        .get_file(rel)
        .and_then(|file| file.contents_utf8())
        .map(str::to_string)
}

/// Every file under `templates/<rel_dir>` as `templates/`-relative paths, sorted.
pub fn list(rel_dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(dir) = TEMPLATES.get_dir(rel_dir) {
        collect(dir, &mut out);
    }
    out.sort();
    out
}

fn collect(dir: &Dir<'_>, out: &mut Vec<String>) {
    for file in dir.files() {
        out.push(file.path().to_string_lossy().replace('\\', "/"));
    }
    for sub in dir.dirs() {
        collect(sub, out);
    }
}

/// Whether built-in or local template content supplied the last generation.
pub fn source_label() -> String {
    match LOCAL_ROOT.with(|slot| slot.borrow().clone()) {
        Some(root) => format!("local templates from {}", root.display()),
        None => "built-in templates".to_string(),
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub structure: TemplateStructure,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TemplateStructure {
    #[serde(default)]
    pub folders: Vec<String>,
    #[serde(default)]
    pub files: Vec<FileDef>,
    #[serde(default)]
    pub rules: GenerationRules,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FileDef {
    pub path: String,
    pub template_source: String,
    #[serde(default)]
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct GenerationRules {
    #[serde(default)]
    pub require_backend: bool,
    #[serde(default)]
    pub require_database: bool,
    #[serde(default)]
    pub docker_compose: bool,
    #[serde(default)]
    pub security_configs: bool,
}

pub struct TemplateRegistry;

/// Display order in the wizard; `content/` under `presets/` holds file bodies.
const PRESET_IDS: &[&str] = &["none", "company-profile", "portfolio", "blog"];

impl TemplateRegistry {
    pub fn load_all() -> Vec<Template> {
        Self::ids().iter().filter_map(|id| Self::get(id)).collect()
    }

    pub fn get(id: &str) -> Option<Template> {
        let rel = format!("presets/{id}/preset.json");
        let raw = read(&rel)?;
        Some(
            serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("invalid template JSON at templates/{rel}: {e}")),
        )
    }

    pub fn ids() -> &'static [&'static str] {
        PRESET_IDS
    }
}
