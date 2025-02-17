use regex::Regex;
use std::collections::HashSet;


#[derive(Debug)]
pub struct Command {
    pub repo: String,
    pub lang: Lang,
}

#[derive(Debug)]
pub enum Lang {
    Csharp,
    Python,
    JS,
    Undefined,
}

#[derive(Debug)]
pub struct Method {
    pub class_name: String,
    pub method_name: String,
    pub body: Vec<String>,
}

#[derive(Debug)]
pub struct AnalysisData {
    pub logic_methods: Vec<Method>,
    pub test_methods: Vec<Method>,
    pub tested_lines: HashSet<String>,
}


#[derive(Debug)]
pub struct CSharpRegex {
    pub class_regex: Regex,
    pub method_regex: Regex,
    pub test_regex: Regex,
}

impl CSharpRegex {
    pub fn new() -> Self {
        Self {
            class_regex: Regex::new(r"class\s+(?P<class_name>\w+)").unwrap(),
            method_regex: Regex::new(r"(public|private|protected|internal)\s+(static\s+)?\w\s+(?P<method_name>\w+)\s*\(").unwrap(),
            test_regex: Regex::new(r"\[Test|Fact\]\s*\n\s*public\s+void\s+(?P<test_method>\w+)\s*\(").unwrap(),
        }
    }
    
}
