# Technical Design Document (TDD)

## Document Information
- **Document Version**: 1.0
- **Created Date**: 2026-07-28
- **Last Updated**: 2026-07-28
- **Author**: SNIPE-CODE Team
- **Project**: SNIPE-CODE CLI
- **Feature**: Enterprise Preset Templates
- **PRD Reference**: `docs/features/SNIPE-PRD.md`

---

## 1. System Architecture

### 1.1 High-Level Architecture

```mermaid
graph TB
    subgraph "CLI Layer"
        A[CLI Entry Point] --> B[Template Selector]
        B --> C[Stack Selection]
        C --> D[Config Prompt]
        D --> E[Structure Generator]
        E --> F[Git Automator]
    end

    subgraph "Core Engine"
        G[Template Registry] --> H[Template Loader]
        H --> I[File Generator]
        I --> J[Folder Creator]
    end

    subgraph "Security Layer"
        K[Security Config Generator] --> L[.env.example]
        K --> M[CORS/Helmet Config]
        K --> N[.gitignore]
    end

    subgraph "External"
        O[Git CLI]
        P[GitHub Actions]
    end

    A --> G
    E --> K
    F --> O
    F --> P
```

### 1.2 Module Dependency Graph

```mermaid
graph LR
    subgraph "snipecode-core"
        A[main] --> B[commands::new]
        B --> C[template::registry]
        B --> D[stack::selector]
        B --> E[generator::engine]
        B --> F[git::automator]
    end

    subgraph "CLI Interaction"
        G[dialoguer] --> B
    end

    subgraph "Serialization"
        H[serde] --> C
        I[serde_json] --> C
    end

    subgraph "File I/O"
        J[std::fs] --> E
        K[tempfile] --> E
    end
```

### 1.3 Request Flow Diagram

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as snipe-code CLI
    participant TS as Template Selector
    participant SS as Stack Selector
    participant SG as Structure Generator
    participant GA as Git Automator
    participant FS as FileSystem

    U->>CLI: snipe-code new
    CLI->>TS: load_templates()
    TS-->>U: display template list
    U->>TS: select_template()
    TS-->>CLI: selected_template

    CLI->>SS: load_stack_options()
    SS-->>U: display stack choices
    U->>SS: select_stack()
    SS-->>CLI: {frontend, backend, database}

    CLI->>CLI: prompt_config(docker, name)
    CLI-->>U: prompt answers

    CLI->>SG: generate(template, stack, config)
    SG->>FS: create_dirs(structure)
    SG->>FS: create_files(templates)
    SG->>FS: create_security_config()
    SG-->>CLI: generation_report

    CLI->>GA: init_git()
    GA->>FS: git_init()
    GA->>FS: create_gitignore()
    GA->>FS: create_ci_cd()
    GA->>FS: initial_commit()
    GA-->>CLI: git_report

    CLI-->>U: "Project created! ✔"
```

---

## 2. Module Design

### 2.1 Template Registry

```rust
// Template definition structure
struct Template {
    id: String,
    name: String,
    description: String,
    icon: String,
    structure: TemplateStructure,
}

struct TemplateStructure {
    folders: Vec<FolderDef>,
    files: Vec<FileDef>,
    rules: GenerationRules,
}

struct FolderDef {
    path: String,        // "dashboard/components/"
    condition: Option<String>, // "stack_frontend == 'nextjs'"
}

struct FileDef {
    path: String,
    template_source: String, // embedded path or content
    condition: Option<String>,
}

struct GenerationRules {
    require_backend: bool,
    require_database: bool,
    docker_compose: bool,
    security_configs: bool,
}
```

### 2.2 Template Definitions (JSON)

```json
{
  "templates": [
    {
      "id": "none",
      "name": "No Template — Clean Start",
      "description": "Stack-only structure, no template files",
      "icon": "📋"
    },
    {
      "id": "iot-dashboard",
      "name": "Industrial IoT Dashboard",
      "description": "Real-time monitoring dashboard with WebSocket",
      "icon": "📊",
      "structure": {
        "folders": [
          "frontend/src/components/charts/",
          "frontend/src/components/layout/dashboard-sidebar/",
          "frontend/src/hooks/useWebSocket/",
          "frontend/src/pages/dashboard/",
          "backend/src/controllers/sensor/",
          "backend/src/services/mqtt/",
          "backend/src/websocket/"
        ],
        "docker_compose": ["mqtt-broker", "timescaledb"]
      }
    },
    {
      "id": "company-profile",
      "name": "Company Profile",
      "description": "Professional company website with SEO",
      "icon": "🏢"
    },
    {
      "id": "portfolio",
      "name": "Portfolio",
      "description": "Personal portfolio with project showcase",
      "icon": "💼"
    },
    {
      "id": "blog",
      "name": "Blog / Posts",
      "description": "Content management with MDX support",
      "icon": "📝"
    }
  ]
}
```

### 2.3 Stack Configurations

```json
{
  "frontend": {
    "nextjs": { "name": "Next.js", "package_json": true, "tsconfig": true },
    "react-vite": { "name": "React (Vite)", "vite_config": true },
    "vue": { "name": "Vue", "vite_config": true },
    "nuxtjs": { "name": "Nuxt.js", "nuxt_config": true }
  },
  "backend": {
    "dotnet8": { "name": ".NET 8", "csproj": true },
    "nestjs": { "name": "NestJS", "tsconfig": true },
    "go-gin": { "name": "Go (Gin)", "go_mod": true },
    "spring-boot": { "name": "Spring Boot", "pom_xml": true },
    "laravel": { "name": "Laravel", "composer_json": true }
  },
  "database": {
    "postgresql": { "name": "PostgreSQL", "driver": "sqlx" },
    "mysql": { "name": "MySQL", "driver": "sqlx" }
  }
}
```

---

## 3. CLI Interface Design

### 3.1 Command Structure

```
snipe-code new              # Interactive mode
snipe-code new --dry-run    # Preview only
snipe-code new --template <id>  # Skip template prompt
snipe-code new --quiet      # Skip all prompts, use defaults
snipe-code new --frontend nextjs --backend nestjs --name my-app  # Full CLI args
```

### 3.2 Interactive Flow

```mermaid
graph TD
    Start[new] --> T[Select Template]
    T --> T1{Template Selected?}
    T1 -->|No Template| F[Select Frontend]
    T1 -->|With Template| F
    F --> F1[Select Backend]
    F1 --> F2[Select Database]
    F2 --> F3[Project Name]
    F3 --> F4{Use Docker?}
    F4 -->|Yes| Gen[Generate + Docker]
    F4 -->|No| Gen2[Generate]
    Gen --> Git[Git Init]
    Gen2 --> Git
    Git --> Done[Done]
```

### 3.3 Error Handling

```
Error: Target folder "my-project" already exists
  Overwrite?  (y/n)
  └─ Yes: remove existing folder and regenerate
  └─ No: cancel, no changes

Error: No stack selected
  └─ Warning: "Select at least Frontend or Backend"

Error: Git not installed
  └─ Warning: "Git not found. Skipping git init. Install git for full experience."
```

---

## 4. Security Design

### 4.1 Generated Security Configs

| Config | Description |
|--------|-------------|
| `.env.example` | All secrets with placeholder values, no defaults |
| `.gitignore` | Excludes `.env`, `*.pem`, `node_modules/`, `target/` |
| `cors.config` | Restrictive CORS for backend frameworks |
| `helmet` (Node) | Security headers for NestJS/Express |
| Rate limiter | Basic rate limit config for API routes |

### 4.2 Secure Generation Rules

```rust
// Security rules applied to every template
const SECURITY_RULES: SecurityConfig = SecurityConfig {
    no_hardcoded_secrets: true,
    env_placeholder: true,
    sql_injection_protection: true,  // parameterized queries
    xss_protection: true,            // input sanitization
    cors_config: true,
    helmet_config: true,             // Node.js only
    gitignore_secure: true,
};
```

### 4.3 .env.example Template

```env
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/mydb

# API
API_PORT=3000
API_SECRET=your-secret-here

# Security
CORS_ORIGIN=http://localhost:3000
RATE_LIMIT_WINDOW=60000
RATE_LIMIT_MAX=100

# [GENERATED BY SNIPE-CODE CLI]
```

---

## 5. Folder Structure — Generated Output

### 5.1 No Template + Next.js + NestJS + PostgreSQL

```
my-project/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   └── layout/
│   │   ├── pages/
│   │   ├── hooks/
│   │   ├── lib/
│   │   └── styles/
│   ├── .env.local.example
│   ├── next.config.js
│   ├── tsconfig.json
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── controllers/
│   │   ├── services/
│   │   ├── repositories/
│   │   ├── middleware/
│   │   ├── routes/
│   │   ├── models/
│   │   └── utils/
│   ├── .env.example
│   ├── tsconfig.json
│   ├── nest-cli.json
│   └── package.json
├── docker-compose.yml
├── .env.example
├── .gitignore
├── README.md
└── .github/
    └── workflows/
        └── main.yml
```

### 5.2 IoT Dashboard Template

```
my-project/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   ├── layout/
│   │   │   │   └── dashboard-sidebar/
│   │   │   └── charts/
│   │   │       ├── LineChart/
│   │   │       ├── GaugeChart/
│   │   │       └── AlertPanel/
│   │   ├── pages/
│   │   │   └── dashboard/
│   │   ├── hooks/
│   │   │   └── useWebSocket/
│   │   ├── lib/
│   │   └── styles/
│   └── ...
├── backend/
│   ├── src/
│   │   ├── controllers/
│   │   │   └── sensor/
│   │   ├── services/
│   │   │   └── mqtt/
│   │   ├── websocket/
│   │   ├── repositories/
│   │   ├── models/
│   │   └── middleware/
│   └── ...
├── docker-compose.yml  # MQTT + TimescaleDB
├── .env.example
├── .gitignore
└── README.md
```

### 5.3 Portfolio Template

```
my-project/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   ├── layout/
│   │   │   └── sections/
│   │   │       ├── hero/
│   │   │       ├── projects-grid/
│   │   │       ├── skills/
│   │   │       ├── experience/
│   │   │       └── contact/
│   │   ├── pages/
│   │   ├── hooks/
│   │   ├── data/
│   │   │   └── projects.json
│   │   └── styles/
│   └── ...
├── .env.example
├── .gitignore
└── README.md
```

### 5.4 Blog Template

```
my-project/
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   ├── layout/
│   │   │   └── blog/
│   │   │       ├── post-card/
│   │   │       ├── post-content/
│   │   │       └── author-page/
│   │   ├── pages/
│   │   ├── posts/          # MDX content
│   │   ├── categories/
│   │   ├── tags/
│   │   ├── hooks/
│   │   └── styles/
│   └── ...
├── backend/                  # optional
│   ├── src/
│   │   ├── controllers/
│   │   ├── services/
│   │   ├── repositories/
│   │   └── models/
│   └── ...
├── .env.example
├── .gitignore
└── README.md
```

---

## 6. CI/CD Pipeline Design

### 6.1 Generated `.github/workflows/main.yml`

```mermaid
graph LR
    A[Push/PR] --> B[Lint]
    B --> C[Test]
    C --> D[Build]
    D --> E[Deploy]
```

### 6.2 Pipeline Config

```yaml
name: CI/CD Pipeline
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Lint Frontend
        run: cd frontend && npm run lint
      - name: Lint Backend
        run: cd backend && npm run lint

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test Frontend
        run: cd frontend && npm test
      - name: Test Backend
        run: cd backend && npm test

  build:
    runs-on: ubuntu-latest
    needs: [lint, test]
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: |
          cd frontend && npm run build
          cd backend && npm run build
```

---

## 7. Performance Design

### 7.1 Performance Targets

| Operation | Target | Strategy |
|-----------|--------|----------|
| Template loading | < 50ms | Embedded binary (include_str!), no runtime file I/O |
| Folder creation | < 100ms | Parallel `fs::create_dir_all`, no sequential writes |
| File generation | < 200ms | Buffer writes, batch file creation |
| Dry-run output | < 100ms | In-memory tree construction, no disk I/O |
| Git init + commit | < 2s | Shell exec `git init && git add . && git commit` |
| Total `snipe-code new` | < 5s | End-to-end |

### 7.2 Optimization Strategies

```rust
// Embedded templates at compile time — no disk reads
const TEMPLATE_IOT: &str = include_str!("../templates/iot-dashboard.json");
const TEMPLATE_COMPANY: &str = include_str!("../templates/company-profile.json");
const TEMPLATE_PORTFOLIO: &str = include_str!("../templates/portfolio.json");
const TEMPLATE_BLOG: &str = include_str!("../templates/blog.json");

// Parallel folder creation
let folders: Vec<_> = structure.folders.iter().map(|f| {
    fs::create_dir_all(&f.path)
}).collect();
join_all(folders).await?;
```

---

## 8. Scalability Design

### 8.1 Template System Extension

```mermaid
graph TD
    A[New Template] --> B[Define JSON]
    B --> C[Add to Registry]
    C --> D[No code changes needed]
    D --> E[Automatically available in CLI]
```

### 8.2 Future Extension Points

| Extension | Current Support | Future Plan |
|-----------|----------------|-------------|
| Community templates | Not in v1 | CLI fetches from GitHub registry |
| Template versioning | Not in v1 | Version field in JSON, CLI prompts upgrade |
| Custom stack configs | Not in v1 | `~/.snipe-code/stacks.json` user-defined stacks |
| Multi-language | Rust only | Language-agnostic JSON definitions |

---

## 9. Monitoring & Logging

### 9.1 CLI Output Levels

```
--quiet    : No output except errors
(default)  : Normal output (prompts + success messages)
--verbose  : Debug output (template loaded, folders created, etc.)
```

### 9.2 Structured Output

```json
{
  "status": "success",
  "template": "iot-dashboard",
  "stack": { "frontend": "nextjs", "backend": "nestjs", "database": "postgresql" },
  "folders_created": 12,
  "files_created": 18,
  "git_initialized": true,
  "duration_ms": 3200
}
```

---

## 10. Testing Strategy

### 10.1 Test Pyramid

```mermaid
graph TB
    A[Unit Tests] --> B[Integration Tests]
    B --> C[E2E Tests]
    
    A -.- A1[Template JSON parsing]
    A -.- A2[Structure validation]
    A -.- A3[Stack config parsing]
    
    B -.- B1[Generator creates correct files]
    B -.- B2[Security configs present]
    B -.- B3[Git init works]
    
    C -.- C1[Full snipe-code new flow]
    C -.- C2[Dry-run output correct]
    C -.- C3[Error handling paths]
```

### 10.2 Key Test Cases

| Test Case | Type | Description |
|-----------|------|-------------|
| Template JSON valid | Unit | Parse all template JSON, assert no deserialization error |
| Structure generation | Integration | Generate for each template, assert expected folders/files exist |
| No Template clean start | Integration | Generate without template, assert clean architecture only |
| Security config present | Integration | Assert `.env.example`, `.gitignore`, CORS config exist in output |
| Git init succeeds | E2E | Run full `snipe-code new`, assert `.git/` exists with commit |
| Dry-run no files | Integration | Run `--dry-run`, assert no files created |
| Error on existing folder | E2E | Run with existing target, assert error + prompt |
| All stacks | Integration | Test each frontend/backend/db combination |

---

## 11. Deployment Strategy

### 11.1 Build Targets

| Target | Platform | Command |
|--------|----------|---------|
| Windows x64 | Win | `cargo build --release --target x86_64-pc-windows-msvc` |
| macOS ARM | Mac | `cargo build --release --target aarch64-apple-darwin` |
| Linux x64 | Linux | `cargo build --release --target x86_64-unknown-linux-gnu` |

### 11.2 Distribution

```
GitHub Releases/
├── snipe-code-windows-x64.exe
├── snipe-code-macos-arm64
└── snipe-code-linux-x64
```

### 11.3 CI/CD for CLI itself

```yaml
release:
  on:
    push:
      tags: ['v*']
  jobs:
    build:
      strategy:
        matrix:
          target: [windows-x64, macos-arm64, linux-x64]
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release --target ${{ matrix.target }}
        - uses: softprops/action-gh-release@v2
          with:
            files: target/${{ matrix.target }}/release/snipe-code*
```

---

## 12. Risk Mitigation (Technical)

| Risk | Mitigation |
|------|------------|
| Template too generic | Validate with real projects; iterate based on feedback |
| Folder collision | Detect existing folder, prompt overwrite/merge/cancel |
| Stack version drift | Generate structure only; user installs deps via package manager |
| Large template size | Embedded at compile-time; each template < 50KB JSON |
| Git not installed | Graceful skip with warning; CLI works without git |

---

## 13. Change History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-07-28 | SNIPE-CODE Team | Initial TDD — Enterprise Preset Templates |
