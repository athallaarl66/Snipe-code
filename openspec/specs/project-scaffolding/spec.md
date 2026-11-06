# project-scaffolding Specification

## Purpose
Defines what a generated project receives for each supported stack — the dependency constraints it declares, the test entry point and scripts it ships, and the CI that exercises them — plus the guarantee that those constraints resolve and those commands pass in a clean environment.

## Requirements

### Requirement: Generated dependency constraints resolve in a clean environment
Every dependency constraint written into a generated project manifest MUST resolve to an installable release under the package manager's current security-advisory policy.

#### Scenario: Laravel stack installs
- **WHEN** a Laravel backend is generated and `composer install --no-interaction` runs in a clean environment
- **THEN** dependency resolution succeeds without a security-advisory block

#### Scenario: Advisory-blocked framework line is not emitted
- **WHEN** the scaffold writes the Laravel framework constraint
- **THEN** the constraint targets a release line that has at least one advisory-free release

### Requirement: Each supported stack is verified end to end in CI
CI MUST generate a project for every supported stack, install its dependencies, and run that stack's build or test command, failing the job when any step fails.

#### Scenario: Laravel backend job
- **WHEN** the Laravel backend CI job runs
- **THEN** it generates the project, completes `composer install`, and passes `php artisan test`

#### Scenario: Failure is attributed to the stack
- **WHEN** a stack's install, build, or test command exits non-zero
- **THEN** the job fails and names the stack and the command that failed

### Requirement: Generated projects include runnable test infrastructure
A generated project MUST include a test entry point for its stack and at least one example test that passes and exercises generated code.

#### Scenario: Frontend stack declares a test script
- **WHEN** a frontend stack is generated
- **THEN** its manifest declares a test script and the project contains an example test

#### Scenario: Backend stack declares a test command
- **WHEN** a backend stack is generated
- **THEN** its manifest or build file declares a test command and the project contains an example test

#### Scenario: The example test exercises generated code
- **WHEN** the example test runs against the generated project
- **THEN** it asserts behaviour of generated code rather than asserting a constant

### Requirement: Generated projects declare lint and type-check commands where available
A generated project MUST declare a lint command when its stack's toolchain provides a linter, and a type-check command when the stack is statically typed.

#### Scenario: Statically typed stack declares a type-check command
- **WHEN** a stack with a static type checker is generated
- **THEN** its manifest declares a type-check script

#### Scenario: Untyped stack declares no type-check command
- **WHEN** a stack whose toolchain has no static type checker is generated
- **THEN** its manifest declares no type-check script

### Requirement: Generated CI invokes only declared package-manager scripts
The generated workflow MUST invoke a package-manager script only when the generated project's manifest declares that script.

#### Scenario: Lint step matches a declared script
- **WHEN** the generated workflow runs a lint step for a manifest-based stack
- **THEN** that project's manifest declares a lint script

#### Scenario: Undeclared script is never invoked
- **WHEN** a generated project's manifest declares no test script
- **THEN** the generated workflow does not invoke a test script for it

