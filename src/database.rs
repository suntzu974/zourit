use libsql::{Connection, Database as LibSqlDatabase};
use std::sync::Arc;
use std::fs;
use std::path::Path;

pub type SharedConnection = Arc<Connection>;
pub type DbResult<T> = Result<T, libsql::Error>;

/// Database connection manager for LibSQL.
/// 
/// Configuration modes:
/// - **Remote only**: Set TURSO_DATABASE_URL + TURSO_AUTH_TOKEN (production)
///   - Direct connection to Turso cloud
///   - All operations go through network
///   - No local storage required
/// 
/// - **Local only**: Leave TURSO_* unset (development)
///   - Uses local SQLite file specified by DATABASE_PATH
///   - Offline development friendly
pub struct Database;

impl Database {
    pub async fn connect(database_path: &str) -> DbResult<(LibSqlDatabase, Connection)> {
        // Check if remote URL is configured
        let remote_url = std::env::var("TURSO_DATABASE_URL").ok();
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();
        
        match (remote_url, auth_token) {
            (Some(url), Some(token)) if !url.is_empty() && !token.is_empty() => {
                // Direct remote connection to Turso (no local replica)
                println!("[database] Connecting to remote Turso database: '{}'", url);
                let db = libsql::Builder::new_remote(url, token)
                    .build()
                    .await?;
                
                let conn = db.connect()?;
                println!("[database] Connected successfully to Turso");
                
                Ok((db, conn))
            }
            _ => {
                // Local-only connection (fallback for development)
                println!("[database] Connecting to local database: '{}'", database_path);
                let db = libsql::Builder::new_local(database_path).build().await?;
                let conn = db.connect()?;
                Ok((db, conn))
            }
        }
    }

    pub async fn create_shared_connection(database_path: &str) -> DbResult<SharedConnection> {
        let (_db, conn) = Self::connect(database_path).await?;
        Self::create_tables(&conn).await?;
        Ok(Arc::new(conn))
    }

    pub async fn create_tables(conn: &Connection) -> DbResult<()> {
        // legacy direct creation (kept for backward compatibility) now handled by migrations
        Self::apply_migrations(conn).await?;
        Ok(())
    }

    async fn apply_migrations(conn: &Connection) -> DbResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            (),
        ).await?;

        let migrations_dir = std::env::var("MIGRATIONS_PATH").unwrap_or_else(|_| "migrations".to_string());
        let path = Path::new(&migrations_dir);
        if !path.exists() { return Ok(()); }

        let mut entries: Vec<_> = match fs::read_dir(path) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(err) => { eprintln!("[migrations] Unable to read dir: {err}"); return Ok(()); }
        };
        entries.sort_by_key(|e| e.path());

        for e in entries {
            let p = e.path();
            if !p.is_file() || p.extension().and_then(|s| s.to_str()) != Some("sql") { continue; }
            let filename = p.file_name().unwrap().to_string_lossy().to_string();
            
            let mut rows = conn.query(
                "SELECT 1 FROM _migrations WHERE filename = ?1 LIMIT 1",
                [libsql::Value::from(filename.clone())],
            ).await?;
            
            let exists = rows.next().await?.is_some();
            if exists { continue; }
            
            let sql = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(err) => { eprintln!("[migrations] Failed to read {}: {err}", filename); continue; }
            };
            
            let tx = conn.transaction().await?;
            for stmt in sql.split(';') {
                let stmt = stmt.trim();
                if stmt.is_empty() { continue; }
                if let Err(err) = tx.execute(stmt, ()).await {
                    eprintln!("[migrations] Error executing statement in {}: {err}", filename);
                    return Err(err);
                }
            }
            tx.execute("INSERT INTO _migrations (filename) VALUES (?1)", [libsql::Value::from(filename.clone())]).await?;
            tx.commit().await?;
            println!("[migrations] Applied {filename}");
        }
        Ok(())
    }
}
