## Purpose

Defines stack discovery, multi-language audit coverage, and honest scope reporting so generated frontend/backend layouts receive meaningful checks without presenting heuristic coverage as a complete security guarantee.

## ADDED Requirements

### Requirement: Stack discovery recognizes generated project layout
The audit SHALL discover project contexts at the audit root and at supported nested application roots, including `frontend/` and `backend/`, using manifest content or equivalent framework signals rather than folder names alone. A root project with both frontend and backend manifests SHALL report both contexts and retain each context's framework identity.

#### Scenario: Root manifest
- **WHEN** a project has a recognized manifest at the audit root
- **THEN** that root is reported as a detected project context

#### Scenario: Generated frontend and backend
- **WHEN** the audit root contains `frontend/package.json` and `backend/go.mod`
- **THEN** JS/TS frontend and Go backend contexts are both discovered and reported

#### Scenario: Nested monorepo contexts
- **WHEN** recognized manifests exist below the audit root in non-generated application directories
- **THEN** those contexts are discovered unless they reside under an explicitly excluded dependency, build, VCS, or generated-report directory

#### Scenario: Misleading filename
- **WHEN** a recognized manifest filename has content that does not describe the expected ecosystem
- **THEN** the report identifies the context as ambiguous or unknown rather than asserting a full framework match

### Requirement: Stack identity distinguishes ecosystem from framework
Manifest evidence SHALL distinguish at least JavaScript/TypeScript from React, Vue, and Nuxt, and PHP from Laravel, Go, Java from Spring, .NET, and Rust. A generic ecosystem SHALL NOT be reported as a specific framework without supporting evidence.

#### Scenario: React package
- **WHEN** `frontend/package.json` declares React and a Vite build toolchain
- **THEN** the context is reported as React with the JavaScript/TypeScript ecosystem

#### Scenario: Generic PHP package
- **WHEN** `composer.json` does not declare Laravel
- **THEN** the context is reported as PHP, not Laravel

#### Scenario: Laravel package
- **WHEN** `composer.json` declares the Laravel framework
- **THEN** the context is reported as Laravel

### Requirement: Audit runs meaningful rules for every advertised generated stack
The audit SHALL run at least common checks plus stack-appropriate security checks for JavaScript/TypeScript, PHP/Laravel, Go, Java/Spring, .NET, and Rust projects. A detected stack SHALL NOT be advertised as covered when no applicable rule set can run.

#### Scenario: JavaScript security rule
- **WHEN** an applicable JS/TS file contains a recognized unsafe HTML or SQL construction
- **THEN** the report includes a finding from the active JS/TS rule set

#### Scenario: PHP security rule
- **WHEN** an applicable PHP file contains a recognized unescaped output, SQL concatenation, or hardcoded secret
- **THEN** the report includes a finding from the active PHP rule set

#### Scenario: Go security rule
- **WHEN** an applicable Go file contains a recognized unsafe query or process construction, or a hardcoded secret
- **THEN** the report includes a finding from the active Go rule set

#### Scenario: Java and Spring security rule
- **WHEN** an applicable Java/Spring configuration or source file contains a recognized exposed management endpoint, unsafe configuration, or hardcoded secret
- **THEN** the report includes a finding from the active Java/Spring rule set

#### Scenario: .NET security rule
- **WHEN** an applicable .NET source or configuration file contains a recognized unsafe query, connection secret, or dangerous HTML construction
- **THEN** the report includes a finding from the active .NET rule set

#### Scenario: Rust security rule
- **WHEN** an applicable Rust source file contains a recognized hardcoded secret, unsafe command construction, or SQL construction covered by the Rust rule set
- **THEN** the report includes a finding from the active Rust rule set

### Requirement: Rule evaluation supports more than line regex matching
The audit rule model SHALL represent line-pattern rules and non-pattern checks required by the product contract, including required-file presence, ignore containment, and framework-specific configuration checks. Extension or filename matching SHALL be supported wherever rules target named configuration files.

#### Scenario: Required security file
- **WHEN** common rules require `.env.example`
- **THEN** absence is evaluated as a file-presence check rather than an impossible regex rule

#### Scenario: Environment ignore rule
- **WHEN** common rules require `.env` to be covered by `.gitignore`
- **THEN** the audit evaluates ignore-file containment independently from source-line patterns

#### Scenario: Named configuration rule
- **WHEN** a rule targets `appsettings.json` or `application.properties`
- **THEN** filename matching selects the file even though it has no source-code extension

### Requirement: Findings retain rule identity during deduplication
When equivalent findings are discovered more than once, audit deduplication SHALL retain rule ID, severity, description, originating context, and source location. Distinct rules on the same line SHALL remain independently visible unless they are proven to represent the same rule instance.

#### Scenario: Same rule across overlapping contexts
- **WHEN** the same rule and source location are reached by overlapping scans
- **THEN** only one instance of that rule is retained

#### Scenario: Different rules on one line
- **WHEN** two different rules match the same file and line
- **THEN** both findings remain distinguishable by rule identity

### Requirement: Reports disclose evaluated and unevaluated scope
Every audit report SHALL identify detected contexts, frameworks, executed rule groups, skipped paths, unknown or ambiguous contexts, and the heuristic nature of regex-based checks. Empty findings SHALL mean only that evaluated rules found nothing.

#### Scenario: No findings in a known project
- **WHEN** all applicable rules run and produce no findings
- **THEN** the report says no findings were detected by the evaluated rules and does not claim the codebase is completely secure or clean

#### Scenario: Unknown project
- **WHEN** no project context can be identified
- **THEN** the report states that stack-specific analysis was not performed and lists the common checks that did run

#### Scenario: Partial support
- **WHEN** a project is only partially covered by available rules
- **THEN** the report identifies unevaluated files or language features and does not present the result as enterprise-grade assurance
