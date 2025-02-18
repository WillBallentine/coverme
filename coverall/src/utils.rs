use regex::Regex;
use std::collections::HashSet;


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
    pub tested_lines: HashSet<String>,
}

#[derive(Debug)]
pub struct LangSettings {
    pub regex: LangRegex,
    pub ext: String,
}

#[derive(Debug)]
pub enum LangRegex {
    CSharp(CSharpRegex),
    Rust(RustRegex),
}

impl LangRegex {
    pub fn get_class_regex(&self) -> &Regex {
        match self {
            LangRegex::CSharp(csharp) => &csharp.class_regex,
            LangRegex::Rust(rust) => &rust.class_regex,
        }
    }

    pub fn get_method_regex(&self) -> &Regex {
        match self {
            LangRegex::CSharp(csharp) => &csharp.method_regex,
            LangRegex::Rust(rust) => &rust.method_regex,
        }
    }

    pub fn get_test_regex(&self) -> &Regex {
        match self {
            LangRegex::CSharp(csharp) => &csharp.test_regex,
            LangRegex::Rust(rust) => &rust.test_regex,
        }
    }
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
            method_regex: Regex::new(r"(?m)\b(public|private|protected|internal)?\s*(static)?\s*(\w+)\s+(?P<method_name>\w+)\s*\(").unwrap(),
            test_regex: Regex::new(r"\[Test|Fact\]\s*\n\s*public\s+void\s+(?P<test_method>\w+)\s*\(").unwrap(),
        }
    }
    
}

#[derive(Debug)]
pub struct RustRegex {
    pub class_regex: Regex,
    pub method_regex: Regex,
    pub test_regex: Regex,
}

impl RustRegex {
    pub fn new() -> Self {
        Self {
            class_regex: Regex::new(r"fn\s+(?P<name>\w+)\s*\((?P<args>[\s\S]*?)\)\s*(->\s*(?P<return_type>[^{\s]+))?\s*\{").unwrap(),
            method_regex: Regex::new(r"(?m)^(pub\s+)?fn\s+(?P<method_name>\w+)\s*(<[^>]+>)?\s*\((?P<args>[^)]*)\)\s*(->\s*[^ ]+)?\s*\{").unwrap(),
            test_regex: Regex::new(r"#\[test\]\s*pub\s+fn\s+(?P<test_name>\w+)\s*\(\)\s*\{").unwrap(),
        }
    }
}

pub fn normalize_line(line: &str) -> String {
    line.replace(" ", "").replace("\t", "")
}