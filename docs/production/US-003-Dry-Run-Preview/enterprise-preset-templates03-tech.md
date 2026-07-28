# US-003: Dry-Run Preview — Technical Specifications

## CLI Commands
```bash
snipe-code new --dry-run                    # preview only
snipe-code new --template iot --dry-run     # preview specific template
```

## Implementation
```rust
fn dry_run_preview(template_id: &str, stack: &StackConfig) -> String {
    // Build tree in-memory, return as string
    // No fs operations
    let tree = build_folder_tree(template_id, stack);
    format_tree_display(&tree)
}
```

## Performance
- Target: < 100ms
- No disk I/O, all in-memory
