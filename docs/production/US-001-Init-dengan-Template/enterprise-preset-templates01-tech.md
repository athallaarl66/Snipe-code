# US-001: Init dengan Template — Technical Specifications

## CLI Commands
```bash
snipe-code new                          # interactive mode
snipe-code new --template iot-dashboard # skip template prompt
snipe-code new --quiet                  # use defaults
```

## Dependencies
- `dialoguer` — interactive prompts
- `serde` + `serde_json` — template parsing
- `std::fs` — file/folder creation
- `indicatif` — progress spinner

## Data Structure
```rust
struct Template {
    id: String,
    name: String,
    description: String,
    icon: String,
    structure: TemplateStructure,
}
```

## Error Handling
| Error | Message |
|-------|---------|
| Target exists | "Folder 'X' already exists. Overwrite? (y/n)" |
| No template selected | "Select at least one template" |
| Invalid template ID | "Template 'X' not found" |

## Performance
- Template load: < 50ms (embedded at compile-time)
- Folder creation: < 100ms (parallel `create_dir_all`)
