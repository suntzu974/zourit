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

## API Endpoints


### Products
- `GET /products` - List all products (JSON)
- `POST /products` - Create new product (JSON)
- `GET /products/{id}` - Get product by ID (JSON)
- `PUT /products/{id}` - Update product (JSON)
- `DELETE /products/{id}` - Delete product

### General
- `GET /` - API welcome page with endpoints documentation

## Content Negotiation

The API supports both JSON and HTML responses:
- Add `?format=json` for JSON response
- Default response is HTML for browser-friendly viewing

## Project Structure

```
src/
├── main.rs              # Application entry point
├── database.rs          # Database connection and setup
├── repository.rs        # Repository trait definition
├── entity.rs           # Generic entity operations
├── models/             # Data models
│   ├── mod.rs
│   ├── person.rs       # Person model and database operations
│   └── product.rs      # Product model and database operations
├── handlers/           # Request handlers
│   ├── mod.rs
│   ├── person_handler.rs
│   └── product_handler.rs
├── routes/             # Route definitions
│   ├── mod.rs
│   ├── person_routes.rs
│   └── product_routes.rs
└── templates/          # Askama template structs
    └── mod.rs
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

## Examples

### Create a Person (JSON)
```bash
curl -X POST http://localhost:3000/persons \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com", "age": 30}'
```

### Get All Persons (JSON)
```bash
curl http://localhost:3000/persons?format=json
```

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

- [ ] Authentication and authorization
- [ ] Database migrations
- [ ] API versioning
- [ ] OpenAPI/Swagger documentation
- [ ] Docker containerization
- [ ] Unit and integration tests
- [ ] Logging and monitoring
