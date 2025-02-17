use std::fs::File;
use std::io::{self, BufRead};
use walkdir::WalkDir;
use std::collections::HashSet;

use crate::utils::{self, AnalysisData, Lang, Method};

pub fn start_analysis(repo: utils::Command) {
    match repo.lang {
        Lang::Csharp => csharp_entry(repo),
        Lang::Python => unimplemented!(),
        Lang::JS => unimplemented!(),
        Lang::Undefined => unimplemented!(),
    }
}

fn csharp_entry(repo: utils::Command) {
    let class_methods = extract_class_methods_csharp(&repo.repo);
    let test_methods = extract_test_methods(&repo.repo);
    let tested_lines = extract_tested_lines(&test_methods);

    let analysis_data = utils::AnalysisData {
        logic_methods: class_methods,
        test_methods: test_methods,
        tested_lines: tested_lines,
    };

    println!("{:?}", analysis_data)
    //call coverage module
}

fn extract_class_methods_csharp(repo: &String) -> Vec<Method> {
    let mut methods: Vec<utils::Method> = Vec::new();
    let regex = utils::CSharpRegex::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry.path().extension().map_or(false, |ext| ext == "cs") {
            if let Ok(file) = File::open(entry.path()) {
                let reader = io::BufReader::new(file);
                let mut current_class = String::new();
                let mut in_method = false;
                let mut method_body: Vec<String> = Vec::new();
                let mut method_name = String::new();

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
                    if let Some(cap) = regex.class_regex.captures(&trimmed_line) {
                        if let Some(class_match) = cap.name("class_name") {
                            current_class = class_match.as_str().to_string();
                        }
                    }

                    if let Some(cap) = regex.method_regex.captures(&trimmed_line) {
                        if let Some(method_match) = cap.name("method_name") {
                            method_name = method_match.as_str().to_string();
                            method_body.clear();
                            in_method = true;
                        }
                    }

                    if in_method {
                        method_body.push(trimmed_line.clone());
                    }

                    if in_method && trimmed_line.contains('}') {
                        methods.push(Method{
                            class_name: current_class.clone(),
                            method_name: method_name.clone(),
                            body: method_body.clone(),
                        });
                        in_method = false;
                    }
                }
            }
        }
    }
    methods
}

fn extract_test_methods(repo: &String) -> Vec<Method> {
    let mut test_methods: Vec<Method> = Vec::new();
    let regex = utils::CSharpRegex::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(Result::ok) {
        if entry.path().extension().map_or(false, |ext| ext == "cs") && entry.path().file_name().map_or(false, |name| name.to_string_lossy().contains("Test")) {
            if let Ok(file) = File::open(entry.path()) {
                let reader = io::BufReader::new(file);
                let mut method_body: Vec<String> = Vec::new();
                let mut method_name = String::new();
                let mut in_method = false;

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
                    if let Some(cap) = regex.test_regex.captures(&trimmed_line) {
                        if let Some(test_method) = cap.name("test_method") {
                            method_name = test_method.as_str().to_string();
                            method_body.clear();
                            in_method = true;
                        }
                    }

                    if in_method {
                        method_body.push(trimmed_line.clone());
                    }

                    if in_method && trimmed_line.contains('}') {
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

fn extract_tested_lines(test_methods: &[Method]) -> HashSet<String> {
    let mut tested_lines = HashSet::new();
    for test in test_methods {
        for line in &test.body {
            let normalized_line = normalize_line(line);
            tested_lines.insert(normalized_line);
        }
    }
    tested_lines
}

fn normalize_line(line: &str) -> String {
    line.replace(" ", "").replace("\t", "")
}

