# US-006: Template Portfolio — Technical Specifications

## Generated Folders
```
frontend/src/components/sections/hero/
frontend/src/components/sections/projects-grid/
frontend/src/components/sections/skills/
frontend/src/components/sections/experience/
frontend/src/components/sections/contact/
frontend/src/data/projects.json
```

## Projects Data Format
```json
{
  "projects": [
    {
      "title": "Project Name",
      "description": "Brief description",
      "image": "/images/project.png",
      "tags": ["React", "TypeScript"],
      "link": "https://github.com/..."
    }
  ]
}
```

## Deploy Target
- Vercel: `vercel deploy`
- Netlify: `netlify deploy`
