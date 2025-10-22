# Configuration : Connexion Directe à Turso

## Mode Actuel : Remote Direct (Sans Réplication)

L'application est maintenant configurée pour se connecter **directement** à Turso sans utiliser de base de données locale ni de synchronisation.

## Architecture Simplifiée

```
┌─────────────────────┐
│   Application       │
│                     │
│   Axum Handlers     │
│         │           │
│         │ Requêtes  │
│         ▼           │
│   LibSQL Client     │
│         │           │
└─────────┼───────────┘
          │
          │ HTTPS/WSS
          │
          ▼
  ┌──────────────────┐
  │  Turso Cloud     │
  │  (AWS Tokyo)     │
  │                  │
  │  zourit.db       │
  └──────────────────┘
```

## Avantages

### ✅ Simplicité
- Pas de gestion de synchronisation
- Pas de fichier local à maintenir
- Architecture directe et claire

### ✅ Cohérence Immédiate
- Toutes les instances voient les mêmes données
- Pas de replication lag
- Pas de conflits possibles

### ✅ Scalabilité
- Turso gère la charge
- Pas de limite locale
- Backups automatiques

### ✅ Multi-Région
- Turso peut répliquer dans plusieurs régions
- Failover automatique géré par Turso
- Optimisation réseau par Turso

## Configuration

### Fichier .env (Mode Production)

```env
# Connexion directe à Turso
TURSO_DATABASE_URL=libsql://zourit-suntzu974.aws-ap-northeast-1.turso.io
TURSO_AUTH_TOKEN=eyJhbGc...

# JWT pour l'application
JWT_SECRET=CHANGE_ME_DEV_SECRET
PORT=3000
```

### Fichier .env (Mode Développement Local)

```env
# Désactiver Turso
# TURSO_DATABASE_URL=
# TURSO_AUTH_TOKEN=

# Base locale
DATABASE_PATH=zourit.db

JWT_SECRET=CHANGE_ME_DEV_SECRET
PORT=3000
```

## Fonctionnement

### Au Démarrage

```rust
// L'application détecte les variables d'environnement
if TURSO_DATABASE_URL && TURSO_AUTH_TOKEN {
    // Connexion directe à Turso
    libsql::Builder::new_remote(url, token)
} else {
    // Fallback sur fichier local
    libsql::Builder::new_local(path)
}
```

### Logs Attendus

**Avec Turso (Production) :**
```
[database] Connecting to remote Turso database: 'libsql://zourit-...'
[database] Connected successfully to Turso
[migrations] Applied 001_init.sql
Server running on http://0.0.0.0:3000
```

**Sans Turso (Développement) :**
```
[database] Connecting to local database: 'zourit.db'
[migrations] Applied 001_init.sql
Server running on http://0.0.0.0:3000
```

## Performances

### Latence Réseau

| Opération | Temps Estimé |
|-----------|--------------|
| SELECT simple | ~20-50ms (Tokyo → votre région) |
| INSERT | ~30-60ms |
| UPDATE | ~30-60ms |
| DELETE | ~30-60ms |

**Note :** Latence dépend de votre distance à Tokyo (AWS ap-northeast-1)

### Optimisation Turso

Turso optimise automatiquement :
- Connection pooling
- Query caching
- Compression réseau
- Keep-alive connections

## Comparaison avec Réplication

| Aspect | Direct (Actuel) | Embedded Replica |
|--------|-----------------|------------------|
| **Latence lecture** | ~20-50ms | < 1ms |
| **Latence écriture** | ~30-60ms | < 1ms local + async sync |
| **Cohérence** | Immédiate | Éventuelle (lag) |
| **Offline** | ❌ Requiert réseau | ✅ Fonctionne |
| **Complexité** | Simple | Moyenne |
| **Conflits** | Impossible | Possibles |

## Limitations

### ⚠️ Dépendance Réseau

Si le réseau tombe :
```
Error: Connection timeout
→ Application ne peut pas fonctionner
```

**Mitigation :**
- Monitoring réseau actif
- Health checks Turso
- Retry automatique (intégré LibSQL)
- Timeout configurables

### ⚠️ Latence

Toutes les requêtes passent par le réseau :
- Pas de cache local
- Dépend de la qualité réseau
- Distance à Tokyo importante

**Mitigation :**
- Utiliser des requêtes batch si possible
- Implémenter caching applicatif (Redis, Memcached)
- Considérer multi-région Turso

## Cas d'Usage Idéaux

### ✅ Parfait pour :
- Applications stateless
- Déploiement multi-instances
- Données critiques nécessitant cohérence immédiate
- Équipe ne voulant pas gérer la réplication

### ❌ Moins idéal pour :
- Applications nécessitant < 10ms de latence
- Besoin de fonctionnement offline
- Trafic très élevé (milliers de req/s)
- Budget très limité (coût réseau)

## Migration Future vers Embedded Replica

Si nécessaire, la migration est simple :

```rust
// Changer dans database.rs
libsql::Builder::new_remote_replica(
    "zourit.db",    // fichier local
    url,             // Turso URL
    token            // Auth token
)
```

Avantages après migration :
- ✅ Latence < 1ms
- ✅ Fonctionnement offline
- ✅ Même cohérence éventuelle
- ⚠️ Complexité accrue

## Troubleshooting

### Problème : "Connection timeout"

**Causes possibles :**
1. Pas de connexion Internet
2. Firewall bloque port 443/8080
3. Token expiré
4. URL incorrecte

**Solutions :**
```bash
# Tester connectivité
curl -v https://zourit-suntzu974.aws-ap-northeast-1.turso.io

# Vérifier token
turso db tokens validate $TURSO_AUTH_TOKEN

# Régénérer token
turso db tokens create zourit
```

### Problème : "Slow queries"

**Diagnostic :**
```bash
# Mesurer latence
time curl http://localhost:3000/products
```

**Solutions :**
1. Vérifier distance à Tokyo
2. Considérer région plus proche
3. Implémenter cache applicatif
4. Migrer vers embedded replica

### Problème : "Too many connections"

**Cause :** Turso limite le nombre de connexions concurrentes

**Solution :**
```rust
// Dans votre code, LibSQL gère déjà le pooling
// Vérifier que vous ne créez pas de connexions inutiles
```

## Monitoring

### Métriques à surveiller

1. **Latence des requêtes**
   - Objectif : < 100ms
   - Alerte : > 500ms

2. **Taux d'erreur réseau**
   - Objectif : < 0.1%
   - Alerte : > 1%

3. **Disponibilité Turso**
   - Objectif : 99.9%
   - Vérifier status.turso.tech

### Logs Importants

```bash
# Succès
[database] Connected successfully to Turso

# Échec
Error: Connection timeout
Error: Authentication failed
Error: Database not found
```

## Coûts Turso

Vérifier les limites du plan :
- Free tier : 500 MB, 1 milliard de row reads/mois
- Croissance : Augmenter selon besoin
- Facturation : Basée sur stockage + transfert

## Conclusion

La configuration actuelle (connexion directe) est :
- ✅ Simple à comprendre et maintenir
- ✅ Idéale pour débuter
- ✅ Scalable pour la plupart des cas
- ⚠️ Dépendante du réseau
- ⚠️ Latence réseau inhérente

Pour des besoins de très haute performance ou offline, considérer embedded replica.
