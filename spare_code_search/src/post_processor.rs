use crate::config::PostProcessorConfig;
use crate::models::{DataPoint, Prediction};
use crate::preprocessor::Preprocessor;
use crate::searcher::ZoektResponse;
use std::path::PathBuf;
use std::fs;

pub struct PostProcessor {
    config: PostProcessorConfig,
    preprocessor: Preprocessor,
}

impl PostProcessor {
    pub fn new(config: PostProcessorConfig, preprocessor: Preprocessor) -> Self {
        Self { config, preprocessor }
    }

    pub fn compose_context(&self, file_name: &str, content: &str) -> String {
        format!("{}{}\n{}", self.config.file_separator, file_name, content)
    }

    pub fn count_tokens(&self, code: &str) -> usize {
        // Heuristic: 1 token ~= 4 chars or split by whitespace
        // For accurate results, we need a tokenizer.
        // For this migration, we'll use a simple approximation: code.len() / 4
        code.len() / 4
    }

    pub fn postprocess(&self, datapoint: &DataPoint, search_results: &ZoektResponse) -> Prediction {
        // Determine prefix and suffix
        let (mut prefix, mut suffix) = (datapoint.prefix.clone(), datapoint.suffix.clone());
        
        // This logic mimics: if self.config.use_diff_prefix and self.config.use_diff_suffix
        // The default config in Python seemed to use diff prefix/suffix by default in PostProcessorConfig (use_diff_prefix=True)
        
        if let Some(diff) = &datapoint.diff {
             let (p, s) = Preprocessor::extract_diff_prefix_and_suffix(diff);
             if !p.trim().is_empty() && !s.trim().is_empty() { // simplistic check
                 prefix = p;
                 suffix = s;
             }
        }

        let num_tokens_prefix_suffix = self.count_tokens(&format!("{}{}", prefix, suffix));
        let max_reserved = self.config.max_reserved_tokens;
        let max_total = self.config.max_tokens;
        
        // Calculate budget
        let possible_context_tokens = if max_total > num_tokens_prefix_suffix + max_reserved {
            max_total - num_tokens_prefix_suffix - max_reserved
        } else {
            0
        };

        let context_str = self.postprocess_search_results(search_results, possible_context_tokens);

        Prediction {
            context: context_str,
            prefix,
            suffix,
        }
    }

    fn postprocess_search_results(&self, search_results: &ZoektResponse, total_max_context_tokens: usize) -> String {
        if let Some(files) = &search_results.result.files {
             if files.is_empty() {
                 return String::new();
             }
             
             let max_context_tokens = total_max_context_tokens / self.config.top_k_file.max(1);
             let mut remaining_tokens = total_max_context_tokens;
             let mut processed_contexts = Vec::new();
             
             for file in files.iter().take(self.config.top_k_file) {
                 // Construct absolute path to file
                 // Python: os.path.join(self.config.data_root,f'repositories-{self.config.language}-{self.config.stage}',file['Repository'], file['FileName'])
                 let repo_dir = format!("repositories-{}-{}", self.config.base.language, self.config.base.stage);
                 let file_path = PathBuf::from(&self.config.base.data_root)
                    .join(repo_dir)
                    .join(&file.repository)
                    .join(&file.file_name);
                 
                 let file_content = match fs::read_to_string(&file_path) {
                     Ok(c) => c,
                     Err(_) => continue, // Skip if file not found
                 };
                 
                 let context_str = self.compose_context(&file.file_name, &file_content);
                 let file_tokens = self.count_tokens(&context_str);
                 
                 if file_tokens <= max_context_tokens && remaining_tokens >= file_tokens {
                     remaining_tokens -= file_tokens;
                     processed_contexts.push(context_str);
                     continue;
                 }
                 
                 // If file too big, use line matches
                 // Python logic: get_line_infos, merge_overlapping_ranges
                 let mut line_ranges = Vec::new();
                 for m in file.line_matches.iter().take(self.config.top_k_matches) {
                      let start = m.line_number.saturating_sub(self.config.num_context_lines + 1);
                      let end = m.line_number + self.config.num_context_lines;
                      line_ranges.push((start, end));
                 }
                 
                 // Merge ranges
                 // Sort
                 line_ranges.sort_by_key(|r| r.0);
                 let mut merged = Vec::new();
                 if !line_ranges.is_empty() {
                     let (mut curr_start, mut curr_end) = line_ranges[0];
                     for &(start, end) in &line_ranges[1..] {
                         if start <= curr_end + 1 {
                             curr_end = curr_end.max(end);
                         } else {
                             merged.push((curr_start, curr_end));
                             curr_start = start;
                             curr_end = end;
                         }
                     }
                     merged.push((curr_start, curr_end));
                 }
                 
                 // Extract snippets
                 // Python: lines[start_line - 1:end_line]
                 // Note: line numbers in matches might be 1-based?
                 // Python: snippet_lines = file_lines[start_line - 1:end_line]
                 
                 let lines: Vec<&str> = file_content.lines().collect();
                 
                 for (start, end) in merged {
                     // clamp
                     let start_idx = start.saturating_sub(1).min(lines.len()); 
                     let end_idx = end.min(lines.len());
                     
                     if start_idx >= end_idx { continue; }
                     
                     let snippet = lines[start_idx..end_idx].join("\n");
                     let context_str = self.compose_context(&file.file_name, &snippet);
                     let tokens = self.count_tokens(&context_str);
                     
                     if tokens <= max_context_tokens && remaining_tokens >= tokens {
                         remaining_tokens -= tokens;
                         processed_contexts.push(context_str);
                     }
                 }
             }
             
             processed_contexts.join("\n")
        } else {
            String::new()
        }
    }
}
