# SNIPE-CODE QA planning

Use this skill to prepare reviewable test cases for this project, not to execute them. These project-local instructions reuse the deterministic CLI; they are not a new engine or permission grant. This workflow takes precedence over generic instructions to run tests or audit while planning.

## Read the selected requirements

1. Confirm the project path and user-selected requirements file inside that project. Read only this selected scope. If no file is selected or requirements are ambiguous, ask the user; do not invent requirements, endpoints, methods, assertions, or expected results.
2. Input is JSON version 1: `requirements` contains unique requirement `id` values and `scenarios`. Every requirement needs positive and negative scenarios. Every scenario needs a unique `id` within its requirement, `category`, `objective`, `precondition`, `input`, and `expected_result`. Preserve IDs and requirement/source traceability. Additional supported categories are `boundary`, `authorization`, `validation`, and `regression`.
3. Use only synthetic, non-secret data. Do not put credentials, tokens, or personal data in input, output, or examples. Missing input is a blocker, not permission to edit project files or create requirements automatically.

Example selected file (already supplied by the user, not a file to create during planning):

```json
{
  "version": 1,
  "requirements": [
    {
      "id": "REQ-ACCESS",
      "scenarios": [
        {
          "id": "allowed",
          "category": "positive",
          "objective": "Allow a synthetic permitted request",
          "precondition": "Synthetic fixture describes permitted access",
          "input": "Read the allowed fixture",
          "expected_result": "HTTP 200",
          "request": {
            "method": "GET",
            "url": "http://127.0.0.1:9/allowed",
            "headers": {},
            "body": null,
            "expected_status": 200
          }
        },
        {
          "id": "denied",
          "category": "negative",
          "objective": "Reject a synthetic forbidden request",
          "precondition": "Synthetic fixture describes forbidden access",
          "input": "Read the forbidden fixture",
          "expected_result": "HTTP 403",
          "request": {
            "method": "GET",
            "url": "http://127.0.0.1:9/denied",
            "headers": {},
            "body": null,
            "expected_status": 403
          }
        }
      ]
    }
  ]
}
```

The URLs above are data for planning, not endpoints to contact.

## Run only the read-only planner

Replace placeholders with the selected paths. Paths are relative to the current directory, not implicitly relative to the project; absolute paths avoid ambiguity.

```text
snipe-code qa plan "<PROJECT>" --input "<SELECTED_FILE>"
```

The CLI returns deterministic JSON on stdout. Basic planning does not edit project files, does not run target tests, does not install tools, and does not access the network. It does not need AI/MCP, dependency setup, application startup, or a database connection. Cases start as `draft`; planning success is not evidence of test execution.

Only when the user requests a Postman draft, append `--adapter postman-cli`. Each scenario then requires structured `request` fields `method`, `url`, `headers`, `body`, `expected_status`: an explicit supported uppercase HTTP method, absolute HTTP(S) URL, header map, nullable raw-string body, and numeric status 100-599. Do not infer them from prose. Credential headers, URL userinfo, and URL templates are rejected by this initial adapter. Requests alone do not select an adapter; script generation requires `--adapter postman-cli`.

Output contains Collection v2.1 under `script.collection`, not a bare collection at the JSON root. Postman CLI target is 1.90.0, runtime unverified. Static validation means `ready-for-review` is not `passed`, executed, or runtime-compatible. Do not install Postman or validate by executing the collection. Other adapters, including unit/regression frameworks, Playwright, Bruno, and Maestro, remain `unsupported` in this slice.

Only on an explicit user request to export, append `--export "<OUTSIDE_PROJECT_NEW_FILE>"` using the exact approved new file outside the project. Default is stdout only. Do not redirect output into the project or overwrite an existing file. Report CLI errors honestly; do not replace rejected input or failed export with fabricated results.

## Approval and unavailable execution

Treat install, test execution, endpoint access, and data transmission as separate actions. Before any such action can be considered, show command/action, working directory, exact target, environment names, data, output, timeout, install effects, network permissions, and access limits. Require separate explicit USER approval bound to that exact scope; setup approval is not run approval. The agent cannot approve on behalf of the user; changed scope requires new approval. Missing approval, unknown scope, or no user approval channel means `blocked`. This applies to local AI/MCP as well as online services; local does not mean exempt or safe. Redact secrets in previews. AI/MCP runtime is deferred, not an available planning feature.

`qa run` and `qa setup` are not available. The fake-runner policy library only tests approval/preflight rules; it is not a working execution path. Do not bypass this limit using direct Postman, shell scripts, package-manager commands, or `snipe-code verify`. Even user approval does not make an unavailable execution path available. Stop at the plan and report execution as NOT RUN.

Future host execution is not a sandbox. A temporary workspace does not isolate filesystem, processes, network, endpoints, or hardcoded database targets. Explicit remote, production-like, or ambiguous database configuration must remain blocked before spawn, not overridden by approval. An undetected database is unknown, not proven safe or absent. Do not claim OS parity, sandbox support, human UAT, or runtime compatibility without evidence for the specific platform and boundary.

## Handoff

Return selected scope, requirement-to-case references, CLI command, actual planning/export result, adapter/version, draft or review status, and missing evidence. Keep planned, statically validated, executed, and passed distinct. No install, target test, endpoint access, or data transmission is implied by a generated skill or successful plan.
