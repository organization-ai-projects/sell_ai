# 📦 Hub Registry - Annuaire des services

Le **Hub Registry** est l'annuaire central du Hub System. Il maintient les **mappings capability → service URL**.

## 📋 Rôle

- **Annuaire** : Registre des handlers disponibles
- **Lookup** : Trouve l'URL d'un service pour une capability donnée
- **Enregistrement** : Accepte les inscriptions de nouveaux services
- **Découverte** : Permet la découverte des services dynamiquement

## 🎯 Responsabilités

### Enregistrement

```plaintext
Service → Registry:8081/register
Registry → Stocke { capability: "invoice.create", url: "http://billing:9001" }
```

### Lookup

```plaintext
Manager → Registry:8081/lookup/invoice.create
Registry → Retourne { "url": "http://billing:9001" }
```

### Listing

```plaintext
Manager → Registry:8081/list
Registry → Retourne { capabilities: ["invoice.create", "payment.process", ...] }
```

## 🚀 Démarrage

### Option 1 : Via le script

```bash
./start_hub.sh  # Démarre Registry automatiquement via Hub Manager
```

### Option 2 : Manuel

```bash
cargo run --release
```

Le service écoute sur le port **8081**.

## 📊 Architecture

```plaintext
┌──────────────────────────────┐
│  Hub Registry (Port 8081)    │
│                              │
│  Registry {                  │
│    handlers: HashMap<        │
│      String,  // capability  │
│      String   // URL         │
│    >                         │
│  }                           │
│                              │
│  API HTTP:                   │
│  - POST /register            │
│  - GET /lookup/:capability   │
│  - GET /list                 │
│  - GET /health               │
└──────────────────────────────┘
         │
         └─ Hub Manager (8080) lancé et supervise
```

## 🔑 API Endpoints

### 1. Health Check

```bash
GET /health
```

**Réponse :**

```json
{
  "service": "registry",
  "status": "ok"
}
```

### 2. Enregistrer un service

```bash
POST /register
Content-Type: application/json

{
  "capability": "invoice.create",
  "url": "http://billing-service:9001"
}
```

**Réponse :**

```json
{
  "status": "registered",
  "capability": "invoice.create",
  "url": "http://billing-service:9001"
}
```

### 3. Lookup une capability

```bash
GET /lookup/invoice.create
```

**Réponse si trouvée :**

```json
{
  "capability": "invoice.create",
  "url": "http://billing-service:9001",
  "status": "found"
}
```

**Réponse si non trouvée :**

```json
{
  "capability": "invoice.create",
  "status": "not_found"
}
```

### 4. Lister tous les handlers

```bash
GET /list
```

**Réponse :**

```json
{
  "handlers": [
    {
      "capability": "invoice.create",
      "url": "http://billing-service:9001"
    },
    {
      "capability": "payment.process",
      "url": "http://payment-service:9002"
    }
  ],
  "count": 2
}
```

## 🧪 Tests

### Health check

```bash
curl http://localhost:8081/health
```

### Enregistrer un service

```bash
curl -X POST http://localhost:8081/register \
  -H "Content-Type: application/json" \
  -d '{
    "capability": "invoice.create",
    "url": "http://billing:9001"
  }'
```

### Lookup curl

```bash
curl http://localhost:8081/lookup/invoice.create
```

### Lister

```bash
curl http://localhost:8081/list
```

## 🏗️ Principes architecturaux

### Seul annuaire centralisé

- ✅ Source de vérité unique pour les mappings
- ✅ Évite la duplication
- ✅ Permet la découverte dynamique

### Lancé par Hub Manager

- ✅ Pas d'accès direct depuis clients
- ✅ Supervise et redémarre automatiquement
- ✅ Health checks toutes les 5s

### Pas de logique métier

- ✅ C'est juste un annuaire
- ✅ Pas de orchestration
- ✅ Pas de routage

### Stateless (à améliorer)

- ⚠️ Actuellement pas de persistence
- 🔧 TODO: Ajouter persistence en fichier ou DB

## 📈 Flux typique

```plaintext
1. Service externe démarre
   ↓
2. POST /register avec ses capabilities
   ↓
3. Registry stocke les mappings
   ↓
4. Manager interroge le Registry
   ↓
5. Registry retourne les URL
   ↓
6. Manager met à jour le cache du Router
```

## 🔧 Configuration

### Fichier `Cargo.toml`

```toml
[package]
name = "hub_registry"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

## 📝 Structeurs de données

### Handler

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Handler {
    pub capability: String,
    pub url: String,
}
```

### Registry

```rust
pub struct Registry {
    handlers: Arc<RwLock<HashMap<String, String>>>,
}

impl Registry {
    pub fn register(&self, capability: String, url: String)
    pub fn lookup(&self, capability: &str) -> Option<String>
    pub fn list(&self) -> Vec<Handler>
}
```

## 🚨 Gestion des erreurs

### Capability non trouvée

```rust
// Retourne `not_found` au lieu de crasher
GET /lookup/unknown_capability
→ { "status": "not_found" }
```

### Enregistrement dupliqué

```rust
// Met à jour l'URL existante
POST /register avec capability déjà enregistrée
→ { "status": "registered", "action": "updated" }
```

## 📊 Monitoring

### Logs

```bash
tail -f logs/registry.log
```

### Vérifier le port

```bash
lsof -i:8081
```

### Tests intégrés

```bash
cargo test
```

## 🎯 Améliorations futures

### Court terme

1. **Persistence** : Sauvegarder en fichier JSON ou SQLite
2. **Unregistration** : Endpoint POST /unregister
3. **TTL** : Expirations automatiques des enregistrements

### Moyen terme

1. **Replication** : Plusieurs Registry en sync
2. **Watch** : Notifier Manager des changements
3. **Metrics** : Compter lookups, registrations

### Long terme

1. **Clustering** : Haute disponibilité
2. **API REST** : Documentation OpenAPI
3. **Cache invalidation** : Smart cache busting

## 📚 Voir aussi

- [Hub Manager](../manager/README.md) - Le superviseur
- [Router Service](../router/README.md) - Le routeur
- [Architecture Hub](../../../docs/hub_services.md)
- [Flux de données](../../../docs/data_flux.md)

---

### Le Registry est le cœur de la découverte de services ! 📦
