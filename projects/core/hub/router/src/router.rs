use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type de message à router
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Command,
    Event,
    Query,
}

/// Requête de routage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub capability: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Réponse de routage
/// Optimisée pour éviter aller-retours inutiles:
/// - Si target_url est Some: utiliser directement
/// - Si needs_registry_lookup: Manager fait UN appel Registry avec la capability fournie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub target_url: Option<String>,
    pub needs_registry_lookup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Service de routage
/// Décide où router un message sans stocker les handlers
/// (c'est le Registry qui les stocke)
pub struct Router {
    // Cache optionnel pour éviter des lookups répétitifs
    // Non critique - peut être reconstruit
    cache: HashMap<String, String>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Router un message
    /// Retourne soit une URL en cache, soit indique qu'il faut interroger le Registry
    /// + inclut la capability pour éviter que Manager ne la reparge
    pub fn route(&self, request: &RouteRequest) -> RouteResponse {
        // Vérifier si on a l'URL en cache
        if let Some(url) = self.cache.get(&request.capability) {
            return RouteResponse {
                target_url: Some(url.clone()),
                needs_registry_lookup: false,
                capability: None,
                error: None,
            };
        }

        // Pas en cache, il faut demander au Registry via le Hub Manager
        // Inclure la capability pour que Manager n'ait pas à reparer la requête originale
        RouteResponse {
            target_url: None,
            needs_registry_lookup: true,
            capability: Some(request.capability.clone()),
            error: None,
        }
    }

    /// Mettre à jour le cache (appelé par Hub Manager après un lookup Registry)
    pub fn update_cache(&mut self, capability: String, url: String) {
        self.cache.insert(capability, url);
    }

    /// Invalider une entrée du cache
    pub fn invalidate(&mut self, capability: &str) {
        self.cache.remove(capability);
    }

    /// Vider tout le cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Obtenir les statistiques du cache
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            capabilities: self.cache.keys().cloned().collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub capabilities: Vec<String>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_without_cache() {
        let router = Router::new();
        let request = RouteRequest {
            message_type: MessageType::Command,
            capability: "invoice.create".to_string(),
            payload: serde_json::json!({}),
        };

        let response = router.route(&request);
        assert!(response.target_url.is_none());
        assert!(response.needs_registry_lookup);
    }

    #[test]
    fn test_route_with_cache() {
        let mut router = Router::new();
        router.update_cache(
            "invoice.create".to_string(),
            "http://localhost:9001".to_string(),
        );

        let request = RouteRequest {
            message_type: MessageType::Command,
            capability: "invoice.create".to_string(),
            payload: serde_json::json!({}),
        };

        let response = router.route(&request);
        assert_eq!(
            response.target_url,
            Some("http://localhost:9001".to_string())
        );
        assert!(!response.needs_registry_lookup);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut router = Router::new();
        router.update_cache(
            "invoice.create".to_string(),
            "http://localhost:9001".to_string(),
        );

        router.invalidate("invoice.create");

        let request = RouteRequest {
            message_type: MessageType::Command,
            capability: "invoice.create".to_string(),
            payload: serde_json::json!({}),
        };

        let response = router.route(&request);
        assert!(response.target_url.is_none());
        assert!(response.needs_registry_lookup);
    }
}
