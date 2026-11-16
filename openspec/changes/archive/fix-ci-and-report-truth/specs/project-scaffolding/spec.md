# Spec Delta

## Purpose

Defines what a generated project receives for each supported stack and template, and the guarantee that the dependency constraints it declares will resolve in a clean environment.

## ADDED Requirements

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
