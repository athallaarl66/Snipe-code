use serde::Deserialize;

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

impl TemplateRegistry {
    pub fn load_all() -> Vec<Template> {
        let mut templates = Vec::new();
        for json in TEMPLATES {
            let template: Template = serde_json::from_str(json).expect("Invalid template JSON");
            templates.push(template);
        }
        templates
    }

    pub fn get(id: &str) -> Option<Template> {
        Self::load_all().into_iter().find(|t| t.id == id)
    }
}

const TEMPLATES: &[&str] = &[
    r#"{
        "id": "none",
        "name": "No Template — Clean Start",
        "description": "Stack-only structure, no template files",
        "icon": "📋",
        "structure": {
            "folders": [],
            "files": [],
            "rules": {}
        }
    }"#,
    r#"{
        "id": "company-profile",
        "name": "Company Profile",
        "description": "Professional company website with SEO",
        "icon": "🏢",
        "structure": {
            "folders": [
                "frontend/src/pages/home/",
                "frontend/src/pages/about/",
                "frontend/src/pages/services/",
                "frontend/src/pages/contact/",
                "frontend/src/pages/team/",
                "frontend/src/components/layout/header/",
                "frontend/src/components/layout/footer/"
            ],
            "files": [],
            "rules": {
                "require_backend": false,
                "require_database": false,
                "docker_compose": false,
                "security_configs": true
            }
        }
    }"#,
    r#"{
        "id": "portfolio",
        "name": "Portfolio",
        "description": "Personal portfolio with project showcase",
        "icon": "💼",
        "structure": {
            "folders": [
                "frontend/src/components/sections/hero/",
                "frontend/src/components/sections/projects-grid/",
                "frontend/src/components/sections/skills/",
                "frontend/src/components/sections/experience/",
                "frontend/src/components/sections/contact/",
                "frontend/src/data/"
            ],
            "files": [
                {
                    "path": "frontend/src/data/projects.json",
                    "template_source": "portfolio-projects"
                }
            ],
            "rules": {
                "require_backend": false,
                "require_database": false,
                "docker_compose": false,
                "security_configs": true
            }
        }
    }"#,
    r#"{
        "id": "blog",
        "name": "Blog / Posts",
        "description": "Content management with MDX support",
        "icon": "📝",
        "structure": {
            "folders": [
                "frontend/src/posts/",
                "frontend/src/categories/",
                "frontend/src/tags/",
                "frontend/src/components/blog/post-card/",
                "frontend/src/components/blog/post-content/",
                "frontend/src/components/blog/author-page/"
            ],
            "files": [
                {
                    "path": "frontend/src/posts/hello-world.mdx",
                    "template_source": "blog-sample-post"
                },
                {
                    "path": "frontend/public/rss.xml",
                    "template_source": "blog-rss-feed"
                }
            ],
            "rules": {
                "require_backend": false,
                "require_database": false,
                "docker_compose": false,
                "security_configs": true
            }
        }
    }"#,
];
