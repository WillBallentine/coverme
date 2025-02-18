use crate::utils::{self, AnalysisData};

//eventually want to be able to pipe output to file
pub fn generage_coverage_report(data: AnalysisData){
    println!("Test Coverage Report");
    println!("---------------------");
    let mut tested_count = 0;
    let mut total_lines = 0;

    for method in data.logic_methods {
        let method_id = format!("{}.{}", method.class_name, method.method_name);
        println!("\nMethod: {}", method_id);

        for line in &method.body {
            let normalized_line = utils::normalize_line(line);
            if normalized_line.is_empty() || normalized_line == "{" || normalized_line == "}" {
                continue;
            }

            total_lines += 1;
            if data.tested_lines.contains(&normalized_line) {
                println!("✅ {}", line);
                tested_count += 1;
            } else {
                println!("❌ {}", line);
            }
        }
    }

    let coverage = if total_lines > 0 {
        (tested_count as f64 / total_lines as f64) * 100.0
    } else {
        0.0
    };
    println!("\nTotal Line Coverage: {:.2}%", coverage);
}