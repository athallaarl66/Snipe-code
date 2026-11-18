use std::fs;
use std::path::Path;

pub fn generate_docker_compose(
    project_path: &str,
    database: &str,
    backend: &str,
) -> Result<(), String> {
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
    lines.push("services:".to_string());

    // Database service
    match database {
        "postgresql" => {
            lines.push("  postgres:".to_string());
            lines.push("    image: postgres:16-alpine".to_string());
            lines.push("    environment:".to_string());
            lines.push("      POSTGRES_USER: ${POSTGRES_USER:-snipe_local}".to_string());
            lines.push(
                "      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-CHANGE_ME_LOCAL_DB_PASSWORD}"
                    .to_string(),
            );
            lines.push("      POSTGRES_DB: ${POSTGRES_DB:-snipe_local}".to_string());
            lines.push("    healthcheck:".to_string());
            lines.push("      test: [\"CMD-SHELL\", \"pg_isready -U \\\"$$POSTGRES_USER\\\" -d \\\"$$POSTGRES_DB\\\"\"]".to_string());
            lines.push("      interval: 10s".to_string());
            lines.push("      timeout: 5s".to_string());
            lines.push("      retries: 5".to_string());
            lines.push("    ports:".to_string());
            lines.push("      - '5432:5432'".to_string());
            lines.push("    volumes:".to_string());
            lines.push("      - postgres_data:/var/lib/postgresql/data".to_string());
        }
        "mysql" => {
            lines.push("  mysql:".to_string());
            lines.push("    image: mysql:8.0".to_string());
            lines.push("    environment:".to_string());
            lines.push(
                "      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD:-CHANGE_ME_LOCAL}".to_string(),
            );
            lines.push("      MYSQL_DATABASE: ${MYSQL_DATABASE:-snipe_local}".to_string());
            lines.push("      MYSQL_USER: ${MYSQL_USER:-snipe_local}".to_string());
            lines.push(
                "      MYSQL_PASSWORD: ${MYSQL_PASSWORD:-CHANGE_ME_LOCAL_DB_PASSWORD}".to_string(),
            );
            lines.push("    healthcheck:".to_string());
            lines.push("      test: [\"CMD-SHELL\", \"MYSQL_PWD=\\\"$$MYSQL_PASSWORD\\\" mysql --protocol=tcp -h 127.0.0.1 -u \\\"$$MYSQL_USER\\\" \\\"$$MYSQL_DATABASE\\\" -e 'SELECT 1'\"]".to_string());
            lines.push("      interval: 10s".to_string());
            lines.push("      timeout: 5s".to_string());
            lines.push("      retries: 5".to_string());
            lines.push("    ports:".to_string());
            lines.push("      - '3306:3306'".to_string());
            lines.push("    volumes:".to_string());
            lines.push("      - mysql_data:/var/lib/mysql".to_string());
        }
        _ => {}
    }

    // Backend service (optional)
    // Only backends with a generated Dockerfile are containerized.
    if backend == "nestjs" {
        lines.push("".to_string());
        lines.push("  backend:".to_string());
        lines.push("    build: ./backend".to_string());
        lines.push("    ports:".to_string());
        lines.push("      - '3000:3000'".to_string());
        match database {
            "postgresql" => {
                lines.push("    depends_on:".to_string());
                lines.push("      postgres:".to_string());
                lines.push("        condition: service_healthy".to_string());
                lines.push("    environment:".to_string());
                lines.push("      DATABASE_URL: postgresql://${POSTGRES_USER:-snipe_local}:${POSTGRES_PASSWORD:-CHANGE_ME_LOCAL_DB_PASSWORD}@postgres:5432/${POSTGRES_DB:-snipe_local}".to_string());
            }
            "mysql" => {
                lines.push("    depends_on:".to_string());
                lines.push("      mysql:".to_string());
                lines.push("        condition: service_healthy".to_string());
                lines.push("    environment:".to_string());
                lines.push("      DATABASE_URL: mysql://${MYSQL_USER:-snipe_local}:${MYSQL_PASSWORD:-CHANGE_ME_LOCAL_DB_PASSWORD}@mysql:3306/${MYSQL_DATABASE:-snipe_local}".to_string());
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
