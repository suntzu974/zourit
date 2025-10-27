use libsql::{Connection, Database as LibSqlDatabase};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub type SharedConnection = Arc<Connection>;
pub type DbResult<T> = Result<T, libsql::Error>;

/// Database state tracking for replica synchronization
pub struct DatabaseState {
    pub is_replica: bool,
    pub sync_enabled: bool,
}

pub type SharedDatabaseState = Arc<RwLock<DatabaseState>>;

/// Database connection manager for LibSQL with local replica and synchronization.
/// 
/// Configuration modes:
/// - **Local replica with sync**: Set TURSO_DATABASE_URL + TURSO_AUTH_TOKEN + DATABASE_PATH
///   - Creates local replica that syncs with remote Turso database
///   - Fast local reads/writes with periodic synchronization
///   - Best of both worlds: performance + data consistency
/// 
/// - **Local only**: Leave TURSO_* unset, set DATABASE_PATH
///   - Uses local SQLite file only
///   - Development mode
pub struct Database;

impl Database {
    pub async fn connect(database_path: &str) -> DbResult<(LibSqlDatabase, Connection, SharedDatabaseState)> {
        // Check if remote URL is configured
        let remote_url = std::env::var("TURSO_DATABASE_URL").ok();
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();
        
        match (remote_url, auth_token) {
            (Some(url), Some(token)) if !url.is_empty() && !token.is_empty() => {
                // Local replica with remote sync
                println!("[database] Creating local replica with sync to: '{}'", url);
                let db = libsql::Builder::new_remote_replica(database_path, url, token)
                    .build()
                    .await?;
                
                let conn = db.connect()?;
                
                // Initial sync
                println!("[database] Attempting initial sync...");
                match db.sync().await {
                    Ok(_) => println!("✅ [database] Initial sync successful"),
                    Err(e) => {
                        eprintln!("⚠️ [database] Initial sync failed: {}", e);
                    }
                }
                
                let state = Arc::new(RwLock::new(DatabaseState {
                    is_replica: true,
                    sync_enabled: true,
                }));
                
                Ok((db, conn, state))
            }
            _ => {
                // Local-only connection (development mode)
                println!("[database] Connecting to local database: '{}'", database_path);
                let db = libsql::Builder::new_local(database_path).build().await?;
                let conn = db.connect()?;
                
                let state = Arc::new(RwLock::new(DatabaseState {
                    is_replica: false,
                    sync_enabled: false,
                }));
                
                Ok((db, conn, state))
            }
        }
    }

    pub async fn create_shared_connection(database_path: &str) -> DbResult<(SharedConnection, SharedDatabaseState)> {
        let (db, conn, state) = Self::connect(database_path).await?;
        Self::create_tables(&conn).await?;
        
        // Start background sync task if replica is enabled
        let state_clone = state.clone();
        let db_clone = Arc::new(db); // Wrap database in Arc for sharing
        tokio::spawn(async move {
            start_sync_task(db_clone, state_clone).await;
        });
        
        Ok((Arc::new(conn), state))
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

/// Start background sync task for replica database
async fn start_sync_task(db: Arc<LibSqlDatabase>, state: SharedDatabaseState) {
    let sync_interval = std::env::var("SYNC_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u64>()
        .unwrap_or(60);
    
    let mut interval = interval(Duration::from_secs(sync_interval));
    
    loop {
        interval.tick().await;
        
        let should_sync = {
            let state_guard = state.read().await;
            state_guard.is_replica && state_guard.sync_enabled
        };
        
        if should_sync {
            match db.sync().await {
                Ok(_) => {
                    println!("🔄 [database] Background sync completed successfully");
                }
                Err(e) => {
                    eprintln!("❌ [database] Background sync failed: {}", e);
                }
            }
        }
    }
}
