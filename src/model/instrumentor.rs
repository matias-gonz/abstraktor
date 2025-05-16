use std::collections::HashMap;

use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstrumentationTargets {
    pub path: String,
    pub targets_line: Vec<usize>,
    pub targets_const: HashMap<usize, String>,
    pub targets_block: Vec<usize>,
}

pub struct Instrumentor {
    target_line_regex: Regex,
    target_const_regex: Regex,
    target_block_regex: Regex,
    block_start_regex: Regex,
}

impl Instrumentor {
    pub fn new() -> Self {
        Self {
            target_line_regex: Regex::new(r"ABSTRAKTOR_LINE").unwrap(),
            target_const_regex: Regex::new(r"ABSTRAKTOR_CONST: (\w+)").unwrap(),
            target_block_regex: Regex::new(r"ABSTRAKTOR_BLOCK_EVENT").unwrap(),
            block_start_regex: Regex::new(r"^(([a-zA-z]{1}.*)|\})").unwrap(),
        }
    }

    fn is_block_start(&self, line: &str) -> bool {
        let trimmed = line.trim_start();
        self.block_start_regex.is_match(trimmed)
    }

    pub fn get_targets(self: &Self, content: &str, path: &str) -> InstrumentationTargets {
        let mut targets = InstrumentationTargets {
            path: path.to_string(),
            ..Default::default()
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let line_num = i + 1;

            if self.target_line_regex.is_match(line) {
                targets.targets_line.push(line_num);
            }
            if self.target_const_regex.is_match(line) {
                let captures = self.target_const_regex.captures(line).unwrap();
                let const_name = captures[1].to_string();
                targets.targets_const.insert(line_num, const_name);
            }
            if self.target_block_regex.is_match(line) {
                // Look ahead for the next line that starts with a letter or closing brace
                let mut next_line_num = line_num;
                while next_line_num < lines.len() {
                    if self.is_block_start(lines[next_line_num]) {
                        targets.targets_block.push(next_line_num + 1);
                        break;
                    }
                    next_line_num += 1;
                }
            }
            i += 1;
        }
        
        targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_targets_with_no_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        let x = 1;
        ";
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert!(targets.targets_line.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_line_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_LINE
        let x = 1;
        // ABSTRAKTOR_LINE
        let y = 2;
        // ABSTRAKTOR
        let z = 3;
        ";
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_line, vec![2, 4]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);

        let expected = HashMap::from([(2, "x".to_string()), (4, "y".to_string()), (6, "z".to_string())]);
        assert_eq!(targets.targets_const, expected);
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_line_and_const_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_LINE
        let x = 1;
        // ABSTRAKTOR_CONST: y
        let y = 2;
        // ABSTRAKTOR_LINE
        let z = 3;
        ";
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);

        let expected_line = vec![2, 6];
        let expected_const = HashMap::from([(4, "y".to_string())]);
        assert_eq!(targets.targets_line, expected_line);
        assert_eq!(targets.targets_const, expected_const);
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_block_instrumentation() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT
        let x = 1;
        ";
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![3]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        let expected_block = vec![3, 7];
        let expected_const = HashMap::from([(4, "y".to_string())]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![3, 5, 7]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![4, 8]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![5, 8]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![4]);
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert!(targets.targets_block.is_empty());
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
        let path = "test.rs";
        let targets = instrumentor.get_targets(&content, &path);
        assert_eq!(targets.targets_block, vec![3]);
        assert_eq!(targets.path, path);
    }
}
