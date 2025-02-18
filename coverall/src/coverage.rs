use crate::utils::{AnalysisData, LangSettings};

//eventually want to be able to pipe output to file
pub fn generage_method_level_coverage_report(data: AnalysisData, lang_settings: LangSettings) {
    println!("Test Coverage Report");
    println!("---------------------");
    let mut tested_count = 0;
    let total_methods = data.logic_methods.len();

    for method in data.logic_methods {
        let method_id = if lang_settings.uses_classes {
            format!("{}.{}", method.class_name, method.method_name)
        } else {
            method.method_name.clone()
        };

        let is_tested = data.tested_methods.contains(&method.method_name);

        if is_tested {
            println!("✅ Method: {}", method_id);
            tested_count += 1;
        } else {
            println!("❌ Method: {}", method_id);
        }
    }

    let coverage = if total_methods > 0 {
        (tested_count as f64 / total_methods as f64) * 100.0
    } else {
        0.0
    };
    println!("\nTotal Method Coverage: {:.2}%", coverage);
}
