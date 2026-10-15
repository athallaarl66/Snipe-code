//! Generated test / lint / type-check contracts, per stack.

use serde_json::Value;
use snipe_code::cicd;
use snipe_code::stack;
use std::fs;
use std::path::PathBuf;

fn files(area: &str, id: &str) -> Vec<stack::FileDef> {
    match area {
        "frontends" => stack::get_frontend_files(id),
        _ => stack::get_backend_files(id),
    }
    .unwrap_or_else(|| panic!("unknown stack {area}/{id}"))
}

fn has_file(area: &str, id: &str, path: &str) -> bool {
    files(area, id).iter().any(|file| file.path == path)
}

fn content(area: &str, id: &str, path: &str) -> String {
    files(area, id)
        .into_iter()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("{area}/{id} has no {path}"))
        .content
}

fn manifest(area: &str, id: &str) -> Value {
    serde_json::from_str(&content(area, id, "package.json")).expect("valid package.json")
}

fn script(area: &str, id: &str, name: &str) -> Option<String> {
    manifest(area, id)
        .get("scripts")
        .and_then(|scripts| scripts.get(name))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn dev_dependency(area: &str, id: &str, name: &str) -> bool {
    manifest(area, id)
        .get("devDependencies")
        .and_then(|deps| deps.get(name))
        .is_some()
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "snipe-cicd-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// 3.1 — Next.js ships vitest, a config, an example test, and both scripts.
#[test]
fn test_nextjs_test_infrastructure() {
    assert_eq!(
        script("frontends", "nextjs", "test").as_deref(),
        Some("vitest run")
    );
    assert_eq!(
        script("frontends", "nextjs", "typecheck").as_deref(),
        Some("tsc --noEmit")
    );
    assert!(dev_dependency("frontends", "nextjs", "vitest"));
    assert!(has_file("frontends", "nextjs", "vitest.config.ts"));
    assert!(has_file("frontends", "nextjs", "src/app/page.test.tsx"));
}

/// 3.2 — React (Vite) and Vue declare the same pair, Vue through `vue-tsc`.
#[test]
fn test_react_vite_and_vue_test_infrastructure() {
    for (id, typecheck) in [("react-vite", "tsc --noEmit"), ("vue", "vue-tsc --noEmit")] {
        assert_eq!(
            script("frontends", id, "test").as_deref(),
            Some("vitest run"),
            "{id} must declare a test script"
        );
        assert_eq!(
            script("frontends", id, "typecheck").as_deref(),
            Some(typecheck),
            "{id} must declare its type-check script"
        );
        assert!(
            dev_dependency("frontends", id, "vitest"),
            "{id} must depend on vitest"
        );
        assert!(has_file("frontends", id, "vitest.config.ts"));
    }
    assert!(has_file("frontends", "react-vite", "src/App.test.tsx"));
    assert!(has_file("frontends", "vue", "src/App.test.ts"));
}

/// 3.3 — Nuxt.js runs its example test through the Nuxt test utilities.
#[test]
fn test_nuxt_test_infrastructure() {
    assert_eq!(
        script("frontends", "nuxtjs", "test").as_deref(),
        Some("vitest run")
    );
    assert_eq!(
        script("frontends", "nuxtjs", "typecheck").as_deref(),
        Some("nuxt typecheck")
    );
    assert!(dev_dependency("frontends", "nuxtjs", "@nuxt/test-utils"));
    assert!(has_file("frontends", "nuxtjs", "vitest.config.ts"));
    assert!(has_file("frontends", "nuxtjs", "tests/index.spec.ts"));
    let config = content("frontends", "nuxtjs", "vitest.config.ts");
    assert!(
        config.contains("defineVitestConfig"),
        "nuxt config must come from the Nuxt test utilities"
    );
}

/// 4.1 — NestJS ships jest, a controller spec, and both scripts.
#[test]
fn test_nestjs_test_infrastructure() {
    assert_eq!(
        script("backends", "nestjs", "test").as_deref(),
        Some("jest")
    );
    assert_eq!(
        script("backends", "nestjs", "typecheck").as_deref(),
        Some("tsc --noEmit")
    );
    for dep in ["jest", "ts-jest", "@nestjs/testing", "supertest"] {
        assert!(
            dev_dependency("backends", "nestjs", dep),
            "nestjs must depend on {dep}"
        );
    }
    assert!(has_file("backends", "nestjs", "src/app.controller.spec.ts"));
}

/// 4.2 — Go exposes a constructible router that its test drives through httptest.
#[test]
fn test_go_router_is_constructible_and_tested() {
    let main = content("backends", "go-gin", "main.go");
    assert!(
        main.contains("func NewRouter() *gin.Engine"),
        "generated main.go must expose a router constructor"
    );
    assert!(main.contains("NewRouter().Run"), "main must use it");
    let test = content("backends", "go-gin", "main_test.go");
    assert!(test.contains("httptest.NewRequest"));
    assert!(test.contains("NewRouter().ServeHTTP"));
}

/// 4.3 — .NET 8 adds an xunit project referencing the generated web project.
#[test]
fn test_dotnet_test_project_is_wired() {
    assert!(has_file("backends", "dotnet8", "tests/app.Tests.csproj"));
    assert!(has_file(
        "backends",
        "dotnet8",
        "tests/HomeEndpointTests.cs"
    ));
    let project = content("backends", "dotnet8", "tests/app.Tests.csproj");
    assert!(project.contains("Microsoft.AspNetCore.Mvc.Testing"));
    assert!(project.contains(r#"Include="../app.csproj""#));
    let program = content("backends", "dotnet8", "Program.cs");
    assert!(
        program.contains("public partial class Program"),
        "WebApplicationFactory needs an accessible entry point"
    );
    let solution = content("backends", "dotnet8", "app.sln");
    assert!(
        solution.contains(r"tests\app.Tests.csproj"),
        "solution must list the test project so `dotnet test` resolves it"
    );
}

/// 4.4 — Spring Boot's placeholder test becomes a real MockMvc test.
#[test]
fn test_spring_boot_test_exercises_an_endpoint() {
    let test = content(
        "backends",
        "spring-boot",
        "src/test/java/com/example/backend/ApplicationTests.java",
    );
    assert!(test.contains("@SpringBootTest"));
    assert!(test.contains("MockMvc"));
    assert!(test.contains("mockMvc.perform(get(\"/\"))"));
    assert!(
        !test.contains("assertTrue(true)"),
        "the placeholder assertion must be gone"
    );
    assert!(has_file(
        "backends",
        "spring-boot",
        "src/main/java/com/example/backend/HelloController.java"
    ));
}

/// 5.1 — every manifest-based stack declares lint and ships a flat config.
#[test]
fn test_js_stacks_declare_lint_with_flat_config() {
    let stacks = [
        ("frontends", "nextjs", "eslint.config.mjs"),
        ("frontends", "react-vite", "eslint.config.js"),
        ("frontends", "vue", "eslint.config.js"),
        ("frontends", "nuxtjs", "eslint.config.mjs"),
        ("backends", "nestjs", "eslint.config.mjs"),
    ];
    for (area, id, config) in stacks {
        assert_eq!(
            script(area, id, "lint").as_deref(),
            Some("eslint ."),
            "{id} must declare a lint script"
        );
        assert!(dev_dependency(area, id, "eslint"), "{id} needs eslint");
        assert!(
            dev_dependency(area, id, "typescript-eslint"),
            "{id} needs typescript-eslint"
        );
        assert!(has_file(area, id, config), "{id} needs {config}");
        assert!(
            content(area, id, config).contains("tseslint.config"),
            "{id} config must be flat"
        );
    }
}

/// 5.2 — generated CI runs test and lint only when the manifest declares them.
#[test]
fn test_generated_ci_is_gated_on_declared_scripts() {
    let dir = temp_dir("declared");
    let frontend = dir.join("frontend");
    fs::create_dir_all(&frontend).unwrap();
    fs::write(
        frontend.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest run","lint":"eslint ."}}"#,
    )
    .unwrap();

    cicd::generate_ci_cd(&dir.to_string_lossy(), "nextjs", "none").unwrap();
    let workflow = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();

    assert!(workflow.contains("npm run test"), "{workflow}");
    assert!(workflow.contains("npm run lint"), "{workflow}");
    assert!(
        workflow.find("npm run test") < workflow.find("npm run build"),
        "the test step must run before the build step"
    );
    assert!(
        !workflow.contains("npm run typecheck"),
        "typecheck is not declared, so it must not be invoked"
    );

    let dir = temp_dir("undeclared");
    let frontend = dir.join("frontend");
    fs::create_dir_all(&frontend).unwrap();
    fs::write(
        frontend.join("package.json"),
        r#"{"scripts":{"build":"vite build"}}"#,
    )
    .unwrap();

    cicd::generate_ci_cd(&dir.to_string_lossy(), "nextjs", "none").unwrap();
    let workflow = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();
    assert!(!workflow.contains("npm run test"), "{workflow}");
    assert!(!workflow.contains("npm run lint"), "{workflow}");

    // A backend with no package.json gets no npm step at all.
    cicd::generate_ci_cd(&dir.to_string_lossy(), "none", "laravel").unwrap();
    let workflow = fs::read_to_string(dir.join(".github/workflows/main.yml")).unwrap();
    assert!(!workflow.contains("npm run test"), "{workflow}");
    assert!(workflow.contains("php artisan test"), "{workflow}");

    let _ = fs::remove_dir_all(&dir);
}

/// 6.2 — the README documents the override variable.
#[test]
fn test_readme_documents_template_override_variable() {
    let readme = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("README.md");

    assert!(
        readme.contains(snipe_code::template::OVERRIDE_ENV),
        "README must document {}",
        snipe_code::template::OVERRIDE_ENV
    );
}
