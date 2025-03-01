use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

use crate::coverage;
use crate::csharp::{
    extract_csharp_assert_targets, extract_csharp_method_calls, traverse_c_sharp_nodes,
};
use crate::utils::{self, extract_body, get_parser, Lang, LangSettings, Method};

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
    let tested_methods = extract_tested_methods(&logic_methods, &lang_settings);

    coverage::generate_method_level_coverage_report(logic_methods, tested_methods, &lang_settings);
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

                    if lang_settings.ext == "cs" {
                        traverse_c_sharp_nodes(
                            root_node,
                            &mut cursor,
                            &source_code,
                            lang_settings,
                            &mut methods,
                        );
                    } else {
                        for node in root_node.children(&mut cursor) {
                            let mut test = false;
                            if is_test_method(&node, &source_code, lang_settings) {
                                test = true;
                            }
                            if node.kind() == "function_item" || node.kind() == "method_declaration"
                            {
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
                                        is_test: test,
                                    });
                                }
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

fn extract_tested_methods(logic_methods: &[Method], lang_settings: &LangSettings) -> Vec<String> {
    let mut tested_methods = Vec::new();
    //println!("{:?}", logic_methods[0].method_name);
    let logic_method_names: HashSet<String> = logic_methods
        .iter()
        .map(|m| m.method_name.clone())
        .collect();

    for method in logic_methods {
        if method.is_test == true {
            for line in &method.body {
                let normalized_line = utils::normalize_line(line);

                if lang_settings.ext == "cs" {
                    if normalized_line.contains("Assert.") {
                        extract_csharp_assert_targets(
                            &line,
                            &logic_method_names,
                            &mut tested_methods,
                        );
                    } else {
                        extract_csharp_method_calls(
                            &line,
                            &logic_method_names,
                            &mut tested_methods,
                        );
                    }
                }

                if let Some(start) = normalized_line.find('!') {
                    let macro_name = &normalized_line[..start];

                    if ["assert", "assert_eq", "assert_ne", "assert_matches"].contains(&macro_name)
                    {
                        // Extract arguments inside the macro
                        if let Some(args_start) = normalized_line.find('(') {
                            if let Some(args_end) = normalized_line.rfind(')') {
                                let args = &normalized_line[args_start + 1..args_end];

                                // Split arguments by comma and check each one for function calls
                                for arg in args.split(',') {
                                    let called_function = arg
                                        .trim()
                                        .split('(')
                                        .next()
                                        .unwrap_or("")
                                        .trim()
                                        .to_string();

                                    if logic_method_names.contains(&called_function) {
                                        tested_methods.push(called_function)
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(pos) = normalized_line.find('(') {
                    let before_paren = &normalized_line[..pos].trim();
                    let called_function = if before_paren.contains("=") {
                        before_paren.split('=').last().unwrap().trim().to_string()
                    } else {
                        before_paren.to_string()
                    };

                    if logic_method_names.contains(&called_function) {
                        tested_methods.push(called_function);
                    }
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

fn path_exists(repo: &String) -> bool {
    Path::new(repo).exists()
}

fn create_lang_settings(lang: &Lang) -> LangSettings {
    match lang {
        Lang::Csharp => LangSettings {
            ext: String::from("cs"),
            uses_classes: true,
            test_pattern: String::from("[Fact]"),
            test_method_start: String::from("Public"),
        },
        Lang::Rust => LangSettings {
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

#[test]
fn test_create_lang_settings_rust() {
    let result = create_lang_settings(&Lang::Rust);
    let expected = LangSettings {
        ext: String::from("rs"),
        uses_classes: false,
        test_pattern: String::from("[test]"),
        test_method_start: String::from("fn"),
    };
    assert_eq!(result, expected);
}

#[test]
fn test_path_exists_existing_path() {
    use std::fs::{self};

    let repo = String::from("tests/existing_dir");
    fs::create_dir_all(&repo).expect("Failed to create test dir");
    assert!(path_exists(&repo));
    fs::remove_dir_all(&repo).expect("Failed to clean up test repo");
}

#[test]
fn test_path_exists_non_existing_path() {
    let repo = String::from("tests/non_existing_dir");
    assert!(!path_exists(&repo));
}

#[test]
fn test_extract_logic_methods() {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("example.rs");

    // Write mock Rust code to the temp file
    let mut file = File::create(&file_path).expect("Failed to create temp file");
    writeln!(
        file,
        r#"
        fn example_function() {{
            let x = 5;
            println!("Example function: {{}}", x);
        }}

        fn another_function() -> i32 {{
            42
        }}
        "#
    )
    .expect("Failed to write to temp file");

    // Run function on temp directory path
    let lang_settings = create_lang_settings(&Lang::Rust);
    let methods = extract_logic_methods(
        &temp_dir.path().to_string_lossy().to_string(),
        &lang_settings,
    );

    // Verify results
    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0].method_name, "example_function");
    assert_eq!(methods[1].method_name, "another_function");
}

#[test]
fn test_is_test_method() {
    let source_code = r#"
        #[test]
        fn sample_test() {
            assert_eq!(2 + 2, 4);
        }

        fn not_a_test() {
            let x = 5;
        }
    "#;
    let mut parser = get_parser("rs");

    let tree = parser
        .parse(source_code, None)
        .expect("Failed to parse test");
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    let lang_settings = create_lang_settings(&Lang::Rust);

    let mut test_found = false;
    let mut non_test_found = false;

    for node in root_node.children(&mut cursor) {
        if node.kind() == "function_item" {
            let result = is_test_method(&node, source_code, &lang_settings);
            let function_name = &source_code[node.start_byte()..node.end_byte()];

            if function_name.contains("sample_test") {
                test_found = result;
            } else if function_name.contains("not_a_test") {
                non_test_found = result;
            }
        }
    }

    assert!(
        test_found,
        "Expected 'sample_test' to be recognized as a test."
    );
    assert!(
        !non_test_found,
        "Expected 'not_a_test' to not be recognized as a test."
    );
}

#[test]
fn test_extract_body() {
    let source_code = r#"
        fn example_function() {
            let x = 5;
            println!("{}", x);
        }
    "#;
    let mut parser = get_parser("rs");

    let tree = parser
        .parse(source_code, None)
        .expect("Failed to parse test");
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    let mut function_node = None;

    for node in root_node.children(&mut cursor) {
        if node.kind() == "function_item" {
            function_node = Some(node);
            break;
        }
    }

    assert!(function_node.is_some(), "Failed to find function node.");

    let extracted_body: Vec<String> = extract_body(function_node.unwrap(), source_code)
        .iter()
        .map(|line| line.trim().to_string()) // Normalize by trimming whitespace
        .collect();

    let expected_body = vec![
        "fn example_function() {".to_string(),
        "let x = 5;".to_string(),
        "println!(\"{}\", x);".to_string(),
        "}".to_string(),
    ];

    assert_eq!(
        extracted_body, expected_body,
        "Extracted body does not match expected."
    );
}

#[test]
fn test_read_to_string_buffered() {
    use std::fs::{self, File};
    use std::io::BufReader;
    use std::io::{BufWriter, Write};
    use tempfile::NamedTempFile;

    // Create a temporary file
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    // Write some content to the file
    {
        let mut writer = BufWriter::new(File::create(&file_path).expect("Failed to create file"));
        writeln!(writer, "Hello, world!").unwrap();
        writeln!(writer, "This is a test file.").unwrap();
        writeln!(writer, "Buffered reading is useful.").unwrap();
    }

    // Read the file using BufReader
    let file = File::open(&file_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let result = read_to_string_buffered(reader);

    // Expected output
    let expected = "Hello, world!\nThis is a test file.\nBuffered reading is useful.\n";

    // Cleanup
    fs::remove_file(file_path).expect("Failed to delete temp file");

    // Assert the result
    assert_eq!(
        result, expected,
        "Buffered read did not match expected output."
    );
}

#[test]
fn test_start_analysis_valid_repo() {
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;
    use utils::Command;

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path().join("repo");
    create_dir_all(&repo_path).expect("Failed to create test repo");

    // Create a mock Rust source file
    let file_path = repo_path.join("main.rs");
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(
        file,
        "fn example_function() {{ println!(\"Hello, world!\"); }}"
    )
    .unwrap();

    let mock_repo = Command {
        repo: repo_path.to_str().unwrap().to_string(),
        lang: Lang::Rust,
    };

    // Redirect output to avoid cluttering the test logs
    let _ = std::io::stdout().lock();

    start_analysis(mock_repo);

    // If we reach this point, the function didn't panic, meaning it handled the input correctly.
    assert!(true);
}
