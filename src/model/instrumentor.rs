use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstrumentationTargets {
    pub path: String,
    pub targets_const: HashMap<usize, String>,
    pub targets_block: HashMap<usize, HashMap<String, Vec<u32>>>,
    pub targets_function: HashMap<usize, HashMap<String, Vec<u32>>>,
}

pub struct Instrumentor {
    target_const_regex: Regex,
    target_block_regex: Regex,
    block_start_regex: Regex,
    target_function_regex: Regex,
}

impl Instrumentor {
    pub fn new() -> Self {
        Self {
            target_const_regex: Regex::new(r"ABSTRAKTOR_CONST: (\w+)").unwrap(),
            target_block_regex: Regex::new(r"ABSTRAKTOR_BLOCK_EVENT(?:[:\s]*(\w+(?:->\d+)*))?").unwrap(),
            block_start_regex: Regex::new(r"^(([a-zA-z]{1}.*)|\})").unwrap(),
            target_function_regex: Regex::new(r"ABSTRAKTOR_FUNC:\s*(\w+(?:->\d+)*(?:\s*,\s*\w+(?:->\d+)*)*)\s*").unwrap(),
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

            if self.target_function_regex.is_match(line){
                let captures = self.target_function_regex.captures(line).unwrap();

                let list = &captures[1]; 

                let mut map: HashMap<String, Vec<u32>> = HashMap::new();
                let regex_variables = Regex::new(r"(\w+)((?:->\d+)*)").unwrap();

                for variables_captures in regex_variables.captures_iter(list) {
                    let var_name = variables_captures[1].to_string(); 
                    
                    let mut numbers: Vec<u32> = Vec::new();
                    if let Some(numbers_match) = variables_captures.get(2){
                        
                        let numbers_str = numbers_match.as_str();
                        if !numbers_str.is_empty() {
                            numbers = numbers_str
                                .split("->")
                                .filter(|s| !s.is_empty())
                                .map(|s| s.parse::<u32>().unwrap())
                                .collect();
                        }
                    } else {}
                        
                    map.insert(var_name, numbers);
                }
                if let Some(block_line) = self.find_next_block_start(&lines, line_num) {
                    targets.targets_function.insert(block_line, map);
                }
                
            }
            if self.target_const_regex.is_match(line) {
                let captures = self.target_const_regex.captures(line).unwrap();
                let const_name = captures[1].to_string();

                if let Some(block_line) = self.find_next_block_start(&lines, line_num) {
                    targets.targets_const.insert(block_line, const_name);
                }
            }
            if self.target_block_regex.is_match(line) {
                
                let captures = self.target_block_regex.captures(line).unwrap();
                let mut var_name: String = "".to_string();
                let mut numbers: Vec<u32> = Vec::new();
                let mut map_block: HashMap<String, Vec<u32>> = HashMap::new();
                if let Some(_) = captures.get(1){
                    let list = &captures[1]; 
                    print!("{}", list);
                    let regex_variables = Regex::new(r"(\w+)((?:->\d+)*)").unwrap();
                    
                    for variables_captures in regex_variables.captures_iter(list) {
                        var_name = variables_captures[1].to_string(); 
                        numbers = Vec::new();
                        if let Some(numbers_match) = variables_captures.get(2){
                            let numbers_str = numbers_match.as_str();
                            if !numbers_str.is_empty() {
                                numbers = numbers_str
                                    .split("->")
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.parse::<u32>().unwrap())
                                    .collect();
                            }
                        }
                    }
                }
                map_block.insert(var_name, numbers);
                if let Some(block_line) = self.find_next_block_start(&lines, line_num) {
                    targets.targets_block.insert(block_line, map_block);
                }
            }
            i += 1;
        }

        targets
    }

    pub fn get_targets(&self, files: Vec<(String, String)>) -> Vec<InstrumentationTargets> {
        files
            .into_iter()
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
        let lines = vec![
            "// comment",
            "  // another comment",
            "  /* block comment */",
        ];
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

        let expected = HashMap::from([
            (3, "x".to_string()),
            (5, "y".to_string()),
            (7, "z".to_string()),
        ]);
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
        let expected_block = HashMap::from([(3_usize, HashMap::from([("".to_string(), Vec::new())])), (7_usize, HashMap::from([("".to_string(), Vec::new())]))]);
        let expected_const = HashMap::from([(5, "y".to_string())]);
        assert_eq!(targets.targets_block, expected_block);
        assert_eq!(targets.targets_const, expected_const);
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_complex_block_annotations() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT: x->4
        let x = 1;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block,HashMap::from([(3_usize, HashMap::from([("x".to_string(), vec![4])]))]));
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_with_complex_block_multiple_fields_annotations() {
        let instrumentor = Instrumentor::new();
        let content = r"
        // ABSTRAKTOR_BLOCK_EVENT: x->4->5
        let x = 1;
        ";
        let path = "test.c";
        let targets = instrumentor.get_targets_single(&content, &path);
        assert_eq!(targets.targets_block,HashMap::from([(3_usize, HashMap::from([("x".to_string(), vec![4,5])]))]));
        assert!(targets.targets_const.is_empty());
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
        assert_eq!(targets.targets_block,HashMap::from([(3_usize, HashMap::from([("".to_string(), Vec::new())])), (5_usize, HashMap::from([("".to_string(), Vec::new())])), (7_usize, HashMap::from([("".to_string(), Vec::new())]))]));
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
        assert_eq!(targets.targets_block, HashMap::from([(4_usize, HashMap::from([("".to_string(), Vec::new())])), (8_usize, HashMap::from([("".to_string(), Vec::new())]))]));
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
        assert_eq!(targets.targets_block, HashMap::from([(5_usize, HashMap::from([("".to_string(), Vec::new())])), (8_usize, HashMap::from([("".to_string(), Vec::new())]))]));
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
        assert_eq!(targets.targets_block, HashMap::from([(4_usize, HashMap::from([("".to_string(), Vec::new())]))]));
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
        assert_eq!(targets.targets_block, HashMap::from([(3_usize, HashMap::from([("".to_string(), Vec::new())]))]));
        assert!(targets.targets_const.is_empty());
        assert_eq!(targets.path, path);
    }

    #[test]
    fn test_parse_targets_one_file_func_only_one_parameter() {
        let instrumentor = Instrumentor::new();
        let files = vec![
            (
                r"
                // ABSTRAKTOR_FUNC: r
                let x = 1;
                // ABSTRAKTOR_CONST: y
                let y = 2;
                "
                .to_string(),
                "file1.c".to_string(),
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 1);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
        assert_eq!(
            targets[0].targets_const,
            HashMap::from([(5, "y".to_string())])
        );
        let inside_map: HashMap<String, Vec<u32>> = HashMap::from([
            ("r".to_string(), Vec::<u32>::new())
        ]);

        let expected: HashMap<usize, HashMap<String, Vec<u32>>> = HashMap::from([
            (3_usize, inside_map)
        ]);

        assert_eq!(targets[0].targets_function, expected);

    }

    #[test]
    fn test_parse_targets_one_file_func_only_one_parameter_optional_fields() {
        let instrumentor = Instrumentor::new();
        let files = vec![
            (
                r"
                // ABSTRAKTOR_FUNC: r->19
                let x = 1;
                // ABSTRAKTOR_CONST: y
                let y = 2;
                "
                .to_string(),
                "file1.c".to_string(),
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 1);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
        assert_eq!(
            targets[0].targets_const,
            HashMap::from([(5, "y".to_string())])
        );
        let inside_map: HashMap<String, Vec<u32>> = HashMap::from([
            ("r".to_string(), vec![19])
        ]);

        let expected: HashMap<usize, HashMap<String, Vec<u32>>> = HashMap::from([
            (3_usize, inside_map)
        ]);

        assert_eq!(targets[0].targets_function, expected);

    }

    #[test]
    fn test_parse_targets_one_file_func_multiple_parameter_optional_fields() {
        let instrumentor = Instrumentor::new();
        let files = vec![
            (
                r"
                // ABSTRAKTOR_FUNC: r->19->4->5
                let x = 1;
                // ABSTRAKTOR_CONST: y
                let y = 2;
                "
                .to_string(),
                "file1.c".to_string(),
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 1);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
        assert_eq!(
            targets[0].targets_const,
            HashMap::from([(5, "y".to_string())])
        );
        let inside_map: HashMap<String, Vec<u32>> = HashMap::from([
            ("r".to_string(), vec![19,4,5])
        ]);

        let expected: HashMap<usize, HashMap<String, Vec<u32>>> = HashMap::from([
            (3_usize, inside_map)
        ]);

        assert_eq!(targets[0].targets_function, expected);

    }

    #[test]
    fn test_parse_targets_one_file_func_multiple_parameter_optional_fields_and_multiple_variables() {
        let instrumentor = Instrumentor::new();
        let files = vec![
            (
                r"
                // ABSTRAKTOR_FUNC: r->19->4->5, r2->15
                let x = 1;
                // ABSTRAKTOR_FUNC: y2->15
                let y = 2;
                "
                .to_string(),
                "file1.c".to_string(),
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 1);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
  
        let inside_map: HashMap<String, Vec<u32>> = HashMap::from([
            ("r".to_string(), vec![19,4,5]),
            ("r2".to_string(), vec![15])
        ]);

        let second_inside_map: HashMap<String, Vec<u32>> = HashMap::from([
            ("y2".to_string(), vec![15])
        ]);

        let expected: HashMap<usize, HashMap<String, Vec<u32>>> = HashMap::from([
            (3_usize, inside_map),
            (5_usize, second_inside_map)
            
        ]);

        assert_eq!(targets[0].targets_function, expected);

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
                "
                .to_string(),
                "file1.c".to_string(),
            ),
            (
                r"
                // ABSTRAKTOR_BLOCK_EVENT
                let z = 3;
                // ABSTRAKTOR_CONST: w
                let w = 4;
                "
                .to_string(),
                "file2.c".to_string(),
            ),
        ];

        let targets = instrumentor.get_targets(files);
        assert_eq!(targets.len(), 2);

        // Check first file
        assert_eq!(targets[0].path, "file1.c");
        //assert_eq!(targets[0].targets_block, vec![3]);
        assert_eq!(
            targets[0].targets_const,
            HashMap::from([(5, "y".to_string())])
        );

        // Check second file
        assert_eq!(targets[1].path, "file2.c");
        //assert_eq!(targets[1].targets_block, vec![3]);
        assert_eq!(
            targets[1].targets_const,
            HashMap::from([(5, "w".to_string())])
        );
    }
}
