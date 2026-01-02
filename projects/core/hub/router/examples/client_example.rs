/// Exemple d'utilisation du Router en tant que client
/// Ce fichier montre comment le Hub Manager interagit avec le Router
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test du Router Service\n");

    let client = reqwest::Client::new();
    let base_url = "http://localhost:8082";

    // 1. Health check
    println!("1️⃣  Health check...");
    let health = client
        .get(format!("{}/health", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   ✅ {}\n", health);

    // 2. Router une commande sans cache (devrait demander un lookup)
    println!("2️⃣  Route invoice.create (sans cache)...");
    let route_request = json!({
        "type": "command",
        "capability": "invoice.create",
        "payload": {"customer_id": "12345"}
    });

    let route_response = client
        .post(format!("{}/route", base_url))
        .json(&route_request)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   📡 {}\n", route_response);

    // 3. Mettre à jour le cache (simuler un lookup Registry)
    println!("3️⃣  Mise à jour du cache...");
    let cache_update = json!({
        "capability": "invoice.create",
        "url": "http://billing-service:9001"
    });

    let update_response = client
        .post(format!("{}/cache/update", base_url))
        .json(&cache_update)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   ✅ {}\n", update_response);

    // 4. Router à nouveau (devrait utiliser le cache)
    println!("4️⃣  Route invoice.create (avec cache)...");
    let route_response2 = client
        .post(format!("{}/route", base_url))
        .json(&route_request)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   📡 {}\n", route_response2);

    // 5. Statistiques du cache
    println!("5️⃣  Statistiques du cache...");
    let stats = client
        .get(format!("{}/cache/stats", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   📊 {}\n", stats);

    // 6. Invalider une entrée
    println!("6️⃣  Invalidation de invoice.create...");
    let invalidate_response = client
        .delete(format!("{}/cache/invoice.create", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   ✅ {}\n", invalidate_response);

    // 7. Vérifier que le cache est vide
    println!("7️⃣  Statistiques après invalidation...");
    let stats2 = client
        .get(format!("{}/cache/stats", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("   📊 {}\n", stats2);

    println!("✅ Test terminé avec succès !");

    Ok(())
}
