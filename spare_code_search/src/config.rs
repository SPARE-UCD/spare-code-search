use clap::Parser;
use std::env;

#[derive(Debug, Clone, Parser)]
pub struct AppConfig {
    #[clap(env = "LANGUAGE", default_value = "python")]
    pub language: String,
    
    #[clap(env = "STAGE", default_value = "practice")]
    pub stage: String,
    
    #[clap(env = "DATA_ROOT", default_value = "/data")]
    pub data_root: String,
    
    #[clap(env = "SAMPLES_ROOT", default_value = "/samples")]
    pub samples_root: String,
    
    #[clap(env = "PREDICTIONS_ROOT", default_value = "/predictions")]
    pub predictions_root: String,
    
    #[clap(env = "QUERIES_ROOT", default_value = "/queries")]
    pub queries_root: String,
}

#[derive(Debug, Clone)]
pub struct PreprocessorConfig {
    pub base: AppConfig,
    pub use_tokenizer: bool,
    pub model_name: String,
}

impl PreprocessorConfig {
    pub fn new(base: AppConfig) -> Self {
        Self {
            base,
            use_tokenizer: true,
            model_name: env::var("EVAL_MODEL_NAME").unwrap_or_else(|_| "Mellum".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGeneratorConfig {
    pub base: AppConfig,
    pub max_terms: usize,
    pub use_temporal_context: bool,
    pub case_sensitive: bool,
    // queries_root is in base
}

impl QueryGeneratorConfig {
    pub fn new(base: AppConfig) -> Self {
        Self {
            base,
            max_terms: 6,
            use_temporal_context: true,
            case_sensitive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub base: AppConfig,
    pub num_context_lines: usize,
    pub max_results: usize,
    pub max_retries: usize,
    pub retry_delay: f64,
    pub zoekt_url: String,
    pub max_candidates_used: usize,
}

impl SearchConfig {
    pub fn new(base: AppConfig) -> Self {
        Self {
            base,
            num_context_lines: env::var("NUM_CONTEXT_LINES").ok().and_then(|s| s.parse().ok()).unwrap_or(10), // Default value?
            max_results: env::var("MAX_RESULTS").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
            max_retries: env::var("MAX_RETRIES").ok().and_then(|s| s.parse().ok()).unwrap_or(3),
            retry_delay: env::var("RETRY_DELAY").ok().and_then(|s| s.parse().ok()).unwrap_or(0.2),
            zoekt_url: env::var("ZOEKT_URL").unwrap_or_else(|_| "http://localhost:6070/api/search".to_string()),
            max_candidates_used: env::var("MAX_CANDIDATES_USED").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostProcessorConfig {
    pub base: AppConfig,
    pub top_k_file: usize,
    pub top_k_matches: usize,
    pub merge_overlapping: bool,
    pub max_tokens: usize,
    pub max_reserved_tokens: usize,
    pub file_separator: String,
    pub num_context_lines: usize,
}

impl PostProcessorConfig {
    pub fn new(base: AppConfig) -> Self {
        Self {
            base,
            top_k_file: 1,
            top_k_matches: 5,
            merge_overlapping: true,
            max_tokens: env::var("MAX_TOKENS").ok().and_then(|s| s.parse().ok()).unwrap_or(8192),
            max_reserved_tokens: env::var("MAX_RESERVED_TOKENS").ok().and_then(|s| s.parse().ok()).unwrap_or(512),
            file_separator: env::var("FILE_SEPARATOR").unwrap_or_else(|_| "\n".to_string()), // Default?
            num_context_lines: env::var("NUM_CONTEXT_LINES").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
        }
    }
}
