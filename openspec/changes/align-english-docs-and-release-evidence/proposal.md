## Why

SNIPE-CODE v0.5.3 is published with release assets and a crates.io package, but README.md still says only source builds work. Existing feature and scenario documents mix Indonesian and English, contain outdated examples, and label draft test snippets as though they were results. Users need accurate English documentation and a clear distinction between automated checks, UAT plans, and actual test evidence.

## What Changes

- Update README.md to reflect independently verified v0.5.3 release channels, a working first-run path, current CLI flags, and remaining limitations. Mark any channel not exercised on a target as unverified rather than unavailable or fully tested.
- Review every tracked Markdown file in README.md, docs/, and templates/ai-tools/content/. Convert Indonesian prose to English without changing CLI literals, code identifiers, source paths, or behavior. Replace obsolete examples and claims with present behavior, and distinguish historical plans from implemented features. Keep historical directories/identifiers unless a referenced path must change.
- Replace docs/README.md claims about nonexistent test reports and unsupported `sdd-gen` commands with an accurate document index.
- Define English UAT cases for the supported CLI flows and an evidence template for a versioned test report. Populate actual results only from executed checks, published artifacts, and a human UAT run; unknowns remain not run or blocked, without fabricated sign-off.
- Keep formal QA/evidence ownership distinct from product documentation. This change does not automatically retag, publish, change remote settings, or declare release-ready from a green CI badge alone.

## Capabilities

### New Capabilities

- `release-acceptance-evidence`: repeatable UAT cases and versioned test reporting that separates executed evidence from unrun checks and human approval.

### Modified Capabilities

- `product-documentation-accuracy`: all published Markdown guidance is in English, points to real commands and files, and clearly labels historical/planned content.
- `release-assurance`: documentation of available channels reflects evidence for the specific published version; CI success is not substituted for installer and platform tests.

## Impact

Documentation surfaces: README.md, docs/README.md, docs/features/, docs/production/, and the Markdown sources under templates/ai-tools/content/. New acceptance/test-report documents belong under docs/, with paths indexed from docs/README.md. Existing CLI/test contracts may require narrowly targeted test updates only if text assertions pin an obsolete claim. The gitignored openspec/ change stays local. No application feature or release publication in the proposal phase.
