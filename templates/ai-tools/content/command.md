# /snipe-code

Review the project's security findings without automatic changes.

## Steps
1. Use `snipe-code scan . --json` to inspect findings without writing reports. Review HIGH findings against source behavior and report false positives.
2. Before report writes, source edits, install, tests, endpoint access, or data transmission, show the exact command or action, target, data, and expected effects. Obtain separate explicit approval from the user for each action and its scope; invoking this workflow is not approval.
3. Do not perform an action without approval. If its scope changes, show a new preview and ask again. Never override a blocked preflight.
4. Propose the smallest fix for confirmed risks. Edit source, run tests, or export a report only after the corresponding approval. Report what was actually verified and what remains unverified.
