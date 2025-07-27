use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub command: String,
    pub expected_pattern: Option<String>,
    pub should_fail: bool,
    pub category: TestCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum TestCategory {
    BasicFunctionality,
    DepthAndStructure,
    FileFiltering,
    DisplayOptions,
    SortingOptions,
    OutputOptions,
    Fromfilefunctionality,
    CombinedOptions,
    ErrorConditions,
}

impl TestCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestCategory::BasicFunctionality => "Basic Functionality",
            TestCategory::DepthAndStructure => "Depth and Structure",
            TestCategory::FileFiltering => "File Filtering",
            TestCategory::DisplayOptions => "Display Options",
            TestCategory::SortingOptions => "Sorting Options",
            TestCategory::OutputOptions => "Output Options",
            TestCategory::Fromfilefunctionality => "Fromfile Functionality",
            TestCategory::CombinedOptions => "Combined Options",
            TestCategory::ErrorConditions => "Error Conditions",
        }
    }
}

pub fn get_all_tests() -> Vec<TestCase> {
    vec![
        // Basic Functionality Tests
        TestCase {
            name: "Basic tree display".to_string(),
            command: "/app/target/release/tree .".to_string(),
            expected_pattern: Some("dir1".to_string()),
            should_fail: false,
            category: TestCategory::BasicFunctionality,
        },
        TestCase {
            name: "Help flag".to_string(),
            command: "/app/target/release/tree --help".to_string(),
            expected_pattern: Some("tree".to_string()),
            should_fail: false,
            category: TestCategory::BasicFunctionality,
        },
        TestCase {
            name: "Version flag".to_string(),
            command: "/app/target/release/tree --version".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::BasicFunctionality,
        },
        TestCase {
            name: "Non-existent directory exit code".to_string(),
            command: "/app/target/release/tree /non/existent/path".to_string(),
            expected_pattern: None,
            should_fail: true,
            category: TestCategory::BasicFunctionality,
        },

        // Depth and Structure Tests
        TestCase {
            name: "Depth level 1".to_string(),
            command: "/app/target/release/tree -L 1 .".to_string(),
            expected_pattern: Some("dir1".to_string()),
            should_fail: false,
            category: TestCategory::DepthAndStructure,
        },
        TestCase {
            name: "Depth level 2".to_string(),
            command: "/app/target/release/tree --level=2 .".to_string(),
            expected_pattern: Some("subdir1".to_string()),
            should_fail: false,
            category: TestCategory::DepthAndStructure,
        },
        TestCase {
            name: "No indentation".to_string(),
            command: "/app/target/release/tree -i .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DepthAndStructure,
        },
        TestCase {
            name: "Full path".to_string(),
            command: "/app/target/release/tree -f .".to_string(),
            expected_pattern: Some("./".to_string()),
            should_fail: false,
            category: TestCategory::DepthAndStructure,
        },
        TestCase {
            name: "ASCII characters".to_string(),
            command: "/app/target/release/tree -A .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DepthAndStructure,
        },

        // File Filtering Tests
        TestCase {
            name: "All files (including hidden)".to_string(),
            command: "/app/target/release/tree -a .".to_string(),
            expected_pattern: Some(".hidden".to_string()),
            should_fail: false,
            category: TestCategory::FileFiltering,
        },
        TestCase {
            name: "Directories only".to_string(),
            command: "/app/target/release/tree -d .".to_string(),
            expected_pattern: Some("dir1".to_string()),
            should_fail: false,
            category: TestCategory::FileFiltering,
        },
        TestCase {
            name: "Pattern include .log files".to_string(),
            command: "/app/target/release/tree -P '*.log' .".to_string(),
            expected_pattern: Some("file2.log".to_string()),
            should_fail: false,
            category: TestCategory::FileFiltering,
        },
        TestCase {
            name: "Pattern exclude .log files".to_string(),
            command: "/app/target/release/tree -I '*.log' .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::FileFiltering,
        },
        TestCase {
            name: "File limit".to_string(),
            command: "/app/target/release/tree --filelimit=5 .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::FileFiltering,
        },

        // Display Options Tests
        TestCase {
            name: "Show file sizes".to_string(),
            command: "/app/target/release/tree -s .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "Human readable sizes".to_string(),
            command: "/app/target/release/tree -H .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "Color output".to_string(),
            command: "/app/target/release/tree -C .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "No color output".to_string(),
            command: "/app/target/release/tree -n .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "File type indicators".to_string(),
            command: "/app/target/release/tree -F .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "Show permissions".to_string(),
            command: "/app/target/release/tree -p .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },
        TestCase {
            name: "Show modification dates".to_string(),
            command: "/app/target/release/tree -D .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::DisplayOptions,
        },

        // Sorting Options Tests
        TestCase {
            name: "Sort by time".to_string(),
            command: "/app/target/release/tree -t .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::SortingOptions,
        },
        TestCase {
            name: "Reverse sort".to_string(),
            command: "/app/target/release/tree -r .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::SortingOptions,
        },
        TestCase {
            name: "Directories first".to_string(),
            command: "/app/target/release/tree --dirsfirst .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::SortingOptions,
        },
        TestCase {
            name: "No report".to_string(),
            command: "/app/target/release/tree --noreport .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::SortingOptions,
        },

        // Output Options Tests  
        TestCase {
            name: "Output to file".to_string(),
            command: "/app/target/release/tree -o /tmp/test_output.txt .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::OutputOptions,
        },

        // Fromfile Functionality Tests
        TestCase {
            name: "Read from file".to_string(),
            command: "/app/target/release/tree --fromfile /tmp/file_list.txt".to_string(),
            expected_pattern: Some("dir1".to_string()),
            should_fail: false,
            category: TestCategory::Fromfilefunctionality,
        },
        TestCase {
            name: "Read from stdin".to_string(),
            command: "echo -e 'test/\\ntest/file.txt' | /app/target/release/tree --fromfile".to_string(),
            expected_pattern: Some("test".to_string()),
            should_fail: false,
            category: TestCategory::Fromfilefunctionality,
        },

        // Combined Options Tests
        TestCase {
            name: "Multiple flags combination".to_string(),
            command: "/app/target/release/tree -f -s -L 2 .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::CombinedOptions,
        },
        TestCase {
            name: "Long flags combination".to_string(),
            command: "/app/target/release/tree --full-path --size --level=2 .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::CombinedOptions,
        },
        TestCase {
            name: "Pattern with exclusion".to_string(),
            command: "/app/target/release/tree -P '*.txt' -I 'large*' .".to_string(),
            expected_pattern: None,
            should_fail: false,
            category: TestCategory::CombinedOptions,
        },

        // Error Conditions Tests
        TestCase {
            name: "Invalid flag".to_string(),
            command: "/app/target/release/tree --invalid-flag .".to_string(),
            expected_pattern: None,
            should_fail: true,
            category: TestCategory::ErrorConditions,
        },
        TestCase {
            name: "Invalid level value".to_string(),
            command: "/app/target/release/tree -L abc .".to_string(),
            expected_pattern: None,
            should_fail: true,
            category: TestCategory::ErrorConditions,
        },
        TestCase {
            name: "Invalid file limit".to_string(),
            command: "/app/target/release/tree --filelimit=-1 .".to_string(),
            expected_pattern: None,
            should_fail: true,
            category: TestCategory::ErrorConditions,
        },
    ]
}

pub fn group_tests_by_category(tests: &[TestCase]) -> HashMap<TestCategory, Vec<&TestCase>> {
    let mut grouped = HashMap::new();
    
    for test in tests {
        grouped
            .entry(test.category.clone())
            .or_insert_with(Vec::new)
            .push(test);
    }
    
    grouped
}