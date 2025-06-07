use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub trait Repository<T, CreateT, UpdateT> 
where 
    T: Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    fn create_table(conn: &Connection) -> Result<()>;
    fn insert(&mut self, conn: &Connection) -> Result<()>;
    fn find_by_id(conn: &Connection, id: i32) -> Result<Option<T>>;
    fn find_all(conn: &Connection) -> Result<Vec<T>>;
    fn update(conn: &Connection, id: i32, update_data: UpdateT) -> Result<Option<T>>;
    fn delete(conn: &Connection, id: i32) -> Result<bool>;
}

pub trait Entity {
    type CreateType;
    type UpdateType;
    
    fn new_from_create(data: Self::CreateType) -> Self;
}
