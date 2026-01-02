// resilience.rs - Timeouts, retries, circuit breaker (sobriété maximale)

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, timeout, Duration};

/// Configuration de résilience (pas de framework, juste des valeurs)
#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    /// Timeout par hop interne (1-2s)
    pub hop_timeout: Duration,
    /// Nombre d'erreurs avant d'ouvrir le circuit
    pub circuit_error_threshold: u32,
    /// Durée avant de retenter après ouverture du circuit
    pub circuit_retry_delay: Duration,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            hop_timeout: Duration::from_secs(2),
            circuit_error_threshold: 5,
            circuit_retry_delay: Duration::from_secs(10),
        }
    }
}

/// Circuit breaker ultra-simple
/// Trois états: Closed (OK) → Open (FAIL) → Half-Open (RETRY)
pub struct CircuitBreaker {
    error_count: AtomicU32,
    last_error_time: AtomicU64,
    state: AtomicU32, // 0=Closed, 1=Open, 2=Half-Open
    config: ResilienceConfig,
}

impl CircuitBreaker {
    pub fn new(config: ResilienceConfig) -> Self {
        Self {
            error_count: AtomicU32::new(0),
            last_error_time: AtomicU64::new(0),
            state: AtomicU32::new(0), // Closed
            config,
        }
    }

    /// Vérifier si on peut faire une requête
    pub fn can_request(&self) -> bool {
        let state = self.state.load(Ordering::SeqCst);

        match state {
            0 => true, // Closed: toujours OK
            1 => {
                // Open: vérifier si on peut passer en Half-Open
                let last_error = self.last_error_time.load(Ordering::SeqCst);
                let now = now_secs();
                let elapsed = now.saturating_sub(last_error);

                if elapsed as u64 >= self.config.circuit_retry_delay.as_secs() {
                    // Assez de temps a passé, essayer Half-Open
                    self.state.store(2, Ordering::SeqCst);
                    true
                } else {
                    false // Circuit toujours ouvert
                }
            }
            2 => true, // Half-Open: tenter une requête
            _ => false,
        }
    }

    /// Enregistrer un succès
    pub fn record_success(&self) {
        self.error_count.store(0, Ordering::SeqCst);
        self.state.store(0, Ordering::SeqCst); // Retour à Closed
    }

    /// Enregistrer une erreur
    pub fn record_error(&self) {
        let errors = self.error_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_error_time.store(now_secs(), Ordering::SeqCst);

        if errors >= self.config.circuit_error_threshold {
            self.state.store(1, Ordering::SeqCst); // Passer à Open
        }
    }

    /// État du circuit (pour debugging)
    pub fn state(&self) -> &'static str {
        match self.state.load(Ordering::SeqCst) {
            0 => "Closed (OK)",
            1 => "Open (FAIL)",
            2 => "Half-Open (RETRY)",
            _ => "Unknown",
        }
    }
}

/// Exécuter une opération avec timeout et gestion des erreurs
pub async fn with_timeout_and_retry<F, T, E>(
    operation_name: &str,
    mut operation: F,
    config: &ResilienceConfig,
) -> Result<T, String>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    // Tentative 1
    match timeout(config.hop_timeout, operation()).await {
        Ok(Ok(result)) => {
            println!("✅ {} réussi", operation_name);
            Ok(result)
        }
        Ok(Err(e)) => {
            // Erreur métier (pas un timeout)
            if is_idempotent(operation_name) {
                println!("⚠️  {} échoué ({}), retrying...", operation_name, e);
                // Attendre un peu avant retry
                sleep(Duration::from_millis(100)).await;

                // Tentative 2 (une seule!)
                match timeout(config.hop_timeout, operation()).await {
                    Ok(Ok(result)) => {
                        println!("✅ {} réussi au retry", operation_name);
                        Ok(result)
                    }
                    Ok(Err(e)) => Err(format!("{} échoué après retry: {}", operation_name, e)),
                    Err(_) => Err(format!("{} timeout au retry", operation_name)),
                }
            } else {
                Err(format!("{} échoué (non idempotent): {}", operation_name, e))
            }
        }
        Err(_) => {
            // Timeout
            println!(
                "⏱️  {} timeout, pas de retry (non idempotent)",
                operation_name
            );
            Err(format!("{} timeout", operation_name))
        }
    }
}

/// Savoir si une opération est idempotente
/// (Règle simple: GET oui, POST avec ID idempotent oui, POST création non)
fn is_idempotent(operation_name: &str) -> bool {
    operation_name.contains("GET")
        || operation_name.contains("lookup")
        || operation_name.contains("health")
        || operation_name.contains("status")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::new(ResilienceConfig::default());
        assert!(cb.can_request());
        assert_eq!(cb.state(), "Closed (OK)");
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(ResilienceConfig {
            circuit_error_threshold: 3,
            ..Default::default()
        });

        // Enregistrer 3 erreurs
        cb.record_error();
        cb.record_error();
        cb.record_error();

        // Le circuit devrait être ouvert
        assert!(!cb.can_request());
        assert_eq!(cb.state(), "Open (FAIL)");
    }

    #[test]
    fn test_circuit_breaker_success_closes_it() {
        let cb = CircuitBreaker::new(ResilienceConfig::default());
        cb.record_error();
        cb.record_error();

        // Succès
        cb.record_success();

        // Le circuit devrait être fermé
        assert!(cb.can_request());
        assert_eq!(cb.state(), "Closed (OK)");
    }

    #[tokio::test]
    async fn test_timeout_detection() {
        let config = ResilienceConfig {
            hop_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let result: Result<(), String> = with_timeout_and_retry(
            "GET /slow",
            || {
                Box::pin(async {
                    sleep(Duration::from_secs(1)).await;
                    Ok::<(), String>(())
                })
            },
            &config,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }
}
