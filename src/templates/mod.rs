use askama::Template;
use serde::Serialize;
use crate::models::Person;

#[derive(Serialize)]
pub struct EndpointInfo {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Template)]
#[template(path = "index.html")]
// If you use custom filters, import them here, e.g.:
// #[template(path = "index.html", escape = "none", filters = "crate::templates::filters")]
pub struct IndexTemplate {
    pub title: String,
    pub message: String,
    pub version: String,
    pub endpoints: Vec<EndpointInfo>,
}


#[derive(Template)]
#[template(path = "persons/show.html")]
pub struct PersonShowTemplate {
    pub person: Person,
}

#[derive(Template)]
#[template(path = "persons/create.html")]
pub struct PersonCreateTemplate;

impl IndexTemplate {
    pub fn new() -> Self {
        Self {
            title: "Zourit API".to_string(),
            message: "Welcome to Zourit API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                EndpointInfo {
                    name: "Persons".to_string(),
                    url: "/persons".to_string(),
                    description: "Manage persons - GET, POST, PUT, DELETE operations".to_string(),
                },
                EndpointInfo {
                    name: "Products".to_string(),
                    url: "/products".to_string(),
                    description: "Manage products - GET, POST, PUT, DELETE operations".to_string(),
                },
            ],
        }
    }
}



impl PersonShowTemplate {
    pub fn new(person: Person) -> Self {
        Self { person }
    }
}

impl PersonCreateTemplate {
    pub fn new() -> Self {
        Self
    }
}

