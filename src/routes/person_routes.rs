use axum::{
    routing::{get, post, delete, put},
    Router,
};
use crate::database::SharedConnection;
use crate::handlers::{
    person_handler::*,
};

pub fn create_person_routes() -> Router<SharedConnection> {
    Router::new()
//        .route("/persons", get(get_all_persons))
        .route("/persons", post(create_person))
        .route("/persons/new", get(show_create_person_form))
        .route("/persons/create", post(create_person_form))
//        .route("/persons/{id}", get(get_person))
        .route("/persons/{id}", put(update_person))
        .route("/persons/{id}", delete(delete_person))
}
