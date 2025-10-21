use libsql::Connection;
use serde::{Deserialize, Serialize};
use crate::repository::{Repository, Entity};
use crate::database::DbResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Option<i32>,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub quantity: Option<i32>,
}

impl Product {
    pub fn new(name: String, description: String, price: f64, quantity: i32) -> Self {
        Product {
            id: None,
            name,
            description,
            price,
            quantity,
        }
    }

    pub async fn create_table(conn: &Connection) -> DbResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS product (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL
            )",
            (),
        ).await?;
        Ok(())
    }

    pub async fn insert(&mut self, conn: &Connection) -> DbResult<()> {
        conn.execute(
            "INSERT INTO product (name, description, price, quantity) VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                self.name.clone(),
                self.description.clone(),
                self.price,
                self.quantity
            ],
        ).await?;
        self.id = Some(conn.last_insert_rowid() as i32);
        Ok(())
    }

    pub async fn find_by_id(conn: &Connection, id: i32) -> DbResult<Option<Product>> {
        let mut rows = conn.query(
            "SELECT id, name, description, price, quantity FROM product WHERE id = ?1",
            [libsql::Value::from(id)]
        ).await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Product {
                id: Some(row.get::<i64>(0)? as i32),
                name: row.get::<String>(1)?,
                description: row.get::<String>(2)?,
                price: row.get::<f64>(3)?,
                quantity: row.get::<i64>(4)? as i32,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn find_all(conn: &Connection) -> DbResult<Vec<Product>> {
        let mut rows = conn.query(
            "SELECT id, name, description, price, quantity FROM product",
            ()
        ).await?;

        let mut products = Vec::new();
        while let Some(row) = rows.next().await? {
            products.push(Product {
                id: Some(row.get::<i64>(0)? as i32),
                name: row.get::<String>(1)?,
                description: row.get::<String>(2)?,
                price: row.get::<f64>(3)?,
                quantity: row.get::<i64>(4)? as i32,
            });
        }
        Ok(products)
    }

    pub async fn update(conn: &Connection, id: i32, update_data: UpdateProduct) -> DbResult<Option<Product>> {
        if let Some(mut product) = Self::find_by_id(conn, id).await? {
            if let Some(name) = update_data.name {
                product.name = name;
            }
            if let Some(description) = update_data.description {
                product.description = description;
            }
            if let Some(price) = update_data.price {
                product.price = price;
            }
            if let Some(quantity) = update_data.quantity {
                product.quantity = quantity;
            }

            conn.execute(
                "UPDATE product SET name = ?1, description = ?2, price = ?3, quantity = ?4 WHERE id = ?5",
                libsql::params![
                    product.name.clone(),
                    product.description.clone(),
                    product.price,
                    product.quantity,
                    id
                ],
            ).await?;

            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(conn: &Connection, id: i32) -> DbResult<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM product WHERE id = ?1",
            [libsql::Value::from(id)]
        ).await?;
        Ok(rows_affected > 0)
    }
}

impl Repository<Product, CreateProduct, UpdateProduct> for Product {
    async fn create_table(conn: &Connection) -> DbResult<()> {
        Product::create_table(conn).await
    }

    async fn insert(&mut self, conn: &Connection) -> DbResult<()> {
        self.insert(conn).await
    }

    async fn find_by_id(conn: &Connection, id: i32) -> DbResult<Option<Product>> {
        Product::find_by_id(conn, id).await
    }

    async fn find_all(conn: &Connection) -> DbResult<Vec<Product>> {
        Product::find_all(conn).await
    }

    async fn update(conn: &Connection, id: i32, data: UpdateProduct) -> DbResult<Option<Product>> {
        Product::update(conn, id, data).await
    }

    async fn delete(conn: &Connection, id: i32) -> DbResult<bool> {
        Product::delete(conn, id).await
    }
}

impl Entity for Product {
    type CreateType = CreateProduct;
    type UpdateType = UpdateProduct;
    
    fn new_from_create(data: CreateProduct) -> Self {
        Product::new(data.name, data.description, data.price, data.quantity)
    }
}
