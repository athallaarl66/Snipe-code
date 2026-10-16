# /snipe-code

Run the project's security audit and act on the result.

## Steps
1. Run `snipe-code audit . --export md`.
2. Read the report the audit prints (path and findings).
3. Fix every HIGH finding; describe each fix in one line.
4. Run the project's test command, then re-run the audit.
5. Stop when no HIGH finding remains and report what was changed.
