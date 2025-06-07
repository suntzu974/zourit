use askama::Template;
use serde::Serialize;

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

impl IndexTemplate {
    pub fn new() -> Self {
        Self {
            title: "Zourit API".to_string(),
            message: "Welcome to Zourit API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                EndpointInfo {
                    name: "Products".to_string(),
                    url: "/products".to_string(),
                    description: "Manage products - GET, POST, PUT, DELETE operations".to_string(),
                },
            ],
        }
    }
}
 