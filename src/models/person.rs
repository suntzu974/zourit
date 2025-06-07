use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use crate::repository::{Repository, Entity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
    pub age: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreatePerson {
    pub name: String,
    pub email: String,
    pub age: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
}

impl Person {
    pub fn new(name: String, email: String, age: i32) -> Self {
        Person {
            id: None,
            name,
            email,
            age,
        }
    }

    pub fn create_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS person (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                age INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO person (name, email, age) VALUES (?1, ?2, ?3)",
            params![self.name, self.email, self.age],
        )?;
        self.id = Some(conn.last_insert_rowid() as i32);
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Person>> {
        let mut stmt = conn.prepare("SELECT id, name, email, age FROM person WHERE id = ?1")?;
        let person_iter = stmt.query_map([id], |row| {
            Ok(Person {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                email: row.get(2)?,
                age: row.get(3)?,
            })
        })?;

        for person in person_iter {
            return Ok(Some(person?));
        }
        Ok(None)
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Person>> {
        let mut stmt = conn.prepare("SELECT id, name, email, age FROM person")?;
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                email: row.get(2)?,
                age: row.get(3)?,
            })
        })?;

        let mut persons = Vec::new();
        for person in person_iter {
            persons.push(person?);
        }
        Ok(persons)
    }

    pub fn update(conn: &Connection, id: i32, update_data: UpdatePerson) -> Result<Option<Person>> {
        if let Some(mut person) = Self::find_by_id(conn, id)? {
            if let Some(name) = update_data.name {
                person.name = name;
            }
            if let Some(email) = update_data.email {
                person.email = email;
            }
            if let Some(age) = update_data.age {
                person.age = age;
            }

            conn.execute(
                "UPDATE person SET name = ?1, email = ?2, age = ?3 WHERE id = ?4",
                params![person.name, person.email, person.age, id],
            )?;

            Ok(Some(person))
        } else {
            Ok(None)
        }
    }

    pub fn delete(conn: &Connection, id: i32) -> Result<bool> {
        let rows_affected = conn.execute("DELETE FROM person WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }
}

impl Repository<Person, CreatePerson, UpdatePerson> for Person {
    fn create_table(conn: &Connection) -> Result<()> {
        Person::create_table(conn)
    }

    fn insert(&mut self, conn: &Connection) -> Result<()> {
        self.insert(conn)
    }

    fn find_by_id(conn: &Connection, id: i32) -> Result<Option<Person>> {
        Person::find_by_id(conn, id)
    }

    fn find_all(conn: &Connection) -> Result<Vec<Person>> {
        Person::find_all(conn)
    }

    fn update(conn: &Connection, id: i32, data: UpdatePerson) -> Result<Option<Person>> {
        Person::update(conn, id, data)
    }

    fn delete(conn: &Connection, id: i32) -> Result<bool> {
        Person::delete(conn, id)
    }
}

impl Entity for Person {
    type CreateType = CreatePerson;
    type UpdateType = UpdatePerson;
    
    fn new_from_create(data: CreatePerson) -> Self {
        Person::new(data.name, data.email, data.age)
    }
}
