curl -X PUT http://localhost:3000/products/5 `
  -H "Content-Type: application/json" `
  -H "Authorization: Bearer YOUR_TOKEN_HERE" `
  -d '{"name": "thinkpadT14s", "description": "Portable Lenovo thinkpad T14s - Updated", "price": 2200.00, "quantity": 5}'