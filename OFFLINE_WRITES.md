# Offline Writes - Read-Your-Writes Mode

## 📝 Configuration

Le système utilise maintenant le mode **read-your-writes** de LibSQL, qui permet:
- ✅ **Écritures locales** même quand le remote Turso est indisponible
- ✅ **Lectures immédiates** des données que vous venez d'écrire
- ✅ **Sync automatique** quand la connexion est restaurée

## 🔧 Implémentation

```rust
let db = libsql::Builder::new_remote_replica(
    database_path,
    url,
    token
)
.sync_interval(std::time::Duration::from_secs(60))
.read_your_writes(true)  // ← Permet les écritures offline
.build()
.await?;
```

## 🎯 Comportement

### Quand le remote est ACCESSIBLE:
1. **POST** produit → Écrit localement
2. **Sync** automatique vers Turso (toutes les 60s)
3. **GET** produits → Lit depuis le local (très rapide)

### Quand le remote est INDISPONIBLE:
1. **POST** produit → ✅ **Fonctionne!** Écrit dans la DB locale
2. **GET** produits → ✅ **Fonctionne!** Lit depuis la DB locale
3. **Sync** échoue, mais les opérations continuent
4. Logs montrent: `⚠️ Remote unreachable - operating in OFFLINE mode`

### Quand la connexion REVIENT:
1. Sync automatique reprend
2. Toutes les écritures locales sont envoyées vers Turso
3. Logs montrent: `✅ Remote connection restored! Sync completed`

## 🧪 Comment tester

### Test 1: Écriture en mode connecté
```bash
# 1. Démarrer l'application
cargo run

# 2. Créer un produit (POST)
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"name":"Test Product","description":"Testing","price":99.99,"quantity":10}'

# 3. Vérifier dans les logs
# → [database] Background sync completed
```

### Test 2: Écriture en mode offline
```bash
# 1. Bloquer l'accès à Turso (Windows Firewall, déconnexion réseau, etc.)

# 2. Créer un produit (POST)
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"name":"Offline Product","description":"Written offline","price":49.99,"quantity":5}'

# 3. Vérifier dans les logs
# → ⚠️ Remote unreachable - operating in OFFLINE mode
# → POST devrait maintenant réussir (code 200/201)

# 4. Lire les produits (GET)
curl http://localhost:3000/products \
  -H "Authorization: Bearer YOUR_TOKEN"

# → Le produit offline devrait apparaître!
```

### Test 3: Synchronisation après reconnexion
```bash
# 1. Restaurer l'accès réseau

# 2. Attendre max 60 secondes

# 3. Vérifier dans les logs
# → ✅ Remote connection restored! Sync completed

# 4. Vérifier dans Turso Web UI
# → Les produits créés offline devraient être visibles
```

## 🔍 Diagnostic

### Si POST échoue encore offline:

1. **Vérifier les logs**:
   ```
   [database] Connecting with embedded replica
   [database] Background sync task started
   ```

2. **Vérifier le mode read-your-writes**:
   ```rust
   // Dans src/database.rs, ligne ~50
   .read_your_writes(true)  // Doit être présent
   ```

3. **Vérifier la version de libsql**:
   ```toml
   # Dans Cargo.toml
   libsql = "0.9.24"  # Version minimale recommandée
   ```

4. **Tester avec SQLite direct**:
   ```bash
   sqlite3 zourit.db "SELECT * FROM product;"
   ```

## 📊 Surveillance

### Logs clés à surveiller:

```
✅ Normal (online):
   [database] Background sync completed

⚠️  Offline détecté:
   ⚠️ [database] Remote unreachable - operating in OFFLINE mode
   → All operations continue on local database

✅ Reconnexion:
   ✅ [database] Remote connection restored! Sync completed
```

## 🎓 Pourquoi ça fonctionne maintenant?

### Avant (sans read_your_writes):
- LibSQL embedded replica attendait la confirmation du remote pour les writes
- Si remote down → timeout → échec du POST

### Après (avec read_your_writes):
- LibSQL écrit immédiatement dans la DB locale
- Sync vers remote se fait en arrière-plan
- Si remote down → write réussit localement + sync retry plus tard

## 📚 Références

- [LibSQL Embedded Replicas](https://docs.turso.tech/sdk/rust/reference#embedded-replicas)
- [Read Your Writes Mode](https://docs.turso.tech/features/embedded-replicas#read-your-writes)
- Configuration dans: `src/database.rs`
- Documentation sync: `EMBEDDED_REPLICA_ACTIVE.md`
