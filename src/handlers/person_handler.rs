use askama::Template;
use axum::{
    extract::{Path, State, Query, Form},
    http::StatusCode,
    response::{Json, Html, Redirect},
};
use either::Either;
use std::collections::HashMap;
use crate::models::{Person, CreatePerson, UpdatePerson};
use crate::database::SharedConnection;
use crate::entity::{create_entity, get_entity, get_all_entities, update_entity, delete_entity};
use crate::templates::{ PersonShowTemplate, PersonCreateTemplate};

pub async fn show_create_person_form() -> Html<String> {
    let template = PersonCreateTemplate::new();
    Html(template.render().unwrap())
}

pub async fn create_person_form(
    State(conn): State<SharedConnection>,
    Form(payload): Form<CreatePerson>,
) -> Result<Redirect, StatusCode> {
    match create_entity::<Person, CreatePerson, UpdatePerson>(&conn, payload).await {
        Ok(_) => Ok(Redirect::to("/persons")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_person(
    State(conn): State<SharedConnection>,
    Json(payload): Json<CreatePerson>,
) -> Result<Json<Person>, StatusCode> {
    create_entity::<Person, CreatePerson, UpdatePerson>(&conn, payload).await
}

pub async fn get_person(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Either<Html<String>, Json<Person>>, StatusCode> {
    let result = get_entity::<Person, CreatePerson, UpdatePerson>(&conn, id).await?;
    
    if params.get("format").map(|s| s.as_str()) == Some("json") {
        Ok(Either::Right(result))
    } else {
        let template = PersonShowTemplate::new(result.0);
        match template.render() {
            Ok(html) => Ok(Either::Left(Html(html))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn get_all_persons(
    State(conn): State<SharedConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Either<Html<String>, Json<Vec<Person>>>, StatusCode> {
    Ok(Either::Right(Json(vec![])))
}

pub async fn update_person(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePerson>,
) -> Result<Json<Person>, StatusCode> {
    update_entity::<Person, CreatePerson, UpdatePerson>(&conn, id, payload).await
}

pub async fn delete_person(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    delete_entity::<Person, CreatePerson, UpdatePerson>(&conn, id).await
}
