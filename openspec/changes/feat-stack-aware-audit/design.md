# Design — feat-stack-aware-audit

## Context

Lihat `proposal.md` untuk motivation. Current source behavior is narrower than the original design:

- `src/audit.rs:31-44` calls five fixed checks and does not build project contexts.
- `src/audit.rs:85-175` hardcodes JS/TS-oriented extensions and cannot express file-presence, ignore, or config-content semantics as typed rules.
- `src/audit.rs:199-231` recursively walks source files, but skips only hidden directories, `node_modules`, and `target`; it does not define project-context ownership or symlink handling.
- `src/audit.rs:431-433` still returns the hardcoded date `2026-07-28` for reports.
- `src/ai_workspace.rs:4-15` writes `.ai-workspace/rules.md`, `rules.mdc`, and `.gitkeep`; it does not write root `AGENTS.md`.
- `src/cicd.rs`, `src/stack.rs`, and `src/generator.rs` generate nested `frontend/` and `backend/` trees, so recursive discovery is required for the audit to see generated projects.

The design below describes target behavior, not completed application implementation. It preserves the original intent of stack-aware rules, code smells, runtime dates, and AI workspace guidance without claiming unsupported coverage.

## Goals / Non-Goals

**Goals:**
- Find project contexts recursively, including generated `frontend/` and `backend/` roots.
- Separate ecosystem identity from framework identity using content-aware manifest/file markers.
- Give every supported detected context a meaningful, non-empty applicable rule set, including Rust.
- Model line, file-presence, ignore-containment, and config-content checks as distinct typed rules.
- Keep stable rule identity through execution and deduplication.
- Exclude and disclose symlink and other non-evaluated paths.
- Report evaluated scope, unknown scope, partial traversal, and heuristic limits honestly.
- Keep code-smell checks, runtime dates, and root `AGENTS.md` generation consistent with the same context model.

**Non-Goals:**
- Not a complete SAST or proof of application security; v1 rules are deterministic heuristics.
- Not AST/type inference, taint analysis, or guaranteed framework bootability.
- Not a claim that every project is fully clean when only common rules or a subset of languages ran.
- Not a Node/Python/Go/Java/.NET/PHP runtime inside the SNIPE-CODE binary.
- Not generated-project build or CI repair; those contracts remain with `harden-generator-project-quality`.

## Decisions

### D1: Recursive, bounded project-context discovery
Model each recognized manifest location as an `AuditContext` containing root path, ecosystem, framework evidence, manifest path, owned file set, applicable rule IDs, and discovery status. Start at the requested audit path, walk nested directories, and discover contexts at any depth, including `frontend/` and `backend/`. A context owns source beneath it until a nested context boundary; a nested context gets its own ownership and rule evaluation.

Traversal is bounded by a fixed implementation limit and an explicit exclusion set for hidden directories, VCS metadata, `node_modules`, `target`, `vendor`, `bin`, `obj`, build/output directories, and report artifacts. A traversal limit, unreadable directory, or skipped boundary is recorded as partial/unknown scope. Do not infer a project solely from a folder named `frontend` or `backend`; a manifest is still required for context identity.

Directory symlinks and symlink files are not followed or read. Their paths are recorded as excluded with reason `symlink`, including the parent context. This prevents loops and prevents report claims about code that was never evaluated.

### D2: Content-aware ecosystem and framework identity
First identify ecosystem from recognized manifest content and file types:

- `package.json` → JavaScript/TypeScript ecosystem.
- `composer.json` → PHP ecosystem.
- `go.mod` → Go ecosystem.
- `*.csproj` or `*.sln` → .NET ecosystem.
- `pom.xml` or `build.gradle`/`build.gradle.kts` → Java ecosystem.
- `Cargo.toml` → Rust ecosystem.

Then identify framework only from content markers, never from directory name alone. Examples include `next`/`react`/`vue`/`nuxt` dependencies or scripts in `package.json`, `laravel/framework` in Composer requirements, `spring-boot-starter` in Maven/Gradle content, Gin markers in `go.mod` or imports, and web SDK/framework references in `.csproj`. A valid manifest with no recognized framework marker reports ecosystem `known` and framework `unknown`; an invalid or misleading manifest reports the reason and remains partial/unknown where appropriate. Multiple contexts remain separate even when they share an ecosystem.

### D3: Typed rule registry, not regex-only checks
Create `src/rules.rs` with a typed `Rule` model and embedded definitions. `RuleRegistry::load_all()` deserializes definitions into one of these variants:

- `LinePattern { file_match, pattern, negative_context }` for token or optional-regex matches on a line, with a line number.
- `FilePresence { path, requirement }` for required or forbidden paths.
- `IgnoreContainment { tracked_path, required_entry }` for checks such as `.env` being contained in `.gitignore`.
- `ConfigContent { path, checks }` for literal or optional-regex content checks in named configuration files.

Every rule carries a stable `id`, owner (`common`, ecosystem, or framework), severity, category, description, and context/file matcher. The registry keeps common rules separate from stack-owned rules. Fixed-token checks use standard-library matching; regex support is an implementation choice for patterns that need it, not a requirement to encode presence or ignore checks as regexes.

The initial applicable set must include:

- common: hardcoded-secret line checks, `.env.example` presence, `.gitignore` presence, and `.env` ignore containment;
- JavaScript/TypeScript and framework variants: SQL construction, XSS sinks, and secret assignments;
- PHP and Laravel variants: SQL concatenation, unescaped output, `APP_DEBUG`/`APP_KEY`, and secret assignments;
- Go and Gin variants: SQL formatting, process execution, and secret assignments;
- .NET variants: interpolated SQL, connection/configuration content, and secret assignments;
- Java/Spring variants: actuator exposure, SQL construction, and secret assignments;
- Rust: hardcoded secret assignments and dynamic process/shell command construction, plus an `unsafe` block rule where the heuristic is meaningful.

A detected supported context SHALL execute at least one non-empty applicable rule group. Unknown contexts execute common rules only and are labeled as such.

### D4: Context ownership and stable rule-ID deduplication
Run common rules against the audit root and context rules only against files owned by their context. A nested context boundary prevents a parent rule set from silently reclassifying child files. If a file belongs to multiple overlapping discovery representations, normalize its relative path and collapse only the same rule instance.

The deduplication key is stable and explicit: `rule_id + context_id + normalized_relative_file + line + match discriminator`. Repeated traversal or overlapping matches with the same key produce one finding. Different rule IDs remain separate even when category, file, and line match. `rule_id` is declared in the registry and never generated from a display description, rule order, or line number.

### D5: Honest evaluated and unknown scope reporting
Every audit result carries one shared snapshot containing detected contexts, ecosystem/framework evidence, evaluated rule groups, evaluated file count, finding count, unknown contexts/files, partial-traversal reasons, and excluded paths with reasons. Markdown, PDF, and DOCX render the same snapshot rather than independently claiming coverage.

Use wording such as `No findings detected by evaluated rules` when the evaluated set is empty of findings. Never replace that wording with a whole-codebase clean claim. Reports distinguish:

- evaluated scope: contexts and rules that actually ran;
- unknown scope: manifests/files whose ecosystem or framework could not be established;
- partial scope: traversal stopped, a path was unreadable, or a rule group was not applicable;
- excluded scope: hidden/dependency/build/report paths and symlinks, with reason.

### D6: Code smells remain deterministic and scope-aware
Keep the existing smell intent—long functions, deep nesting, magic numbers, `TODO`/`FIXME`/`HACK`, empty catch/except blocks, assignment-in-condition, and repeated string literals—but run them only on evaluated source files owned by a discovered context. Report them as heuristic findings, not proof of defects. Brace-based heuristics apply to supported curly-brace languages including Rust; other languages receive only applicable simpler checks. Excluded paths and unknown contexts are listed in scope output, not silently treated as passing.

### D7: Runtime date without assuming a new dependency
Replace the audit report's hardcoded date with a shared runtime date derived from `std::time::SystemTime` and a small civil-date conversion, using one date value for all exporters. Keep the implementation free of a timezone library for this v1. The artifact does not claim that a current-date test has run; verification must compare output to a controlled clock or an accepted date contract.

### D8: Root `AGENTS.md` uses discovered context
Generate `AGENTS.md` at project root instead of `.ai-workspace/`. Content names the discovered ecosystem/framework contexts, security rules that are actually applicable, code-style guidance, and known unknown/partial scope. Do not create `.ai-workspace/` or include slash commands that the CLI does not implement. AI guidance must not imply that heuristic rules prove the project is secure.

## Risks / Trade-offs

- [Nested projects can be unrelated] → use manifest boundaries, bounded traversal, explicit context ownership, and report every discovered context.
- [Content markers can be incomplete] → distinguish ecosystem-known/framework-unknown from fully identified and disclose partial scope.
- [Typed rules add model complexity] → use one enum and one registry instead of parallel ad hoc check functions; preserve stable IDs for deduplication and reporting.
- [Symlink exclusion hides user code] → never follow symlinks, but list every excluded symlink path and reason in the report.
- [Heuristics produce false positives/negatives] → label rule groups heuristic, retain rule IDs, and avoid clean-codebase wording.
- [Optional regex support may affect dependency policy] → prefer fixed-token stdlib matching and require review before adding a regex crate.
- [Noisy or incomplete generated projects] → do not imply framework bootability; generated-project contracts remain in hardening change.

## Migration Plan

1. Add typed rule definitions and context/scope data structures in `src/rules.rs` and `src/audit.rs`.
2. Implement bounded recursive discovery, content-aware ecosystem/framework markers, symlink/exclusion recording, and context ownership.
3. Replace fixed checks with typed rule evaluation and stable rule-ID deduplication; add Rust rule coverage.
4. Update code-smell execution and shared report snapshot so evaluated/unknown/partial/excluded scope is visible.
5. Replace report date and move generated AI context to root `AGENTS.md` without fictional commands.
6. Add tests for nested generated contexts, monorepos, framework distinctions, every supported stack including Rust, typed non-regex rules, stable-ID dedupe, symlink paths, and honest reports.
7. Keep generated-project build/CI contracts and native verification out of this change; consume them from `harden-generator-project-quality`.
8. Rollback: revert implementation and generated context format together; no user data migration is required.

## Open Questions

None. Regex dependency choice is deliberately deferred until a concrete pattern requires it; typed rule semantics and scope reporting are fixed by this design.
