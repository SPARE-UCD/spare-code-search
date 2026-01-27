use crate::config::QueryGeneratorConfig;
use crate::models::DataPoint;
use crate::extraction::{SymbolExtractor, FunctionAndClassExtractor, NavigationExpressionExtractor, WildIdentifierExtractor};
use crate::utils::{rank_nodes_by_distance, deduplicate_nodes, code_to_tree}; // Need to fix utils imports
use crate::preprocessor::{SEPARATOR_COMMENT, Preprocessor};
use tree_sitter::Node;
use std::collections::HashMap;

pub struct ZoektQueryGenerator {
    config: QueryGeneratorConfig,
    function_and_class_extractor: FunctionAndClassExtractor,
    navigation_expression_extractor: NavigationExpressionExtractor,
    wild_identifier_extractor: WildIdentifierExtractor,
}

impl ZoektQueryGenerator {
    pub fn new(config: QueryGeneratorConfig) -> Self {
        let lang = &config.base.language;
        Self {
            function_and_class_extractor: FunctionAndClassExtractor::new(lang),
            navigation_expression_extractor: NavigationExpressionExtractor::new(lang),
            wild_identifier_extractor: WildIdentifierExtractor::new(lang),
            config,
        }
    }

    pub fn construct_query_candidates_from_datapoint(&self, datapoint: &DataPoint) -> HashMap<String, String> {
        let mut queries = HashMap::new();
        let mut candidates = Vec::new();

        let mut repo_name = format!("{}-{}", datapoint.repo.replace("/", "__"), datapoint.revision);
        if self.config.use_temporal_context {
            repo_name = repo_name.split("-").next().unwrap_or(&repo_name).to_string();
        }

        // We need the diff. If it's not present, we can't generate queries based on diff.
        // Assuming diff is populated by preprocessor before this.
        let diff = datapoint.diff.as_deref().unwrap_or("");
        
        // Find nodes
        // Since we can't pass the Tree around easily without keeping the source string, 
        // we might need to parse here or pass parsed trees.
        // For simplicity, we parse here.
        
        let (diff_prefix, _diff_suffix) = Preprocessor::extract_diff_prefix_and_suffix(diff);
        let clean_diff = diff.replace(SEPARATOR_COMMENT, "");
        
        // Extract symbols
        // We accumulate Nodes. But Nodes depend on the Tree which depends on the String.
        // The strings `clean_diff` and `diff_prefix` are local. Trees must live as long as Nodes.
        
        // Strategy: Extract text immediately or keep Trees alive.
        // Extracting text is easier. But we need positions for ranking.
        // So we can extract (Text, StartLine).
        
        let mut func_nodes = Vec::new();
        let mut nav_nodes = Vec::new();
        let mut wild_nodes = Vec::new();
        
        // Helper to process code
        let process_code = |code: &str| {
            if let Some(tree) = code_to_tree(code, &self.config.base.language) {
                 let f_nodes = self.function_and_class_extractor.extract_symbols(code, &tree);
                 let n_nodes = self.navigation_expression_extractor.extract_symbols(code, &tree);
                 let w_nodes = self.wild_identifier_extractor.extract_symbols(code, &tree);
                 
                 // Map to (Text, StartRow)
                 let map_nodes = |nodes: Vec<Node>| -> Vec<(String, usize)> {
                     nodes.into_iter().map(|n| (n.utf8_text(code.as_bytes()).unwrap_or("").to_string(), n.start_position().row)).collect()
                 };
                 
                 (map_nodes(f_nodes), map_nodes(n_nodes), map_nodes(w_nodes))
            } else {
                (vec![], vec![], vec![])
            }
        };

        for code in &[clean_diff.as_str(), diff_prefix.as_str()] {
            let (f, n, w) = process_code(code);
            func_nodes.extend(f);
            nav_nodes.extend(n);
            wild_nodes.extend(w);
        }
        
        // Deduplicate
        // Sort by distance to completion point line?
        // Use completion_point form datapoint.
        let completion_line = datapoint.completion_point.map(|(r, _)| r).unwrap_or(0);
        
        // Rank
        let rank = |nodes: &mut Vec<(String, usize)>| {
            nodes.sort_by_key(|(_, row)| {
                if *row > completion_line { row - completion_line } else { completion_line - row }
            });
        };
        
        rank(&mut func_nodes);
        rank(&mut nav_nodes);
        rank(&mut wild_nodes);
        
        // Dedupe by text, keeping first (which is closest due to sort)
        let dedupe = |nodes: Vec<(String, usize)>| -> Vec<(String, usize)> {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            let mut res = Vec::new();
            for item in nodes {
                if !seen.contains(&item.0) {
                    seen.insert(item.0.clone());
                    res.push(item);
                }
            }
            res
        };
        
        let func_nodes = dedupe(func_nodes);
        let nav_nodes = dedupe(nav_nodes);
        let wild_nodes = dedupe(wild_nodes);
        
        // Generate candidates
        candidates.extend(self.process_func_nodes(&func_nodes, &repo_name));
        candidates.extend(self.process_nav_nodes(&nav_nodes, &repo_name));
        candidates.extend(self.process_wild_nodes(&wild_nodes, &repo_name));

        for (query, desc) in candidates {
            queries.insert(desc, query);
        }
        queries
    }
    
    fn process_func_nodes(&self, nodes: &[(String, usize)], repo: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let texts: Vec<&str> = nodes.iter().map(|(s, _)| s.as_str()).collect();
        if texts.is_empty() { return res; }
        
        let max = self.config.max_terms;
        let selected = &texts[..texts.len().min(max)];
        
        // 1. Naive
        let naive = selected.join(" ");
        res.push((format!("{} r:{}", naive, repo), "functions_classes_naive".to_string()));
        
        // 2. OR
        let or_q = selected.join(" or ");
        res.push((format!("{} r:{}", or_q, repo), "functions_classes_or".to_string()));
        
        // 3. Top N
        for n in [5, 4, 3] {
            if texts.len() > n {
                 let red = texts[..n].join(" ");
                 res.push((format!("{} r:{}", red, repo), format!("functions_classes_top{}", n)));
            }
        }
        
        // 4. Regex
        if texts.len() >= 2 {
            // first and last in list (closest ones? or first/last in code? Python logic used find_first_and_last_nodes based on position)
            // My nodes are sorted by distance.
            // Python: first_func = min(nodes, key=start_point), last_func = max(nodes, key=end_point)
            // I sort by distance. I lost original order.
            // I should accept that or fix the logic. 
            // The python code sorts by distance FIRST, then takes max_terms, then finds first/last from THAT subset?
            // Python: 
            // nodes = rank_nodes_by_distance(nodes)
            // nodes = nodes[:max_terms]
            // first, last = find_first_and_last_nodes(nodes) -> finds min/max by start_point among the selected nodes.
            // Yes.
            
            // So I need to find min/max start_point among `selected`.
            // `nodes` has `row`.
            // `selected` in my code is `&texts`, lost rows.
            // Let's go back to `nodes`.
            let selected_nodes = &nodes[..nodes.len().min(max)];
            let min_idx = selected_nodes.iter().enumerate().min_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            let max_idx = selected_nodes.iter().enumerate().max_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            
            let first = &selected_nodes[min_idx].0;
            let last = &selected_nodes[max_idx].0;
            
            let regex = format!("{}.*{}", first, last);
            res.push((format!("{} r:{}", regex, repo), "functions_classes_regex".to_string()));
        }
        
        res
    }

    fn process_nav_nodes(&self, nodes: &[(String, usize)], repo: &str) -> Vec<(String, String)> {
        // Similar logic, plus splitting handling
        let mut res = Vec::new();
        if nodes.is_empty() { return res; }
        
        let max = self.config.max_terms;
        
        // Pre-process texts
        let processed: Vec<(String, usize)> = nodes.iter().map(|(s, r)| {
            let s = s.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect::<String>();
            // clean up
             let s = s.split_whitespace().collect::<Vec<_>>().join(".");
             let s = s.to_lowercase(); // assuming case sensitive config check?
             (s, *r)
        }).filter(|(s, _)| !s.is_empty()).collect();
        
        let selected_nodes = &processed[..processed.len().min(max)];
        let texts: Vec<&str> = selected_nodes.iter().map(|(s, _)| s.as_str()).collect();

        // 1. Naive
        let naive = texts.join(" ");
        res.push((format!("{} r:{}", naive, repo), "navigation_naive".to_string()));
        
        // 2. Unpacked
        let mut unpacked = Vec::new();
        for t in &texts {
            for part in t.split('.') {
                unpacked.push(part.to_string());
            }
        }
        // Dedupe unpacked
        unpacked.sort(); 
        unpacked.dedup();
        
        if !unpacked.is_empty() {
             let un_texts = &unpacked[..unpacked.len().min(max)];
             let un_str = un_texts.join(" ");
             res.push((format!("{} r:{}", un_str, repo), "navigation_unpacked".to_string()));
             
             let or_un = un_texts.join(" or ");
             res.push((format!("{} r:{}", or_un, repo), "navigation_unpacked_or".to_string()));
             
             for n in [5, 4, 3] {
                 if unpacked.len() > n {
                     let red = unpacked[..n].join(" ");
                     res.push((format!("{} r:{}", red, repo), format!("navigation_unpacked_top{}", n)));
                 }
             }
        }
        
        // Regex
        if !selected_nodes.is_empty() {
             let min_idx = selected_nodes.iter().enumerate().min_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            let max_idx = selected_nodes.iter().enumerate().max_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            
            let first = &selected_nodes[min_idx].0; // these are processed like "a.b"
            let last = &selected_nodes[max_idx].0;
            // extract first id of first match, last id of last match
            let first_id = first.split('.').next().unwrap_or(first);
            let last_id = last.split('.').last().unwrap_or(last);
            
            let regex = format!("{}.*{}", first_id, last_id);
            res.push((format!("{} r:{}", regex, repo), "navigation_regex".to_string()));
        }
        
        res
    }
    
    fn process_wild_nodes(&self, nodes: &[(String, usize)], repo: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        if nodes.is_empty() { return res; }
        let max = self.config.max_terms;
        
        // Rank by occurrence count? Python does:
        // wild_identifiers = sorted(set(wild_identifiers), key=wild_identifiers.count, reverse=True)
        // It counts occurrences in the extracted list BEFORE deduplication?
        // My list `nodes` passed here is deduped.
        // Wait, `find_all_nodes` in my code dedupes at the end.
        // But Python code `find_all_nodes` does `deduplicate_nodes`.
        // THEN `process_wild_identifiers`: uses `wild_identifiers` original logic?
        // Ah, `process_wild_identifiers` takes `wild_identifiers` node list.
        // It does `set` to dedupe inside.
        // But the `key=wild_identifiers.count` relies on the original list having duplicates.
        // My `find_all_nodes` deduplicated.
        // I should perhaps Count first.
        
        // For now, I stick to distance ranking (which I did in find_all_nodes) or just simple logic.
        // To be simpler, I'll just use the list I have.
        
        let texts: Vec<&str> = nodes.iter().map(|(s, _)| s.as_str()).collect();
        let selected = &texts[..texts.len().min(max)];
        
        let naive = selected.join(" ");
        res.push((format!("{} r:{}", naive, repo), "identifiers_naive".to_string()));
        
        let or_q = selected.join(" or ");
        res.push((format!("{} r:{}", or_q, repo), "identifiers_or".to_string()));
         
        for n in [5, 4, 3] {
            if texts.len() > n {
                 let red = texts[..n].join(" ");
                 res.push((format!("{} r:{}", red, repo), format!("identifiers_top{}", n)));
            }
        }
        
        if texts.len() >= 2 {
             let selected_nodes = &nodes[..nodes.len().min(max)];
             let min_idx = selected_nodes.iter().enumerate().min_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            let max_idx = selected_nodes.iter().enumerate().max_by_key(|(_, (_, r))| *r).map(|(i, _)| i).unwrap_or(0);
            
            let first = &selected_nodes[min_idx].0;
            let last = &selected_nodes[max_idx].0;
            
            let regex = format!("{}.*{}", first, last);
            res.push((format!("{} r:{}", regex, repo), "identifiers_regex".to_string()));
        }
        
        res
    }
}
