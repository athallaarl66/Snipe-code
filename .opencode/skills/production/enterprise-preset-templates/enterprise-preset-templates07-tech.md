# US-007: Template Blog/Posts — Technical Specifications

## Generated Folders
```
frontend/src/posts/
frontend/src/categories/
frontend/src/tags/
frontend/src/components/blog/post-card/
frontend/src/components/blog/post-content/
frontend/src/components/blog/author-page/
```

## Content Format
```markdown
---
title: "My First Post"
date: "2026-07-28"
tags: ["rust", "cli"]
category: "tutorial"
author: "SNIPE-CODE Team"
---

Content goes here...
```

## RSS Feed
- Auto-generated from posts
- Location: `/public/rss.xml` or `/feed.xml`
- Standard RSS 2.0 format

## Backend
Optional — bisa static generation (SSG) atau pake headless CMS (Strapi, Sanity)
