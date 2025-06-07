use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::models::{Product, CreateProduct, UpdateProduct};
use crate::database::SharedConnection;
use crate::entity::{create_entity, get_entity, get_all_entities, update_entity, delete_entity};

pub async fn create_product(
    State(conn): State<SharedConnection>,
    Json(payload): Json<CreateProduct>,
) -> Result<Json<Product>, StatusCode> {
    create_entity::<Product, CreateProduct, UpdateProduct>(&conn, payload).await
}

pub async fn get_product(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
) -> Result<Json<Product>, StatusCode> {
    get_entity::<Product, CreateProduct, UpdateProduct>(&conn, id).await
}

pub async fn get_all_products(
    State(conn): State<SharedConnection>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    get_all_entities::<Product, CreateProduct, UpdateProduct>(&conn).await
}

pub async fn update_product(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateProduct>,
) -> Result<Json<Product>, StatusCode> {
    update_entity::<Product, CreateProduct, UpdateProduct>(&conn, id, payload).await
}

pub async fn delete_product(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    delete_entity::<Product, CreateProduct, UpdateProduct>(&conn, id).await
}
