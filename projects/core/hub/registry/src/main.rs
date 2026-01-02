use hub_registry::registry::Registry;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("📦 Hub Registry Service starting...");

    // Initialiser le Registry
    let registry = Arc::new(RwLock::new(Registry::new()));

    println!("✅ Registry initialized");

    // === API HTTP ===

    // Health check
    let health = warp::path("health").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "ok",
            "service": "registry"
        }))
    });

    // POST /register - Enregistrer une capacité → URL
    let registry_clone = registry.clone();
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let registry = registry_clone.clone();
            async move {
                if let Some(product_id) = body.get("product_id").and_then(|v| v.as_str()) {
                    if let Some(url) = body.get("url").and_then(|v| v.as_str()) {
                        if let Some(capabilities) =
                            body.get("capabilities").and_then(|v| v.as_array())
                        {
                            println!(
                                "📝 Enregistrement: {} -> {} ({} capacités)",
                                product_id,
                                url,
                                capabilities.len()
                            );

                            // Stocker les mappings capability → (product_id, url)
                            let mut registry_write = registry.write().await;
                            for cap in capabilities {
                                if let Some(cap_str) = cap.as_str() {
                                    registry_write.register_capability(
                                        cap_str.to_string(),
                                        product_id.to_string(),
                                        url.to_string(),
                                    );
                                    println!("  ✓ Capability '{}' registered", cap_str);
                                }
                            }
                            drop(registry_write);

                            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "status": "registered",
                                "product_id": product_id,
                                "url": url,
                                "capabilities": capabilities
                            })))
                        } else {
                            Ok(warp::reply::json(&serde_json::json!({
                                "error": "Missing capabilities"
                            })))
                        }
                    } else {
                        Ok(warp::reply::json(&serde_json::json!({
                            "error": "Missing url"
                        })))
                    }
                } else {
                    Ok(warp::reply::json(&serde_json::json!({
                        "error": "Missing product_id"
                    })))
                }
            }
        });

    // GET /lookup/{capability} - Trouver l'URL pour une capacité
    let registry_clone = registry.clone();
    let lookup =
        warp::path!("lookup" / String)
            .and(warp::get())
            .and_then(move |capability: String| {
                let registry = registry_clone.clone();
                async move {
                    println!("🔍 Lookup: {}", capability);

                    let registry_read = registry.read().await;
                    if let Some(info) = registry_read.lookup_capability(&capability) {
                        drop(registry_read);
                        Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                            "capability": capability,
                            "status": "found",
                            "product_id": info.product_id,
                            "url": info.url
                        })))
                    } else {
                        drop(registry_read);
                        Ok(warp::reply::json(&serde_json::json!({
                            "capability": capability,
                            "status": "not_found",
                            "message": "Capability not yet registered"
                        })))
                    }
                }
            });

    // GET /list - Lister tous les handlers
    let registry_clone = registry.clone();
    let list = warp::path("list").and(warp::get()).and_then(move || {
        let _registry = registry_clone.clone();
        async move {
            println!("📋 Listing handlers");

            // TODO: Implémenter le listing réel
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                "handlers": [],
                "total": 0
            })))
        }
    });

    let routes = health.or(register).or(lookup).or(list);

    println!("🌐 Registry service running on http://localhost:8081");
    println!("   - Health: GET http://localhost:8081/health");
    println!("   - Register: POST http://localhost:8081/register");
    println!("   - Lookup: GET http://localhost:8081/lookup/{{capability}}");
    println!("   - List: GET http://localhost:8081/list\n");

    // Démarrage du serveur HTTP
    warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
}
