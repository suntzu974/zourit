use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{SaltString, PasswordHash, rand_core::OsRng}};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm, TokenData};
use serde::{Serialize, Deserialize};
use time::{Duration, OffsetDateTime};
use libsql::Database as LibSqlDatabase;
use crate::database::DbResult;

const TOKEN_EXP_HOURS: i64 = 12;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,          // user id
    pub exp: usize,        // expiration (epoch seconds)
    pub role: String,      // simple role string
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed = PasswordHash::new(hash);
    if parsed.is_err() { return false; }
    Argon2::default().verify_password(password.as_bytes(), &parsed.unwrap()).is_ok()
}

pub fn generate_token(user_id: i32, role: &str, secret: &str) -> jsonwebtoken::errors::Result<String> {
    let exp = OffsetDateTime::now_utc() + Duration::hours(TOKEN_EXP_HOURS);
    let claims = Claims { sub: user_id, exp: exp.unix_timestamp() as usize, role: role.to_string() };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn validate_token(token: &str, secret: &str) -> jsonwebtoken::errors::Result<TokenData<Claims>> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256))
}

// Simple user storage (username unique)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User { pub id: Option<i32>, pub username: String, pub password_hash: String, pub role: String }

#[derive(Debug, Deserialize)]
pub struct RegisterUser { pub username: String, pub password: String }

#[derive(Debug, Deserialize)]
pub struct LoginUser { pub username: String, pub password: String }

impl User {
    pub async fn create_table(db: &LibSqlDatabase) -> DbResult<()> {
        let conn = db.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT NOT NULL)",
            ()
        ).await?;
        Ok(())
    }
    
    pub async fn insert(&mut self, db: &LibSqlDatabase) -> DbResult<()> {
        let conn = db.connect()?;
        conn.execute(
            "INSERT INTO user (username, password_hash, role) VALUES (?1, ?2, ?3)",
            libsql::params![
                self.username.clone(),
                self.password_hash.clone(),
                self.role.clone()
            ]
        ).await?;
        self.id = Some(conn.last_insert_rowid() as i32);
        Ok(())
    }
    
    pub async fn find_by_username(db: &LibSqlDatabase, username: &str) -> DbResult<Option<User>> {
        let conn = db.connect()?;
        let mut rows = conn.query(
            "SELECT id, username, password_hash, role FROM user WHERE username = ?1",
            [libsql::Value::from(username.to_string())]
        ).await?;
        
        if let Some(row) = rows.next().await? {
            Ok(Some(User {
                id: Some(row.get::<i64>(0)? as i32),
                username: row.get::<String>(1)?,
                password_hash: row.get::<String>(2)?,
                role: row.get::<String>(3)?
            }))
        } else {
            Ok(None)
        }
    }
}
