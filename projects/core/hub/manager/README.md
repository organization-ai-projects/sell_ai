# 🚀 Hub Manager - Gateway + Superviseur

Le **Hub Manager** est le composant central du Hub System. C'est le **seul processus** qui connaît tous les autres services et produits.

## 📋 Rôle

- **Gateway** : Point d'entrée unique pour toutes les requêtes externes
- **Superviseur** : Lance et supervise les services du hub (Registry, Router)
- **Proxy** : Route les commandes vers les services appropriés
- **Orchestre** : Gère la coordination entre tous les composants

## 🎯 Responsabilités

### Lancement des services

```
Hub Manager
├── Lance hub_registry sur port 8081
├── Lance hub_router sur port 8082
└── Vérifie qu'ils répondent aux health checks
```

### Supervision continue

```
Hub Manager (boucle)
├── Check health du Registry toutes les 5s
├── Check health du Router toutes les 5s
├── Redémarre si un service crash
└── Logs les incidents
```

### API Gateway

Expose une API HTTP unique sur le port **8080** :

- `GET /health` - Health check du Manager
- `GET /status` - État complet du hub (Manager + Registry + Router)
- `POST /command` - Recevoir une commande client
- `POST /register` - Enregistrer un produit externe

## 🚀 Démarrage

### Option 1 : Via le script

```bash
./start_hub.sh  # Démarre Manager, Registry et Router ensemble
```

### Option 2 : Manuel

```bash
cargo run --release
```

## 📊 Architecture

```
┌──────────────────────────────────────┐
│   Hub Manager (Port 8080)            │
│                                      │
│  ServiceConfig {                     │
│    - name: String                    │
│    - binary_name: String             │
│    - port: u16                       │
│    - health_check_url: String        │
│  }                                   │
│                                      │
│  HubManager {                        │
│    - services: HashMap<Name, Config> │
│    - processes: RwLock<Children>     │
│    - client: reqwest::Client         │
│  }                                   │
└──────────────────────────────────────┘
         │                 │
         ▼                 ▼
    [Registry:8081]   [Router:8082]
```

## 🔑 Fonctionnalités

### 1. Lancement automatique

```rust
// Détecte les binaires compilés et les lance
// Avec retry et vérification de santé (timeout 10s)
```

### 2. Health checks asynchrones

```rust
// Polling toutes les 5 secondes
// GET http://service/health
// Redémarrage si pas de réponse
```

### 3. Supervision continue

```rust
// Boucle async qui monitor les services
// Détecte les crashs
// Redémarre automatiquement
// Log les incidents
```

### 4. API d'état

```bash
curl http://localhost:8080/status
# Retourne JSON avec état complet du hub
```

## 🧪 Tests

### Health check local

```bash
curl http://localhost:8080/health
# {"status":"ok"}
```

### Status complet

```bash
curl http://localhost:8080/status
# {
#   "hub_manager": "running",
#   "components": {
#     "registry": {"status": "healthy"},
#     "router": {"status": "healthy"}
#   },
#   "timestamp": "2025-12-31T17:19:26.234909035+01:00"
# }
```

### Vérifier les binaires lancés

```bash
# Voir les processus enfants
ps aux | grep hub_

# Vérifier les ports
lsof -i:8080
lsof -i:8081
lsof -i:8082
```

## 🏗️ Principes architecturaux

### Hub Manager stateless

- ✅ Pas de donnée critique
- ✅ Peut crasher et redémarrer
- ✅ Pas de persistence

### Seul point connu de tous

- ✅ Hub Manager connaît Registry et Router
- ✅ Registry ne connaît que lui
- ✅ Router ne connaît que lui
- ✅ Produits ne connaissent que lui

### Pas de communication directe

```
❌ Registry ←→ Router
✅ Registry ←→ Manager ←→ Router
```

## 📈 Flux typique

```
1. Client → Manager:8080/command
2. Manager → Router:8082/route (où router ?)
3. Router → Manager (URL du handler ou lookup needed)
4. Manager → Registry:8081/lookup/{capability}
5. Registry → Manager (URL du service)
6. Manager → Router:8082/cache/update (cache l'URL)
7. Manager → Service:9001/handle (exécute)
8. Service → Manager (résultat)
9. Manager → Client (résultat)
```

## 🔧 Configuration

### Fichier `Cargo.toml`

```toml
[package]
name = "hub_manager"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = "0.11"
chrono = "0.4"
anyhow = "1.0"
thiserror = "1.0"
```

## 📝 Code principal

### Structure `ServiceConfig`

```rust
struct ServiceConfig {
    name: String,              // "registry" ou "router"
    binary_name: String,       // "hub_registry" ou "hub_router"
    port: u16,                 // 8081 ou 8082
    health_check_url: String,  // http://localhost:8081/health
}
```

### Structure `HubManager`

```rust
struct HubManager {
    services: HashMap<String, ServiceConfig>,
    processes: Arc<RwLock<HashMap<String, Option<Child>>>>,
    client: reqwest::Client,
}
```

## 🚨 Gestion des erreurs

### Processus qui crash

```
Manager détecte : GET /health timeout ou erreur
→ Market service comme "unhealthy"
→ Tue le processus enfant
→ Redémarre le binaire
→ Vérifie avec health checks
```

### Ctrl+C gracieux

```rust
// Intercepte SIGINT
// Arrête tous les services proprement
// Tue les processus enfants
// Ferme le manager
```

## 📊 Monitoring

### Logs

```bash
tail -f logs/manager.log
```

### Processus actifs

```bash
ps aux | grep hub_manager
lsof -i:8080
```

## 🎯 Prochaines étapes

1. **Intégration complète** : Implémenter le flux complet Client → Manager → Router → Registry → Service
2. **Produit exemple** : Créer un `billing_service` pour tester
3. **Lifecycle Service** : Gestion start/stop/restart
4. **Config Service** : Configuration centralisée
5. **Observability** : Logs structurés et métriques

## 📚 Voir aussi

- [README principal](../../../README.md)
- [Router Service](../router/README.md)
- [Architecture Hub](../../../docs/hub_services.md)
- [Flux de données](../../../docs/data_flux.md)

---

**Le Hub Manager est le cœur du système ! 🎉**
