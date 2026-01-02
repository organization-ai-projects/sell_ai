/// Configuration d'un service interne du hub
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub binary_name: String,
    pub port: u16,
    pub health_check_url: String,
}

impl ServiceConfig {
    pub fn new(name: &str, binary_name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            binary_name: binary_name.to_string(),
            port,
            health_check_url: format!("http://localhost:{}/health", port),
        }
    }
}