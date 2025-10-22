use libsql::Database as LibSqlDatabase;
use serde::{Deserialize, Serialize};
use crate::database::DbResult;

#[allow(dead_code)]
pub trait Repository<T, CreateT, UpdateT> 
where 
    T: Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    async fn create_table(db: &LibSqlDatabase) -> DbResult<()>;
    async fn insert(&mut self, db: &LibSqlDatabase) -> DbResult<()>;
    async fn find_by_id(db: &LibSqlDatabase, id: i32) -> DbResult<Option<T>>;
    async fn find_all(db: &LibSqlDatabase) -> DbResult<Vec<T>>;
    async fn update(db: &LibSqlDatabase, id: i32, update_data: UpdateT) -> DbResult<Option<T>>;
    async fn delete(db: &LibSqlDatabase, id: i32) -> DbResult<bool>;
}

pub trait Entity {
    type CreateType;
    type UpdateType;
    
    fn new_from_create(data: Self::CreateType) -> Self;
}
