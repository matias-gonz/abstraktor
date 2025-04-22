use std::collections::HashMap;

use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstrumentationTargets {
    pub path: String,
    pub targets_line: Vec<usize>,
    pub targets_const: HashMap<usize, String>,
}

pub struct Instrumentor {
    target_line_regex: Regex,
    target_const_regex: Regex,
}

impl Instrumentor {
    pub fn new() -> Self {
        Self {
            target_line_regex: Regex::new(r"ABSTRAKTOR_LINE").unwrap(),
            target_const_regex: Regex::new(r"ABSTRAKTOR_CONST: (\w+)").unwrap(),
        }
    }

    pub fn get_targets(self: &Self, content: &str, path: &str) -> InstrumentationTargets {
        let mut targets = InstrumentationTargets {
            path: path.to_string(),
            ..Default::default()
        };

        for (index, line) in content.lines().enumerate() {
            let line_num = index + 1;
            if self.target_line_regex.is_match(line) {
                targets.targets_line.push(line_num);
            }
            if self.target_const_regex.is_match(line) {
                let captures = self.target_const_regex.captures(line).unwrap();
                let const_name = captures[1].to_string();
                targets.targets_const.insert(line_num, const_name);
            }
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
    fn test_parse_targets_with_block_instrumentation() {
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
    fn test_parse_targets_with_block_and_const_instrumentation() {
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
    
}
