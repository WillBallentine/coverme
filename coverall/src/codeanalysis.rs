use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

use crate::coverage;
use crate::utils::{
    self, get_parser, CSharpRegex, Lang, LangRegex, LangSettings, Method, RustRegex,
};

pub fn start_analysis(repo: utils::Command) {
    let lang_settings = create_lang_settings(&repo.lang);

    //eventually I want to return a custom error but panicing will work for now
    if !path_exists(&repo.repo) {
        panic!(
            "{} does not exist. Please check your data is correct.",
            &repo.repo
        );
    }

    let logic_methods = extract_logic_methods(&repo.repo, &lang_settings);
    let test_methods = extract_test_methods(&repo.repo, &lang_settings);
    let tested_methods = extract_tested_methods(&test_methods, &logic_methods);

    // println!("{:?}", test_methods);
    // println!("{:?}", tested_methods);

    let analysis_data = utils::AnalysisData {
        logic_methods: logic_methods,
        test_methods: test_methods,
        tested_methods: tested_methods,
    };

    coverage::generate_method_level_coverage_report(analysis_data, &lang_settings);
}

fn extract_logic_methods(repo: &String, lang_settings: &LangSettings) -> Vec<Method> {
    let mut methods = Vec::new();
    let mut parser = get_parser(&lang_settings.ext);

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry
            .path()
            .extension()
            .map_or(false, |ext| *ext == *lang_settings.ext)
        {
            if let Ok(file) = File::open(entry.path()) {
                let reader = BufReader::new(file);
                let source_code = read_to_string_buffered(reader);

                if let Some(tree) = parser.parse(&source_code, None) {
                    let root_node = tree.root_node();
                    let mut cursor = root_node.walk();

                    for node in root_node.children(&mut cursor) {
                        if is_test_method(&node, &source_code, lang_settings) {
                            continue;
                        }
                        if node.kind() == "function_item" || node.kind() == "method_declaration" {
                            let class_name = if lang_settings.uses_classes {
                                find_class_name(&node, &source_code)
                            } else {
                                String::new()
                            };
                            if let Some(identifier) = node.child_by_field_name("name") {
                                let method_name = source_code
                                    [identifier.start_byte()..identifier.end_byte()]
                                    .to_string();
                                methods.push(Method {
                                    class_name,
                                    method_name,
                                    body: extract_body(node, &source_code),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    methods
}

fn read_to_string_buffered(reader: BufReader<File>) -> String {
    let mut source_code = String::new();
    for line in reader.lines().flatten() {
        source_code.push_str(&line);
        source_code.push('\n');
    }

    source_code
}

fn find_class_name(node: &tree_sitter::Node, source: &str) -> String {
    let _cursor = node.walk();
    node.parent().map_or(String::new(), |parent| {
        if parent.kind() == "class_declaration" {
            source[parent.start_byte()..parent.end_byte()].to_string()
        } else {
            String::new()
        }
    })
}

fn extract_body(node: tree_sitter::Node, source: &str) -> Vec<String> {
    node.utf8_text(source.as_bytes())
        .unwrap_or("")
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn extract_test_methods(repo: &str, lang_settings: &LangSettings) -> Vec<Method> {
    let mut test_methods: Vec<Method> = Vec::new();
    let mut parser = get_parser(&lang_settings.ext);

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry
            .path()
            .extension()
            .map_or(false, |ext| *ext == *lang_settings.ext)
        {
            if let Ok(file) = File::open(entry.path()) {
                let mut reader = BufReader::new(file);
                let mut source_code = String::new();

                // Read file content
                reader
                    .read_to_string(&mut source_code)
                    .expect("Failed to read file");

                // Parse the source code with Tree-sitter
                //let tree = parse_with_tree_sitter(&source_code, lang_settings);
                let tree = parser.parse(&source_code, None).expect("failed to parse");
                let root_node = tree.root_node();

                // Traverse syntax tree
                let mut cursor = root_node.walk();
                for node in root_node.children(&mut cursor) {
                    //all methods are showing as false for this is_test_method call
                    //println!("{:?}", is_test_method(&node, &source_code, lang_settings));
                    if is_test_method(&node, &source_code, lang_settings) {
                        let method_name = extract_method_name(&node, &source_code);
                        test_methods.push(Method {
                            class_name: "Test".to_string(),
                            method_name,
                            body: extract_method_body(&node, &source_code),
                        });
                    }
                }
            }
        }
    }
    test_methods
}

fn extract_tested_methods(test_methods: &[Method], logic_methods: &[Method]) -> HashSet<String> {
    let mut tested_methods = HashSet::new();
    //println!("{:?}", logic_methods[0].method_name);
    let logic_method_names: HashSet<String> = logic_methods
        .iter()
        .map(|m| m.method_name.clone())
        .collect();

    for test in test_methods {
        for line in &test.body {
            let normalized_line = utils::normalize_line(line);
            if normalized_line.starts_with("assert") || normalized_line.starts_with("dbg!") {
                continue;
            }
            if let Some(pos) = normalized_line.find('(') {
                let before_paren = &normalized_line[..pos].trim();
                let called_function = if before_paren.contains("=") {
                    before_paren.split('=').last().unwrap().trim().to_string()
                } else {
                    before_paren.to_string()
                };

                if logic_method_names.contains(&called_function) {
                    tested_methods.insert(called_function.clone());
                }
            }
        }
    }
    tested_methods
}

fn is_test_method(
    node: &tree_sitter::Node,
    source_code: &str,
    lang_settings: &LangSettings,
) -> bool {
    // Ensure we're checking a function or method definition
    if node.kind() != "function_item" && node.kind() != "method_definition" {
        return false;
    }

    //println!("made it here");
    // Check preceding siblings (Tree-sitter places attributes before functions)
    let mut _cursor = node.walk();
    let mut sibling = node.prev_sibling();

    while let Some(prev) = sibling {
        let text = &source_code[prev.start_byte()..prev.end_byte()].trim();

        // Check if the text matches the test attribute pattern
        if text.contains(&lang_settings.test_pattern) {
            //println!("{:?}", text);
            return true;
        }

        // If we reach a different statement (not an attribute), stop checking
        if !text.starts_with("#") && !text.starts_with("[") {
            break;
        }

        sibling = prev.prev_sibling();
    }

    false
}

fn extract_method_name(node: &tree_sitter::Node, source_code: &str) -> String {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "identifier" {
            return source_code[child.start_byte()..child.end_byte()].to_string();
        }
    }
    "<unknown>".to_string()
}

fn extract_method_body(node: &tree_sitter::Node, source_code: &str) -> Vec<String> {
    let mut body_lines = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let body_text = &source_code[body.start_byte()..body.end_byte()];
        body_lines.extend(body_text.lines().map(String::from));
    }
    body_lines
}

fn path_exists(repo: &String) -> bool {
    Path::new(repo).exists()
}

fn create_lang_settings(lang: &Lang) -> LangSettings {
    match lang {
        Lang::Csharp => LangSettings {
            regex: LangRegex::CSharp(CSharpRegex::new()),
            ext: String::from("cs"),
            uses_classes: true,
            test_pattern: String::from("[Fact]"),
            test_method_start: String::from("Public"),
        },
        Lang::Rust => LangSettings {
            regex: LangRegex::Rust(RustRegex::new()),
            ext: String::from("rs"),
            uses_classes: false,
            test_pattern: String::from("[test]"),
            test_method_start: String::from("fn"),
        },
        Lang::Python => unimplemented!(),
        Lang::JS => unimplemented!(),
        Lang::Undefined => unimplemented!(),
    }
}
