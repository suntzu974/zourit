use rusqlite::{Connection, Result, params, OptionalExtension};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;

pub type SharedConnection = Arc<Mutex<Connection>>;

pub struct Database;

impl Database {
    pub fn connect(database_path: &str) -> Result<Connection> {
        Connection::open(database_path)
    }

    pub fn create_shared_connection(database_path: &str) -> Result<SharedConnection> {
        let conn = Self::connect(database_path)?;
        Self::create_tables(&conn)?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    pub fn create_tables(conn: &Connection) -> Result<()> {
        // legacy direct creation (kept for backward compatibility) now handled by migrations
        Self::apply_migrations(conn)?;
        Ok(())
    }

    fn apply_migrations(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

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
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM _migrations WHERE filename = ?1 LIMIT 1",
                    params![filename],
                    |_| Ok(1),
                )
                .optional()? // None => not applied
                .is_some();
            if exists { continue; }
            let sql = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(err) => { eprintln!("[migrations] Failed to read {}: {err}", filename); continue; }
            };
            let tx = conn.unchecked_transaction()?;
            for stmt in sql.split(';') {
                let stmt = stmt.trim();
                if stmt.is_empty() { continue; }
                if let Err(err) = tx.execute(stmt, []) {
                    eprintln!("[migrations] Error executing statement in {}: {err}", filename);
                    return Err(err);
                }
            }
            tx.execute("INSERT INTO _migrations (filename) VALUES (?1)", params![filename])?;
            tx.commit()?;
            println!("[migrations] Applied {filename}");
        }
        Ok(())
    }
}
