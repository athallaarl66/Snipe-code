## 1. Context Discovery and Identity

- [ ] 1.1 Implement bounded recursive project-context discovery under the audit target, including nested `frontend/` and `backend/` roots, manifest boundaries, hidden/VCS/dependency/build exclusions, and explicit symlink exclusion paths without following symlinks
- [ ] 1.2 Implement content-aware identity that separates ecosystem from framework for JavaScript/TypeScript, React, Vue, Nuxt, Next.js, PHP, Laravel, Go, Gin, Java, Spring, .NET, and Rust; report unknown/partial identity honestly

## 2. Typed Rules and Coverage

- [ ] 2.1 Create `src/rules.rs` with a typed `Rule`/`RuleRegistry` model for line-pattern, file-presence, ignore-containment, and config-content rules, stable rule IDs, owner scope, severity, category, description, and file matchers
- [ ] 2.2 Define embedded common rules and applicable JavaScript/TypeScript, PHP/Laravel, Go/Gin, Java/Spring, .NET, and Rust rules; every detected supported context must have a non-empty applicable rule group
- [ ] 2.3 Add Rust coverage for hardcoded secrets, dynamic process/shell command construction, and the selected unsafe-block heuristic; do not classify an unknown framework as a supported framework

## 3. Rule Engine, Ownership, and Deduplication

- [ ] 3.1 Execute each rule only in its owning project context, preserve context/file scope, and apply common rules to the audit root
- [ ] 3.2 Deduplicate only identical instances using stable `rule_id + context_id + normalized_relative_file + line + match discriminator`; retain distinct rule IDs on the same file and line
- [ ] 3.3 Make every evaluated, unknown, partial, and excluded path/rule group available to the shared audit result snapshot

## 4. Code Smells and Reports

- [ ] 4.1 Keep deterministic code-smell checks for long functions, nesting, magic numbers, TODO/FIXME/HACK, empty catch/except, assignment-in-condition, and duplicate literals, limited to evaluated files
- [ ] 4.2 Update Markdown/PDF/DOCX output to show detected contexts, evaluated rule groups, unknown/partial scope, excluded paths/reasons, and `No findings detected by evaluated rules` instead of a whole-codebase clean claim
- [ ] 4.3 Replace the hardcoded audit date with one shared runtime date for all exporters; add controlled-clock/current-date coverage before claiming verification

## 5. AI Workspace

- [ ] 5.1 Generate root `AGENTS.md` with discovered ecosystem/framework context, applicable security rules, code-style guidance, and honest unknown/partial scope; do not create `.ai-workspace/`
- [ ] 5.2 Remove fictional slash commands and ensure generated guidance does not claim heuristic rules prove security

## 6. Tests and Verification

- [ ] 6.1 Test recursive `frontend/`/`backend/` discovery, nested monorepos, bounded traversal, excluded paths, and symlink paths
- [ ] 6.2 Test ecosystem-versus-framework content markers, misleading manifests, unknown/partial identity, and all supported contexts including Rust
- [ ] 6.3 Test each typed rule variant, common rules, stack-owned rules, stable rule-ID deduplication, distinct same-line rules, and placeholder handling
- [ ] 6.4 Test code smells, runtime report date, scope wording, root `AGENTS.md`, and absence of fictional commands
- [ ] 6.5 Run the repository test/lint commands only during implementation verification; this artifact-only task does not claim those commands passed

## 7. Boundary with Hardening

- [ ] 7.1 Keep generated-project bootability, capability-aware CI lint/test steps, native stack matrix, report pagination/completeness, and release assurance in `harden-generator-project-quality`; do not duplicate those implementation tasks here
