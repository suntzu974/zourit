use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use rusqlite::{Connection, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ProductRead,
    ProductCreate,
    ProductUpdate,
    ProductDelete,
    UserList,
    UserPromote,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ProductRead => "product.read",
            Permission::ProductCreate => "product.create",
            Permission::ProductUpdate => "product.update",
            Permission::ProductDelete => "product.delete",
            Permission::UserList => "user.list",
            Permission::UserPromote => "user.promote",
        }
    }

    pub fn all() -> &'static [Permission] {
        &[
            Permission::ProductRead,
            Permission::ProductCreate,
            Permission::ProductUpdate,
            Permission::ProductDelete,
            Permission::UserList,
            Permission::UserPromote,
        ]
    }
}

static ROLE_MAP: Lazy<HashMap<&'static str, HashSet<&'static str>>> = Lazy::new(|| {
    use Permission::*;
    let mut m: HashMap<&'static str, HashSet<&'static str>> = HashMap::new();
    // user role: read + create/update own maybe; we simplify to read/create/update
    let user_perms = vec![ProductRead.as_str(), ProductCreate.as_str(), ProductUpdate.as_str()];
    m.insert("user", user_perms.into_iter().collect());
    // admin role: all perms
    m.insert("admin", Permission::all().iter().map(|p| p.as_str()).collect());
    m
});

/// Check permission given a role. If role_permissions table exists, prefer its dynamic mapping.
pub fn has_permission(conn: &Connection, role: &str, perm: Permission) -> bool {
    // Try dynamic table; ignore errors (fallback to static map)
    if let Ok(mut stmt) = conn.prepare("SELECT 1 FROM role_permissions WHERE role = ?1 AND permission = ?2 LIMIT 1") {
        if let Ok(mut rows) = stmt.query(params![role, perm.as_str()]) {
            if let Ok(Some(_)) = rows.next() { return true; }
            // If row not found in dynamic table we treat as deny even if static would allow? We'll fallback to static if table present but no row.
        }
    }
    ROLE_MAP.get(role).map(|set| set.contains(perm.as_str())).unwrap_or(false)
}
