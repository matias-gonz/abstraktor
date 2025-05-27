use std::collections::HashMap;

use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstrumentationTargets {
    pub path: String,
    pub targets_const: HashMap<usize, String>,
    pub targets_block: Vec<usize>,
}

pub struct Instrumentor {
    target_const_regex: Regex,
    target_block_regex: Regex,
    block_start_regex: Regex,
}

impl Instrumentor {
    pub fn new() -> Self {
        Self {
            target_const_regex: Regex::new(r"ABSTRAKTOR_CONST: (\w+)").unwrap(),
            target_block_regex: Regex::new(r"ABSTRAKTOR_BLOCK_EVENT").unwrap(),
            block_start_regex: Regex::new(r"^(([a-zA-z]{1}.*)|\})").unwrap(),
        }
    }

    fn is_block_start(&self, line: &str) -> bool {
        let trimmed = line.trim_start();
        self.block_start_regex.is_match(trimmed)
    }

    fn find_next_block_start(&self, lines: &[&str], start_line_num: usize) -> Option<usize> {
        let mut next_line_num = start_line_num;
        while next_line_num < lines.len() {
            if self.is_block_start(lines[next_line_num]) {
                return Some(next_line_num + 1);
            }
            next_line_num += 1;
        }
        None
    }

    fn get_targets_single(&self, content: &str, path: &str) -> InstrumentationTargets {
        let mut targets = InstrumentationTargets {
            path: path.to_string(),
            ..Default::default()
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let line_num = i + 1;

            if self.target_const_regex.is_match(line) {
                let captures = self.target_const_regex.captures(line).unwrap();
                let const_name = captures[1].to_string();
                
                if let Some(block_line) = self.find_next_block_start(&lines, line_num) {
                    targets.targets_const.insert(block_line, const_name);
                }
            }
            if self.target_block_regex.is_match(line) {
                if let Some(block_line) = self.find_next_block_start(&lines, line_num) {
                    targets.targets_block.push(block_line);
                }
            }
            i += 1;
        }
        
        targets
    }

    pub fn get_targets(&self, files: Vec<(String, String)>) -> Vec<InstrumentationTargets> {
        files.into_iter()
            .map(|(content, path)| self.get_targets_single(&content, &path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_next_block_start_with_immediate_code() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["// comment", "let x = 1;", "let y = 2;"];
        let result = instrumentor.find_next_block_start(&lines, 0);
        assert_eq!(result, Some(2)); // line 2 (1-indexed)
    }

    #[test]
    fn test_find_next_block_start_with_empty_lines() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["// comment", "", "  ", "let x = 1;"];
        let result = instrumentor.find_next_block_start(&lines, 0);
        assert_eq!(result, Some(4)); // line 4 (1-indexed)
    }

    #[test]
    fn test_find_next_block_start_with_closing_brace() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["// comment", "  // another comment", "}", "let x = 1;"];
        let result = instrumentor.find_next_block_start(&lines, 0);
        assert_eq!(result, Some(3)); // line 3 (1-indexed) - the closing brace
    }

    #[test]
    fn test_find_next_block_start_no_valid_start() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["// comment", "  // another comment", "  /* block comment */"];
        let result = instrumentor.find_next_block_start(&lines, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_next_block_start_from_middle() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["let a = 1;", "// comment", "", "let x = 1;"];
        let result = instrumentor.find_next_block_start(&lines, 1); // start from line 1 (0-indexed)
        assert_eq!(result, Some(4)); // line 4 (1-indexed)
    }

    #[test]
    fn test_find_next_block_start_at_end_of_file() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["let a = 1;", "// comment"];
        let result = instrumentor.find_next_block_start(&lines, 1); // start from line 1 (0-indexed)
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_next_block_start_with_indented_code() {
        let instrumentor = Instrumentor::new();
        let lines = vec!["// comment", "    if (condition) {", "        let x = 1;"];
        let result = instrumentor.find_next_block_start(&lines, 0);
        assert_eq!(result, Some(2)); // line 2 (1-indexed) - indented code should work
    }

    #[test]
    fn test_parse_targets_with_no_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        let x = 1;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert!(targets.targets_block.is_empty());
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_const_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_CONST: x
        let x = 1;
        // ABSTRAKTOR_CONST: y
        let y = 2;
        // ABSTRAKTOR_CONST: z
        let z = 3;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);

        let expected = HashMap::from([(3, "x".to_string()), (5, "y".to_string()), (7, "z".to_string())]);
        assert_eq!(targets.targets_const, expected);
        assert!(targets.targets_block.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_block_and_const_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        let x = 1;
        // ABSTRAKTOR_CONST: y
        let y = 2;
        // ABSTRAKTOR_BLOCK_EVENT
        let z = 3;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        let expected_block = vec![3, 7];
        let expected_const = HashMap::from([(5, "y".to_string())]);
        assert_eq!(targets.targets_block, expected_block);
        assert_eq!(targets.targets_const, expected_const);
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_consecutive_block_annotations() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        let x = 1;
        // ABSTRAKTOR_BLOCK_EVENT
        let y = 2;
        // ABSTRAKTOR_BLOCK_EVENT
        let z = 3;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block, vec![3, 5, 7]);
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_empty_lines_between_blocks() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT

        let x = 1;

        // ABSTRAKTOR_BLOCK_EVENT

        let y = 2;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block, vec![4, 8]);
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_comments_between_blocks() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        // This is a comment
        // Another comment
        let x = 1;
        // ABSTRAKTOR_BLOCK_EVENT
        // Some other comment
        let y = 2;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block, vec![5, 8]);
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_block_at_end_of_file() {
        let instrumentor = Instrumentor::new();
        let content = r"
        let x = 1;
        // ABSTRAKTOR_BLOCK_EVENT
        let y = 2;
        // ABSTRAKTOR_BLOCK_EVENT
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block, vec![4]);
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_no_valid_block_start() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        // Only comments here
        // No actual code
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert!(targets.targets_block.is_empty());
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_no_valid_const_block_start() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_CONST: myconst
        // Only comments here
        // No actual code
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert!(targets.targets_block.is_empty());
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_block_starting_with_brace() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        } // end of previous block
        // ABSTRAKTOR_BLOCK_EVENT
        { // start of new block
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block, vec![3]);
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_multiple_files() {
        let instrumentor = Instrumentor::new();
        let files = vec![
            (
                r"
                // ABSTRAKTOR_BLOCK_EVENT
                let x = 1;
                // ABSTRAKTOR_CONST: y
                let y = 2;
                ".to_string(),
                "file1.c".to_string()
            ),
            (
                r"
                // ABSTRAKTOR_BLOCK_EVENT
                let z = 3;
                // ABSTRAKTOR_CONST: w
                let w = 4;
                ".to_string(),
                "file2.c".to_string()
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 2);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
        assert_eq!(targets[0].targets_block, vec![3]);
        assert_eq!(targets[0].targets_const, HashMap::from([(5, "y".to_string())]));

        // Check second file
        assert_eq!(targets[1].path, "file2.c");
        assert_eq!(targets[1].targets_block, vec![3]);
        assert_eq!(targets[1].targets_const, HashMap::from([(5, "w".to_string())]));
    }
}
