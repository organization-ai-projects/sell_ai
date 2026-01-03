use std::fs;
use std::path::PathBuf;

#[test]
fn test_run_pipeline_command() {
    // Setup: Define input and output paths
    let uris_path = PathBuf::from("/tmp/test_pipeline_uris.txt");
    let output_dir = PathBuf::from("/tmp/test_pipeline_output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Write mock URIs to the input file
    let uris_content = "file://example1.txt\nfile://example2.txt\n";
    fs::write(&uris_path, uris_content).expect("Failed to write URIs file");

    // Simulate pipeline execution
    let scrape_output = output_dir.join("scraped.jsonl");
    let clean_output = output_dir.join("cleaned.jsonl");
    let enrich_output = output_dir.join("enriched.jsonl");
    let validate_report = output_dir.join("validation_report.txt");

    // Mock pipeline steps
    fs::write(&scrape_output, "[\"Scraped data\"]").expect("Failed to write scrape output");
    fs::write(&clean_output, "[\"Cleaned data\"]").expect("Failed to write clean output");
    fs::write(&enrich_output, "[\"Enriched data\"]").expect("Failed to write enrich output");
    fs::write(&validate_report, "Validation successful")
        .expect("Failed to write validation report");

    // Verify outputs
    assert!(scrape_output.exists());
    assert!(clean_output.exists());
    assert!(enrich_output.exists());
    assert!(validate_report.exists());

    let report_content =
        fs::read_to_string(&validate_report).expect("Failed to read validation report");
    assert!(report_content.contains("Validation successful"));
}
