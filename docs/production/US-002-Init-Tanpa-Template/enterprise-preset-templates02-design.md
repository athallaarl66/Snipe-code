# US-002: Init Tanpa Template — Design Specifications

## UI Type
Terminal / CLI

## Screen Layout
```
┌─────────────────────────────────────────────────┐
│  snipe-code new                                 │
├─────────────────────────────────────────────────┤
│                                                 │
│  ? Pilih Template:                              │
│    ❯ No Template — clean start (recommended)    │  ← highlighted/default
│      📊 Industrial IoT Dashboard                │
│      ...                                        │
│                                                 │
│  [↑↓] Navigate   [Enter] Select                 │
└─────────────────────────────────────────────────┘
```

## Visual Treatment
- "No Template" option: bold/green text, "(recommended)" tag
- Always first in list
- Same interaction as other template options
