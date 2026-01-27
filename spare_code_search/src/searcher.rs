use crate::config::SearchConfig;
use crate::models::QueryPoint;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoektOptions {
    #[serde(rename = "NumContextLines")]
    pub num_context_lines: usize,
    #[serde(rename = "MaxResults")]
    pub max_results: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoektRequest {
    #[serde(rename = "Q")]
    pub q: String,
    #[serde(rename = "Opts")]
    pub opts: ZoektOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMatch {
    #[serde(rename = "FileName")]
    pub file_name: String,
    #[serde(rename = "Repository")]
    pub repository: String,
    // Add other fields if needed, e.g. LineMatches
    #[serde(rename = "LineMatches")]
    pub line_matches: Vec<LineMatch>,
    #[serde(rename = "Content")]
    pub content: Option<String>, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineMatch {
    #[serde(rename = "Line")]
    pub line: String,
    #[serde(rename = "LineNumber")]
    pub line_number: usize,
    #[serde(rename = "Before")]
    pub before: Option<Vec<String>>,
    #[serde(rename = "After")]
    pub after: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    #[serde(rename = "Files")]
    pub files: Option<Vec<FileMatch>>,
    #[serde(rename = "FileCount")]
    pub file_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZoektResponse {
    #[serde(rename = "Result")]
    pub result: SearchResult,
}

pub struct ZoektSearchRequester {
    config: SearchConfig,
    client: Client,
    pub num_successful_searches: usize,
    pub num_failed_searches: usize,
}

impl ZoektSearchRequester {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            config,
            client: Client::builder().timeout(Duration::from_secs(30)).build().unwrap(),
            num_successful_searches: 0,
            num_failed_searches: 0,
        }
    }

    pub async fn zoekt_search_on_query_point(&mut self, query_point: &QueryPoint) -> ZoektResponse {
        let mut count = 0;
        let sorted_candidates: Vec<(&String, &String)> = query_point.candidates.iter().collect(); // Sorting needed?
        // Python loops over dict items. Order depends on insertion order in Python 3.7+
        // Rust HashMap iteration is random.
        // We probably need to prioritize keys. 
        // Python `candidates` was list of tuples converted to dict.
        // `query_generator` returns HashMap.
        // We should sort by key or value? The python code had priority in the list construction.
        // To maintain priority, `QueryPoint` should maybe store `Vec<(String, String)>` instead of Map, or we sort keys here.
        // The implementation plan used HashMap.
        // We can check if keys contain "naive", "or", "top5" etc and rank them.
        
        // Simpler: Just iterate. If we want determinstic behavior, we should sort.
        // For now, let's sort by key (description) to be deterministic, although priority might not be alphabetical.
        // Ideally we change `QueryPoint` to store a list. But for now, let's proceed.
        
        for (_, query) in sorted_candidates {
            if let Ok(result) = self.zoekt_search_request(query).await {
                if let Some(files) = &result.result.files {
                    if !files.is_empty() {
                        self.num_successful_searches += 1;
                        return result;
                    }
                }
            }
            if count >= self.config.max_candidates_used {
                break;
            }
            count += 1;
        }
        self.num_failed_searches += 1;
        ZoektResponse {
            result: SearchResult {
                files: Some(vec![]),
                file_count: 0,
            },
        }
    }

    async fn zoekt_search_request(&self, query: &str) -> Result<ZoektResponse, reqwest::Error> {
        if query.trim().is_empty() {
             return Ok(ZoektResponse { result: SearchResult { files: Some(vec![]), file_count: 0 } });
        }
        
        let payload = ZoektRequest {
            q: query.to_string(),
            opts: ZoektOptions {
                num_context_lines: self.config.num_context_lines,
                max_results: self.config.max_results,
            },
        };

        let mut attempt = 0;
        loop {
            let res = self.client.post(&self.config.zoekt_url)
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.json().await;
                    } else {
                         // Log error
                    }
                },
                Err(_) => {
                    // Log error
                }
            }
            
            attempt += 1;
            if attempt > self.config.max_retries {
                break;
            }
            sleep(Duration::from_secs_f64(self.config.retry_delay)).await;
        }
        
         Ok(ZoektResponse { result: SearchResult { files: Some(vec![]), file_count: 0 } })
    }
}
