# SNIPE-CODE project rules

Use this skill when editing code in a SNIPE-CODE generated project.

## Steps
1. Read `.ai-workspace/rules.md` before changing code.
2. Make the smallest change that satisfies the request.
3. Run the project's declared build and test command before claiming success.
4. When the change touches input handling, secrets, or queries, run
   `snipe-code audit . --export md` and fix every HIGH finding.

## Rules
- No hardcoded secrets; environment variables only.
- Parameterized queries only; validate input at trust boundaries.
- Report what was verified and what was not.
