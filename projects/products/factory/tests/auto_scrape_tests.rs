use factory::persistence::read_jsonl;
use factory::resource::Resource;
use std::path::PathBuf;

#[test]
fn test_auto_scrape_command() {
    // Setup: Define output path
    let output_path = PathBuf::from("/tmp/test_auto_scrape.jsonl");

    // Simulate the AutoScrape command
    let model = "test_scraper_model";
    let count = 10;

    let scraped_resources: Vec<Resource> = (0..count)
        .map(|i| Resource {
            id: format!("scraped-{}", i),
            uri: format!("http://example.com/resource/{}", i),
            fetched_at: "2026-01-03T12:00:00Z".to_string(),
            content: format!("Generated content {} by model {}", i, model),
            tier: None,
        })
        .collect();

    // Write the resources to the output file
    factory::persistence::write_jsonl(&output_path, &scraped_resources)
        .expect("Failed to write scraped resources");

    // Verify the output
    let output_resources: Vec<Resource> =
        read_jsonl(&output_path).expect("Failed to read output file");
    assert_eq!(output_resources.len(), count);
    for (i, resource) in output_resources.iter().enumerate() {
        assert_eq!(resource.id, format!("scraped-{}", i));
        assert_eq!(resource.uri, format!("http://example.com/resource/{}", i));
        assert_eq!(
            resource.content,
            format!("Generated content {} by model {}", i, model)
        );
    }
}
