use std::fs;
use std::path::Path;

pub fn generate_ai_workspace(project_path: &str) -> Result<(), String> {
    let path = Path::new(project_path);
    let workspace_dir = path.join(".ai-workspace");
    fs::create_dir_all(&workspace_dir).map_err(|e| e.to_string())?;

    // Generate rules for different AI IDEs
    fs::write(workspace_dir.join("rules.md"), windsurf_rules()).map_err(|e| e.to_string())?;
    fs::write(workspace_dir.join("rules.mdc"), cursor_rules()).map_err(|e| e.to_string())?;
    fs::write(workspace_dir.join("audit/.gitkeep"), "").map_err(|e| e.to_string())?;

    Ok(())
}

fn windsurf_rules() -> &'static str {
    r#"# SNIPE-CODE AI Workspace Rules

## Security Rules (CRITICAL)
- NEVER hardcode secrets, API keys, or passwords
- ALWAYS use environment variables for sensitive data
- ALWAYS use parameterized queries (no string concatenation for SQL)
- ALWAYS sanitize user input before rendering
- NEVER use `eval()` or `exec()` with user input
- ALWAYS validate and sanitize all inputs at trust boundaries

## Code Style
- Use TypeScript/JavaScript strict mode
- Follow framework conventions (Next.js App Router, NestJS modules, etc.)
- Prefer functional components over class components
- Use proper error handling with try/catch or Result types

## Token Optimization
- Focus on code blocks only, minimal explanations
- Use concise variable names
- Avoid redundant comments
- Output only what's necessary

## Commands
- `/pentest` - Review codebase for security vulnerabilities (OWASP)
- `/snipe-code` - Enter extreme token efficiency mode
- `/fix-smells` - Read audit report and fix code smells
- `/gen-docs` - Generate SDD documentation from backend structure
"#
}

fn cursor_rules() -> &'static str {
    r#"# SNIPE-CODE AI Rules

## Security First
- No hardcoded secrets
- Use environment variables
- Parameterized queries only
- Input sanitization required
- XSS protection enabled

## Code Quality
- TypeScript strict mode
- Proper error handling
- Functional components
- Clean architecture patterns

## Token Efficiency
- Minimal explanations
- Code-focused output
- Concise responses
- No unnecessary verbosity
"#
}
