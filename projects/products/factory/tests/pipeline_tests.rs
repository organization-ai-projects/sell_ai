#[cfg(test)]
mod tests {

    use factory::resource::scrape_resources;
    use factory::{resource::validate_resources, resource::Resource, tier::Tier};

    #[tokio::test]
    async fn test_scraping_with_real_uris() {
        let uris = vec![
            "https://jsonplaceholder.typicode.com/posts/1".to_string(),
            "https://jsonplaceholder.typicode.com/posts/2".to_string(),
        ];

        let result = scrape_resources(&uris).await;
        assert!(result.is_ok(), "Le scraping a échoué");
        let resources = result.unwrap();
        assert!(!resources.is_empty(), "Aucune ressource n'a été récupérée");
    }

    #[test]
    fn test_validation_with_real_model() {
        let resources = vec![Resource {
            id: "1".to_string(),
            uri: "https://example.com".to_string(),
            fetched_at: "2026-01-03T12:00:00Z".to_string(),
            content: "Valid content for testing.".to_string(),
            tier: Some(Tier::Medium),
        }];

        let model = "real_validation_model";
        let results = validate_resources(&resources, model);

        assert!(
            !results.is_empty(),
            "Validation results should not be empty"
        );
        assert!(
            results[0].contains("valid"),
            "Validation result should indicate validity"
        );
    }

    #[test]
    fn test_tier_enum() {
        let resource = Resource {
            id: "1".to_string(),
            uri: "https://example.com".to_string(),
            fetched_at: "2026-01-03T12:00:00Z".to_string(),
            content: "Test content".to_string(),
            tier: Some(Tier::Low),
        };
        assert_eq!(resource.tier.unwrap().as_str(), "low");
    }
}
