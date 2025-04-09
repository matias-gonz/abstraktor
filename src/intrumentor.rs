use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstrumentationTargets {
    pub path: String,
    pub targets_line: Vec<usize>,
}

pub struct Instrumentor {
    target_line_regex: Regex,
}

impl Instrumentor {
    pub fn new() -> Self {
        Self {
            target_line_regex: Regex::new(r"ABSTRAKTOR_LINE").unwrap(),
        }
    }

    pub fn parse_targets(self: &Self, content: &str, path: &str) -> InstrumentationTargets {
        let mut targets = InstrumentationTargets {
            path: path.to_string(),
            targets_line: Vec::new(),
        };

        for (index, line) in content.lines().enumerate() {
            let line_num = index + 1;
            if self.target_line_regex.is_match(line) {
                targets.targets_line.push(line_num);
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
        let targets = instrumentor.parse_targets(&content, &path);
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
        let targets = instrumentor.parse_targets(&content, &path);
        assert_eq!(targets.targets_line, vec![2, 4]);
        assert_eq!(targets.path, path);
    }
}
