# Réplication LibSQL - Guide de Résilience

## Comment ça marche ?

### Mode Embedded Replica

Lorsque `TURSO_DATABASE_URL` et `TURSO_AUTH_TOKEN` sont configurés, l'application utilise un **embedded replica** LibSQL :

```
┌─────────────────────┐
│   Application       │
│                     │
│  ┌──────────────┐   │
│  │ Local SQLite │   │  ← Toutes les lectures/écritures (RAPIDE)
│  │  (zourit.db) │   │
│  └──────┬───────┘   │
│         │           │
│         │ Sync      │
│         ▼           │
│  ┌──────────────┐   │
│  │  Background  │   │  ← Synchro périodique en arrière-plan
│  │  Sync Task   │   │
│  └──────┬───────┘   │
└─────────┼───────────┘
          │
          │ Network
          ▼
  ┌──────────────────┐
  │  Turso Cloud     │  ← Base de données distante (backup + multi-région)
  │  (AWS Tokyo)     │
  └──────────────────┘
```

## Scénarios de panne réseau

### ✅ Scénario 1 : Remote inaccessible au démarrage

**Ce qui se passe :**
```
[database] Connecting with embedded replica: local='zourit.db' remote='libsql://...'
[database] Syncing with remote...
[database] Sync warning: connection error (continuing anyway)
[migrations] Applied 001_init.sql
Server running on http://0.0.0.0:3000
[database] Background sync task started (interval: 60s)
```

**Résultat :**
- ✅ L'application démarre normalement
- ✅ Utilise les données locales existantes
- ✅ Accepte les requêtes HTTP immédiatement
- ⚠️ Affiche un warning pour la sync initiale
- 🔄 Continue d'essayer en arrière-plan

### ✅ Scénario 2 : Perte de connexion pendant l'exécution

**Ce qui se passe :**
```
[database] Background sync completed
[database] Background sync completed
⚠️  [database] Remote unreachable - operating in OFFLINE mode
    Error: network timeout
    → All operations continue on local database
    → Will retry sync every 60s
[database] Still offline (failed 10 times) - continuing local ops
```

**Résultat :**
- ✅ L'application continue de fonctionner normalement
- ✅ Toutes les opérations CRUD fonctionnent (local)
- ⚠️ Mode offline détecté et signalé
- 🔄 Retry automatique avec exponential backoff

### ✅ Scénario 3 : Retour de connexion

**Ce qui se passe :**
```
[database] Still offline (failed 20 times) - continuing local ops
✅ [database] Remote connection restored! Sync completed
[database] Background sync completed
```

**Résultat :**
- ✅ Reconnexion automatique détectée
- ✅ Toutes les modifications locales envoyées au cloud
- ✅ Données synchronisées bidirectionnellement
- 🎉 Retour au mode normal

## Algorithme de Retry

```rust
Tentative 1-3:   Retry immédiat (toutes les 60s)
Tentative 4:     Backoff 2× = 120s
Tentative 5:     Backoff 4× = 240s
Tentative 6+:    Backoff 8× = 300s (max 5 minutes)
```

**Avantages :**
- Ne spam pas le serveur distant si down longtemps
- Reconnecion rapide si panne courte
- Log réduit (1 message toutes les 10 tentatives)

## Configuration

### Variables d'environnement

```env
# Intervalle de sync normal (secondes)
SYNC_INTERVAL_SECONDS=60

# Intervalle maximum en cas d'échecs répétés (secondes)
SYNC_MAX_RETRY_INTERVAL=300
```

### Recommandations par environnement

**Développement local :**
```env
SYNC_INTERVAL_SECONDS=10      # Sync rapide pour tests
SYNC_MAX_RETRY_INTERVAL=30    # Retry rapide
```

**Production :**
```env
SYNC_INTERVAL_SECONDS=60      # Balance perf/freshness
SYNC_MAX_RETRY_INTERVAL=300   # Évite de marteler le serveur
```

**Faible bande passante :**
```env
SYNC_INTERVAL_SECONDS=300     # Sync toutes les 5 min
SYNC_MAX_RETRY_INTERVAL=600   # Backoff 10 min
```

## Garanties et Limitations

### ✅ Garanties

1. **Disponibilité** : App fonctionne même si Turso est down
2. **Performance** : Lectures/écritures toujours locales (< 1ms)
3. **Durabilité** : Données écrites localement ne sont jamais perdues
4. **Résilience** : Reconnexion automatique sans intervention

### ⚠️ Limitations

1. **Replication lag** : Les changements prennent jusqu'à `SYNC_INTERVAL_SECONDS` pour se propager
2. **Conflits** : Si plusieurs instances modifient les mêmes données offline, la dernière sync gagne ("last write wins")
3. **Pas de sync temps réel** : Pas de notifications push depuis Turso
4. **Isolation des instances** : Chaque instance a sa propre copie locale

### 🔒 Cas d'usage idéaux

✅ **Bon pour :**
- Applications read-heavy (beaucoup de lectures)
- Données qui tolèrent un délai de sync (< 60s)
- Besoin de haute disponibilité locale
- Déploiements multi-régions

❌ **Pas idéal pour :**
- Transactions distribuées critiques
- Besoin de cohérence immédiate entre instances
- Données financières nécessitant ACID global

## Monitoring

### Logs à surveiller

**Santé normale :**
```
[database] Background sync completed
```

**Problème détecté :**
```
⚠️  [database] Remote unreachable - operating in OFFLINE mode
```

**Problème résolu :**
```
✅ [database] Remote connection restored! Sync completed
```

### Métriques recommandées

- Nombre de sync failures consécutives
- Durée du dernier offline period
- Temps depuis la dernière sync réussie
- Taille du lag de réplication (si mesurable)

## Troubleshooting

### Problème : "Remote unreachable" persistant

**Causes possibles :**
1. Firewall bloque les connexions sortantes sur port 443
2. Token expiré (`TURSO_AUTH_TOKEN`)
3. URL incorrecte (`TURSO_DATABASE_URL`)
4. Turso en maintenance

**Diagnostic :**
```bash
# Test manuel de connexion
curl -v https://zourit-suntzu974.aws-ap-northeast-1.turso.io

# Vérifier le token
turso db tokens validate $TURSO_AUTH_TOKEN

# Régénérer un token
turso db tokens create zourit
```

### Problème : Sync trop lente

**Solutions :**
1. Réduire `SYNC_INTERVAL_SECONDS` (attention à la bande passante)
2. Vérifier la latence réseau vers la région Turso
3. Considérer une région Turso plus proche
4. Vérifier la taille de la base (>100MB peut ralentir)

### Problème : Conflits de données

**Symptôme :** Données écrasées entre instances

**Solution :**
- Utiliser des UUID plutôt que des ID auto-incrémentés
- Implémenter versioning (colonne `version` + check optimiste)
- Séparer les données par instance (colonne `instance_id`)
- Migrer vers un mode pure remote si besoin de cohérence forte

## Architecture Alternative : Pure Remote

Si la réplication embedded ne convient pas, LibSQL peut aussi fonctionner en mode **pure remote** :

```rust
// Dans database.rs (alternative)
let db = libsql::Builder::new_remote(url, token).build().await?;
```

**Avantages :**
- Cohérence immédiate entre toutes les instances
- Pas de gestion de conflits
- Pas de stockage local nécessaire

**Inconvénients :**
- Latence réseau sur chaque requête
- Pas de fonctionnement offline
- Dépendance totale à Turso

## Conclusion

Le mode embedded replica est optimal pour :
- **99.9% des cas d'usage**
- Applications web classiques
- APIs avec données non financières
- Besoin de résilience et performance

Passez en pure remote seulement si vous avez un besoin critique de cohérence immédiate.
