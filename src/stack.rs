//! Stack registry. Stack content lives in `templates/frontends/<id>/` and
//! `templates/backends/<id>/` and is embedded at build time (see `template`).

use crate::template;

#[derive(Debug, PartialEq, Clone)]
pub struct StackOption {
    pub id: String,
    pub name: String,
    pub folders: Vec<String>,
    pub files: Vec<FileDef>,
    pub init_command: String,
    pub install_command: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileDef {
    pub path: String,
    pub content: String,
}

#[derive(Debug, PartialEq)]
pub struct DbOption {
    pub id: &'static str,
    pub name: &'static str,
}

/// Display order in the wizard; `none` is a valid choice with no content.
pub const FRONTEND_IDS: &[&str] = &["nextjs", "react-vite", "vue", "nuxtjs", "none"];
pub const BACKEND_IDS: &[&str] = &[
    "dotnet8",
    "nestjs",
    "go-gin",
    "spring-boot",
    "laravel",
    "none",
];

pub const DATABASE_STACKS: &[DbOption] = &[
    DbOption {
        id: "postgresql",
        name: "PostgreSQL",
    },
    DbOption {
        id: "mysql",
        name: "MySQL",
    },
    DbOption {
        id: "none",
        name: "None",
    },
];

pub fn frontend_stacks() -> Vec<StackOption> {
    FRONTEND_IDS
        .iter()
        .map(|id| load("frontends", id))
        .collect()
}

pub fn backend_stacks() -> Vec<StackOption> {
    BACKEND_IDS.iter().map(|id| load("backends", id)).collect()
}

fn find(area: &str, ids: &[&str], id: &str) -> Option<StackOption> {
    ids.contains(&id).then(|| load(area, id))
}

fn load(area: &str, id: &str) -> StackOption {
    let manifest = template::read(&format!("{area}/{id}/manifest.txt")).unwrap_or_default();
    let (name, init_command, install_command) = parse_manifest(&manifest, id);

    let folders = template::read(&format!("{area}/{id}/folders.txt"))
        .map(|raw| {
            raw.lines()
                .filter(|line| !line.trim().is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let prefix = format!("{area}/{id}/files/");
    let files = template::list(&format!("{area}/{id}/files"))
        .into_iter()
        .filter_map(|rel| {
            let content = template::read(&rel)?;
            Some(FileDef {
                path: rel.strip_prefix(&prefix).unwrap_or(&rel).to_string(),
                content,
            })
        })
        .collect();

    StackOption {
        id: id.to_string(),
        name,
        folders,
        files,
        init_command,
        install_command,
    }
}

fn parse_manifest(raw: &str, id: &str) -> (String, String, String) {
    let mut name = None;
    let mut init = String::new();
    let mut install = String::new();
    for line in raw.lines() {
        match line.split_once('=') {
            Some(("name", value)) => name = Some(value.to_string()),
            Some(("init", value)) => init = value.to_string(),
            Some(("install", value)) => install = value.to_string(),
            _ => {}
        }
    }
    let fallback = if id == "none" {
        "None".to_string()
    } else {
        id.to_string()
    };
    (name.unwrap_or(fallback), init, install)
}

pub fn get_frontend_folders(id: &str) -> Option<Vec<String>> {
    find("frontends", FRONTEND_IDS, id).map(|stack| stack.folders)
}

pub fn get_backend_folders(id: &str) -> Option<Vec<String>> {
    find("backends", BACKEND_IDS, id).map(|stack| stack.folders)
}

pub fn get_frontend_files(id: &str) -> Option<Vec<FileDef>> {
    find("frontends", FRONTEND_IDS, id).map(|stack| stack.files)
}

pub fn get_backend_files(id: &str) -> Option<Vec<FileDef>> {
    find("backends", BACKEND_IDS, id).map(|stack| stack.files)
}

pub fn get_frontend_init_command(id: &str) -> String {
    find("frontends", FRONTEND_IDS, id)
        .map(|stack| stack.init_command)
        .unwrap_or_default()
}

pub fn get_frontend_install_command(id: &str) -> String {
    find("frontends", FRONTEND_IDS, id)
        .map(|stack| stack.install_command)
        .unwrap_or_default()
}

pub fn get_backend_init_command(id: &str) -> String {
    find("backends", BACKEND_IDS, id)
        .map(|stack| stack.init_command)
        .unwrap_or_default()
}

pub fn get_backend_install_command(id: &str) -> String {
    find("backends", BACKEND_IDS, id)
        .map(|stack| stack.install_command)
        .unwrap_or_default()
}

pub fn get_database_url(id: &str) -> &'static str {
    match id {
        "postgresql" => "postgresql://user:***@localhost:5432/mydb",
        "mysql" => "mysql://user:***@localhost:3306/mydb",
        _ => "",
    }
}

pub fn validate_selection(frontend: &str, backend: &str) -> Result<(), &'static str> {
    if frontend == "none" && backend == "none" {
        return Err("Pilih minimal Frontend atau Backend");
    }
    Ok(())
}
