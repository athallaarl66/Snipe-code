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

/// Validate portable relative paths, including Windows forms on Unix hosts.
pub(crate) fn validate_relative(path: &str) -> Result<(), String> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty()
        || path.contains(['\\', ':', '\0'])
        || trimmed.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' '])
        })
    {
        return Err(format!("invalid template path '{path}'"));
    }
    Ok(())
}

fn is_link(metadata: &std::fs::Metadata) -> bool {
    #[allow(unused_mut)]
    let mut link = metadata.file_type().is_symlink();
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        link |= metadata.file_attributes() & 0x400 != 0;
    }
    link
}

/// Reject links anywhere in an override tree before generation reads it.
pub(crate) fn preflight_override() -> Result<(), String> {
    fn visit(path: &Path) -> Result<(), String> {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|e| format!("cannot inspect template '{}': {e}", path.display()))?;
        if is_link(&metadata) {
            return Err(format!(
                "template path contains a link: '{}'",
                path.display()
            ));
        }
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path)
                .map_err(|e| format!("cannot inspect template '{}': {e}", path.display()))?
            {
                visit(&entry.map_err(|e| e.to_string())?.path())?;
            }
        }
        Ok(())
    }
    if let Some(root) = std::env::var_os(OVERRIDE_ENV).map(PathBuf::from) {
        let absolute = if root.is_absolute() {
            root.clone()
        } else {
            #[cfg(windows)]
            if matches!(
                root.components().next(),
                Some(std::path::Component::Prefix(_))
            ) {
                return Err(format!(
                    "cannot inspect template '{}': drive-relative path",
                    root.display()
                ));
            }
            std::env::current_dir()
                .map_err(|e| format!("cannot inspect template '{}': {e}", root.display()))?
                .join(&root)
        };
        for ancestor in absolute.ancestors().collect::<Vec<_>>().into_iter().rev() {
            match std::fs::symlink_metadata(ancestor) {
                Ok(metadata) if is_link(&metadata) => {
                    return Err(format!(
                        "template path contains a link: '{}'",
                        ancestor.display()
                    ));
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                Err(e) => {
                    return Err(format!(
                        "cannot inspect template '{}': {e}",
                        ancestor.display()
                    ));
                }
            }
        }
        match std::fs::symlink_metadata(&root) {
            Ok(_) => visit(&root)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("cannot inspect template '{}': {e}", root.display())),
        }
    }
    Ok(())
}

/// Reads `templates/<rel>`: the override directory wins per file, the embedded
/// default is the fallback for anything the override does not provide.
pub fn read(rel: &str) -> Option<String> {
    validate_relative(rel).ok()?;
    let local = override_root().and_then(|root| {
        let mut path = root.clone();
        if std::fs::symlink_metadata(&path)
            .ok()
            .is_some_and(|m| is_link(&m))
        {
            return None;
        }
        for component in rel.split('/') {
            path.push(component);
            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if is_link(&metadata) => return None,
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
                Err(_) => return None,
            }
        }
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
