use hub_router::router::{MessageType, RouteRequest, Router};

#[test]
fn test_router_initialization() {
    let router = Router::new();
    let stats = router.cache_stats();
    assert_eq!(stats.entries, 0);
}

#[test]
fn test_route_needs_lookup_when_no_cache() {
    let router = Router::new();

    let request = RouteRequest {
        message_type: MessageType::Command,
        capability: "invoice.create".to_string(),
        payload: serde_json::json!({"test": "data"}),
    };

    let response = router.route(&request);

    assert!(response.target_url.is_none());
    assert!(response.needs_registry_lookup);
    assert!(response.error.is_none());
}

#[test]
fn test_route_uses_cache_when_available() {
    let mut router = Router::new();

    // Ajouter une entrée dans le cache
    router.update_cache(
        "invoice.create".to_string(),
        "http://billing-service:9001".to_string(),
    );

    let request = RouteRequest {
        message_type: MessageType::Command,
        capability: "invoice.create".to_string(),
        payload: serde_json::json!({}),
    };

    let response = router.route(&request);

    assert_eq!(
        response.target_url,
        Some("http://billing-service:9001".to_string())
    );
    assert!(!response.needs_registry_lookup);
}

#[test]
fn test_cache_update_and_stats() {
    let mut router = Router::new();

    router.update_cache(
        "invoice.create".to_string(),
        "http://localhost:9001".to_string(),
    );
    router.update_cache(
        "order.create".to_string(),
        "http://localhost:9002".to_string(),
    );

    let stats = router.cache_stats();
    assert_eq!(stats.entries, 2);
    assert!(stats.capabilities.contains(&"invoice.create".to_string()));
    assert!(stats.capabilities.contains(&"order.create".to_string()));
}

#[test]
fn test_cache_invalidation() {
    let mut router = Router::new();

    router.update_cache(
        "invoice.create".to_string(),
        "http://localhost:9001".to_string(),
    );

    // Vérifier que le cache contient l'entrée
    let stats = router.cache_stats();
    assert_eq!(stats.entries, 1);

    // Invalider l'entrée
    router.invalidate("invoice.create");

    // Vérifier que le cache ne contient plus l'entrée
    let stats = router.cache_stats();
    assert_eq!(stats.entries, 0);

    // Vérifier qu'un nouveau routage nécessite un lookup
    let request = RouteRequest {
        message_type: MessageType::Command,
        capability: "invoice.create".to_string(),
        payload: serde_json::json!({}),
    };

    let response = router.route(&request);
    assert!(response.needs_registry_lookup);
}

#[test]
fn test_cache_clear() {
    let mut router = Router::new();

    router.update_cache(
        "invoice.create".to_string(),
        "http://localhost:9001".to_string(),
    );
    router.update_cache(
        "order.create".to_string(),
        "http://localhost:9002".to_string(),
    );
    router.update_cache(
        "payment.process".to_string(),
        "http://localhost:9003".to_string(),
    );

    let stats = router.cache_stats();
    assert_eq!(stats.entries, 3);

    // Vider le cache
    router.clear_cache();

    let stats = router.cache_stats();
    assert_eq!(stats.entries, 0);
}

#[test]
fn test_multiple_message_types() {
    let router = Router::new();

    let command = RouteRequest {
        message_type: MessageType::Command,
        capability: "invoice.create".to_string(),
        payload: serde_json::json!({}),
    };

    let event = RouteRequest {
        message_type: MessageType::Event,
        capability: "invoice.created".to_string(),
        payload: serde_json::json!({}),
    };

    let query = RouteRequest {
        message_type: MessageType::Query,
        capability: "invoice.get".to_string(),
        payload: serde_json::json!({}),
    };

    // Tous devraient nécessiter un lookup sans cache
    assert!(router.route(&command).needs_registry_lookup);
    assert!(router.route(&event).needs_registry_lookup);
    assert!(router.route(&query).needs_registry_lookup);
}
