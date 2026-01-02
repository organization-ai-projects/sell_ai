use factory::{clean_stats::CleanStats, resource::Resource};

#[test]
fn test_resource_tier_assignment() {
    let mut resource = Resource {
        id: "1".to_string(),
        uri: "http://example.com".to_string(),
        fetched_at: "2026-01-02T12:00:00Z".to_string(),
        content: "This is a test content.".to_string(),
        tier: None,
    };

    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("medium"));

    resource.content = "Short".to_string();
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("low"));

    resource.content = "This content is definitely longer than fifty characters.".to_string();
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("high"));
}

#[test]
fn test_resource_tier_edge_cases() {
    let mut resource = Resource {
        id: "1".to_string(),
        uri: "http://example.com".to_string(),
        fetched_at: "2026-01-02T12:00:00Z".to_string(),
        content: "".to_string(),
        tier: None,
    };

    // Contenu vide
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("low"));

    // Contenu égal au seuil bas
    resource.content = "1234567890".to_string(); // 10 caractères
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("medium"));

    // Contenu égal au seuil haut
    resource.content = "12345678901234567890123456789012345678901234567890".to_string(); // 50 caractères
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("medium"));

    // Contenu très long
    resource.content = "a".repeat(100);
    resource.assign_tier(&[10, 50]);
    assert_eq!(resource.tier.as_deref(), Some("high"));
}

#[test]
fn test_clean_stats_tier_tracking() {
    let mut stats = CleanStats::new();

    stats.add_kept("low");
    stats.add_kept("medium");
    stats.add_kept("high");
    stats.add_kept("medium");

    assert_eq!(stats.low_tier, 1);
    assert_eq!(stats.medium_tier, 2);
    assert_eq!(stats.high_tier, 1);
    assert_eq!(stats.kept, 4);
}

#[test]
fn test_clean_stats_unknown_tier() {
    let mut stats = CleanStats::new();

    stats.add_kept("unknown");

    assert_eq!(stats.low_tier, 0);
    assert_eq!(stats.medium_tier, 0);
    assert_eq!(stats.high_tier, 0);
    assert_eq!(stats.kept, 1); // Compte toujours comme "kept"
}

#[test]
fn test_clean_stats_no_resources() {
    let stats = CleanStats::new();

    assert_eq!(stats.low_tier, 0);
    assert_eq!(stats.medium_tier, 0);
    assert_eq!(stats.high_tier, 0);
    assert_eq!(stats.kept, 0);
    assert_eq!(stats.too_short, 0);
    assert_eq!(stats.too_long, 0);
}

#[test]
fn test_filter_resources_by_tier() {
    use factory::persistence::{read_jsonl, write_jsonl};
    use factory::resource::Resource;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("resources.jsonl");
    let output_path = dir.path().join("filtered.jsonl");

    let resources = vec![
        Resource {
            id: "1".to_string(),
            uri: "http://example.com/1".to_string(),
            fetched_at: "2026-01-02T12:00:00Z".to_string(),
            content: "Short content".to_string(),
            tier: Some("low".to_string()),
        },
        Resource {
            id: "2".to_string(),
            uri: "http://example.com/2".to_string(),
            fetched_at: "2026-01-02T12:00:00Z".to_string(),
            content: "Medium content".to_string(),
            tier: Some("medium".to_string()),
        },
    ];

    write_jsonl(&input_path, &resources).unwrap();

    let filtered_resources: Vec<_> = resources
        .into_iter()
        .filter(|r| r.tier.as_deref() == Some("low"))
        .collect();

    write_jsonl(&output_path, &filtered_resources).unwrap();

    let result: Vec<Resource> = read_jsonl(&output_path).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].tier.as_deref(), Some("low"));
}
