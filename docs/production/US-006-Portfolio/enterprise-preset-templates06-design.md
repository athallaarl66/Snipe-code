# US-006: Template Portfolio — Design Specifications

## Page Structure
```
frontend/src/
├── pages/
├── components/
│   ├── sections/
│   │   ├── hero/
│   │   ├── projects-grid/
│   │   ├── skills/
│   │   ├── experience/
│   │   └── contact/
│   ├── ui/
│   └── layout/
├── data/
│   └── projects.json
└── styles/
```

## Visual Treatment
- Hero: full-screen with name + tagline + CTA
- Projects: grid layout with image + title + description
- Skills: progress bars or tag chips
- Experience: timeline layout
- Contact: simple form + social links

## Dark/Light Mode
- Toggle button in header
- Default: system preference
- Persisted in localStorage

## Responsive
- Desktop: full grid
- Tablet: 2-column grid
- Mobile: single column, stacked sections
