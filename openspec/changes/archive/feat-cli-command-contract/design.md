# Design: CLI command contract

## Context

Three command surfaces already exist and are stable in behavior: `new` (scaffolding), `analyze` (read-only report), `audit` (heuristic scan plus export), and `verify` (gate runner). The gap this change closes is not a missing capability but an undocumented contract: where a command runs, what it writes, and what a skipped or empty run exits with. One behavior change is needed, because documenting the workaround "do not run `verify` over a directory that contains other projects" is worse than offering the option that makes it safe.

## Decision: skip the install gate by rewriting the plan, not by filtering at execution

`verify` builds a `GatePlan` per project part and then executes it. `--no-install` clears the install gate's commands and sets its `not_applicable` reason in the plan (`Options::skip_install` handled in `verify::run`). The alternative, filtering the gate inside `execute`, would lose the reason in the report and would leave two places that decide which gates exist.

Consequences of this placement:

- The gate stays visible in the report as `not-applicable`, with the reason `skipped by --no-install`, so a reader can tell a skipped gate from a project that declares none.
- `Report::executed` does not count it, so a run where the install gate was the only applicable gate exits `3` ("nothing executed") rather than `0`. That is the honest outcome: no command ran, and pretending success would hide an unverified project.
- `--only install` combined with `--no-install` asks for a gate and skips it in the same run, so it is rejected as a usage error (exit `2`) instead of silently choosing one of the two.

## Decision: one reason string, exported

The reason is exported as `verify::SKIP_INSTALL_REASON` so tests assert the same literal the CLI prints. A free-form string in the report would drift from the test that checks it.

## Decision: documentation carries the contract, in one table

Both READMEs gain a table of command, run directory, reads, writes, and exit codes, placed directly under the existing command table. Prose is kept to two bullets about the filesystem effects, because the surprise users hit is "the file appeared", not "the semantics are unclear". The FAQ gains the same two facts in question form, since `faq` is what a user runs when the README failed them.

## Decision: generated `.gitignore` covers tool-created artifacts

Two artifacts appear inside a generated project through documented workflows: `bin/` and `obj/` from the .NET build the generated CI performs, and `Audit_Report.*` from the `/snipe-code` command whose body runs `snipe-code audit . --export md`. Ignoring them is the smallest fix that keeps a generated project's `git status` clean. `package-lock.json` is deliberately not ignored, because a lockfile belongs in the repository.

## Trade-offs and limits

- `--no-install` is per run, not persisted; there is no config file, which matches the rest of the CLI.
- Skipping the install gate on a project whose dependencies are absent makes the later gates fail. That failure is reported as it happens, and the run's exit code is `1`, not `0`.
- The documentation table describes directories, not mounted volumes or container paths; the container image section keeps its own usage line.
