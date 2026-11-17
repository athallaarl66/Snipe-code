# Test authoring

1. Read relevant source and user-selected requirements. Ask the user when expected behavior is ambiguous; do not invent assertions or requirements.
2. Propose tests to author or repair. Preview exact files and edits and obtain separate explicit approval for source edits before writing tests.
3. Preview `snipe-code verify --only test`, its target and working directory, test commands, database target, endpoint access, network/data transmission, installation and expected effects. Obtain separate explicit approval for each applicable effect before running. Do not install, access endpoints or share data without their own approval.
4. Report actual test results and any blocked or unavailable execution; never claim a pass without execution. The agent authors tests; the executable runs declared tests and produces deterministic results, not inferred test cases.

## Safety

Invoking this workflow is not approval. Before report writes, source edits, install, tests, endpoint access, or data transmission, show the exact action, target, data, and effects and obtain separate explicit approval for each scope. Changed or unknown scope requires a new preview and approval. Stop without approval. Default to offline operation: do not transmit project data, contact endpoints, or install dependencies without separate scope-bound approval. Commands and tests can spawn processes and access files or networks; they are not sandboxed. If effects or targets are unknown, stop and ask rather than assume safety. Instructions are not a runtime permission boundary: never bypass core preflight. Explicit remote, production-like, or ambiguous database targets remain blocked even with approval. An undetected database target is unknown, not safe or absent; follow the CLI's explicit approval policy and honor any current preflight rejection. The generated-project harness exception never applies to this workflow.
