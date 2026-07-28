# US-001: Init dengan Template — Design Specifications

## UI Type
Terminal / CLI — no GUI

## Screen Layout
```
┌─────────────────────────────────────────────────┐
│  snipe-code new                                 │
├─────────────────────────────────────────────────┤
│                                                 │
│  ? Pilih Template:                              │
│    ❯ No Template — clean start (recommended)    │
│      📊 Industrial IoT Dashboard                │
│      🏢 Company Profile                         │
│      💼 Portfolio                               │
│      📝 Blog / Posts                            │
│                                                 │
│  [↑↓] Navigate   [Enter] Select   [Ctrl+C] Exit │
└─────────────────────────────────────────────────┘
```

## Components
- `dialoguer::Select` — arrow key navigation
- `console::Term` — terminal output
- Colored output: green success, red error, yellow warning

## Interaction
- Arrow key ↑↓: navigate
- Enter: select
- Ctrl+C: cancel (clean exit, no partial files)

## Responsive
N/A — terminal-based

## Accessibility
- Colorblind-friendly: icons (📊, 🏢) alongside colors
- Screen reader: plain text fallback with `--quiet` flag
