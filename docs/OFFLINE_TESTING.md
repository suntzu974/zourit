# Test de Résilience - Scénario Offline

Ce script simule différents scénarios de panne réseau pour tester la résilience.

## Scénario 1 : Démarrage avec remote inaccessible

### Simulation
```bash
# Temporairement pointer vers une URL invalide
export TURSO_DATABASE_URL="libsql://invalid-url-does-not-exist.turso.io"
cargo run
```

### Résultat attendu
```
[database] Connecting with embedded replica: local='zourit.db' remote='libsql://invalid...'
[database] Syncing with remote...
[database] Sync warning: ... (continuing anyway)
Server running on http://0.0.0.0:3000
[database] Background sync task started (interval: 60s)
⚠️  [database] Remote unreachable - operating in OFFLINE mode
```

✅ L'application démarre et fonctionne normalement en mode local

## Scénario 2 : Test des opérations en mode offline

### Étapes
1. Démarrer l'app avec remote invalide (comme ci-dessus)
2. Créer un produit via API:
```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Offline","description":"Créé en mode offline","price":99.99,"quantity":10}'
```

3. Lire les produits:
```bash
curl http://localhost:3000/products
```

### Résultat attendu
✅ Toutes les opérations CRUD fonctionnent
✅ Données stockées dans zourit.db local
✅ Logs montrent "operating in OFFLINE mode"

## Scénario 3 : Reconnexion automatique

### Étapes
1. Démarrer en mode offline (URL invalide)
2. Attendre quelques cycles de sync (logs "Still offline...")
3. Changer l'URL vers la vraie:
```bash
# Dans un autre terminal, modifier .env
# TURSO_DATABASE_URL=libsql://zourit-suntzu974.aws-ap-northeast-1.turso.io

# Redémarrer l'app
cargo run
```

### Résultat attendu
```
[database] Syncing with remote...
✅ [database] Remote connection restored! Sync completed
[database] Background sync completed
```

✅ Données créées offline sont maintenant synchronisées vers Turso

## Scénario 4 : Test avec délai réseau

### Simulation Windows (PowerShell avec privilèges admin)
```powershell
# Bloquer temporairement le trafic vers Turso
New-NetFirewallRule -DisplayName "Block Turso" -Direction Outbound -RemoteAddress "52.84.0.0/16" -Action Block

# Démarrer l'app
cargo run

# Observer les logs offline

# Débloquer après quelques minutes
Remove-NetFirewallRule -DisplayName "Block Turso"

# Observer la reconnexion automatique
```

### Résultat attendu
```
⚠️  [database] Remote unreachable - operating in OFFLINE mode
[database] Backing off to 120s before next retry
[database] Still offline (failed 10 times) - continuing local ops
✅ [database] Remote connection restored! Sync completed
```

## Vérification de la sync

### Après reconnexion, vérifier dans Turso CLI:
```bash
# Connecter à la base remote
turso db shell zourit

# Vérifier que les données créées offline sont présentes
SELECT * FROM product WHERE name = 'Test Offline';
```

✅ Les données doivent apparaître dans Turso

## Test de charge pendant offline

### Créer 100 produits en mode offline
```bash
for i in {1..100}; do
  curl -X POST http://localhost:3000/products \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Product $i\",\"description\":\"Bulk test\",\"price\":$i.99,\"quantity\":$i}"
  echo ""
done
```

### Après reconnexion
Tous les 100 produits doivent être synchronisés vers Turso (peut prendre 1-2 minutes selon `SYNC_INTERVAL_SECONDS`).

## Monitoring des logs

### Logs normaux (mode online)
```
[database] Background sync completed
[database] Background sync completed
```

### Logs problème (mode offline)
```
⚠️  [database] Remote unreachable - operating in OFFLINE mode
[database] Backing off to 120s before next retry
[database] Still offline (failed 10 times)
```

### Logs récupération
```
✅ [database] Remote connection restored! Sync completed
[database] Background sync completed
```

## Métriques de performance

Mesurer l'impact de la réplication:

### Temps de réponse API (mode online)
```bash
# Devrait être < 10ms (local)
time curl http://localhost:3000/products
```

### Temps de réponse API (mode offline)
```bash
# Devrait être identique (< 10ms, local)
time curl http://localhost:3000/products
```

**Conclusion:** Aucune différence de performance entre online/offline car les opérations sont toujours locales.

## Troubleshooting

### Problème: Les logs ne montrent pas "offline mode"

**Cause:** URL remote est peut-être valide et accessible.

**Solution:** Vérifier connectivité:
```bash
curl -v https://zourit-suntzu974.aws-ap-northeast-1.turso.io
```

### Problème: App ne démarre pas du tout

**Cause:** Fichier local zourit.db peut être corrompu.

**Solution:**
```bash
# Sauvegarder
mv zourit.db zourit.db.backup

# Redémarrer (créera un nouveau fichier)
cargo run
```

### Problème: Données ne se synchronisent pas après reconnexion

**Cause:** Token peut être invalide ou expiré.

**Solution:**
```bash
# Générer un nouveau token
turso db tokens create zourit

# Mettre à jour .env
# TURSO_AUTH_TOKEN=nouveau_token_ici

# Redémarrer
cargo run
```

## Checklist de validation

- [ ] ✅ App démarre avec remote inaccessible
- [ ] ✅ CRUD fonctionne en mode offline
- [ ] ✅ Logs montrent "OFFLINE mode" clairement
- [ ] ✅ Reconnexion automatique après rétablissement
- [ ] ✅ Données offline synchronisées vers remote
- [ ] ✅ Performance identique online/offline
- [ ] ✅ Exponential backoff visible dans les logs
- [ ] ✅ Pas de crash/panic en cas de panne réseau
