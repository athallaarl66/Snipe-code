use std::fs;
use std::path::Path;

pub fn generate_ci_cd(project_path: &str, frontend: &str, backend: &str) -> Result<(), String> {
    let path = Path::new(project_path);
    let workflows_dir = path.join(".github/workflows");
    fs::create_dir_all(&workflows_dir).map_err(|e| e.to_string())?;

    let content = workflow_content(path, frontend, backend);
    fs::write(workflows_dir.join("main.yml"), content).map_err(|e| e.to_string())?;

    Ok(())
}

/// Workflows are written into the staged project, so the generated manifests
/// are already on disk: every step below is gated on the script being declared,
/// and the generated CI never invokes a script the project does not have.
fn workflow_content(project_path: &Path, frontend: &str, backend: &str) -> String {
    let mut steps = vec![
        "name: CI/CD Pipeline".to_string(),
        "on:".to_string(),
        "  push:".to_string(),
        "    branches: [main, develop]".to_string(),
        "  pull_request:".to_string(),
        "    branches: [main]".to_string(),
        "".to_string(),
        "jobs:".to_string(),
    ];

    // Frontend job
    if frontend != "none" {
        steps.push("  frontend:".to_string());
        steps.push("    runs-on: ubuntu-latest".to_string());
        steps.push("    defaults:".to_string());
        steps.push("      run:".to_string());
        steps.push("        working-directory: frontend".to_string());
        steps.push("    steps:".to_string());
        steps.push("      - uses: actions/checkout@v4".to_string());

        match frontend {
            "nextjs" | "react-vite" | "vue" | "nuxtjs" => {
                steps.push("      - uses: actions/setup-node@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          node-version: '24'".to_string());
                steps.push("      - run: npm install".to_string());
                steps.extend(npm_script_steps(&project_path.join("frontend")));
            }
            _ => {}
        }
        steps.push("".to_string());
    }

    // Backend job
    if backend != "none" {
        steps.push("  backend:".to_string());
        steps.push("    runs-on: ubuntu-latest".to_string());
        steps.push("    defaults:".to_string());
        steps.push("      run:".to_string());
        steps.push("        working-directory: backend".to_string());
        steps.push("    steps:".to_string());
        steps.push("      - uses: actions/checkout@v4".to_string());

        match backend {
            "nestjs" => {
                steps.push("      - uses: actions/setup-node@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          node-version: '24'".to_string());
                steps.push("      - run: npm install".to_string());
                steps.extend(npm_script_steps(&project_path.join("backend")));
            }
            "go-gin" => {
                steps.push("      - uses: actions/setup-go@v5".to_string());
                steps.push("        with:".to_string());
                steps.push("          go-version: '1.21'".to_string());
                steps.push("      - run: go mod tidy".to_string());
                steps.push("      - run: go test ./...".to_string());
                steps.push("      - run: go vet ./...".to_string());
                steps.push("      - run: go build ./...".to_string());
            }
            "spring-boot" => {
                steps.push("      - uses: actions/setup-java@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          java-version: '17'".to_string());
                steps.push("          distribution: 'temurin'".to_string());
                steps.push("      - run: mvn test package".to_string());
            }
            "dotnet8" => {
                steps.push("      - uses: actions/setup-dotnet@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          dotnet-version: '8.0.x'".to_string());
                steps.push("      - run: dotnet restore".to_string());
                steps.push("      - run: dotnet test --no-restore".to_string());
                steps.push("      - run: dotnet build --no-restore".to_string());
            }
            "laravel" => {
                steps.push("      - uses: shivammathur/setup-php@v2".to_string());
                steps.push("        with:".to_string());
                steps.push("          php-version: '8.2'".to_string());
                steps.push("      - run: composer install".to_string());
                steps.push("      - run: php artisan test".to_string());
            }
            _ => {}
        }
    }

    steps.join("\n")
}

/// typecheck, then lint, then test, then build — cheapest first, build last so a
/// failure is attributed to the phase that caused it.
fn npm_script_steps(dir: &Path) -> Vec<String> {
    let declared = crate::generator::declared_scripts(dir);
    ["typecheck", "lint", "test", "build"]
        .iter()
        .filter(|script| declared.iter().any(|name| name == *script))
        .map(|script| format!("      - run: npm run {script}"))
        .collect()
}
