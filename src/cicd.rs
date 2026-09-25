use std::fs;
use std::path::Path;

pub fn generate_ci_cd(project_path: &str, frontend: &str, backend: &str) -> Result<(), String> {
    let path = Path::new(project_path);
    let workflows_dir = path.join(".github/workflows");
    fs::create_dir_all(&workflows_dir).map_err(|e| e.to_string())?;

    let content = workflow_content(frontend, backend);
    fs::write(workflows_dir.join("main.yml"), content).map_err(|e| e.to_string())?;

    Ok(())
}

fn workflow_content(frontend: &str, backend: &str) -> String {
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
                steps.push("          node-version: '20'".to_string());
                steps.push("      - run: npm install".to_string());
                steps.push("      - run: npm run lint".to_string());
                steps.push("      - run: npm run build".to_string());
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
                steps.push("          node-version: '20'".to_string());
                steps.push("      - run: npm install".to_string());
                steps.push("      - run: npm run lint".to_string());
                steps.push("      - run: npm run build".to_string());
            }
            "go-gin" => {
                steps.push("      - uses: actions/setup-go@v5".to_string());
                steps.push("        with:".to_string());
                steps.push("          go-version: '1.21'".to_string());
                steps.push("      - run: go mod tidy".to_string());
                steps.push("      - run: go build ./...".to_string());
            }
            "spring-boot" => {
                steps.push("      - uses: actions/setup-java@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          java-version: '17'".to_string());
                steps.push("          distribution: 'temurin'".to_string());
                steps.push("      - run: mvn clean install".to_string());
            }
            "dotnet8" => {
                steps.push("      - uses: actions/setup-dotnet@v4".to_string());
                steps.push("        with:".to_string());
                steps.push("          dotnet-version: '8.0.x'".to_string());
                steps.push("      - run: dotnet restore".to_string());
                steps.push("      - run: dotnet build --no-restore".to_string());
            }
            "laravel" => {
                steps.push("      - uses: shivammathur/setup-php@v2".to_string());
                steps.push("        with:".to_string());
                steps.push("          php-version: '8.1'".to_string());
                steps.push("      - run: composer install".to_string());
                steps.push("      - run: php artisan test".to_string());
            }
            _ => {}
        }
    }

    steps.join("\n")
}
