## Purpose

Audit rules are typed data with stable identity. Line, file-presence, ignore-containment, and config-content semantics remain distinct; stack coverage includes Rust.

## ADDED Requirements

### Requirement: Typed rule variants
The rule registry SHALL support line-pattern, file-presence, ignore-containment, and config-content rule variants, each with a stable ID, owner, severity, category, description, and file matcher.

#### Scenario: Line rule
- **WHEN** a line-pattern rule matches a configured file
- **THEN** the finding includes its stable rule ID, file, and line number

#### Scenario: File-presence rule
- **WHEN** a required or forbidden file-presence rule is evaluated
- **THEN** the rule reports presence or absence at the context-relative path without requiring a line match

#### Scenario: Ignore-containment rule
- **WHEN** an ignore-containment rule checks a tracked path against `.gitignore`
- **THEN** it reports whether the required ignore entry contains that path

#### Scenario: Config-content rule
- **WHEN** a config-content rule checks a named configuration file
- **THEN** it reports the configured literal or pattern match with file and line information when applicable

### Requirement: Context-owned applicable rules
Common rules SHALL be evaluated for the audit root, and stack/framework rules SHALL run only for their owning detected context. Every detected supported context SHALL have at least one non-empty applicable rule group.

#### Scenario: Common rules run without a framework
- **WHEN** audit evaluates a known ecosystem whose framework is unknown
- **THEN** common rules run and the report does not invent a framework rule group

#### Scenario: PHP Laravel rules
- **WHEN** a PHP context contains a `laravel/framework` marker
- **THEN** PHP common and Laravel rules are eligible for that context

#### Scenario: Rust rules are non-empty
- **WHEN** audit evaluates a Rust context
- **THEN** Rust secret, dynamic process/shell, and selected unsafe-block rules are eligible

### Requirement: Stable rule-ID deduplication
The system SHALL deduplicate only identical rule instances using stable rule ID, context ID, normalized relative file, line, and match discriminator. Distinct rule IDs SHALL remain separate findings even when file, line, category, or severity match.

#### Scenario: Repeated identical instance
- **WHEN** the same rule ID, context, file, line, and match discriminator is produced more than once
- **THEN** only one finding remains

#### Scenario: Different rules share a line
- **WHEN** two different stable rule IDs match the same file and line
- **THEN** both findings remain and each retains its own rule ID

#### Scenario: Rule ID is independent of display text
- **WHEN** a rule description changes without changing its registry ID
- **THEN** deduplication still uses the stable ID rather than the new description

### Requirement: Language-specific and common security rules
The active rule set SHALL cover common security/config checks and meaningful SQL, XSS, secret, and configuration patterns for JavaScript/TypeScript, PHP/Laravel, Go/Gin, Java/Spring, .NET, and Rust, without applying a framework rule when its content marker is absent.

#### Scenario: PHP SQL concatenation
- **WHEN** a PHP file contains a SQL keyword adjacent to string concatenation
- **THEN** a HIGH SQL-injection finding is reported with file and line

#### Scenario: Go SQL formatting
- **WHEN** a Go file uses SQL keywords with a format verb such as `fmt.Sprintf`
- **THEN** a HIGH SQL-injection finding is reported

#### Scenario: JavaScript template query
- **WHEN** a JavaScript/TypeScript file contains a SQL keyword in a template literal with interpolation
- **THEN** a HIGH SQL-injection finding is reported

#### Scenario: Rust secret
- **WHEN** a Rust source file assigns a literal to a secret-named variable or constant
- **THEN** a HIGH hardcoded-secret finding is reported unless the value is an environment reference or placeholder

#### Scenario: Placeholder is not a secret
- **WHEN** a secret-named assignment uses `process.env`, `getenv`, `env(...)`, or a `your-` placeholder
- **THEN** no hardcoded-secret finding is produced for that line

### Requirement: Unknown scope runs common rules only
When no recognized ecosystem can be established, the system SHALL run only common rules and SHALL label language-specific coverage as unknown.

#### Scenario: Unknown directory
- **WHEN** audit runs on a directory with no recognized manifest
- **THEN** common rules may run, but no language/framework-specific rule is claimed
