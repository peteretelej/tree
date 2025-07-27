use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::tests::{group_tests_by_category, TestCase, TestCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub category: TestCategory,
    pub passed: bool,
    pub command: String,
    pub exit_code: Option<i32>,
    pub output: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub platform: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub execution_time_ms: u64,
    pub results_by_category: HashMap<String, CategorySummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

pub struct TestExecutor {
    platform: String,
}

impl TestExecutor {
    pub fn new(platform: String) -> Self {
        Self { platform }
    }

    pub async fn execute_all_tests(&self, container_output: &str) -> Result<TestSummary> {
        let start_time = std::time::Instant::now();
        let tests = crate::tests::get_all_tests();
        let mut results = Vec::new();

        println!("  {} Executing {} tests", "🧪".cyan(), tests.len());

        // Group tests by category for organized execution
        let grouped_tests = group_tests_by_category(&tests);
        let mut category_order = vec![
            TestCategory::BasicFunctionality,
            TestCategory::DepthAndStructure,
            TestCategory::FileFiltering,
            TestCategory::DisplayOptions,
            TestCategory::SortingOptions,
            TestCategory::OutputOptions,
            TestCategory::Fromfilefunctionality,
            TestCategory::CombinedOptions,
            TestCategory::ErrorConditions,
        ];

        for category in category_order {
            if let Some(category_tests) = grouped_tests.get(&category) {
                println!("    {} {}", "📁".blue(), category.as_str().bright_white());

                for test in category_tests {
                    let result = self.execute_single_test(test, container_output).await?;
                    let status_icon = if result.passed { "✅" } else { "❌" };
                    println!("      {} {}", status_icon, result.name);

                    if !result.passed {
                        if let Some(error) = &result.error_message {
                            println!("        {} {}", "⚠️".yellow(), error);
                        }
                    }

                    results.push(result);
                }
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;
        let summary = self.generate_summary(results, execution_time);

        Ok(summary)
    }

    async fn execute_single_test(
        &self,
        test: &TestCase,
        container_output: &str,
    ) -> Result<TestResult> {
        let start_time = std::time::Instant::now();

        // Parse the container output to find this test's execution
        let result = self.parse_test_result_from_output(test, container_output);

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(TestResult {
            name: test.name.clone(),
            category: test.category.clone(),
            passed: result.0,
            command: test.command.clone(),
            exit_code: result.1,
            output: result.2,
            error_message: result.3,
            execution_time_ms: execution_time,
        })
    }

    fn parse_test_result_from_output(
        &self,
        test: &TestCase,
        output: &str,
    ) -> (bool, Option<i32>, String, Option<String>) {
        // Look for test execution in container output
        let mut found_test = false;
        let mut test_output = String::new();
        let mut passed = false;
        let mut exit_code = None;
        let mut error_message = None;

        let lines: Vec<&str> = output.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Look for the test start marker
            if line.contains(&format!("TEST: {}", test.name)) {
                found_test = true;

                // Collect subsequent lines until we find the result
                for j in (i + 1)..lines.len() {
                    test_output.push_str(lines[j]);
                    test_output.push('\n');

                    if lines[j].contains("✓ PASS:") && lines[j].contains(&test.name) {
                        passed = true;
                        break;
                    } else if lines[j].contains("✗ FAIL:") && lines[j].contains(&test.name) {
                        passed = false;
                        // Look for error details
                        if j + 1 < lines.len() && lines[j + 1].contains("Details:") {
                            error_message =
                                Some(lines[j + 1].trim_start_matches("  Details: ").to_string());
                        }
                        break;
                    } else if lines[j].contains("Exit code:") {
                        if let Some(code_str) = lines[j].split("Exit code: ").nth(1) {
                            exit_code = code_str.trim().parse().ok();
                        }
                    }

                    // Stop if we hit the next test
                    if lines[j].contains("TEST: ") && !lines[j].contains(&test.name) {
                        break;
                    }
                }
                break;
            }
        }

        if !found_test {
            // Test wasn't found in output - maybe it didn't run
            error_message = Some("Test not found in container output".to_string());
        }

        (passed, exit_code, test_output, error_message)
    }

    fn generate_summary(&self, results: Vec<TestResult>, execution_time_ms: u64) -> TestSummary {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let success_rate = if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        // Group results by category
        let mut results_by_category = HashMap::new();
        for result in &results {
            let category_name = result.category.as_str().to_string();
            let entry = results_by_category
                .entry(category_name)
                .or_insert(CategorySummary {
                    total: 0,
                    passed: 0,
                    failed: 0,
                });

            entry.total += 1;
            if result.passed {
                entry.passed += 1;
            } else {
                entry.failed += 1;
            }
        }

        TestSummary {
            platform: self.platform.clone(),
            total_tests,
            passed_tests,
            failed_tests,
            success_rate,
            execution_time_ms,
            results_by_category,
        }
    }

    pub fn print_summary(&self, summary: &TestSummary) {
        println!("\n{}", "=".repeat(50).bright_blue());
        println!(
            "{} {} Test Summary",
            "📊".bright_blue(),
            summary.platform.bright_white().bold()
        );
        println!("{}", "=".repeat(50).bright_blue());

        println!(
            "Total tests: {}",
            summary.total_tests.to_string().bright_white()
        );
        println!("Passed: {}", summary.passed_tests.to_string().green());
        println!("Failed: {}", summary.failed_tests.to_string().red());
        println!(
            "Success rate: {:.1}%",
            summary.success_rate.to_string().bright_cyan()
        );
        println!(
            "Execution time: {}ms",
            summary.execution_time_ms.to_string().bright_yellow()
        );

        if !summary.results_by_category.is_empty() {
            println!("\n{} Category Breakdown:", "📁".bright_blue());
            for (category, stats) in &summary.results_by_category {
                let success_rate = if stats.total > 0 {
                    (stats.passed as f64 / stats.total as f64) * 100.0
                } else {
                    0.0
                };

                println!(
                    "  {}: {}/{} ({:.1}%)",
                    category.bright_white(),
                    stats.passed.to_string().green(),
                    stats.total,
                    success_rate.to_string().bright_cyan()
                );
            }
        }

        println!("{}", "=".repeat(50).bright_blue());
    }
}
