use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod extraction;
mod models;
mod post_processor;
mod preprocessor;
mod query_generator;
mod searcher;
mod utils;

use config::*;
use models::*;
use post_processor::PostProcessor;
use preprocessor::Preprocessor;
use query_generator::ZoektQueryGenerator;
use searcher::ZoektSearchRequester;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set subscriber: {}", e);
    }

    // Load config
    let app_config = AppConfig::parse();
    
    eprintln!("Starting Spare Code Search...");
    eprintln!("Config: {:?}", app_config);

    // Derived configs
    let preprocessor_config = PreprocessorConfig::new(app_config.clone());
    let query_generator_config = QueryGeneratorConfig::new(app_config.clone());
    let search_config = SearchConfig::new(app_config.clone());
    let post_processor_config = PostProcessorConfig::new(app_config.clone());

    info!("Starting Spare Code Search with config: {:?}", app_config);

    // Initialize components
    let preprocessor = Preprocessor::new(preprocessor_config.clone());
    let query_generator = ZoektQueryGenerator::new(query_generator_config);
    let mut search_requester = ZoektSearchRequester::new(search_config);
    // PostProcessor needs a copy of preprocessor or we share it. 
    // Since Preprocessor is stateless (except config), we can clone it or make new one.
    // Our struct takes ownership.
    let post_processor = PostProcessor::new(post_processor_config, Preprocessor::new(preprocessor_config.clone()));

    // Input/Output paths
    let input_path = format!("{}/{}-{}.jsonl", app_config.data_root, app_config.language, app_config.stage);
    let output_path = format!("{}/{}-{}-predictions.jsonl", app_config.predictions_root, app_config.language, app_config.stage);
    let queries_output_path = format!("{}/{}-{}-queries.jsonl", app_config.queries_root, app_config.language, app_config.stage);

    info!("Reading from {}", input_path);
    let file = File::open(&input_path).context(format!("Failed to open input file: {}", input_path))?;
    let reader = BufReader::new(file);

    let mut predictions_file = File::create(&output_path).context("Failed to create predictions file")?;
    let mut queries_file = File::create(&queries_output_path).context("Failed to create queries file")?;

    for line in reader.lines() {
        let line = line?;
        let mut datapoint: DataPoint = serde_json::from_str(&line).context("Failed to parse DataPoint")?; // Mutable for updates

        info!("Processing datapoint {}", datapoint.id);

        // 1. Preprocess
        // In Rust, we calculate and set fields.
        let diff = preprocessor.generate_diff(&datapoint).unwrap_or_default();
        let completion_point = preprocessor.detect_completion_point(&datapoint);
        
        datapoint.diff = Some(diff);
        datapoint.completion_point = Some(completion_point);

        // 2. Generate Queries
        let queries = query_generator.construct_query_candidates_from_datapoint(&datapoint);
        let query_point = QueryPoint { candidates: queries };
        
        // Write query point
        serde_json::to_writer(&mut queries_file, &query_point)?;
        queries_file.write_all(b"\n")?;

        // 3. Search
        let search_results = search_requester.zoekt_search_on_query_point(&query_point).await;

        // 4. Postprocess
        let prediction = post_processor.postprocess(&datapoint, &search_results);

        // Write prediction
        serde_json::to_writer(&mut predictions_file, &prediction)?;
        predictions_file.write_all(b"\n")?;
    }

    info!("Finished processing.");
    info!("Successful searches: {}", search_requester.num_successful_searches);
    info!("Failed searches: {}", search_requester.num_failed_searches);

    Ok(())
}
