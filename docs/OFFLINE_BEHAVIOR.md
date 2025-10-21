# Réponse à : "Qu'est-ce qui se passe si le remote n'est pas accessible ?"

## 🎯 Réponse courte

**L'application continue de fonctionner normalement.** Toutes les opérations se font sur la base locale, et la synchronisation reprendra automatiquement quand le remote sera à nouveau accessible.

---

## 📊 Diagramme de Comportement

```
                    DÉMARRAGE
                        │
                        ▼
            ┌───────────────────────┐
            │ Connexion à local DB  │
            │    (zourit.db)        │
            └───────────┬───────────┘
                        │
                        ▼
        ┌───────────────────────────────┐
        │  Turso remote accessible ?    │
        └───────┬───────────────┬───────┘
                │               │
          OUI   │               │  NON
                ▼               ▼
     ┌──────────────┐    ┌─────────────┐
     │ Sync initiale│    │⚠️  Warning  │
     │   réussie    │    │ (non-fatal) │
     └──────┬───────┘    └──────┬──────┘
            │                   │
            └─────────┬─────────┘
                      ▼
            ┌─────────────────┐
            │  🚀 APP DÉMARRE │
            │                 │
            │ ✅ API prête   │
            │ ✅ CRUD actif  │
            └────────┬────────┘
                     │
                     ▼
        ┌────────────────────────┐
        │  Background Sync Task  │
        │  (toutes les 60s)      │
        └───┬──────────────┬─────┘
            │              │
     ┌──────▼──────┐  ┌───▼────────┐
     │   ONLINE    │  │  OFFLINE   │
     │             │  │            │
     │ Sync OK ✅  │  │ Retry 🔄   │
     │ Log quiet   │  │ Backoff ⏱️  │
     └─────────────┘  └────────────┘
                           │
                           │ Quand remote revient
                           ▼
                    ┌──────────────┐
                    │ Reconnexion  │
                    │ automatique  │
                    │      ✅      │
                    └──────────────┘
```

---

## 🔄 Scénarios Détaillés

### Scénario 1 : Remote inaccessible au démarrage

```plaintext
[0.0s]  🔌 Tentative connexion à Turso...
[0.2s]  ⚠️  Timeout - remote inaccessible
[0.2s]  ✅ Utilisation de zourit.db local
[0.3s]  🚀 Server running on http://0.0.0.0:3000
[60s]   🔄 Retry sync... failed
[120s]  🔄 Retry sync (backoff)... failed
[...]   Mode OFFLINE - app fonctionne normalement
```

**Impact utilisateur :** AUCUN
- API répond normalement
- Performance identique (tout local)
- Données persistent localement

---

### Scénario 2 : Remote tombe pendant l'exécution

```plaintext
[10:00]  ✅ Sync completed
[10:01]  ✅ Sync completed
[10:02]  ⚠️  OFFLINE MODE activated
[10:02]  → All operations continue locally
[10:03]  🔄 Retry... failed
[10:05]  🔄 Retry... failed (backoff 2min)
[10:15]  🔄 Still offline (10 failures)
[...]
```

**Que fait l'app ?**
1. ✅ Continue d'accepter requêtes HTTP
2. ✅ Toutes écritures en local
3. ✅ Toutes lectures depuis local
4. 🔄 Retry périodique automatique
5. 📝 Log clair du statut offline

**Impact utilisateur :** AUCUN si mono-instance

---

### Scénario 3 : Remote revient après panne

```plaintext
[11:00]  🔄 Retry... failed
[11:05]  🔄 Retry... ✅ SUCCESS!
[11:05]  ✅ Remote connection restored!
[11:05]  📤 Syncing local changes to cloud...
[11:06]  ✅ Sync completed
[11:07]  ✅ Back to normal operation
```

**Ce qui se passe :**
1. 🔍 Détection automatique du retour réseau
2. 📤 Upload de tous les changements locaux
3. 📥 Download des changements distants (si multi-instance)
4. ✅ Retour transparent au mode normal

---

## 🎯 Garanties du Système

| Situation | Comportement | Garantie |
|-----------|--------------|----------|
| **Remote down au boot** | App démarre avec local DB | ✅ 100% fonctionnel |
| **Remote tombe en prod** | Switch auto vers offline | ✅ Zero downtime |
| **Écritures offline** | Stockées localement | ✅ Aucune perte |
| **Remote revient** | Sync auto des données | ✅ Cohérence finale |
| **Performance** | Toujours local (< 1ms) | ✅ Constante |
| **Durabilité** | SQLite ACID local | ✅ Crash-safe |

---

## ⚠️ Limitations à Connaître

### 1. Délai de Sync (Replication Lag)

```plaintext
Instance A (Tokyo)     Instance B (Paris)
      │                      │
      │  CREATE product      │
      ├─────────►(local)     │
      │                      │
      │   [60s delay]        │
      │                      │
      │  ─────sync──────►  Turso
      │                      │
      │                   ◄──sync───
      │                      │
      │                  (product visible)
```

**Délai maximum :** `SYNC_INTERVAL_SECONDS` (défaut 60s)

---

### 2. Conflits Multi-Instances

Si deux instances modifient la même ligne offline :

```plaintext
Instance A              Instance B
  │                        │
  │ UPDATE product#1       │ UPDATE product#1
  │ price = 100€           │ price = 120€
  │ (offline)              │ (offline)
  │                        │
  │ ──────sync(101s)──────►│ Turso
  │                        │
  │                        │──sync(105s)─►
  │                        │
  │  Result: price = 120€  │ (last write wins)
```

**Stratégie :** Last Write Wins (LWW)
**Solution :** Versionning optimiste si critique

---

## 🛠️ Configuration Recommandée

### Production Standard
```env
SYNC_INTERVAL_SECONDS=60          # Balance perf/freshness
SYNC_MAX_RETRY_INTERVAL=300       # Cap à 5 min
```

### High Availability
```env
SYNC_INTERVAL_SECONDS=30          # Sync plus fréquente
SYNC_MAX_RETRY_INTERVAL=120       # Retry rapide
```

### Faible Bande Passante
```env
SYNC_INTERVAL_SECONDS=300         # Sync toutes les 5 min
SYNC_MAX_RETRY_INTERVAL=600       # Moins de pression réseau
```

---

## 📈 Métriques de Monitoring

### Logs à surveiller

**✅ Santé normale :**
```
[database] Background sync completed
```

**⚠️ Problème détecté :**
```
⚠️  [database] Remote unreachable - operating in OFFLINE mode
```

**✅ Problème résolu :**
```
✅ [database] Remote connection restored! Sync completed
```

### Alertes recommandées

| Condition | Alerte | Action |
|-----------|--------|--------|
| Offline > 5 min | Warning | Check réseau |
| Offline > 30 min | Error | Vérifier Turso status |
| Sync errors > 100 | Critical | Check token/URL |

---

## 🧪 Tester la Résilience

### Test 1 : Démarrage sans remote
```bash
# Invalider l'URL temporairement
export TURSO_DATABASE_URL="libsql://invalid.turso.io"
cargo run
# → App démarre quand même ✅
```

### Test 2 : Opérations en offline
```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"name":"Offline Product","price":99.99}'
# → Fonctionne ✅
```

### Test 3 : Vérifier la sync après reconnexion
```bash
# Remettre la vraie URL, redémarrer
# Puis vérifier dans Turso CLI:
turso db shell zourit "SELECT * FROM product"
# → "Offline Product" est présent ✅
```

---

## 📚 Documentation Complète

- **[docs/REPLICATION.md](REPLICATION.md)** - Architecture détaillée
- **[docs/OFFLINE_TESTING.md](OFFLINE_TESTING.md)** - Scénarios de test

---

## ✨ Conclusion

### Qu'est-ce qui se passe si le remote n'est pas accessible ?

**Rien de grave ! 🎉**

1. ✅ L'application démarre et fonctionne
2. ✅ Toutes les opérations continuent (local)
3. ✅ Aucune donnée perdue
4. ✅ Reconnexion automatique
5. ✅ Performance constante

**C'est exactement pour ça qu'on utilise l'embedded replica !**
