#[derive(Debug, PartialEq)]
pub struct StackOption {
    pub id: &'static str,
    pub name: &'static str,
    pub folders: &'static [&'static str],
}

pub const FRONTEND_STACKS: &[StackOption] = &[
    StackOption { id: "nextjs", name: "Next.js", folders: &["src/components/ui/", "src/components/layout/", "src/pages/", "src/hooks/", "src/lib/"] },
    StackOption { id: "react-vite", name: "React (Vite)", folders: &["src/components/ui/", "src/components/layout/", "src/pages/", "src/hooks/", "src/lib/"] },
    StackOption { id: "vue", name: "Vue", folders: &["src/components/ui/", "src/components/layout/", "src/views/", "src/composables/", "src/lib/"] },
    StackOption { id: "nuxtjs", name: "Nuxt.js", folders: &["src/components/ui/", "src/components/layout/", "src/pages/", "src/composables/", "src/lib/"] },
    StackOption { id: "none", name: "None", folders: &[] },
];

pub const BACKEND_STACKS: &[StackOption] = &[
    StackOption { id: "dotnet8", name: ".NET 8", folders: &["src/Controllers/", "src/Services/", "src/Repositories/", "src/Middleware/", "src/Models/"] },
    StackOption { id: "nestjs", name: "NestJS", folders: &["src/controllers/", "src/services/", "src/repositories/", "src/middleware/", "src/routes/"] },
    StackOption { id: "go-gin", name: "Go (Gin)", folders: &["src/controllers/", "src/services/", "src/repositories/", "src/middleware/", "src/routes/"] },
    StackOption { id: "spring-boot", name: "Spring Boot", folders: &["src/main/java/controllers/", "src/main/java/services/", "src/main/java/repositories/", "src/main/java/middleware/", "src/main/java/models/"] },
    StackOption { id: "laravel", name: "Laravel", folders: &["app/Http/Controllers/", "app/Services/", "app/Repositories/", "app/Http/Middleware/", "routes/"] },
    StackOption { id: "none", name: "None", folders: &[] },
];

pub const DATABASE_STACKS: &[StackOption] = &[
    StackOption { id: "postgresql", name: "PostgreSQL", folders: &[] },
    StackOption { id: "mysql", name: "MySQL", folders: &[] },
    StackOption { id: "none", name: "None", folders: &[] },
];

pub fn get_frontend_folders(id: &str) -> Option<&'static [&'static str]> {
    FRONTEND_STACKS.iter().find(|s| s.id == id).map(|s| s.folders)
}

pub fn get_backend_folders(id: &str) -> Option<&'static [&'static str]> {
    BACKEND_STACKS.iter().find(|s| s.id == id).map(|s| s.folders)
}

pub fn validate_selection(frontend: &str, backend: &str) -> Result<(), &'static str> {
    if frontend == "none" && backend == "none" {
        return Err("Pilih minimal Frontend atau Backend");
    }
    Ok(())
}
