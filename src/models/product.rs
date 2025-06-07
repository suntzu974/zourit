use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use crate::repository::{Repository, Entity};

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

    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS product (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                price REAL NOT NULL,
                quantity INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO product (name, description, price, quantity) VALUES (?1, ?2, ?3, ?4)",
            params![self.name, self.description, self.price, self.quantity],
        )?;
        self.id = Some(conn.last_insert_rowid() as i32);
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Product>> {
        let mut stmt = conn.prepare("SELECT id, name, description, price, quantity FROM product WHERE id = ?1")?;
        let product_iter = stmt.query_map([id], |row| {
            Ok(Product {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
            })
        })?;

        for product in product_iter {
            return Ok(Some(product?));
        }
        Ok(None)
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Product>> {
        let mut stmt = conn.prepare("SELECT id, name, description, price, quantity FROM product")?;
        let product_iter = stmt.query_map([], |row| {
            Ok(Product {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
            })
        })?;

        let mut products = Vec::new();
        for product in product_iter {
            products.push(product?);
        }
        Ok(products)
    }

    pub fn update(conn: &Connection, id: i32, update_data: UpdateProduct) -> Result<Option<Product>> {
        if let Some(mut product) = Self::find_by_id(conn, id)? {
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
            )?;

            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    pub fn delete(conn: &Connection, id: i32) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM product WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }
}

impl Repository<Product, CreateProduct, UpdateProduct> for Product {
    fn create_table(conn: &Connection) -> Result<()> {
        Product::create_table(conn)
    }

    fn insert(&mut self, conn: &Connection) -> Result<()> {
        self.insert(conn)
    }

    fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Product>> {
        Product::find_by_id(conn, id)
    }

    fn find_all(conn: &Connection) -> Result<Vec<Product>> {
        Product::find_all(conn)
    }

    fn update(conn: &Connection, id: i32, data: UpdateProduct) -> Result<Option<Product>> {
        Product::update(conn, id, data)
    }

    fn delete(conn: &Connection, id: i32) -> Result<bool> {
        Product::delete(conn, id)
    }
}

impl Entity for Product {
    type CreateType = CreateProduct;
    type UpdateType = UpdateProduct;
    
    fn new_from_create(data: CreateProduct) -> Self {
        Product::new(data.name, data.description, data.price, data.quantity)
    }
}
