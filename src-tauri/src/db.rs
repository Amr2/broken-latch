use tauri_plugin_sql::{Migration, MigrationKind};

pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "initial_schema",
        sql: include_str!("../migrations/001_initial.sql"),
        kind: MigrationKind::Up,
    }]
}

pub async fn init_db(_app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Database is auto-initialized by tauri-plugin-sql with migrations
    // This function can be used for additional setup if needed
    println!("Database initialized with migrations");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_exist() {
        let migrations = get_migrations();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "initial_schema");
    }

    #[test]
    fn test_migration_sql_not_empty() {
        let migrations = get_migrations();
        assert!(!migrations[0].sql.is_empty());
        assert!(migrations[0].sql.contains("CREATE TABLE"));
    }
}
