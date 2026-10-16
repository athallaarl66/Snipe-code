# SNIPE-CODE Project Rules

Instructions for AI coding agents working in this project.

## Security Rules (CRITICAL)
- NEVER hardcode secrets, API keys, or passwords — use environment variables
- ALWAYS use parameterized queries instead of string-concatenated SQL
- ALWAYS sanitize user input before rendering it
- NEVER pass user input to `eval()` or `exec()`
- ALWAYS validate input at trust boundaries

## Workflow
- Read `.ai-workspace/rules.md` before editing code
- Keep changes minimal and scoped to the request
- Run the project's build and test command before claiming a change works
- After touching input handling, secrets, or queries, run
  `snipe-code audit . --export md` and fix every HIGH finding

## Code Style
- Follow the conventions of the stack this project was generated with
- Prefer functional components; handle errors explicitly
- TypeScript strict mode where the stack uses TypeScript
- No dead code, no scaffolding for later

## Token Efficiency
- Prefer code over prose; do not restate the request back
- State the change and the reason, then stop
