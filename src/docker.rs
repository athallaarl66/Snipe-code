use std::fs;
use std::path::Path;

pub fn generate_docker_compose(project_path: &str, database: &str, backend: &str) -> Result<(), String> {
    if database == "none" && backend == "none" {
        return Ok(());
    }

    let path = Path::new(project_path);
    let content = docker_compose_content(database, backend);
    fs::write(path.join("docker-compose.yml"), content).map_err(|e| e.to_string())?;

    Ok(())
}

fn docker_compose_content(database: &str, backend: &str) -> String {
    let mut lines = Vec::new();
    lines.push("version: '3.8'".to_string());
    lines.push("".to_string());
    lines.push("services:".to_string());

    // Database service
    match database {
        "postgresql" => {
            lines.push("  postgres:".to_string());
            lines.push("    image: postgres:16-alpine".to_string());
            lines.push("    environment:".to_string());
            lines.push("      POSTGRES_USER: user".to_string());
            lines.push("      POSTGRES_PASSWORD: password".to_string());
            lines.push("      POSTGRES_DB: mydb".to_string());
            lines.push("    ports:".to_string());
            lines.push("      - '5432:5432'".to_string());
            lines.push("    volumes:".to_string());
            lines.push("      - postgres_data:/var/lib/postgresql/data".to_string());
        }
        "mysql" => {
            lines.push("  mysql:".to_string());
            lines.push("    image: mysql:8.0".to_string());
            lines.push("    environment:".to_string());
            lines.push("      MYSQL_ROOT_PASSWORD: rootpassword".to_string());
            lines.push("      MYSQL_DATABASE: mydb".to_string());
            lines.push("      MYSQL_USER: user".to_string());
            lines.push("      MYSQL_PASSWORD: password".to_string());
            lines.push("    ports:".to_string());
            lines.push("      - '3306:3306'".to_string());
            lines.push("    volumes:".to_string());
            lines.push("      - mysql_data:/var/lib/mysql".to_string());
        }
        _ => {}
    }

    // Backend service (optional)
    if backend != "none" {
        lines.push("".to_string());
        lines.push("  backend:".to_string());
        lines.push("    build: ./backend".to_string());
        lines.push("    ports:".to_string());
        lines.push("      - '3000:3000'".to_string());
        lines.push("    depends_on:".to_string());

        match database {
            "postgresql" => {
                lines.push("      - postgres".to_string());
            }
            "mysql" => {
                lines.push("      - mysql".to_string());
            }
            _ => {}
        }

        lines.push("    environment:".to_string());
        match database {
            "postgresql" => {
                lines.push("      DATABASE_URL: postgresql://user:password@postgres:5432/mydb".to_string());
            }
            "mysql" => {
                lines.push("      DATABASE_URL: mysql://user:password@mysql:3306/mydb".to_string());
            }
            _ => {}
        }
    }

    // Volumes
    if database != "none" {
        lines.push("".to_string());
        lines.push("volumes:".to_string());
        match database {
            "postgresql" => {
                lines.push("  postgres_data:".to_string());
            }
            "mysql" => {
                lines.push("  mysql_data:".to_string());
            }
            _ => {}
        }
    }

    lines.join("\n")
}
