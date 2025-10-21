use libsql::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use crate::repository::{Repository, Entity};

#[derive(Debug, Clone, Serialize, Deserialize,Copy)]
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

    pub async fn create_table(conn: &Connection) -> Result<()> {
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

    pub async fn insert(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO product (name, description, price, quantity) VALUES (?1, ?2, ?3, ?4)",
            params![self.name.clone(), self.description.clone(), self.price, self.quantity],
        ).await?;
        self.id = Some(conn.last_insert_rowid() as i32);
        Ok(())
    }

    pub async fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Product>> {
        let mut stmt = conn.prepare("SELECT id, name, description, price, quantity FROM product WHERE id = ?1").await?;
        let mut rows = stmt.query([id]).await?;

        if let Some(row) = rows.next().await? {
            return Ok(Some(Product {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
            }));
        }
        Ok(None)
    }

    pub async fn find_all(conn: &Connection) -> Result<Vec<Product>> {
        let mut stmt = conn.prepare("SELECT id, name, description, price, quantity FROM product").await?;
    pub async fn find_all(conn: &Connection) -> Result<Vec<Product>> {
        let mut stmt = conn.prepare("SELECT id, name, description, price, quantity FROM product").await?;
        let mut rows = stmt.query([]).await?;

        let mut products = Vec::new();
        while let Some(row) = rows.next().await? {
            products.push(Product {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
            });
        }
        Ok(products)
    }
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
                params![product.name, product.description, product.price, product.quantity, id],
            ).await?;

            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(conn: &Connection, id: i32) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM product WHERE id = ?1", [id]).await?;
        Ok(rows_affected > 0)
    }
}

impl Repository<Product, CreateProduct, UpdateProduct> for Product {
    async fn create_table(conn: &Connection) -> Result<()> {
        Product::create_table(conn).await
    }

    async fn insert(&mut self, conn: &Connection) -> Result<()> {
        self.insert(conn).await
    }

    async fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Product>> {
        Product::find_by_id(conn, id).await
    }

    async fn find_all(conn: &Connection) -> Result<Vec<Product>> {
        Product::find_all(conn).await
    }

    async fn update(conn: &Connection, id: i32, data: UpdateProduct) -> Result<Option<Product>> {
        Product::update(conn, id, data).await
    }

    async fn delete(conn: &Connection, id: i32) -> Result<bool> {
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
