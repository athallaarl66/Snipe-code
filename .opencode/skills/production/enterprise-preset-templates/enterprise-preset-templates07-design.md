# US-007: Template Blog/Posts — Design Specifications

## Page Structure
```
frontend/src/
├── pages/
├── posts/              # MDX content files
├── categories/
├── tags/
├── components/
│   ├── blog/
│   │   ├── post-card/
│   │   ├── post-content/
│   │   └── author-page/
│   ├── ui/
│   └── layout/
└── styles/
```

## Visual Treatment
- Post list: card layout with title, excerpt, date, tags
- Post content: clean typography, code blocks, images
- Sidebar: categories, recent posts, author info
- Author page: bio, avatar, social links, post list

## Responsive
- Desktop: content + sidebar
- Tablet: content + collapsible sidebar
- Mobile: single column, hamburger nav
