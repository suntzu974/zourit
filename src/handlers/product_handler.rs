use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::models::{Product, CreateProduct, UpdateProduct};
use crate::database::SharedDatabase;
use crate::entity::{create_entity, get_entity, get_all_entities, update_entity, delete_entity};

pub async fn create_product(
    State(db): State<SharedDatabase>,
    Json(payload): Json<CreateProduct>,
) -> Result<Json<Product>, StatusCode> {
    create_entity::<Product, CreateProduct, UpdateProduct>(&db, payload).await
}

pub async fn get_product(
    State(db): State<SharedDatabase>,
    Path(id): Path<i32>,
) -> Result<Json<Product>, StatusCode> {
    get_entity::<Product, CreateProduct, UpdateProduct>(&db, id).await
}

pub async fn get_all_products(
    State(db): State<SharedDatabase>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    get_all_entities::<Product, CreateProduct, UpdateProduct>(&db).await
}

pub async fn update_product(
    State(db): State<SharedDatabase>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateProduct>,
) -> Result<Json<Product>, StatusCode> {
    update_entity::<Product, CreateProduct, UpdateProduct>(&db, id, payload).await
}

pub async fn delete_product(
    State(db): State<SharedDatabase>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    delete_entity::<Product, CreateProduct, UpdateProduct>(&db, id).await
}
