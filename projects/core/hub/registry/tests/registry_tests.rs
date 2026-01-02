use hub_registry::registry::{CommandHandler, Registry};
use std::sync::Arc;

struct TestHandler;

impl CommandHandler for TestHandler {
    fn handle(&self, command: String) {
        println!("Handling command: {}", command);
    }
}

#[test]
fn test_registry_register_and_get() {
    let mut registry = Registry::new();

    let handler: Arc<dyn CommandHandler> = Arc::new(TestHandler);
    registry.register("test_product", handler.clone());

    let retrieved_handler = registry.get("test_product");
    assert!(retrieved_handler.is_some());
    assert!(Arc::ptr_eq(&retrieved_handler.unwrap(), &handler));
}

#[test]
fn test_registry_get_nonexistent() {
    let registry = Registry::new();

    let retrieved_handler = registry.get("nonexistent_product");
    assert!(retrieved_handler.is_none());
}
