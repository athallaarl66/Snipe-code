## Purpose

Defines observable build, startup, CI, and container contracts for every stack advertised by SNIPE-CODE so generated projects are usable scaffolds rather than collections of unverified files.

## ADDED Requirements

### Requirement: Every advertised frontend scaffold has a buildable application entry
Each advertised frontend stack SHALL generate all framework-required entry files, valid configuration, and package scripts needed for its declared toolchain to install and build successfully.

#### Scenario: Next.js scaffold
- **WHEN** a project is generated with the Next.js frontend
- **THEN** the generated frontend contains a valid App Router entry, TypeScript configuration, and scripts whose install and build commands succeed with the declared Node.js version

#### Scenario: React Vite scaffold
- **WHEN** a project is generated with the React Vite frontend
- **THEN** `index.html`, the root mount element, and Vite/TypeScript configuration exist and the declared build command succeeds

#### Scenario: Vue scaffold
- **WHEN** a project is generated with the Vue frontend
- **THEN** `index.html`, the root mount element, Vue entry component, and Vite/TypeScript configuration exist and the declared build command succeeds

#### Scenario: Nuxt scaffold
- **WHEN** a project is generated with the Nuxt frontend
- **THEN** the generated application root, Nuxt configuration, package manifest, and declared build command form a valid Nuxt project

### Requirement: Every advertised backend scaffold builds with its declared toolchain
Each advertised backend stack SHALL generate a complete minimal application or framework skeleton, valid build manifest, source entry, and required framework files needed for its declared build command to succeed.

#### Scenario: NestJS scaffold
- **WHEN** a project is generated with NestJS
- **THEN** `package.json`, TypeScript configuration, Nest CLI configuration, module, and bootstrap entry are complete and `npm run build` succeeds

#### Scenario: Go Gin scaffold
- **WHEN** a project is generated with Go Gin
- **THEN** `go.mod` and package entry form a valid module and `go build ./...` succeeds without instructions to run `go mod init` again

#### Scenario: Spring Boot scaffold
- **WHEN** a project is generated with Spring Boot
- **THEN** the Maven manifest, conventional source path, package declaration, and application class form a valid project and `mvn package` succeeds

#### Scenario: .NET 8 scaffold
- **WHEN** a project is generated with .NET 8
- **THEN** the project manifest targets a supported .NET 8 SDK, source entry is valid, and `dotnet build` succeeds

#### Scenario: Laravel scaffold
- **WHEN** a project is generated with Laravel
- **THEN** Composer autoload and scripts, Artisan bootstrap files, application bootstrap, HTTP kernel, exception handling, routes, and environment template form a valid Laravel project whose declared test command succeeds

### Requirement: Generated dependency instructions match generated manifests
Commands written into generated documentation SHALL match files already present in the scaffold. The generator SHALL NOT instruct users to initialize or recreate a manifest that the generator already wrote.

#### Scenario: Go generated README
- **WHEN** a Go backend is generated
- **THEN** its README does not instruct the user to run `go mod init` for the already generated module

#### Scenario: Node generated README
- **WHEN** any Node-based frontend or backend is generated
- **THEN** its setup commands operate on the existing package manifest and do not overwrite it with `npm init -y`

### Requirement: Generated CI invokes only declared project capabilities
Generated CI SHALL invoke lint, typecheck, test, and build commands only when the generated manifest or build system declares a corresponding capability. Each generated frontend and backend job SHALL include at least one successful build or validation command appropriate to that stack.

#### Scenario: Stack without a lint script
- **WHEN** CI is generated for a stack whose package manifest has no lint script
- **THEN** the workflow does not run `npm run lint` or another nonexistent lint command

#### Scenario: Stack with a lint script
- **WHEN** CI is generated for a stack whose package manifest declares a lint script
- **THEN** the workflow runs that script before its build command

#### Scenario: Test-capable stack
- **WHEN** CI is generated for a stack with an executable test command
- **THEN** the workflow runs tests as a distinct step before build or packaging

#### Scenario: Build validation
- **WHEN** CI is generated for any advertised frontend or backend
- **THEN** its job contains a command that validates the generated project using the stack's declared toolchain

### Requirement: Docker output references runnable artifacts
A generated Docker Compose file SHALL reference only services whose image or build context exists and is runnable. A backend build service SHALL be emitted only when the selected backend includes a valid Dockerfile and required runtime files; otherwise the backend service SHALL be omitted or generation SHALL fail before replacing the destination.

#### Scenario: Backend with Docker support
- **WHEN** Docker Compose is generated for a backend that has a valid generated Dockerfile
- **THEN** the compose backend build context resolves to that Dockerfile and configuration validation succeeds

#### Scenario: Backend without Docker support
- **WHEN** Docker Compose is generated for a backend that has no valid Dockerfile
- **THEN** the compose file does not contain an unusable `build: ./backend` service

#### Scenario: Database-only selection
- **WHEN** Docker Compose is generated for a supported database with backend set to none
- **THEN** the compose file contains a valid database service without a backend service

### Requirement: Generated files and reports preserve completeness
Generated project files SHALL NOT silently contain omitted content. Where a report format cannot represent all findings, it SHALL either paginate or state the exact omitted count as required by `fix-generated-output-bugs`; project manifests and generated reports SHALL not claim completeness when they are not complete.

#### Scenario: Multi-page audit report
- **WHEN** an audit report contains more findings than fit on one PDF page
- **THEN** all findings are represented across pages or the report discloses the exact number omitted

#### Scenario: Generated source placeholder
- **WHEN** a generated file is a configuration, manifest, entry point, or CI definition
- **THEN** it contains the fields and references required by its declared format and is not silently truncated
