# SNIPE-CODE project rules

Use this skill when editing code in a SNIPE-CODE generated project.

## Approval and safety
These instructions do not authorize actions and are not a runtime permission boundary. Before source edits, install, tests, endpoint access, data transmission, or report writes, preview exact command or edits, target, working directory, data, and effects; obtain separate explicit user approval for each scope. Re-preview and ask again when scope changes. Stop without approval. Default to offline operation: do not transmit project data, contact endpoints, or install dependencies without separate scope-bound approval. Commands and tests can spawn processes and access files or networks; they are not sandboxed. If effects or targets are unknown, stop and ask rather than assume safety. Never override core preflight: explicit remote, production-like, or ambiguous database targets remain blocked even with approval. An undetected database target is unknown, not safe or absent; follow CLI approval and honor current rejection. The generated-project harness exception does not apply here.

## Steps
1. Read `.ai-workspace/rules.md` before changing code.
2. Make the smallest change that satisfies the request.
3. Run the project's declared build and test command before claiming success.
4. When the change touches input handling, secrets, or queries, run
   `snipe-code audit . --export md`.
5. Review each HIGH finding against source behavior. Report false positives unchanged. For confirmed risks, make the smallest safe fix, run tests, then repeat the audit.

## Rules
- No hardcoded secrets; environment variables only.
- Parameterized queries only; validate input at trust boundaries.
- Report what was verified and what was not.
