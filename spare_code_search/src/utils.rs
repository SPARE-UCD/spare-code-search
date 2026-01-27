use tree_sitter::{Parser, Node, Tree};
use dissimilar::Chunk;

pub fn extract_diff(incomplete_code: &str, original_code: &str) -> String {
    // Note: dissimilar::diff(a, b) calculates diff from a to b.
    // We want diff from original_code to incomplete_code?
    // Python: dmp.diff_main(original_code, incomplete_code)
    let chunks = dissimilar::diff(original_code, incomplete_code);
    
    let mut result = String::new();
    for chunk in chunks {
        match chunk {
            Chunk::Equal(_) => {},
            Chunk::Delete(s) => {
                 // Python: if diff[0] != 0. 
                 // Python diffs can be -1 (delete), 1 (insert), 0 (equal).
                 // We want to reconstruct the diff representation.
                 // The python code: diffs = "\n".join([diff[1] for diff in diffs if diff[0] != 0])
                 // It collects ALL changes (insert and delete).
                 result.push_str(s);
                 result.push('\n'); 
            },
            Chunk::Insert(s) => {
                 result.push_str(s);
                 result.push('\n'); 
            }
        }
    }
    result
}

// Tree function
pub fn code_to_tree(code: &str, language: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let lang = match language {
        "kotlin" => tree_sitter_kotlin::language(),
        "python" => tree_sitter_python::language(),
        _ => return None,
    };
    parser.set_language(&lang).ok()?;
    parser.parse(code, None)
}

pub fn rank_nodes_by_distance<'a>(nodes: Vec<Node<'a>>, completion_line: usize) -> Vec<Node<'a>> {
    let mut nodes = nodes;
    nodes.sort_by_key(|node| {
        let start_line = node.start_position().row;
        if start_line > completion_line {
             start_line - completion_line
        } else {
             completion_line - start_line
        }
    });
    nodes
}

pub fn deduplicate_nodes<'a>(nodes: Vec<Node<'a>>, source: &str) -> Vec<Node<'a>> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for node in nodes {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        if !seen.contains(text) {
            seen.insert(text.to_string());
            result.push(node);
        }
    }
    result
}

pub struct AdjustedNode<'a> {
    pub node: Node<'a>,
    pub start_offset: (usize, usize),
}

impl<'a> AdjustedNode<'a> {
    pub fn new(node: Node<'a>, start_offset: (usize, usize)) -> Self {
        Self { node, start_offset }
    }

    pub fn text(&self, source: &str) -> String {
        self.node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }
    
    // Position methods could be added here if needed
}
