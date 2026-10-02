mod common;

use snipe_code::generator::{self, GenerateConfig};
use std::fs;

fn fe_config(frontend: &str) -> GenerateConfig {
    GenerateConfig {
        project_name: "app".to_string(),
        template_id: "none".to_string(),
        frontend: frontend.to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    }
}

/// Every framework-required entry/config file must exist in the generated output.
#[test]
fn test_react_vite_required_files() {
    let _ws = common::Workspace::new("fe-react-vite");
    generator::generate(&fe_config("react-vite"), false).unwrap();
    for rel in [
        "app/frontend/index.html",
        "app/frontend/vite.config.ts",
        "app/frontend/tsconfig.json",
        "app/frontend/src/main.tsx",
        "app/frontend/src/App.tsx",
    ] {
        assert!(fs::exists(rel).unwrap(), "missing {}", rel);
    }
    // index.html must reference the same mount element main.tsx mounts
    let html = fs::read_to_string("app/frontend/index.html").unwrap();
    assert!(html.contains(r#"<div id="root"></div>"#));
    assert!(html.contains("src/main.tsx"));
    let main = fs::read_to_string("app/frontend/src/main.tsx").unwrap();
    assert!(main.contains("getElementById('root')"));
    // package.json must declare a build script
    let pkg = fs::read_to_string("app/frontend/package.json").unwrap();
    assert!(pkg.contains("\"build\""));
}

#[test]
fn test_vue_required_files() {
    let _ws = common::Workspace::new("fe-vue");
    generator::generate(&fe_config("vue"), false).unwrap();
    for rel in [
        "app/frontend/index.html",
        "app/frontend/vite.config.ts",
        "app/frontend/tsconfig.json",
        "app/frontend/src/main.ts",
        "app/frontend/src/App.vue",
    ] {
        assert!(fs::exists(rel).unwrap(), "missing {}", rel);
    }
    let html = fs::read_to_string("app/frontend/index.html").unwrap();
    assert!(html.contains(r#"<div id="app"></div>"#));
    assert!(html.contains("src/main.ts"));
    let main = fs::read_to_string("app/frontend/src/main.ts").unwrap();
    assert!(main.contains(".mount('#app')"));
    let pkg = fs::read_to_string("app/frontend/package.json").unwrap();
    assert!(pkg.contains("\"build\""));
    // Typecheck must be part of the build (vue-tsc)
    assert!(pkg.contains("vue-tsc"));
}

#[test]
fn test_nextjs_required_files() {
    let _ws = common::Workspace::new("fe-nextjs");
    generator::generate(&fe_config("nextjs"), false).unwrap();
    for rel in [
        "app/frontend/package.json",
        "app/frontend/tsconfig.json",
        "app/frontend/next.config.js",
        "app/frontend/src/app/layout.tsx",
        "app/frontend/src/app/page.tsx",
    ] {
        assert!(fs::exists(rel).unwrap(), "missing {}", rel);
    }
    let layout = fs::read_to_string("app/frontend/src/app/layout.tsx").unwrap();
    assert!(layout.contains("<html"), "layout must render <html> root");
    assert!(layout.contains("<body"), "layout must render <body>");
    let pkg = fs::read_to_string("app/frontend/package.json").unwrap();
    assert!(pkg.contains("\"build\""));
}

#[test]
fn test_nuxt_required_files() {
    let _ws = common::Workspace::new("fe-nuxt");
    generator::generate(&fe_config("nuxtjs"), false).unwrap();
    for rel in [
        "app/frontend/package.json",
        "app/frontend/nuxt.config.ts",
        "app/frontend/tsconfig.json",
        "app/frontend/app.vue",
        "app/frontend/pages/index.vue",
    ] {
        assert!(fs::exists(rel).unwrap(), "missing {}", rel);
    }
    let app = fs::read_to_string("app/frontend/app.vue").unwrap();
    assert!(app.contains("<NuxtPage />"), "app.vue must render NuxtPage");
    let tsconfig = fs::read_to_string("app/frontend/tsconfig.json").unwrap();
    assert!(
        tsconfig.contains(".nuxt/tsconfig.json"),
        "tsconfig must extend generated types"
    );
    let pkg = fs::read_to_string("app/frontend/package.json").unwrap();
    assert!(pkg.contains("\"build\""));
}

/// Every generated frontend package must declare install-compatible scripts and
/// a build entrypoint.
#[test]
fn test_all_frontends_declare_build_and_install() {
    for fe in ["nextjs", "react-vite", "vue", "nuxtjs"] {
        let _ws = common::Workspace::new(&format!("fe-build-{}", fe));
        generator::generate(&fe_config(fe), false).unwrap();
        let pkg: serde_json::Value =
            serde_json::from_str(&fs::read_to_string("app/frontend/package.json").unwrap())
                .unwrap();
        let scripts = pkg.get("scripts").expect("scripts");
        assert!(
            scripts.get("build").is_some(),
            "{} must declare a build script",
            fe
        );
        assert!(
            scripts.get("dev").is_some(),
            "{} must declare a dev script",
            fe
        );
    }
}
