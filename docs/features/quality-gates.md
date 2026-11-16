# Quality gates

The `quality` command evaluates versioned metric policies against a local scan of a project and reports a gate outcome. It is read-only analysis: it does not install dependencies, does not run the project's tests or build commands, and does not contact any remote service, including pull request APIs. Test coverage is imported from an LCOV report the caller supplies; the static scan does not compute coverage.

Config and metric definitions below use version 1. Coverage and baseline inputs can be unavailable even when source metrics evaluate successfully.

## Command

```text
snipe-code quality [PATH] --config <FILE> [--coverage <LCOV>] [--baseline <SCAN_JSON> --base-ref <REF>] [--json] [--save]
```

PATH defaults to the current directory. The command scans PATH first, then evaluates the config policies against that scan.

- `--config <FILE>`: required. Duplicate flags, unknown options, `--baseline` without `--base-ref` or the reverse, and unreadable, oversized, or invalid config exit 2 before anything is scanned.
- `--coverage <LCOV>`: imports line coverage from an LCOV report. Without it a coverage metric is unknown, not zero. A missing or invalid report also leaves the metric unknown; it is not a usage error.
- `--baseline <SCAN_JSON> --base-ref <REF>`: supplied together. The baseline is a saved scan JSON file, the ref is a Git reference such as a commit, tag, or branch name. An unreadable or invalid baseline file leaves the delta unknown, not a usage error.
- `--json`: prints the full scan envelope with the quality report at `result.quality`. Text output prints the aggregate state, one summary per metric, and a baseline reason when comparison is unavailable. Both outputs use pass, fail, unknown, and not-applicable state names.
- `--save`: stores the run in the local history database, the same store `scan --save` uses. Without it the command writes nothing. A JSON file only appears when you redirect `--json` output yourself.

## Configuration

Config is JSON with all fields mandatory; unknown fields are rejected. The file is a regular UTF-8 file of at most 1 MiB.

```json
{
  "version": 1,
  "enabled": true,
  "metrics": [
    {
      "id": "rust-function-lines",
      "version": 1,
      "unit": "lines",
      "required": true,
      "comparison": "maximum",
      "threshold": 100,
      "unknown": "unknown",
      "exclusions": ["src/generated"]
    },
    {
      "id": "rust-duplicate-blocks",
      "version": 1,
      "unit": "blocks",
      "required": true,
      "comparison": "maximum",
      "threshold": 20,
      "unknown": "fail",
      "exclusions": []
    },
    {
      "id": "lcov-line-coverage",
      "version": 1,
      "unit": "basis-points",
      "required": false,
      "comparison": "minimum",
      "threshold": 6000,
      "unknown": "unknown",
      "exclusions": []
    },
    {
      "id": "new-findings",
      "version": 1,
      "unit": "findings",
      "required": false,
      "comparison": "maximum",
      "threshold": 0,
      "unknown": "unknown",
      "exclusions": []
    }
  ]
}
```

- `version` is 1 for the config and for every metric entry.
- Between 1 and 4 metrics, each id used once.
- Each id has a fixed unit and comparison direction; values that disagree with the id are rejected.
- `threshold` is inclusive. A maximum metric passes when value is less than or equal to the threshold; a minimum metric passes when value is greater than or equal to it. The coverage threshold is capped at 10000.
- `unknown` is `unknown` or `fail` and decides what a missing value does.
- `required` is recorded in the policy and in the report. It does not by itself change the outcome, because an unknown metric is already non-pass.
- `exclusions` holds up to 128 relative paths with no leading or trailing slash and no traversal segments. A path is excluded when it equals an entry or sits below it as a directory prefix.
- `enabled: false` disables the whole gate: every metric reports not-applicable with no values, and the command exits 0.

## Metrics

| id | unit | comparison | measures |
| --- | --- | --- | --- |
| `rust-function-lines` | lines | maximum | longest function body in the scanned Rust source |
| `rust-duplicate-blocks` | blocks | maximum | additional exact duplicate 50-token blocks |
| `lcov-line-coverage` | basis-points | minimum | line coverage imported from an LCOV report, 10000 means 100 percent |
| `new-findings` | findings | maximum | findings in the current scan absent from the baseline |

### rust-function-lines v1

The value is the maximum physical line span of a function body, counting the first and last line, so a body on one line counts as 1. Measured functions are free functions, inherent impl functions, and trait functions that carry a default body, including nested ones. The reported number is the maximum span, not an average.

Only Rust source is measured. A file with a recognized non-Rust source extension (js, jsx, ts, tsx, vue, php, go, java, cs, py, c, cpp, rb, swift, kt) makes the whole metric unknown with that reason; unsupported languages are not skipped silently. Item macros, external module declarations (`mod foo;` without a body), and closures also make the metric unknown, because measuring function bodies behind macro expansion or in closures is outside this slice. An empty corpus, an unparseable file, or a budget overrun is unknown, not zero.

Budgets: at most 4096 files after walking, 262144 bytes per file, and delimiter nesting depth 64 before parsing.

### rust-duplicate-blocks v1

Exact token sequence matching. Tokenization drops comments and whitespace and keeps identifiers, literals, punctuation with its spacing, and group delimiters (each delimiter is its own token). No alpha renaming, no semantic equivalence: two blocks match only when all 50 tokens are equal.

The algorithm: files are processed in lexical path order. Within each file, blocks are fixed 50-token windows starting at offset zero: tokens 0 through 49, 50 through 99, and so on. The first window whose content has not been seen is accepted and recorded; any later window whose 50 tokens exactly equal an accepted block adds 1 to the count, including a repeat inside the same file. Windows do not overlap: after a window is accepted or counted, the next window starts 50 tokens later. A final window shorter than 50 tokens is ignored and can never match.

The value is the number of additional duplicate blocks, not the number of distinct duplicated contents. The same budgets apply, plus a total budget of 100000 tokens across the corpus; exceeding it makes the metric unknown.

### lcov-line-coverage v1

Imported, not computed. The value is covered lines times 10000 divided by total lines, rounded down. The metric records provenance: the format string, the SHA-256 of the report text, and the covered and total line counts. Provenance identifies the input file; it is not a claim that tests ran during this command or that the coverage is fresh for the current working tree.

Accepted records: complete `SF`, `DA`, `end_of_record` groups per file; `TN`; `FN`, `FNDA`, `BRDA` with non-empty values; and the summary fields `LF`, `LH`, `FNF`, `FNH`, `BRF`, `BRH`, each at most once per file. `LF` and `LH` must agree with the counts derived from the `DA` records. `DA` carries 2 or 3 comma-separated fields, the line number is nonzero, and a line appears at most once per file. Anything else in the report is rejected.

Path and size constraints: `SF` resolves to an existing regular file inside the canonical scan root. Absolute paths sit under the root; relative paths are root-relative. Symlinks and Windows reparse points anywhere on the resolved path are rejected, as are traversal segments. Every `DA` line number exists in the source file on disk, and each mapped source is read within the input budget. The report is a regular UTF-8 file of at most 1 MiB, at most 100000 record lines, and at most 4096 file records; duplicate file records are rejected. Excluded paths are left out of both the counts and the file list.

A coverage policy with no report, or with a report failing any check above, is unknown with the failure reason, or fail when the policy sets `unknown` to `fail`. It is never reported as zero.

### new-findings v1

Needs `--baseline` and `--base-ref`. The value counts findings in the current scan, after policy exclusions, whose fingerprints are absent from the baseline. It is evaluated only when the delta comparison reached pass state; an unknown delta leaves the metric without a value. The delta carries the specific comparison reason.

## Baseline and delta

The baseline file is read-only input; it is never rewritten. It accepts the current versioned scan envelope and legacy v1 audit JSON.

Comparison runs only after compatibility is proven: equal schema version, project id, rule version, exclusions, evaluated rule groups, detected scopes, and the same set of SAST language and rule-pack versions, plus every finding in both scans carrying fingerprint version 1 with one consistent rule version per rule id. Both scans also need a complete evaluated scope: every rule group and every SAST report evaluated or not-applicable. A baseline from an earlier scan whose scope stayed unknown does not become complete by being selected as a baseline; the comparison stays unknown.

Git checks are local and read-only, run with `GIT_OPTIONAL_LOCKS=0`, no pager, and prompts disabled:

- the ref resolves through `git rev-parse --verify` to exactly one commit, 40 or 64 hex characters;
- `git ls-files --unmerged` shows no merge conflict entries;
- `git diff --name-status --find-renames` between that commit and the working tree shows no rename, copy, or unmerged status;
- a deleted path combined with untracked files is treated as a possible unstaged rename and stops the comparison.

Git output is capped at 1 MiB; Git must be installed and the checks run in the scanned root.

Any failure produces delta state unknown with empty new, unchanged, and resolved lists and a reason string. Ambiguity is reported, not guessed away.

The resolved base commit is recorded in the report. The binding between the baseline file you passed and that commit is caller-supplied provenance: the tool records what you supplied and the commit the ref resolved to, does not cryptographically tie the baseline file to the commit, and queries no remote pull request.

## Outcomes and exit codes

Each metric and the aggregate report one of four states: pass, fail, unknown, not-applicable. Aggregation precedence: any metric fail makes the gate fail even when other metrics are unknown; otherwise any unknown makes the gate unknown; otherwise at least one pass makes the gate pass; with no evaluated metric the gate is not-applicable. Unknown is not promoted to pass.

| exit code | condition |
| --- | --- |
| 0 | aggregate pass, or the config is disabled |
| 1 | a metric failed its threshold, or a missing metric had `unknown` set to `fail`; takes precedence over unknown |
| 2 | usage error or invalid config, before any scan |
| 3 | runtime error, or no metric produced a value |
| 4 | at least one metric produced a value and at least one is unknown |

These states are separate from `snipe-code verify` execution states (passed, failed, unavailable, blocked, not-applicable). `verify` runs the project's own commands and those commands write into the project; `quality` reads source files and imported reports only.

## Output and history

`--json` prints the scan envelope with the quality report at `result.quality`. Envelopes without a quality report still read, so older saved scans stay usable. On save and on read the report is validated: a report whose aggregate state disagrees with its metrics is rejected.

`--save` writes the envelope to the local history database under the same data directory as `scan --save` (`SNIPE_CODE_DATA_DIR` overrides it) and prints the saved run id to stderr. `history show` and `history export` round-trip the saved envelope unchanged.
