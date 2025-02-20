use std::collections::HashSet;
use tree_sitter::Parser;
use tree_sitter_c_sharp;
use tree_sitter_javascript;
use tree_sitter_python;
use tree_sitter_rust;

#[derive(Debug)]
pub struct Command {
    pub repo: String,
    pub lang: Lang,
}

#[derive(PartialEq, Debug)]
pub enum Lang {
    Csharp,
    Python,
    JS,
    Rust,
    Undefined,
}

//eventually I want this to carry data like file name, line number, etc
#[derive(Debug)]
pub struct Method {
    pub class_name: String,
    pub method_name: String,
    pub body: Vec<String>,
}

// pub struct Line {
//     pub number: i32,
//     pub file_name: String,
//     pub tested: bool,
// }

#[derive(Debug)]
pub struct AnalysisData {
    pub logic_methods: Vec<Method>,
    pub test_methods: Vec<Method>,
    pub tested_methods: HashSet<String>,
}

#[derive(PartialEq, Debug)]
pub struct LangSettings {
    pub ext: String,
    pub uses_classes: bool,
    pub test_pattern: String,
    pub test_method_start: String,
}

pub fn normalize_line(line: &str) -> String {
    line.replace(" ", "").replace("\t", "")
}

pub fn get_parser(lang: &str) -> Parser {
    let mut parser = Parser::new();
    match lang {
        "rs" => parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap(),
        "py" => parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap(),
        "js" => parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap(),
        "cs" => parser
            .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
            .unwrap(),
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}

#[test]
fn test_normalize_line() {
    // Test case 1: Line with spaces and tabs
    let input = "This is \t a test  line.";
    let expected = "Thisisatestline."; // Expecting all spaces and tabs removed
    let result = normalize_line(input);
    assert_eq!(result, expected);

    // Test case 2: Line with only spaces
    let input = "This is a line with spaces.";
    let expected = "Thisisalinewithspaces."; // Expecting spaces removed
    let result = normalize_line(input);
    assert_eq!(result, expected);

    // Test case 3: Line with only tabs
    let input = "This\tis\ta\tline\twith\ttabs.";
    let expected = "Thisisalinewithtabs."; // Expecting tabs removed
    let result = normalize_line(input);
    assert_eq!(result, expected);

    // Test case 4: Line with no spaces or tabs
    let input = "Thisisacleanline.";
    let expected = "Thisisacleanline."; // No changes expected
    let result = normalize_line(input);
    assert_eq!(result, expected);

    // Test case 5: Empty line
    let input = "";
    let expected = ""; // An empty string should return an empty string
    let result = normalize_line(input);
    assert_eq!(result, expected);
}
