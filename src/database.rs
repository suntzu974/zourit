use libsql::{Connection, Database as LibSqlDatabase};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use tokio::time::{interval, Duration};

pub type SharedConnection = Arc<Connection>;
pub type SharedDatabase = Arc<LibSqlDatabase>;
pub type DbResult<T> = Result<T, libsql::Error>;

/// Database connection manager with embedded replica support.
/// 
/// Configuration modes:
/// - **Local only**: Set DATABASE_PATH to a file path, leave TURSO_* unset
/// - **Embedded replica**: Set DATABASE_PATH (local file) + TURSO_DATABASE_URL + TURSO_AUTH_TOKEN
///   - Reads/writes go to local file (fast)
///   - Changes automatically sync to Turso cloud (periodic background sync)
///   - On startup, attempts to sync from Turso (non-blocking)
/// 
/// Resilience:
/// - If remote is unreachable at startup, app continues with local data
/// - Background sync retries automatically until remote is available
/// - App never blocks waiting for remote sync
pub struct Database;

impl Database {
    pub async fn connect(database_path: &str) -> DbResult<(LibSqlDatabase, Connection)> {
        // Check if remote URL is configured for replication
        let remote_url = std::env::var("TURSO_DATABASE_URL").ok();
        let auth_token = std::env::var("TURSO_AUTH_TOKEN").ok();
        
        match (remote_url, auth_token) {
            (Some(url), Some(token)) if !url.is_empty() && !token.is_empty() => {
                // Embedded replica: local file synced with remote Turso
                println!("[database] Connecting with embedded replica: local='{}' remote='{}'", database_path, url);
                let db = libsql::Builder::new_remote_replica(
                    database_path.to_string(),
                    url,
                    token
                )
                .build()
                .await?;
                
                let conn = db.connect()?;
                
                // Perform initial sync
                println!("[database] Syncing with remote...");
                match db.sync().await {
                    Ok(_) => println!("[database] Sync completed successfully"),
                    Err(e) => eprintln!("[database] Sync warning: {} (continuing anyway)", e),
                }
                
                Ok((db, conn))
            }
            _ => {
                // Pure local connection (no replication)
                println!("[database] Connecting to local database: '{}'", database_path);
                let db = libsql::Builder::new_local(database_path).build().await?;
                let conn = db.connect()?;
                Ok((db, conn))
            }
        }
    }

    pub async fn create_shared_connection(database_path: &str) -> DbResult<SharedConnection> {
        let (db, conn) = Self::connect(database_path).await?;
        Self::create_tables(&conn).await?;
        
        // Start background sync task if replication is enabled
        let is_replica = std::env::var("TURSO_DATABASE_URL").ok()
            .and_then(|url| std::env::var("TURSO_AUTH_TOKEN").ok().map(|_| url))
            .filter(|url| !url.is_empty())
            .is_some();
            
        if is_replica {
            Self::start_sync_task(Arc::new(db));
        }
        
        Ok(Arc::new(conn))
    }
    
    fn start_sync_task(db: SharedDatabase) {
        tokio::spawn(async move {
            let sync_interval_secs = std::env::var("SYNC_INTERVAL_SECONDS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60); // Default: sync every 60 seconds
            
            let max_retry_interval = std::env::var("SYNC_MAX_RETRY_INTERVAL")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(300); // Max 5 minutes between retries
            
            let mut ticker = interval(Duration::from_secs(sync_interval_secs));
            let mut consecutive_failures = 0u32;
            let mut is_offline = false;
            
            ticker.tick().await; // Skip first immediate tick
            
            println!("[database] Background sync task started (interval: {}s)", sync_interval_secs);
            
            loop {
                ticker.tick().await;
                
                match db.sync().await {
                    Ok(_) => {
                        if is_offline {
                            println!("✅ [database] Remote connection restored! Sync completed");
                            is_offline = false;
                            consecutive_failures = 0;
                        } else {
                            println!("[database] Background sync completed");
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        
                        if !is_offline {
                            eprintln!("⚠️  [database] Remote unreachable - operating in OFFLINE mode");
                            eprintln!("    Error: {}", e);
                            eprintln!("    → All operations continue on local database");
                            eprintln!("    → Will retry sync every {}s", sync_interval_secs);
                            is_offline = true;
                        } else if consecutive_failures % 10 == 0 {
                            // Log every 10 failures to avoid spam
                            eprintln!("[database] Still offline (failed {} times) - continuing local ops", consecutive_failures);
                        }
                        
                        // Exponential backoff capped at max_retry_interval
                        if consecutive_failures > 3 {
                            let backoff_secs = (sync_interval_secs * 2_u64.pow((consecutive_failures - 3).min(5)))
                                .min(max_retry_interval);
                            if backoff_secs > sync_interval_secs {
                                println!("[database] Backing off to {}s before next retry", backoff_secs);
                                tokio::time::sleep(Duration::from_secs(backoff_secs - sync_interval_secs)).await;
                            }
                        }
                    }
                }
            }
        });
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
