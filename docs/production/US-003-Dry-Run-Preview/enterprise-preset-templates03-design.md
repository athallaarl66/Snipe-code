# US-003: Dry-Run Preview — Design Specifications

## Screen Layout
```
$ snipe-code new --dry-run

? Pilih Template: No Template
? Pilih Frontend: Next.js
? Pilih Backend: NestJS

[DRY RUN] Would create:

my-project/
├── frontend/
│   └── src/
│       ├── components/
│       ├── pages/
│       └── hooks/
├── backend/
│   └── src/
│       ├── controllers/
│       ├── services/
│       └── repositories/
├── .env.example
├── .gitignore
└── README.md

(0 files created — dry run mode)
```

## Visual Treatment
- "[DRY RUN]" prefix in yellow
- Folder tree with tree characters (├──, └──)
- Final line: "(0 files created — dry run mode)"
