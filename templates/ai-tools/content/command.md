# /snipe-code

Run the project's security audit and act on the result.

## Steps
1. Run `snipe-code audit . --export md`.
2. Read the report the audit prints (path and findings).
3. Review each HIGH finding against source behavior. Report false positives unchanged.
4. For confirmed risks, make the smallest safe fix and describe what was verified.
5. Run the project's test command, then re-run the audit.
