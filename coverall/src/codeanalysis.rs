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

    let class_methods = extract_class_methods(&repo.repo, &lang_settings);
    let test_methods = extract_test_methods(&repo.repo, &lang_settings);
    let tested_lines = extract_tested_lines(&test_methods);

    let analysis_data = utils::AnalysisData {
        logic_methods: class_methods,
        test_methods: test_methods,
        tested_lines: tested_lines,
    };

    coverage::generage_coverage_report(analysis_data, lang_settings);
}

fn extract_class_methods(repo: &String, lang_settings: &LangSettings) -> Vec<Method> {
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

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
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

                    if in_method && trimmed_line.contains("}") {
                        methods.push(Method {
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

                for line in reader.lines().flatten() {
                    let trimmed_line = line.trim().to_string();
                    //this line is where it stops working...
                    if let Some(cap) = lang_settings.regex.get_test_regex().captures(&trimmed_line)
                    {
                        println!("{:?}", cap);
                        if let Some(test_method) = cap.name("test_method") {
                            method_name = test_method.as_str().to_string();
                            method_body.clear();
                            in_method = true;
                            println!("{}", method_name);
                        }
                    }

                    if in_method {
                        method_body.push(trimmed_line.clone());
                        println!("2nd: {}", trimmed_line)
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
    println!("{:?}", test_methods);
    test_methods
}

fn extract_tested_lines(test_methods: &[Method]) -> HashSet<String> {
    let mut tested_lines = HashSet::new();
    for test in test_methods {
        for line in &test.body {
            let normalized_line = utils::normalize_line(line);
            tested_lines.insert(normalized_line);
        }
    }
    tested_lines
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
        },
        Lang::Rust => LangSettings {
            regex: LangRegex::Rust(RustRegex::new()),
            ext: String::from("rs"),
            uses_classes: false,
        },
        Lang::Python => unimplemented!(),
        Lang::JS => unimplemented!(),
        Lang::Undefined => unimplemented!(),
    }
}
