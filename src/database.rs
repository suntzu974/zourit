use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};
use crate::models::{Product};

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
        Product::create_table(conn)?;
        Ok(())
    }
}
