## Why

SNIPE-CODE currently advertises production-grade scaffolding and security automation, but several generated projects, failure paths, verification gates, release claims, and documentation claims are not backed by executable guarantees. A single hardening change will establish honest, testable quality boundaries before the project is called enterprise-grade.

## What Changes

- Keep SNIPE-CODE as a Rust-native CLI; target-language toolchains run only in generated projects and verification jobs, never as internal runtime dependencies.
- Restrict project generation to a safe destination and prevent accidental deletion of roots, arbitrary absolute paths, traversal paths, or symlink targets.
- Define buildable output contracts for every supported frontend, backend, database, Docker, and generated CI combination; remove or correct templates that cannot satisfy their contract.
- Make generated CI commands match generated manifests and validate every supported target stack in isolated directories.
- Make generated multi-stack projects discoverable by the audit, run meaningful rules for JS/TS, PHP, Go, .NET, Java/Spring, and Rust, and report unknown or partial coverage without claiming the whole codebase is clean.
- Make audit argument errors, execution errors, unsupported export formats, and policy-level findings return stable non-zero exit codes; replace unconditional “clean” wording with scope-aware results.
- Add root CI, pinned Rust toolchain, cross-platform build verification, packaged release artifacts, checksums, and verifiable release provenance/signing.
- Align README and product documentation with behavior proven by tests; remove “enterprise-grade” and “security-baked” claims until corresponding guarantees pass in CI.
- Build on `fix-generated-output-bugs` and consume stack detection/rules from `feat-stack-aware-audit` rather than duplicating their unfinished scope.

## Capabilities

### New Capabilities

- `safe-project-generation`: Destination validation, symlink protection, and non-destructive collision behavior for project generation.
- `generated-project-contracts`: Runnable-file, framework, Docker, and CI contracts for every supported generated stack.
- `stack-aware-audit`: Multi-location stack discovery, meaningful cross-language coverage, and honest reporting of evaluated versus unsupported scope.
- `generator-verification`: Isolated build/test validation of generated projects using their native toolchains.
- `cli-process-semantics`: Stable exit codes, export validation, and scope-aware audit reporting for scripts and CI.
- `release-assurance`: Reproducible root checks, pinned toolchain, multi-platform artifacts, checksums, and signed/verifiable provenance.
- `product-documentation-accuracy`: Documentation claims constrained to capabilities demonstrated by the test and release suites.

### Modified Capabilities

_(none) — no canonical baseline specs currently exist under `openspec/specs/`._

## Impact

- **Code:** `src/generator.rs`, `src/commands/new.rs`, `src/stack.rs`, `src/docker.rs`, `src/cicd.rs`, `src/audit.rs`, `src/main.rs`, `src/lib.rs`, and audit rule data.
- **Tests:** Existing Rust integration tests plus stack-specific generated-project verification and CLI exit-code tests.
- **Tooling:** Rust toolchain pin, root CI/release workflow, and target-language build environments.
- **Documentation:** `README.md`, PRD, technical design, installation/release instructions, and known limitations.
- **Dependencies:** No Node, Python, or service runtime is added to the SNIPE-CODE binary. Any target-language build dependency remains inside generated-project verification jobs.
- **Breaking:** Project names containing path separators, absolute/root destinations, and traversal paths are rejected; unsafe existing symlink targets are never recursively deleted.
