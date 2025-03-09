use std::collections::HashSet;
use std::{fs::File, io::BufReader};

use walkdir::WalkDir;

use crate::codeanalysis::read_to_string_buffered;
use crate::utils::{
    extract_body, get_parser, normalize_line, should_skip_dir, LangSettings, Method,
};

pub fn extract_js_tested_methods(
    repo: &String,
    lang_settings: &LangSettings,
    logic_methods: &Vec<Method>,
) -> Vec<String> {
    let mut tested_methods = Vec::new();
    let mut parser = get_parser(&lang_settings.ext);

    let logic_method_names = extract_js_logic_method_names(logic_methods);

    for entry in WalkDir::new(repo)
        .into_iter()
        .filter_entry(|e| !should_skip_dir(e))
        .filter_map(Result::ok)
    {
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
                        if node.kind() == "expression_statement" {
                            let body = extract_body(node, &source_code);
                            for line in body {
                                let normalized_line = normalize_line(&line);
                                if normalized_line.contains("expect")
                                    || normalized_line.contains("assert")
                                    || normalized_line.contains("should")
                                    || normalized_line.contains("assertThat")
                                    || normalized_line.contains("done")
                                    || normalized_line.contains("beforeEach")
                                    || normalized_line.contains("afterEach")
                                    || normalized_line.contains("beforeAll")
                                    || normalized_line.contains("afterAll")
                                    || normalized_line.contains("not")
                                {
                                    if let Some(args_start) = line.find('(') {
                                        if let Some(args_end) = line.rfind(')') {
                                            let args = &line[args_start + 1..args_end];

                                            for arg in args.split('(') {
                                                let potential_method = arg.trim();

                                                let method_name = if potential_method.contains('.')
                                                {
                                                    potential_method
                                                        .split('.')
                                                        .last()
                                                        .unwrap_or("")
                                                        .trim()
                                                } else {
                                                    potential_method
                                                };

                                                if !method_name.is_empty()
                                                    && logic_method_names.contains(method_name)
                                                {
                                                    tested_methods.push(method_name.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    tested_methods
}

fn extract_js_logic_method_names(logic_methods: &Vec<Method>) -> HashSet<String> {
    let logic_method_names: HashSet<String> = logic_methods
        .iter()
        .map(|m| m.method_name.clone())
        .collect();

    logic_method_names
}
