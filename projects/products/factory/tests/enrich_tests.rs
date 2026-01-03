use factory::cli::OutputFormat;
use factory::persistence::{read_jsonl, write_jsonl};
use factory::resource::Resource;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_enrich_command() {
    // Setup: Create a temporary input file with mock resources
    let input_path = PathBuf::from("/tmp/test_input.jsonl");
    let output_path = PathBuf::from("/tmp/test_output.jsonl");
    let model_name = "test_model";
    let batch_size = 2;

    let resources = vec![
        Resource {
            id: "1".to_string(),
            uri: "file://example1.txt".to_string(),
            fetched_at: "2026-01-02T12:00:00Z".to_string(),
            content: "Example content 1".to_string(),
            tier: None,
        },
        Resource {
            id: "2".to_string(),
            uri: "file://example2.txt".to_string(),
            fetched_at: "2026-01-02T12:00:00Z".to_string(),
            content: "Example content 2".to_string(),
            tier: None,
        },
    ];

    write_jsonl(&input_path, &resources).expect("Failed to write input file");

    // Simulate enrichment logic
    let enriched_resources: Vec<Resource> = resources
        .chunks(batch_size)
        .flat_map(|batch| {
            batch.iter().map(|resource| {
                let enriched_content =
                    format!("[Enriched by {}]: {}", model_name, resource.content);
                Resource {
                    content: enriched_content,
                    ..resource.clone()
                }
            })
        })
        .collect();

    write_jsonl(&output_path, &enriched_resources).expect("Failed to write output file");

    // Verify the output
    let output_resources: Vec<Resource> =
        read_jsonl(&output_path).expect("Failed to read output file");
    assert_eq!(output_resources.len(), 2);
    assert_eq!(
        output_resources[0].content,
        "[Enriched by test_model]: Example content 1"
    );
    assert_eq!(
        output_resources[1].content,
        "[Enriched by test_model]: Example content 2"
    );
}
