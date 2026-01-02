use hub_router::router::{RouteRequest, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("🚦 Hub Router starting...");

    // Initialisation du Router
    let router = Arc::new(RwLock::new(Router::new()));

    println!("✅ Router initialized");

    // === API HTTP ===

    // Health check
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok", "service": "router"})));

    // POST /route - Router un message
    let router_clone = router.clone();
    let route = warp::path("route")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |request: RouteRequest| {
            let router = router_clone.clone();
            async move {
                let router = router.read().await;
                let response = router.route(&request);

                println!(
                    "🔀 Route request for '{}' -> needs_lookup: {}",
                    request.capability, response.needs_registry_lookup
                );

                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            }
        });

    // POST /cache/update - Mettre à jour le cache (appelé par Hub Manager)
    let router_clone = router.clone();
    let cache_update = warp::path!("cache" / "update")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let router = router_clone.clone();
            async move {
                if let (Some(capability), Some(url)) = (
                    body.get("capability").and_then(|v| v.as_str()),
                    body.get("url").and_then(|v| v.as_str()),
                ) {
                    let mut router = router.write().await;
                    router.update_cache(capability.to_string(), url.to_string());

                    println!("📝 Cache updated: {} -> {}", capability, url);

                    Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                        "status": "updated",
                        "capability": capability,
                        "url": url
                    })))
                } else {
                    Ok(warp::reply::json(&serde_json::json!({
                        "error": "Missing capability or url"
                    })))
                }
            }
        });

    // DELETE /cache/{capability} - Invalider une entrée du cache
    let router_clone = router.clone();
    let cache_invalidate = warp::path!("cache" / String)
        .and(warp::delete())
        .and_then(move |capability: String| {
            let router = router_clone.clone();
            async move {
                let mut router = router.write().await;
                router.invalidate(&capability);

                println!("🗑️  Cache invalidated: {}", capability);

                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "status": "invalidated",
                    "capability": capability
                })))
            }
        });

    // DELETE /cache - Vider tout le cache
    let router_clone = router.clone();
    let cache_clear = warp::path("cache")
        .and(warp::delete())
        .and_then(move || {
            let router = router_clone.clone();
            async move {
                let mut router = router.write().await;
                router.clear_cache();

                println!("🗑️  Cache cleared");

                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "status": "cleared"
                })))
            }
        });

    // GET /cache/stats - Statistiques du cache
    let router_clone = router.clone();
    let cache_stats = warp::path!("cache" / "stats")
        .and(warp::get())
        .and_then(move || {
            let router = router_clone.clone();
            async move {
                let router = router.read().await;
                let stats = router.cache_stats();
                Ok::<_, warp::Rejection>(warp::reply::json(&stats))
            }
        });

    // Combiner toutes les routes
    let routes = health
        .or(route)
        .or(cache_update)
        .or(cache_invalidate)
        .or(cache_clear)
        .or(cache_stats);

    println!("🌐 Router service running on http://localhost:8082");
    println!("   - Health: GET http://localhost:8082/health");
    println!("   - Route: POST http://localhost:8082/route");
    println!("   - Cache update: POST http://localhost:8082/cache/update");
    println!("   - Cache invalidate: DELETE http://localhost:8082/cache/{{capability}}");
    println!("   - Cache clear: DELETE http://localhost:8082/cache");
    println!("   - Cache stats: GET http://localhost:8082/cache/stats");

    // Démarrage du serveur HTTP
    warp::serve(routes).run(([127, 0, 0, 1], 8082)).await;
}
