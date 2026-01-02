# Hub Router

Service de routage pour le Hub - Décide où router les commandes, événements et requêtes.

## 🎯 Rôle

Le Router Service est responsable de :

- Recevoir des requêtes de routage du Hub Manager
- Déterminer vers quel service router un message
- Maintenir un cache optionnel pour optimiser les performances
- Indiquer quand un lookup Registry est nécessaire

## ⚠️ Principes architecturaux

- **Ne stocke PAS les handlers** (c'est le Registry qui les a)
- **Ne communique JAMAIS directement** avec d'autres services
- **Toute communication** passe par le Hub Manager
- **Stateless avec cache optionnel** - le cache peut être vidé sans perte de données critiques

## 🚀 Démarrage

```bash
cargo run --package hub_router
```

Le service démarre sur le port **8082**.

## 📡 API exposée

### Health Check

```bash
GET http://localhost:8082/health
```

**Réponse :**

```json
{
  "status": "ok",
  "service": "router"
}
```

### Router un message

```bash
POST http://localhost:8082/route
Content-Type: application/json

{
  "type": "command",
  "capability": "invoice.create",
  "payload": {...}
}
```

**Réponse (cache hit) :**

```json
{
  "target_url": "http://billing-service:9001",
  "needs_registry_lookup": false
}
```

**Réponse (cache miss) :**

```json
{
  "target_url": null,
  "needs_registry_lookup": true
}
```

### Mettre à jour le cache

```bash
POST http://localhost:8082/cache/update
Content-Type: application/json

{
  "capability": "invoice.create",
  "url": "http://billing-service:9001"
}
```

### Invalider une entrée du cache

```bash
DELETE http://localhost:8082/cache/invoice.create
```

### Vider tout le cache

```bash
DELETE http://localhost:8082/cache
```

### Statistiques du cache

```bash
GET http://localhost:8082/cache/stats
```

**Réponse :**

```json
{
  "entries": 2,
  "capabilities": ["invoice.create", "order.create"]
}
```

## 🔄 Flux typique (OPTIMISÉ)

**Avant (problématique)** :

```plaintext
Manager → Router → Manager → Registry → Manager → Router (cache update) → Produit
(4 appels Router!)
```

**Après (optimisé)** :

```plaintext
Manager → Router (inclut capability si lookup needed)
          ↓
    [Cache hit] → Retourne target_url
          ↓
    [Cache miss] → Retourne {needs_lookup: true, capability: "..."}
          ↓
Manager → Registry (lookup avec capability fournie)
          ↓
Manager → Produit
          ↓
Manager → Router (cache update, optionnel)
```

**Avantages** :

- ✅ UN SEUL appel Router (pas deux)
- ✅ Capability dans la réponse (pas besoin de reparer la requête)
- ✅ Sauts réseau réduits de 50%
- ✅ Flux clair et prévisible

## 🧪 Tests

```bash
# Lancer tous les tests
cargo test --package hub_router

# Tests unitaires uniquement
cargo test --package hub_router --lib

# Tests d'intégration uniquement
cargo test --package hub_router --test router_tests
```

## 🏗️ Architecture

Le Router utilise :

- **Warp** pour le serveur HTTP
- **Tokio** pour l'async/await
- **RwLock** pour le partage du cache entre threads
- **HashMap** pour le cache capability → URL

## 📝 Types de messages supportés

- `command` - Faire quelque chose (ex: invoice.create)
- `event` - Quelque chose est arrivé (ex: invoice.created)
- `query` - Interroger un service (ex: invoice.get)

## ⚡ Optimisations

Le cache est optionnel et peut être :

- Activé pour réduire les lookups Registry
- Désactivé en mode debug/dev
- Invalidé automatiquement si un service change d'URL
- Vidé complètement en cas de problème

## 🔒 Sécurité

- Aucune donnée sensible stockée
- Cache reconstruisible à tout moment
- Pas d'authentification (géré par Hub Manager)
- Logs de toutes les opérations

## 📦 Dépendances

```toml
tokio = { version = "1", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

## 🔧 Configuration

Le Router n'a pas de configuration externe. Il est configuré au runtime par le Hub Manager via les endpoints de mise à jour du cache.
