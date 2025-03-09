use std::collections::HashSet;

use crate::utils::{extract_body, LangSettings, Method};

pub fn traverse_c_sharp_nodes(
    node: tree_sitter::Node,
    cursor: &mut tree_sitter::TreeCursor,
    source_code: &str,
    lang_settings: &LangSettings,
    methods: &mut Vec<Method>,
) {
    // Process method declarations in C# syntax
    if node.kind() == "method_declaration" {
        let mut test = false;
        if is_csharp_test_method(&node, source_code, lang_settings) {
            test = true;
        }

        // Find class name for C# methods
        let class_name = find_csharp_class_name(&node, source_code);

        // Extract method name
        if let Some(identifier) = node.child_by_field_name("name") {
            let method_name =
                source_code[identifier.start_byte()..identifier.end_byte()].to_string();

            methods.push(Method {
                class_name,
                method_name,
                body: extract_body(node, source_code),
                is_test: test,
            });
        }
    } else if node.kind() == "constructor_declaration" {
        // Handle constructor declarations
        let class_name = find_csharp_class_name(&node, source_code);

        if let Some(identifier) = node.child_by_field_name("name") {
            let method_name =
                source_code[identifier.start_byte()..identifier.end_byte()].to_string();

            methods.push(Method {
                class_name,
                method_name,
                body: extract_body(node, source_code),
                is_test: false, // Constructors are typically not tests
            });
        }
    }

    // Recursively traverse child nodes
    for child_idx in 0..node.child_count() {
        if let Some(child) = node.child(child_idx) {
            traverse_c_sharp_nodes(child, cursor, source_code, lang_settings, methods);
        }
    }
}

fn find_csharp_class_name(node: &tree_sitter::Node, source: &str) -> String {
    let mut current = *node;

    // Traverse up to find class declaration
    while let Some(parent) = current.parent() {
        if parent.kind() == "class_declaration" {
            if let Some(class_name_node) = parent.child_by_field_name("name") {
                return source[class_name_node.start_byte()..class_name_node.end_byte()]
                    .to_string();
            }
        }
        current = parent;
    }

    String::new()
}

fn is_csharp_test_method(
    node: &tree_sitter::Node,
    source_code: &str,
    lang_settings: &LangSettings,
) -> bool {
    // First, check if this is a method declaration
    if node.kind() != "method_declaration" {
        return false;
    }

    // Look for attribute lists before the method
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let current = cursor.node();
            if current.kind() == "attribute_list" {
                let attribute_text = &source_code[current.start_byte()..current.end_byte()];

                if attribute_text.contains(&lang_settings.test_pattern)
                    || attribute_text.contains("[Test]")
                    || attribute_text.contains("[TestMethod]")
                    || attribute_text.contains("[Theory]")
                    || attribute_text.contains("[Fact]")
                {
                    return true;
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    false
}

pub fn extract_csharp_assert_targets(
    line: &str,
    logic_method_names: &HashSet<String>,
    tested_methods: &mut Vec<String>,
) {
    // Common C# assertion patterns: Assert.Equal(expected, actual), Assert.True(condition), etc.
    if let Some(args_start) = line.find('(') {
        if let Some(args_end) = line.rfind(')') {
            let args = &line[args_start + 1..args_end];

            // Split arguments by comma and check each one for function calls
            for arg in args.split(',') {
                // Look for method invocations inside arguments
                if let Some(inner_call_end) = arg.rfind(')') {
                    if let Some(inner_call_start) = arg[..inner_call_end].rfind('(') {
                        let potential_method = arg[..inner_call_start].trim();

                        // Handle obj.Method() pattern
                        let method_name = if potential_method.contains('.') {
                            potential_method.split('.').last().unwrap_or("").trim()
                        } else {
                            potential_method
                        };

                        if !method_name.is_empty() && logic_method_names.contains(method_name) {
                            tested_methods.push(method_name.to_string());
                        }
                    }
                }

                // Also check direct method references
                let trimmed_arg = arg.trim();
                if logic_method_names.contains(trimmed_arg) {
                    tested_methods.push(trimmed_arg.to_string());
                }
            }
        }
    }
}

pub fn extract_csharp_method_calls(
    line: &str,
    logic_method_names: &HashSet<String>,
    tested_methods: &mut Vec<String>,
) {
    // Look for all potential method calls in the line
    // Pattern: something like "result = target.Method()" or just "Method()"
    let mut start_idx = 0;

    while let Some(paren_idx) = line[start_idx..].find('(') {
        let actual_paren_idx = start_idx + paren_idx;
        let call_substr = &line[start_idx..actual_paren_idx];

        // Get the method name part
        let method_part = if call_substr.contains('=') {
            call_substr.split('=').last().unwrap().trim()
        } else {
            call_substr.trim()
        };

        // Handle obj.Method pattern
        let method_name = if method_part.contains('.') {
            method_part.split('.').last().unwrap_or("").trim()
        } else {
            method_part
        };

        if !method_name.is_empty() && logic_method_names.contains(method_name) {
            tested_methods.push(method_name.to_string());
        }

        start_idx = actual_paren_idx + 1;
    }
}
