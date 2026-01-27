use tree_sitter::{Node, Query, QueryCursor, Tree, Parser};
use crate::utils::code_to_tree;

pub trait SymbolExtractor {
    fn extract_symbols<'a>(&self, code: &'a str, tree: &'a Tree) -> Vec<Node<'a>>;
}

pub struct FunctionAndClassExtractor {
    query: Query,
}

impl FunctionAndClassExtractor {
    pub fn new(language: &str) -> Self {
        let query_str = match language {
            "kotlin" => include_str!("queries/kotlin/function_and_class_names.scm"),
            "python" => include_str!("queries/python/function_and_class_names.scm"),
            _ => "",
        };
        let lang = match language {
            "kotlin" => tree_sitter_kotlin::language(),
            "python" => tree_sitter_python::language(),
            _ => panic!("Unsupported language"),
        };
        let query = Query::new(&lang, query_str).expect("Failed to parse query");
        Self { query }
    }
}

impl SymbolExtractor for FunctionAndClassExtractor {
    fn extract_symbols<'a>(&self, _code: &'a str, tree: &'a Tree) -> Vec<Node<'a>> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), _code.as_bytes());
        
        let mut nodes = Vec::new();
        for m in matches {
            for capture in m.captures {
                 nodes.push(capture.node);
            }
        }
        nodes
    }
}

pub struct NavigationExpressionExtractor {
    query: Query,
}

impl NavigationExpressionExtractor {
    pub fn new(language: &str) -> Self {
        let query_str = match language {
            "kotlin" => include_str!("queries/kotlin/navigation_expression.scm"),
            "python" => include_str!("queries/python/navigation_expression.scm"),
            _ => "",
        };
        let lang = match language {
            "kotlin" => tree_sitter_kotlin::language(),
            "python" => tree_sitter_python::language(),
            _ => panic!("Unsupported language"),
        };
        let query = Query::new(&lang, query_str).expect("Failed to parse query");
        Self { query }
    }
}

impl SymbolExtractor for NavigationExpressionExtractor {
    fn extract_symbols<'a>(&self, _code: &'a str, tree: &'a Tree) -> Vec<Node<'a>> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), _code.as_bytes());
        let mut nodes = Vec::new();
        for m in matches {
            for capture in m.captures {
                 nodes.push(capture.node);
            }
        }
        nodes
    }
}

pub struct WildIdentifierExtractor {
    query: Query,
}

impl WildIdentifierExtractor {
    pub fn new(language: &str) -> Self {
        let query_str = match language {
            "kotlin" => include_str!("queries/kotlin/wild_identifiers.scm"),
            "python" => include_str!("queries/python/wild_identifiers.scm"),
            _ => "",
        };
        let lang = match language {
            "kotlin" => tree_sitter_kotlin::language(),
            "python" => tree_sitter_python::language(),
            _ => panic!("Unsupported language"),
        };
        let query = Query::new(&lang, query_str).expect("Failed to parse query");
        Self { query }
    }
}

impl SymbolExtractor for WildIdentifierExtractor {
    fn extract_symbols<'a>(&self, _code: &'a str, tree: &'a Tree) -> Vec<Node<'a>> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), _code.as_bytes());
        let mut nodes = Vec::new();
        for m in matches {
            for capture in m.captures {
                 nodes.push(capture.node);
            }
        }
        nodes
    }
}
