use std::collections::HashMap;
use std::sync::Arc;

use crate::service_info::ServiceInfo;

pub type ProductId = &'static str;

pub trait CommandHandler: Send + Sync {
    fn handle(&self, command: String);
}

pub struct Registry {
    pub handlers: HashMap<ProductId, Arc<dyn CommandHandler>>,
    pub capability_map: HashMap<String, ServiceInfo>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            capability_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, product_id: ProductId, handler: Arc<dyn CommandHandler>) {
        self.handlers.insert(product_id, handler);
    }

    pub fn get(&self, product_id: ProductId) -> Option<Arc<dyn CommandHandler>> {
        self.handlers.get(product_id).cloned()
    }

    pub fn register_capability(&mut self, capability: String, product_id: String, url: String) {
        self.capability_map
            .insert(capability, ServiceInfo { product_id, url });
    }

    pub fn lookup_capability(&self, capability: &str) -> Option<ServiceInfo> {
        self.capability_map.get(capability).cloned()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
