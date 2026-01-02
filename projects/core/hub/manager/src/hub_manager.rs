use std::{
    collections::HashMap,
    process::{Child, Command},
    sync::Arc,
    time::Duration,
};

use tokio::{sync::RwLock, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{
    resilience::{CircuitBreaker, ResilienceConfig},
    service_config::ServiceConfig,
};

/// Structure pour gérer les services du hub
/// Responsabilités :
/// 1. Lancer les services
/// 2. Superviser leur santé
/// 3. Les relancer si crash
/// 4. Maintenir l'état des processus
/// 5. Gérer la résilience (timeouts, retries, circuit breaker)
pub struct HubManager {
    pub services: HashMap<String, ServiceConfig>,
    pub processes: Arc<RwLock<HashMap<String, Option<Child>>>>,
    pub client: reqwest::Client,
    pub resilience_config: ResilienceConfig,
    pub circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    pub shutdown_token: CancellationToken,
}

impl HubManager {
    pub fn new() -> Self {
        let mut services = HashMap::new();

        // Configuration des services obligatoires
        services.insert(
            "registry".to_string(),
            ServiceConfig::new("Registry Service", "hub_registry", 8081),
        );
        services.insert(
            "router".to_string(),
            ServiceConfig::new("Router Service", "hub_router", 8082),
        );

        let resilience_config = ResilienceConfig::default();

        // Créer un circuit breaker pour chaque service
        let mut circuit_breakers = HashMap::new();
        for service_id in &["registry", "router"] {
            circuit_breakers.insert(
                service_id.to_string(),
                CircuitBreaker::new(resilience_config.clone()),
            );
        }

        Self {
            services,
            processes: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
            resilience_config,
            circuit_breakers: Arc::new(RwLock::new(circuit_breakers)),
            shutdown_token: CancellationToken::new(),
        }
    }

    /// Lancer tous les services
    pub async fn start_all(&self) -> anyhow::Result<()> {
        println!("🚀 Lancement des services du hub...\n");

        for (service_id, config) in &self.services {
            match self.start_service(service_id, config).await {
                Ok(_) => println!("   ✅ {} lancé sur le port {}", config.name, config.port),
                Err(e) => {
                    eprintln!("   ❌ Erreur lors du lancement de {}: {}", config.name, e);
                    return Err(e);
                }
            }
        }

        println!("\n✅ Tous les services sont opérationnels");
        Ok(())
    }

    /// Lancer un service spécifique
    async fn start_service(&self, service_id: &str, config: &ServiceConfig) -> anyhow::Result<()> {
        // Essayer de lancer le binaire avec différents chemins
        let binary_paths = vec![
            format!("./target/release/{}", config.binary_name),
            format!("target/release/{}", config.binary_name),
            config.binary_name.clone(),
        ];

        let mut child = None;

        for path in binary_paths {
            match Command::new(&path).spawn() {
                Ok(c) => {
                    child = Some(c);
                    break;
                }
                Err(_) => continue,
            }
        }

        let child = child.ok_or_else(|| {
            anyhow::anyhow!(
                "Impossible de lancer {} - binaire non trouvé. Compilez d'abord avec: cargo build --release",
                config.binary_name
            )
        })?;

        // Attendre que le service soit prêt (max 10 secondes)
        for _attempt in 0..20 {
            if self.is_service_healthy(service_id, config).await {
                self.processes
                    .write()
                    .await
                    .insert(service_id.to_string(), Some(child));
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }

        anyhow::bail!(
            "Service {} n'a pas répondu au health check après 10 secondes",
            config.name
        )
    }

    /// Vérifier la santé d'un service avec circuit breaker
    async fn is_service_healthy(&self, service_id: &str, config: &ServiceConfig) -> bool {
        // Vérifier le circuit breaker
        let breakers = self.circuit_breakers.read().await;
        let breaker = match breakers.get(service_id) {
            Some(b) => b,
            None => {
                eprintln!("❌ ERREUR: Circuit breaker introuvable pour {}", service_id);
                return false;
            }
        };

        if !breaker.can_request() {
            eprintln!(
                "⚠️  Circuit breaker pour {} est {}",
                service_id,
                breaker.state()
            );
            return false;
        }
        drop(breakers);

        // Faire le health check avec timeout
        match tokio::time::timeout(
            self.resilience_config.hop_timeout,
            self.client.get(&config.health_check_url).send(),
        )
        .await
        {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    if let Some(breaker) = self.circuit_breakers.read().await.get(service_id) {
                        breaker.record_success();
                    } else {
                        eprintln!(
                            "⚠️  Circuit breaker introuvable pour {} lors du record_success",
                            service_id
                        );
                    }
                    true
                } else {
                    if let Some(breaker) = self.circuit_breakers.read().await.get(service_id) {
                        breaker.record_error();
                    } else {
                        eprintln!(
                            "⚠️  Circuit breaker introuvable pour {} lors du record_error",
                            service_id
                        );
                    }
                    false
                }
            }
            _ => {
                if let Some(breaker) = self.circuit_breakers.read().await.get(service_id) {
                    breaker.record_error();
                } else {
                    eprintln!(
                        "⚠️  Circuit breaker introuvable pour {} lors du timeout",
                        service_id
                    );
                }
                false
            }
        }
    }

    /// Superviser les services et les relancer si nécessaire
    pub async fn supervise_services(&self) {
        println!("👀 Supervision des services activée\n");

        loop {
            // Utiliser select! pour pouvoir s'arrêter proprement
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    println!("🛑 Supervision stoppée (shutdown token activé)");
                    break;
                }
                _ = sleep(Duration::from_secs(5)) => {
                    for (service_id, config) in &self.services {
                        if !self.is_service_healthy(service_id, config).await {
                            eprintln!(
                                "\n⚠️  Service {} est DOWN - Redémarrage en cours...",
                                config.name
                            );

                            // Arrêter le processus existant
                            if let Some(process) = self.processes.write().await.remove(service_id) {
                                if let Some(mut child) = process {
                                    let _ = child.kill();
                                }
                            }

                            // Relancer le service
                            match self.start_service(service_id, config).await {
                                Ok(_) => println!("✅ {} redémarré avec succès", config.name),
                                Err(e) => eprintln!("❌ Erreur lors du redémarrage de {}: {}", config.name, e),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Obtenir l'état de tous les services
    pub async fn get_status(&self) -> serde_json::Value {
        let mut components = serde_json::json!({});

        for (service_id, config) in &self.services {
            let is_healthy = self.is_service_healthy(service_id, config).await;
            components[service_id] = serde_json::json!({
                "name": config.name,
                "port": config.port,
                "status": if is_healthy { "healthy" } else { "down" }
            });
        }

        serde_json::json!({
            "hub_manager": "running",
            "timestamp": chrono::Local::now().to_rfc3339(),
            "components": components
        })
    }

    /// Arrêter tous les services proprement
    pub async fn shutdown(&self) {
        println!("\n🛑 Arrêt des services du hub...");

        // Signaler à la supervision qu'elle doit s'arrêter
        self.shutdown_token.cancel();

        // Attendre un peu pour laisser la supervision se terminer
        sleep(Duration::from_millis(100)).await;

        // Arrêter tous les processus enfants
        for service_id in self.services.keys() {
            if let Some(process) = self.processes.write().await.remove(service_id) {
                if let Some(mut child) = process {
                    let _ = child.kill();
                }
            }
        }

        println!("✅ Tous les services ont été arrêtés proprement");
    }

    /// Router une commande via le Router Service
    pub async fn route_command(
        &self,
        message_type: &str,
        capability: &str,
        payload: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let router_url = "http://localhost:8082";

        let request = serde_json::json!({
            "type": message_type,
            "capability": capability,
            "payload": payload
        });

        let response = self
            .client
            .post(format!("{}/route", router_url))
            .json(&request)
            .timeout(self.resilience_config.hop_timeout)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Enregistrer un produit dans le Registry Service
    pub async fn register_product(
        &self,
        product_id: &str,
        url: &str,
        capabilities: Vec<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let registry_url = "http://localhost:8081";

        let request = serde_json::json!({
            "product_id": product_id,
            "url": url,
            "capabilities": capabilities
        });

        let response = self
            .client
            .post(format!("{}/register", registry_url))
            .json(&request)
            .timeout(self.resilience_config.hop_timeout)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Lookup une capacité dans le Registry Service
    pub async fn lookup_capability(&self, capability: &str) -> anyhow::Result<serde_json::Value> {
        let registry_url = "http://localhost:8081";

        let response = self
            .client
            .get(format!("{}/lookup/{}", registry_url, capability))
            .timeout(self.resilience_config.hop_timeout)
            .send()
            .await?;

        Ok(response.json().await?)
    }
}

impl Default for HubManager {
    fn default() -> Self {
        Self::new()
    }
}
