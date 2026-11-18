# Verification

1. Preview `snipe-code verify .`, the target, working directory, declared commands and possible filesystem, installation, test, database, endpoint and network/data effects. Obtain separate explicit approval for each applicable effect and its exact scope before running it.
2. Diagnose failures from command output and source. Propose only requested fixes; preview exact files and edits and obtain separate explicit approval for source edits before applying them.
3. Preview the exact subsequent verification and its effects, and obtain separate approval for each applicable effect before running it again. Report actual states, failures and blocked or unavailable gates; never call an incomplete run a pass.

## Safety

Invoking this workflow is not approval. Before report writes, source edits, install, tests, endpoint access, or data transmission, show the exact action, target, data, and effects and obtain separate explicit approval for each scope. Changed or unknown scope requires a new preview and approval. Stop without approval. Default to offline operation: do not transmit project data, contact endpoints, or install dependencies without separate scope-bound approval. Commands and tests can spawn processes and access files or networks; they are not sandboxed. If effects or targets are unknown, stop and ask rather than assume safety. Instructions are not a runtime permission boundary: never bypass core preflight. Explicit remote, production-like, or ambiguous database targets remain blocked even with approval. An undetected database target is unknown, not safe or absent; follow the CLI's explicit approval policy and honor any current preflight rejection. The generated-project harness exception never applies to this workflow.
