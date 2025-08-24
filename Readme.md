# Zourit API

A RESTful API built with Rust using Axum web framework, SQLite database, and Askama templating engine. This project demonstrates SOLID principles with a clean architecture for managing persons and products.

## Features

- 🚀 **Fast & Safe**: Built with Rust for performance and memory safety
- 🌐 **REST API**: Full CRUD operations for Persons and Products
- 🎨 **Web Interface**: HTML templates with Askama for browser interaction
- 📊 **SQLite Database**: Lightweight, embedded database with rusqlite
- 🔧 **SOLID Principles**: Clean architecture with proper separation of concerns
- 🎯 **Content Negotiation**: Supports both JSON and HTML responses
- 📝 **Form Support**: Web forms for creating and managing entities
- 🔐 **Authentication**: JWT-based register/login
- 🔒 **Authorization**: Role check (admin required for destructive operations)

## API Endpoints


### Products
- `GET /products` - List all products (JSON)
- `POST /products` - Create new product (JSON)
- `GET /products/{id}` - Get product by ID (JSON)
- `PUT /products/{id}` - Update product (JSON)
- `DELETE /products/{id}` - Delete product

### General
- `GET /` - API welcome page with endpoints documentation

### Auth
- `POST /auth/register` - Register user, returns JWT (role user)
- `POST /auth/login` - Login, returns JWT
- `GET /auth/me` - Get current user (requires Bearer token)
- `GET /auth/refresh` - Issue a new JWT (requires valid token)
- `GET /auth/users` - List users (admin role required)

## Content Negotiation

The API supports both JSON and HTML responses:
- Add `?format=json` for JSON response
- Default response is HTML for browser-friendly viewing

## Project Structure

```
src/
├── main.rs              # Application entry point (loads .env)
├── database.rs          # DB connection + table creation
├── repository.rs        # Generic repository trait
├── entity.rs            # Generic CRUD helpers (create/get/update/delete)
├── auth.rs              # JWT + password hashing (argon2) + user model
├── middleware.rs        # Auth & admin middlewares
├── models/              # Data models (domain + DTO)
│   └── product.rs
├── handlers/            # HTTP handlers (products, auth)
│   ├── product_handler.rs
│   └── auth_handler.rs
├── routes/              # Route groups
│   ├── mod.rs           # Root router assembly
│   ├── product_routes.rs
│   └── auth_routes.rs   # Auth routes (register/login/me/...)
└── templates/           # Askama template structs & HTML
templates/              # HTML templates
├── index.html          # Welcome page
└── persons/
    ├── index.html      # List persons
    ├── show.html       # Show person details
    └── create.html     # Create person form
```

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd zourit
```

2. Install dependencies:
```bash
cargo build
```

3. Run the application:
```bash
cargo run
```

The server will start on `http://localhost:3000`

### Development

Run with auto-reload:
```bash
cargo watch -x run
```

### Configuration (.env)

Create a `.env` file (see `.env.example`):
```
JWT_SECRET=PLEASE_CHANGE_THIS_SECRET
DATABASE_PATH=zourit.db
PORT=3000
```
Environment variables are loaded via `dotenv` in `main.rs`.

## Dependencies

- **axum** - Modern web framework
- **tokio** - Async runtime
- **rusqlite** - SQLite database driver
- **serde** - Serialization/deserialization
- **askama** - Template engine
- **tower-http** - HTTP middleware

## Architecture

This project follows SOLID principles:

- **Single Responsibility**: Each module has a single purpose
- **Open/Closed**: Easy to extend with new entities
- **Liskov Substitution**: Repository trait ensures consistent behavior
- **Interface Segregation**: Small, focused traits
- **Dependency Inversion**: Depends on abstractions, not concretions

### Key Components

1. **Repository Pattern**: Generic CRUD operations
2. **Entity Trait**: Common entity behavior
3. **Handlers**: HTTP request processing
4. **Routes**: URL routing and organization
5. **Templates**: HTML view generation

### Generic Repository & Entity Pattern

Le cœur de l'architecture de persistance repose sur deux traits génériques :

1. `Repository<T, CreateT, UpdateT>` : définit les opérations CRUD que chaque modèle doit implémenter (création de table, insertion, recherche, mise à jour partielle, suppression).
2. `Entity` : relie le type domaine (`T`) à ses DTO d'entrée (`CreateType` et `UpdateType`) et fournit une fabrique `new_from_create`.

Généricité utilisée dans les handlers (`src/entity.rs`) :

```rust
pub async fn create_entity<T, CreateT, UpdateT>(...) -> Result<Json<T>, StatusCode>
where
    T: Repository<T, CreateT, UpdateT> + ...,
    T: Entity<CreateType = CreateT>
```

Ainsi un même code gère création / lecture / mise à jour / suppression pour n'importe quel modèle implémentant ces traits (DRY et facile à étendre).

#### Rôle de CreateT vs UpdateT

| Type | Objectif | Champs | Exemple Product |
|------|----------|--------|-----------------|
| `CreateT` | Données nécessaires à la création | Tous requis (sauf id auto) | `CreateProduct { name, description, price, quantity }` |
| `UpdateT` | Mise à jour partielle (PATCH-like) | Tous optionnels (`Option<T>`) | `UpdateProduct { name: Option<String>, ... }` |

Avantages :
- Empêche l'utilisateur d'envoyer un `id` à la création
- Permet des mises à jour partielles sans payload complet
- Clarifie les invariants (un produit créé est toujours complet)

#### Exemple concret (Product)

Définition (extrait de `models/product.rs`) :
```rust
#[derive(Debug, Deserialize)]
pub struct CreateProduct { name: String, description: String, price: f64, quantity: i32 }

#[derive(Debug, Deserialize)]
pub struct UpdateProduct { 
    name: Option<String>,
    description: Option<String>,
    price: Option<f64>,
    quantity: Option<i32>,
}
```

#### Flux de création
1. Le JSON est désérialisé en `CreateProduct`
2. `create_entity` appelle `Product::new_from_create(...)`
3. `insert` affecte l'`id` généré
4. L'objet complet est renvoyé en JSON

#### Flux de mise à jour
1. Le JSON partiel devient `UpdateProduct`
2. `update_entity` charge l'enregistrement + applique seulement les `Some(...)`
3. Écrit en base puis renvoie l'état à jour

### Exemples API Produit

Créer un produit :
```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"name":"Laptop","description":"Ultrabook 14","price":1299.99,"quantity":5}'
```

Mettre à jour seulement le prix et le stock :
```bash
curl -X PUT http://localhost:3000/products/1 \
  -H "Content-Type: application/json" \
  -d '{"price":1199.00,"quantity":7}'
```

Récupérer tous les produits en JSON :
```bash
curl http://localhost:3000/products?format=json
```

Supprimer :
```bash
curl -X DELETE http://localhost:3000/products/1 -i
```

### Auth Examples

Register:
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"Secret123!"}'
```

Login:
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"Secret123!"}'
```

Use token:
```bash
TOKEN=... # JWT from login
curl http://localhost:3000/auth/me -H "Authorization: Bearer $TOKEN"
```

Refresh token:
```bash
curl http://localhost:3000/auth/refresh -H "Authorization: Bearer $TOKEN"
```

List users (admin only):
```bash
curl http://localhost:3000/auth/users -H "Authorization: Bearer $ADMIN_TOKEN"
```

#### Roles

- Default registered users have role `user`.
- Admin-only endpoints (`/auth/users`, product deletion) require `role == "admin"`.
- (Current implementation: promote a user manually in the SQLite DB by updating the `role` column to `admin`).

## Examples

### Web Interface
Visit `http://localhost:3000` in your browser to use the HTML interface.

## License

This project is open source and available under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Future Enhancements

- [ ] Database migrations
- [ ] API versioning
- [ ] OpenAPI/Swagger documentation
- [ ] Docker containerization
- [ ] Unit and integration tests
- [ ] Logging and monitoring
- [ ] Admin promotion endpoint / user roles management UI
