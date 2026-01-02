use hub_manager::hub_manager::HubManager;
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║   🚀 Hub Manager - Gateway Principal   ║");
    println!("║   Orchestrateur et Superviseur du Hub  ║");
    println!("╚════════════════════════════════════════╝\n");

    let manager = Arc::new(HubManager::new());

    // Lancer tous les services
    manager.start_all().await?;

    // Lancer la supervision en arrière-plan
    let manager_supervision = manager.clone();
    tokio::spawn(async move {
        manager_supervision.supervise_services().await;
    });

    // === API HTTP - Point d'entrée unique ===

    let manager_health = manager.clone();
    let health = warp::path("health").and(warp::get()).map(move || {
        let _ = &manager_health; // Utiliser la variable pour éviter le warning
        warp::reply::json(&serde_json::json!({
            "status": "ok",
            "service": "hub_manager"
        }))
    });

    let manager_status = manager.clone();
    let status = warp::path("status").and(warp::get()).and_then(move || {
        let manager = manager_status.clone();
        async move {
            let status = manager.get_status().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&status))
        }
    });

    // POST /command - Recevoir une commande d'un client
    // (À implémenter: router vers Router puis produit)
    let manager_command = manager.clone();
    let command = warp::path("command")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let manager = manager_command.clone();
            async move {
                println!("📨 Commande reçue: {:?}", body);

                // Extraire les données de la requête
                let message_type = body
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("command");
                let capability = body
                    .get("capability")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let payload = body
                    .get("payload")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                // 1. Router la commande via Router Service
                match manager
                    .route_command(message_type, capability, payload.clone())
                    .await
                {
                    Ok(route_response) => {
                        println!("🔀 Route response: {:?}", route_response);

                        // Vérifier si on a une URL directe
                        if let Some(target_url) =
                            route_response.get("target_url").and_then(|v| v.as_str())
                        {
                            // Envoyer directement au produit
                            println!("✅ Found direct URL: {}", target_url);
                            return Ok::<_, warp::Rejection>(warp::reply::json(
                                &serde_json::json!({
                                    "status": "routed",
                                    "target": target_url,
                                    "capability": capability,
                                    "message": "Command routed to product"
                                }),
                            ));
                        }

                        // 2. Si besoin d'un lookup Registry
                        if route_response
                            .get("needs_registry_lookup")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                        {
                            println!("🔍 Need Registry lookup for: {}", capability);

                            match manager.lookup_capability(capability).await {
                                Ok(lookup_result) => {
                                    println!("📦 Lookup result: {:?}", lookup_result);
                                    if let Some(url) =
                                        lookup_result.get("url").and_then(|v| v.as_str())
                                    {
                                        println!("✅ Found product URL from Registry: {}", url);

                                        // Construire la requête pour le produit
                                        let product_request = serde_json::json!({
                                            "capability": capability,
                                            "code": body.get("payload")
                                                .and_then(|p| p.get("code"))
                                                .and_then(|c| c.as_str()),
                                            "error": body.get("payload")
                                                .and_then(|p| p.get("error"))
                                                .and_then(|e| e.as_str()),
                                            "description": body.get("payload")
                                                .and_then(|p| p.get("description"))
                                                .and_then(|d| d.as_str())
                                        });

                                        // Déterminer l'endpoint selon la capability
                                        let endpoint = match capability {
                                            "debug_rust_code" => "debug",
                                            "improve_rust_code" => "improve",
                                            "fix_rust_error" => "fix",
                                            "generate_rust_code" => "generate",
                                            _ => "command"
                                        };

                                        let target_url = format!("{}/{}", url, endpoint);
                                        println!("📤 Forwarding to: {}", target_url);

                                        // Appeler le produit
                                        let client = reqwest::Client::new();
                                        match client.post(&target_url)
                                            .json(&product_request)
                                            .timeout(std::time::Duration::from_secs(5))
                                            .send()
                                            .await
                                        {
                                            Ok(response) => {
                                                match response.json::<serde_json::Value>().await {
                                                    Ok(product_response) => {
                                                        println!("✅ Product response received");
                                                        return Ok(warp::reply::json(&product_response));
                                                    }
                                                    Err(e) => {
                                                        eprintln!("❌ Failed to parse product response: {}", e);
                                                        return Ok(warp::reply::json(&serde_json::json!({
                                                            "status": "error",
                                                            "error": format!("Product response parsing failed: {}", e)
                                                        })));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("❌ Failed to call product: {}", e);
                                                return Ok(warp::reply::json(&serde_json::json!({
                                                    "status": "error",
                                                    "error": format!("Product call failed: {}", e)
                                                })));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Registry lookup failed: {}", e);
                                    return Ok(warp::reply::json(&serde_json::json!({
                                        "status": "error",
                                        "error": format!("Registry lookup failed: {}", e),
                                        "capability": capability
                                    })));
                                }
                            }
                        }

                        // 3. Pas de route trouvée
                        Ok(warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "error": "No route found for capability",
                            "capability": capability
                        })))
                    }
                    Err(e) => {
                        eprintln!("❌ Router error: {}", e);
                        Ok(warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "error": format!("Router error: {}", e),
                            "capability": capability
                        })))
                    }
                }
            }
        });

    // POST /register - Enregistrer un produit
    let manager_register = manager.clone();
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |body: serde_json::Value| {
            let manager = manager_register.clone();
            async move {
                println!("📦 Produit à enregistrer: {:?}", body);

                // Extraire les données du produit
                let product_id = match body.get("product_id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => {
                        return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "error": "Missing product_id"
                        })))
                    }
                };

                let url = match body.get("url").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => {
                        return Ok(warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "error": "Missing url"
                        })))
                    }
                };

                let capabilities: Vec<String> = body
                    .get("capabilities")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|cap| cap.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();

                if capabilities.is_empty() {
                    return Ok(warp::reply::json(&serde_json::json!({
                        "status": "error",
                        "error": "No capabilities provided"
                    })));
                }

                // Enregistrer via Registry
                match manager
                    .register_product(&product_id, &url, capabilities.clone())
                    .await
                {
                    Ok(registry_response) => {
                        println!("✅ Product {} registered successfully", product_id);
                        println!("   URL: {}", url);
                        println!("   Capabilities: {:?}", capabilities);
                        Ok(warp::reply::json(&serde_json::json!({
                            "status": "registered",
                            "product_id": product_id,
                            "url": url,
                            "capabilities": capabilities,
                            "registry_response": registry_response
                        })))
                    }
                    Err(e) => {
                        eprintln!("❌ Registration error: {}", e);
                        Ok(warp::reply::json(&serde_json::json!({
                            "status": "error",
                            "error": format!("Registration failed: {}", e),
                            "product_id": product_id
                        })))
                    }
                }
            }
        });

    let routes = health.or(status).or(command).or(register);

    println!("🌐 Hub Manager démarré sur http://localhost:8080");
    println!("   - Health: GET http://localhost:8080/health");
    println!("   - Status: GET http://localhost:8080/status");
    println!("   - Command: POST http://localhost:8080/command");
    println!("   - Register: POST http://localhost:8080/register");
    println!("\n💡 Services supervisés:");
    for config in manager.services.values() {
        println!("   - {} (port {})", config.name, config.port);
    }
    println!("\nAppuyez sur Ctrl+C pour arrêter\n");

    // Gérer Ctrl+C proprement
    let manager_shutdown = manager.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        manager_shutdown.shutdown().await;
        std::process::exit(0);
    });

    // Démarrer le serveur HTTP
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}
