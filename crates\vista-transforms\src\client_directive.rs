//! Client Directive Transform
//! 
//! Detects `'client load'` directive at the top of files
//! and marks them as client components.

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

/// Configuration for client directive detection
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClientDirectiveConfig {
    /// The directive string to look for (default: "client load")
    pub directive: String,
}

impl ClientDirectiveConfig {
    pub fn new() -> Self {
        Self {
            directive: "client load".to_string(),
        }
    }
}

/// Result of parsing a file for client directive
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientDirectiveResult {
    /// Whether the file has the client directive
    pub is_client: bool,
    /// The line number where directive was found (0 if not found)
    pub directive_line: usize,
    /// List of exported component names (populated by full parsing)
    pub exports: Vec<String>,
}

impl Default for ClientDirectiveResult {
    fn default() -> Self {
        Self {
            is_client: false,
            directive_line: 0,
            exports: Vec::new(),
        }
    }
}

const DIRECTIVE: &str = "'client load'";
const DIRECTIVE_QUOTED: [&str; 2] = ["'client load'", "\"client load\""];

/// Check if a source string contains the client directive
/// This is a fast string-based check without full parsing
pub fn has_client_directive(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with(DIRECTIVE) || trimmed.starts_with(DIRECTIVE_QUOTED[0]) || trimmed.starts_with(DIRECTIVE_QUOTED[1])
}

/// Quick check without full AST parsing
pub fn detect_client_directive_fast(source: &str) -> ClientDirectiveResult {
    let lines = source.lines();
    let mut directive_line = 0;
    let is_client = lines.enumerate().any(|(idx, line)| {
        let line = line.trim();
        if line.starts_with(DIRECTIVE_QUOTED[0]) || line.starts_with(DIRECTIVE_QUOTED[1]) {
            directive_line = idx + 1; // 1-indexed
            true
        } else {
            false
        }
    });

    ClientDirectiveResult {
        is_client,
        directive_line,
        exports: Vec::new(),
    }
}

/// Scan a source file and extract information about exports
/// This uses simple regex-like matching, not full AST parsing
pub fn analyze_file(source: &str) -> ClientDirectiveResult {
    let mut result = detect_client_directive_fast(source);
    
    let mut exports = FxHashSet::default();
    for line in source.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("export") {
            let export_str = trimmed;
            if let Some(name) = extract_export_name(export_str) {
                if !name.is_empty() {
                    exports.insert(name.to_string());
                }
            }
        }
    }
    
    result.exports = exports.into_iter().collect();
    result
}

fn extract_export_name(export_str: &str) -> Option<&str> {
    if export_str.starts_with("export default function ") {
        export_str
            .strip_prefix("export default function ")
            .and_then(|s| s.split(['(', ' ', '<']).next())
    } else if export_str.starts_with("export function ") {
        export_str
            .strip_prefix("export function ")
            .and_then(|s| s.split(['(', ' ', '<']).next())
    } else if export_str.starts_with("export const ") {
        export_str
            .strip_prefix("export const ")
            .and_then(|s| s.split(['=', ':', ' ']).next())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_client_directive() {
        assert!(has_client_directive("'client load';\nexport default function() {}"));
        assert!(has_client_directive("\"client load\";\nexport default function() {}"));
        assert!(has_client_directive("  'client load'\n"));
        assert!(!has_client_directive("export default function() {}"));
        assert!(!has_client_directive("// comment\n'client load'")); // directive not on first significant line
    }

    #[test]
    fn test_detect_fast() {
        let result = detect_client_directive_fast("'client load';\n");
        assert!(result.is_client);
        assert_eq!(result.directive_line, 1);
        
        let result2 = detect_client_directive_fast("\n\n'client load'\n");
        assert!(result2.is_client);
        assert_eq!(result2.directive_line, 3);
    }
    
    #[test]
    fn test_analyze_exports() {
        let source = r#"
'client load';

export default function MyComponent() {
    return <div>Hello</div>;
}

export const helper = () => {};
export function utilFunc() {}
"#;
        let result = analyze_file(source);
        assert!(result.is_client);
        assert!(result.exports.contains(&"MyComponent".to_string()));
        assert!(result.exports.contains(&"helper".to_string()));
        assert!(result.exports.contains(&"utilFunc".to_string()));
    }
}