use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

fn compose(database: &str, backend: &str) -> String {
    let dir = std::env::var_os("TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "snipe-compose-config-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir(&dir).unwrap();
    snipe_code::docker::generate_docker_compose(&dir.to_string_lossy(), database, backend).unwrap();
    let content = fs::read_to_string(dir.join("docker-compose.yml")).unwrap();
    fs::remove_dir_all(dir).unwrap();
    content
}

fn value<'a>(section: &'a str, key: &str) -> &'a str {
    let prefix = format!("      {key}: ");
    section
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing {key} in {section}"))
}

#[test]
fn compose_database_identity_and_healthcheck_match_backend() {
    for (database, service, scheme, port, user, password, name, probe) in [
        (
            "postgresql",
            "postgres",
            "postgresql",
            "5432",
            "POSTGRES_USER",
            "POSTGRES_PASSWORD",
            "POSTGRES_DB",
            "pg_isready",
        ),
        (
            "mysql",
            "mysql",
            "mysql",
            "3306",
            "MYSQL_USER",
            "MYSQL_PASSWORD",
            "MYSQL_DATABASE",
            "mysql --protocol=tcp",
        ),
    ] {
        let content = compose(database, "nestjs");
        assert!(!content.lines().any(|line| line.starts_with("version:")));
        let (db, backend) = content.split_once("\n  backend:\n").unwrap();
        let url = value(backend, "DATABASE_URL");
        assert_eq!(
            url,
            format!(
                "{scheme}://{}:{}@{service}:{port}/{}",
                value(db, user),
                value(db, password),
                value(db, name)
            )
        );
        assert!(db.contains("    healthcheck:\n"));
        assert!(db.contains(probe));
        for key in [user, name] {
            assert!(db.contains(&format!("$${key}")));
        }
        if database == "mysql" {
            assert!(db.contains("$$MYSQL_PASSWORD"));
        }
        assert!(backend.contains(&format!(
            "    depends_on:\n      {service}:\n        condition: service_healthy"
        )));
        let db_only = compose(database, "none");
        assert!(db_only.contains("    healthcheck:\n"));
        assert!(!db_only.contains("  backend:"));
    }
}

#[test]
fn compose_mysql_defaults_match_laravel_environment_example() {
    let content = compose("mysql", "laravel");
    let example = include_str!("../templates/backends/laravel/files/.env.example");
    for (compose_key, backend_key) in [
        ("MYSQL_DATABASE", "DB_DATABASE"),
        ("MYSQL_USER", "DB_USERNAME"),
        ("MYSQL_PASSWORD", "DB_PASSWORD"),
    ] {
        let default = value(&content, compose_key)
            .split_once(":-")
            .unwrap()
            .1
            .strip_suffix('}')
            .unwrap();
        assert!(
            example
                .lines()
                .any(|line| line == format!("{backend_key}={default}"))
        );
    }
}

#[test]
fn compose_backend_without_database_has_no_empty_database_configuration() {
    let content = compose("none", "nestjs");
    assert!(content.contains("  backend:\n"));
    assert!(!content.contains("    depends_on:"));
    assert!(!content.contains("    environment:"));
    assert!(!content.lines().any(|line| line.starts_with("version:")));
}
