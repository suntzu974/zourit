use axum::{
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::database::SharedConnection;
use crate::repository::Repository;

pub async fn create_entity<T, CreateT, UpdateT>(
    conn: &SharedConnection,
    payload: CreateT,
) -> Result<Json<T>, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + Clone + Serialize + std::fmt::Debug,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
    T: crate::repository::Entity<CreateType = CreateT>,
{
    let mut entity = T::new_from_create(payload);
    
    match entity.insert(conn).await {
        Ok(()) => Ok(Json(entity)),
        Err(e) => {
            eprintln!("[ERROR] Failed to insert entity: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_entity<T, CreateT, UpdateT>(
    conn: &SharedConnection,
    id: i32,
) -> Result<Json<T>, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    match T::find_by_id(conn, id).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("[ERROR] Failed to find entity by id: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_all_entities<T, CreateT, UpdateT>(
    conn: &SharedConnection,
) -> Result<Json<Vec<T>>, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    match T::find_all(conn).await {
        Ok(entities) => Ok(Json(entities)),
        Err(e) => {
            eprintln!("[ERROR] Failed to find all entities: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_entity<T, CreateT, UpdateT>(
    conn: &SharedConnection,
    id: i32,
    payload: UpdateT,
) -> Result<Json<T>, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    match T::update(conn, id, payload).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("[ERROR] Failed to update entity: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_entity<T, CreateT, UpdateT>(
    conn: &SharedConnection,
    id: i32,
) -> Result<StatusCode, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + Clone + Serialize,
    CreateT: for<'de> Deserialize<'de>,
    UpdateT: for<'de> Deserialize<'de>,
{
    match T::delete(conn, id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("[ERROR] Failed to delete entity: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
