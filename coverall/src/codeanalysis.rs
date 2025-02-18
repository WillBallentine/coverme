use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::WalkDir;

use crate::coverage;
use crate::utils::{self, CSharpRegex, Lang, LangRegex, LangSettings, Method, RustRegex};

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

    let analysis_data = utils::AnalysisData {
        logic_methods: logic_methods,
        test_methods: test_methods,
        tested_methods: tested_methods,
    };

    coverage::generage_method_level_coverage_report(analysis_data, lang_settings);
}

fn extract_logic_methods(repo: &String, lang_settings: &LangSettings) -> Vec<Method> {
    let mut methods: Vec<utils::Method> = Vec::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry
            .path()
            .extension()
            .map_or(false, |ext| *ext == *lang_settings.ext)
        {
            if let Ok(file) = File::open(entry.path()) {
                let reader = io::BufReader::new(file);
                let mut current_class = String::new();
                let mut in_method = false;
                let mut method_body: Vec<String> = Vec::new();
                let mut method_name = String::new();
                let mut is_test = false;

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
                    if trimmed_line.contains(&lang_settings.test_pattern) {
                        is_test = true;
                    }
                    if lang_settings.uses_classes {
                        if let Some(cap) = lang_settings
                            .regex
                            .get_class_regex()
                            .captures(&trimmed_line)
                        {
                            if let Some(class_match) = cap.name("class_name") {
                                current_class = class_match.as_str().to_string();
                            }
                        }
                    }
                    if let Some(cap) = lang_settings
                        .regex
                        .get_method_regex()
                        .captures(&trimmed_line)
                    {
                        if let Some(method_match) = cap.name("method_name") {
                            method_name = method_match.as_str().to_string();
                            method_body.clear();
                            in_method = true;
                        }
                    }

                    if in_method {
                        method_body.push(trimmed_line.clone());
                    }

                    if in_method && trimmed_line.contains("}") && !is_test {
                        methods.push(Method {
                            class_name: current_class.clone(),
                            method_name: method_name.clone(),
                            body: method_body.clone(),
                        });
                        in_method = false;
                        is_test = false;
                    }
                }
            }
        }
    }
    methods
}

fn extract_test_methods(repo: &String, lang_settings: &LangSettings) -> Vec<Method> {
    let mut test_methods: Vec<Method> = Vec::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry
            .path()
            .extension()
            .map_or(false, |ext| *ext == *lang_settings.ext)
        {
            if let Ok(file) = File::open(entry.path()) {
                let reader = io::BufReader::new(file);
                let mut method_body: Vec<String> = Vec::new();
                let mut method_name = String::new();
                let mut in_method = false;
                let mut in_test = false;

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
                    if trimmed_line.contains(&lang_settings.test_pattern) {
                        in_test = true;
                        continue;
                    }

                    if in_test && trimmed_line.starts_with(&lang_settings.test_method_start) {
                        if let Some(start) = trimmed_line.find(&lang_settings.test_method_start) {
                            if let Some(end) = trimmed_line[start + 3..].find('(') {
                                method_name =
                                    trimmed_line[start + 3..start + 3 + end].trim().to_string();
                                method_body.clear();
                                in_method = true;
                            }
                        }
                    }

                    if in_method {
                        method_body.push(trimmed_line.clone());
                    }

                    if in_method && trimmed_line.contains("}") {
                        test_methods.push(Method {
                            class_name: "Test".to_string(),
                            method_name: method_name.clone(),
                            body: method_body.clone(),
                        });
                        in_method = false;
                    }
                }
            }
        }
    }
    test_methods
}

fn extract_tested_methods(test_methods: &[Method], logic_methods: &[Method]) -> HashSet<String> {
    let mut tested_methods = HashSet::new();

    // Create a set of all logic method names for quick lookup
    let logic_method_names: HashSet<String> = logic_methods
        .iter()
        .map(|m| m.method_name.clone())
        .collect();

    for test in test_methods {
        for line in &test.body {
            let normalized_line = utils::normalize_line(line);

            // Skip assertion macros and debugging statements
            if normalized_line.starts_with("assert")
                || normalized_line.starts_with("dbg!")
                || normalized_line.starts_with("println!")
            {
                continue;
            }

            // Check if the line contains a function call
            if let Some(pos) = normalized_line.find('(') {
                let before_paren = &normalized_line[..pos].trim();

                // Handle cases where the function is assigned to a variable
                let called_function = if before_paren.contains('=') {
                    // Extract function name after '='
                    before_paren.split('=').last().unwrap().trim().to_string()
                } else {
                    before_paren.to_string()
                };

                // Check if the function being called is a logic method
                if logic_method_names.contains(&called_function) {
                    tested_methods.insert(called_function.clone());
                }
            }
        }
    }

    tested_methods
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
