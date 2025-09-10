# Postman Collection for Zourit API

This directory contains Postman collection and environment files for testing the Zourit API.

## Files

- `Zourit API.postman_collection.json` - Complete API collection with all endpoints
- `Zourit Development.postman_environment.json` - Environment variables for development

## Import Instructions

1. Open Postman
2. Click "Import" in the top left
3. Import both files:
   - `Zourit API.postman_collection.json`
   - `Zourit Development.postman_environment.json`
4. Select "Zourit Development" environment in the top right

## Usage Workflow

### 1. Authentication Setup
Run these requests in order to set up authentication:

1. **Create Admin** - Creates the first admin user (bootstrap)
   - Sets `admin_token` variable automatically
2. **Register User** - Creates a regular user
3. **Login User** - Logs in the regular user
   - Sets `jwt_token` variable automatically

### 2. Product Operations
With authentication tokens set, you can:

1. **Get All Products** - List all products (no auth required)
2. **Create Product** - Create a new product (requires user token)
   - Automatically stores the created product ID in `product_id` variable
3. **Get Product by ID** - Retrieve specific product
4. **Update Product** - Modify product (requires user token)
5. **Delete Product (Admin Only)** - Delete product (requires admin token)

### 3. Admin Operations
Using the admin token:

1. **List All Users (Admin)** - JSON list of all users
2. **Admin UI - Users Page** - HTML user management interface

## Environment Variables

The collection uses these variables:

- `base_url` - API base URL (default: http://localhost:3000)
- `jwt_token` - Regular user JWT token (auto-set by login)
- `admin_token` - Admin user JWT token (auto-set by admin creation)
- `product_id` - Last created product ID (auto-set by create product)

## Authentication Requirements

- **No auth**: GET /products, GET /products/{id}, GET /
- **User auth**: POST /products, PUT /products/{id}, GET /auth/me
- **Admin auth**: DELETE /products/{id}, GET /auth/users, GET /admin/users

## Testing the Delete Product Endpoint

The delete product endpoint specifically requires:

1. **Admin authentication** - Use the `admin_token`
2. **Valid product ID** - Create a product first to get an ID
3. **Proper HTTP method** - DELETE request

Example workflow:
1. Run "Create Admin" to get admin token
2. Run "Create Product" to create a test product
3. Run "Delete Product (Admin Only)" to delete it

## Security Notes

- Admin token is required for destructive operations
- CSRF protection is implemented for HTML forms (not needed for API calls)
- All tokens are stored as secret variables in the environment

## Troubleshooting

- **401 Unauthorized**: Check that the correct token is set and valid
- **403 Forbidden**: Ensure you're using admin token for admin-only endpoints
- **404 Not Found**: Verify the product_id variable is set correctly
- **500 Internal Server Error**: Check server logs and database connectivity
