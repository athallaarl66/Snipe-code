# US-002: Init Tanpa Template — Technical Specifications

## CLI Commands
```bash
snipe-code new --template none   # explicit no-template
snipe-code new                   # default is "no template"
```

## Generated Structure (No Template + Next.js + NestJS)
```
project/
├── frontend/src/
│   ├── components/ui/
│   ├── components/layout/
│   ├── pages/
│   ├── hooks/
│   └── lib/
├── backend/src/
│   ├── controllers/
│   ├── services/
│   ├── repositories/
│   ├── middleware/
│   └── routes/
├── .env.example
├── .gitignore
└── README.md
```

## Business Rules
| Rule | Condition | Action |
|------|-----------|--------|
| BR-004 | Frontend = None | Skip frontend folder |
| BR-005 | Backend = None | Skip backend folder |
