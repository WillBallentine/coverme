use crate::utils::{LangSettings, Method};

//eventually want to be able to pipe output to file
pub fn generate_method_level_coverage_report(
    data: Vec<Method>,
    tests: Vec<String>,
    lang_settings: &LangSettings,
) {
    println!("Test Coverage Report");
    println!("---------------------");

    let mut total_methods = 0;
    let mut tested_count = 0;

    for method in data {
        let method_id = if lang_settings.uses_classes {
            format!("{}.{}", method.class_name, method.method_name)
        } else {
            method.method_name.clone()
        };

        if !method.is_test {
            total_methods += 1;
            if tests.contains(&method.method_name) {
                println!("✅ Method: {}", method_id);
                tested_count += 1;
            } else {
                println!("❌ Method: {}", method_id);
            }
        }
    }

    let coverage = if total_methods > 0 {
        (tested_count as f64 / total_methods as f64) * 100.0
    } else {
        0.0
    };

    println!("\nTotal Method Coverage: {:.2}%", coverage);
}
