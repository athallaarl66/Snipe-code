# Documentation

Index of the documents in this directory. Every entry below exists in the repository at
the time of writing; anything planned but absent is named as a plan, not as a link.

## Product and design

| Document | Content |
| --- | --- |
| `features/SNIPE-PRD.md` | Product requirements: user stories, supported stacks, requirements, and the features deliberately out of scope |
| `features/technical.md` | Technical design of the 0.5.3 code base: modules, template layout, generated output, audit, analyze, verify, release |

## Scenario breakdowns

One directory per user story, four files each: production requirements (`prod`), design,
technical specification, and test cases.

| Directory | Story |
| --- | --- |
| `production/US-001-Init-dengan-Template/` | Generate a project from a preset template |
| `production/US-002-Init-Tanpa-Template/` | Generate a project without a template |
| `production/US-003-Dry-Run-Preview/` | Preview the generated structure without writing |
| `production/US-005-Company-Profile/` | Company profile preset |
| `production/US-006-Portfolio/` | Portfolio preset |
| `production/US-007-Blog/` | Blog preset |

The `-testing.md` files describe intended test cases. They are not executed test
reports; the executed evidence is the Rust test suite in `tests/` and the CI runs behind
the repository badge.

## Acceptance evidence

| Document | Content |
| --- | --- |
| `uat/UAT-MATRIX.md` | Acceptance cases for installation, generation, analyze, audit, verify, and opt-in agent files, with the status of each case |

Formal acceptance needs a human run of that matrix. What the automation proves is listed
per case in the matrix itself, so a green CI run is never counted as a passed UAT case.

## Not in this directory

- `test-reports/` does not exist. Reports belong to QA and are added when a run happens.
- The `sdd-gen` commands that older revisions of this file mentioned belong to a
  different tool and do not exist in this repository.
- `openspec/` and `ROADMAP.md` live in the working tree but are gitignored, so a clone
  does not carry them.
