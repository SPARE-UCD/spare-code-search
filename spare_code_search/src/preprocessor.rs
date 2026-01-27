use crate::config::PreprocessorConfig;
use crate::models::DataPoint;
use crate::utils::{extract_diff, code_to_tree};
use std::fs;
use std::path::PathBuf;
use tree_sitter::Parser;

pub const SEPARATOR_COMMENT: &str = "\n/**<FIM>*/\n";

pub struct Preprocessor {
    config: PreprocessorConfig,
    // tokenizer: Option<Tokenizer>, // Skip for now
}

impl Preprocessor {
    pub fn new(config: PreprocessorConfig) -> Self {
        Self { config }
    }

    pub fn get_original_file_path(&self, datapoint: &DataPoint) -> PathBuf {
        let repo_dirname = format!("{}-{}-{}", "repositories", self.config.base.language, self.config.base.stage);
        let repo_subdir = format!("{}-{}", datapoint.repo.replace("/", "__"), datapoint.revision);
        
        let path = PathBuf::from(&self.config.base.data_root)
            .join(repo_dirname)
            .join(repo_subdir)
            .join(&datapoint.path);
        
        path
    }

    pub fn get_original_code(&self, datapoint: &DataPoint) -> Result<String, std::io::Error> {
        let path = self.get_original_file_path(datapoint);
        fs::read_to_string(path)
    }

    pub fn generate_incomplete_code(datapoint: &DataPoint) -> String {
        format!("{}{}{}", datapoint.prefix, SEPARATOR_COMMENT, datapoint.suffix)
    }

    pub fn generate_diff(&self, datapoint: &DataPoint) -> Result<String, std::io::Error> {
        let original_code = self.get_original_code(datapoint)?;
        let incomplete_code = Self::generate_incomplete_code(datapoint);
        Ok(extract_diff(&incomplete_code, &original_code))
    }

    pub fn detect_completion_point(&self, datapoint: &DataPoint) -> (usize, usize) {
        // Detect the completion point by taking the end point of the prefix.
        let tree_opt = code_to_tree(&datapoint.prefix, &self.config.base.language);
        if let Some(tree) = tree_opt {
             let root = tree.root_node();
             if root.child_count() == 0 {
                 let end = root.end_position();
                 return (end.row, end.column);
             }
             // The completion point is the end point of the last child node?
             // Python code: last_child = tree.root_node.children[-1]
             // return last_child.end_point
             // Actually, if we just parse the prefix, the end of the input is the completion point, 
             // but tree-sitter might have error nodes if it's incomplete. 
             // However, the python code relies on the last child.
             
             // Wait, in Rust `child_count()` returns count. `child(i)` gets it.
             if let Some(last_child) = root.child(root.child_count() - 1) {
                 let end = last_child.end_position();
                 return (end.row, end.column);
             }
        }
        
        // Fallback: line count of prefix
        let lines: Vec<&str> = datapoint.prefix.lines().collect();
        let row = lines.len().saturating_sub(1);
        let column = lines.last().map(|l| l.len()).unwrap_or(0);
        (row, column)
    }

    pub fn extract_diff_prefix_and_suffix(diff: &str) -> (String, String) {
        let parts: Vec<&str> = diff.split(SEPARATOR_COMMENT).collect();
        if parts.len() < 2 {
            (parts.get(0).unwrap_or(&"").to_string(), "".to_string())
        } else {
            (parts[0].to_string(), parts[1].to_string())
        }
    }
}
