-- Dynamic role to permission mapping (optional override of static mapping)
CREATE TABLE IF NOT EXISTS role_permissions (
    role TEXT NOT NULL,
    permission TEXT NOT NULL,
    PRIMARY KEY (role, permission)
);

-- Seed basic permissions only if empty
INSERT INTO role_permissions(role, permission)
SELECT 'user', 'product.read' WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role='user' AND permission='product.read');
INSERT INTO role_permissions(role, permission)
SELECT 'user', 'product.create' WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role='user' AND permission='product.create');
INSERT INTO role_permissions(role, permission)
SELECT 'user', 'product.update' WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role='user' AND permission='product.update');

-- Admin gets everything
INSERT INTO role_permissions(role, permission)
SELECT 'admin', p FROM (
    SELECT 'product.read' AS p UNION ALL
    SELECT 'product.create' UNION ALL
    SELECT 'product.update' UNION ALL
    SELECT 'product.delete' UNION ALL
    SELECT 'user.list' UNION ALL
    SELECT 'user.promote'
)
WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role='admin' AND permission=p);
