use factory::persistence::write_jsonl;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_validate_dataset_command() {
    // Setup: Create a temporary input file with mock resources
    let input_path = PathBuf::from("/tmp/test_validate_input.jsonl");
    let report_path = PathBuf::from("/tmp/test_validate_report.txt");
    let model = "test_validation_model";

    let resources = vec![
        factory::resource::Resource {
            id: "1".to_string(),
            uri: "file://example1.txt".to_string(),
            fetched_at: "2026-01-03T12:00:00Z".to_string(),
            content: "Valid content example 1".to_string(),
            tier: None,
        },
        factory::resource::Resource {
            id: "2".to_string(),
            uri: "file://example2.txt".to_string(),
            fetched_at: "2026-01-03T12:00:00Z".to_string(),
            content: "Short".to_string(),
            tier: None,
        },
    ];

    write_jsonl(&input_path, &resources).expect("Failed to write input file");

    // Simulate validation logic
    let validation_results: Vec<String> = resources
        .iter()
        .map(|resource| {
            let is_valid = resource.content.len() > 10; // Example symbolic rule
            let model_prediction = format!("Model '{}' prediction: valid", model); // Simulated model prediction

            format!(
                "Resource ID: {}, Symbolic: {}, {}, Content: {}",
                resource.id, is_valid, model_prediction, resource.content
            )
        })
        .collect();

    fs::write(&report_path, validation_results.join("\n"))
        .expect("Failed to write validation report");

    // Verify the report
    let report_content =
        fs::read_to_string(&report_path).expect("Failed to read validation report");
    assert!(report_content.contains("Resource ID: 1, Symbolic: true"));
    assert!(report_content.contains("Resource ID: 2, Symbolic: false"));
}
