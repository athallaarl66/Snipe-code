# SNIPE-CODE Project Rules

Instructions for AI coding agents working in this project.

## Security Rules (CRITICAL)
- NEVER hardcode secrets, API keys, or passwords — use environment variables
- ALWAYS use parameterized queries instead of string-concatenated SQL
- ALWAYS sanitize user input before rendering it
- NEVER pass user input to `eval()` or `exec()`
- ALWAYS validate input at trust boundaries

## Approval and safety
- These instructions do not authorize actions and are not a runtime permission boundary.
- Before source edits, install, tests, endpoint access, data transmission, or report writes, preview exact command or edits, target, working directory, data, and effects; obtain separate explicit user approval for each scope. Re-preview and ask again when scope changes. Stop without approval.
- Default to offline operation: do not transmit project data, contact endpoints, or install dependencies without separate scope-bound approval. Commands and tests can spawn processes and access files or networks; they are not sandboxed. If effects or targets are unknown, stop and ask rather than assume safety.
- Never override core preflight: explicit remote, production-like, or ambiguous database targets remain blocked even with approval. An undetected database target is unknown, not safe or absent; follow CLI approval and honor current rejection. The generated-project harness exception does not apply here.

## Workflow
- Read `.ai-workspace/rules.md` before editing code
- Keep changes minimal and scoped to the request
- Run the project's build and test command before claiming a change works
- After touching input handling, secrets, or queries, run
  `snipe-code audit . --export md`
- Review each HIGH finding against source behavior. Report false positives unchanged. For confirmed risks, make the smallest safe fix, run tests, then repeat the audit

## Code Style
- Follow the conventions of the stack this project was generated with
- Prefer functional components; handle errors explicitly
- TypeScript strict mode where the stack uses TypeScript
- No dead code, no scaffolding for later

## Token Efficiency
- Prefer code over prose; do not restate the request back
- State the change and the reason, then stop
