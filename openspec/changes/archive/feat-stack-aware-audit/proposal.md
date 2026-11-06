## Why

The current audit in `src/audit.rs` runs the same small set of JS/TS-oriented checks against every project. Generated projects place manifests and source under `frontend/` and `backend/`, so root-only manifest detection would miss both contexts. A manifest name also identifies an ecosystem only: `composer.json` does not prove Laravel, `pom.xml` does not prove Spring, and `package.json` does not prove React, Vue, Nuxt, or Next.js.

The current check model cannot express file-presence, ignore-containment, or config-content rules without pretending every check is a regular expression. Deduplication also needs stable rule identity: two different rules can legitimately report the same file and line. The report currently lacks evaluated, unknown, and excluded scope, and hidden symlink paths are not an explicit discovery boundary. This change keeps the useful stack-aware, code-smell, runtime-date, and `AGENTS.md` intent while making coverage truthful.

## What Changes

- Discover project contexts recursively through the audit target, including generated `frontend/` and `backend/` roots, with bounded traversal and explicit exclusions.
- Identify ecosystem and framework from manifest and file content, not folder names: JavaScript/TypeScript, PHP, Go, .NET, Java, and Rust as ecosystems; React, Vue, Nuxt, Next.js, Laravel, Spring, and other supported framework markers as content-aware framework identities.
- Replace pretend-everything-is-regex checks with a typed registry for line-pattern, file-presence, ignore-containment, and config-content rules.
- Add meaningful common and ecosystem/framework rules, including non-empty Rust coverage for secrets and unsafe process/shell construction.
- Scope each rule to its owning project context and deduplicate only identical stable rule instances; distinct rule IDs on one file and line remain distinct findings.
- Exclude and report dependency/build/hidden paths and symlink paths without following symlinks, including reason and scope impact.
- Report detected contexts, evaluated rule groups/files, unknown or partial scope, exclusions, and heuristic limitations instead of claiming a whole codebase is clean.
- Preserve code-smell heuristics, runtime report dates, and root `AGENTS.md` generation as secondary capabilities with the same honest-scope boundary.

## Capabilities

### New Capabilities
- `stack-detection`: Recursive project-context discovery plus content-aware ecosystem/framework identity, with unknown and partial results.
- `stack-specific-rules`: Typed common and stack-owned audit rules, including Rust, stable IDs, context ownership, and scope-aware execution.
- `code-smells`: Deterministic structural smell checks on evaluated source files, with heuristic limitations disclosed.
- `ai-workspace`: Root `AGENTS.md` containing detected context, applicable security guidance, code-style guidance, and no fictional commands.

### Modified Capabilities
_(none)_

## Impact

- **Code**: `src/audit.rs` (recursive context model, typed rule execution, scope snapshot, and runtime date), new `src/rules.rs`, and `src/ai_workspace.rs` (root `AGENTS.md` and context-aware content).
- **Data**: Embedded rule definitions deserialize into typed Rust variants. Fixed-token checks use standard-library matching; a regex dependency is optional and is not assumed by this artifact.
- **Dependencies**: No target-language runtime enters SNIPE-CODE. Any future regex dependency requires implementation review; no dependency change is claimed here.
- **Testing**: Planned unit/integration coverage for nested contexts, ecosystem/framework markers, typed rules, Rust rules, stable-ID deduplication, symlink exclusions, unknown scope, and report wording. No application tests are claimed complete by this artifact-only change.
- **Breaking**: Generated AI context moves from `.ai-workspace/rules.md` to root `AGENTS.md`; report language becomes scope-aware. Existing findings are not silently reinterpreted as a clean result.
