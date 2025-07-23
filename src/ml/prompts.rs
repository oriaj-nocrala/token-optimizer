//! Structured prompts for reliable ML model interactions

/// Structured prompt templates to reduce ML model "overthinking"
pub struct StructuredPrompts;

impl StructuredPrompts {
    /// Create a structured function analysis prompt with few-shot examples
    pub fn function_analysis(function_name: &str, ast_context: &str) -> String {
        format!(
            r#"Analyze the following function and provide EXACTLY the JSON structure shown:

EXAMPLE OUTPUT (follow this format exactly):
{{
  "function": "validateUser",
  "complexity": "medium",
  "dependencies": ["auth.service", "user.model"],
  "impact_scope": "component",
  "recommendations": ["Add error handling", "Consider caching"]
}}

TASK: Analyze this function:
Function: {}
Context: {}

REQUIRED: Respond with ONLY valid JSON matching the example structure above."#,
            function_name,
            ast_context.chars().take(500).collect::<String>() // Limit context to prevent overthinking
        )
    }

    /// Create a structured change risk analysis prompt with constraints
    pub fn change_risk_analysis(changed_file: &str, changed_functions: &[String]) -> String {
        let functions_str = changed_functions.join(", ");
        
        format!(
            r#"Analyze change risk and provide EXACTLY this JSON structure:

EXAMPLE OUTPUT (copy this structure):
{{
  "risk_level": "medium",
  "breaking_changes": ["API signature change"],
  "affected_components": ["login.component", "auth.guard"],
  "testing_strategy": ["unit tests", "integration tests"],
  "rollback_plan": "Revert commit abc123"
}}

TASK: Analyze changes to:
File: {}
Functions: {}

CONSTRAINTS:
- risk_level must be: "low", "medium", or "high"
- Keep arrays concise (max 3 items each)
- Respond with ONLY valid JSON matching the example above."#,
            changed_file,
            functions_str
        )
    }

    /// Create a structured pattern detection prompt with specific output format
    pub fn pattern_detection(code_patterns: &str) -> String {
        format!(
            r#"Detect code patterns and provide EXACTLY this JSON structure:

EXAMPLE OUTPUT (match this format):
{{
  "anti_patterns": [
    {{"pattern": "God Class", "file": "dashboard.component.ts", "severity": "high"}}
  ],
  "refactoring_recommendations": [
    {{"action": "Extract service", "benefit": "Better separation", "effort": "medium"}}
  ],
  "design_patterns": [
    {{"pattern": "Observer", "file": "event.service.ts", "usage": "correct"}}
  ]
}}

TASK: Analyze these patterns:
{}

CONSTRAINTS:
- severity must be: "low", "medium", or "high"
- effort must be: "low", "medium", or "high"
- Max 3 items per array
- Respond with ONLY valid JSON matching the example above."#,
            code_patterns.chars().take(800).collect::<String>()
        )
    }

    /// Create a structured token optimization prompt with clear constraints
    pub fn token_optimization(task: &str, available_files: &[String], token_budget: usize) -> String {
        let files_preview = available_files.iter()
            .take(10) // Limit to prevent overthinking
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        
        format!(
            r#"Optimize file selection for token budget and provide EXACTLY this JSON:

EXAMPLE OUTPUT (copy this structure):
{{
  "recommended_files": [
    {{"file": "auth.service.ts", "priority": "critical", "estimated_tokens": 800}},
    {{"file": "login.component.ts", "priority": "high", "estimated_tokens": 600}}
  ],
  "excluded_files": ["dashboard.component.ts", "profile.component.ts"],
  "total_estimated": 1400,
  "optimization_ratio": 0.85
}}

TASK: Optimize for:
Task: {}
Available files: {}
Token budget: {}

CONSTRAINTS:
- priority must be: "critical", "high", "medium", or "low"
- Max 5 recommended files
- total_estimated must be <= token budget
- Respond with ONLY valid JSON matching the example above."#,
            task,
            files_preview,
            token_budget
        )
    }

    /// Create a structured semantic search prompt with bounded results
    pub fn semantic_search(query: &str, max_results: usize) -> String {
        format!(
            r#"Perform semantic search and provide EXACTLY this JSON structure:

EXAMPLE OUTPUT (match this format):
{{
  "query": "authentication logic",
  "results": [
    {{"file": "auth.service.ts", "relevance": 0.95, "context": "Login and logout functions"}},
    {{"file": "auth.guard.ts", "relevance": 0.87, "context": "Route protection logic"}}
  ],
  "total_results": 2
}}

TASK: Search for: "{}"

CONSTRAINTS:
- Max {} results
- relevance must be between 0.0 and 1.0
- context must be under 50 characters
- Respond with ONLY valid JSON matching the example above."#,
            query,
            max_results
        )
    }

    /// Create a basic yes/no analysis prompt to reduce complex reasoning
    pub fn simple_classification(question: &str, context: &str) -> String {
        format!(
            r#"Answer with EXACTLY this JSON structure:

EXAMPLE OUTPUT (copy this format):
{{
  "answer": "yes",
  "confidence": 0.85,
  "reason": "Clear indicators present"
}}

QUESTION: {}
CONTEXT: {}

CONSTRAINTS:
- answer must be: "yes", "no", or "unclear"
- confidence between 0.0 and 1.0
- reason under 30 characters
- Respond with ONLY valid JSON matching the example above."#,
            question,
            context.chars().take(300).collect::<String>()
        )
    }

    /// Validate if a response matches expected JSON structure
    pub fn validate_json_response(response: &str) -> Result<serde_json::Value, String> {
        match serde_json::from_str(response) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Invalid JSON response: {}", e))
        }
    }

    /// Extract clean JSON from potentially messy model output
    pub fn extract_json_from_response(response: &str) -> Result<String, String> {
        // Find first '{' and last '}'
        if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
            if start <= end {
                let json_part = &response[start..=end];
                // Validate it's proper JSON
                match serde_json::from_str::<serde_json::Value>(json_part) {
                    Ok(_) => Ok(json_part.to_string()),
                    Err(e) => Err(format!("Extracted text is not valid JSON: {}", e))
                }
            } else {
                Err("No valid JSON structure found".to_string())
            }
        } else {
            Err("No JSON brackets found in response".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_analysis_prompt() {
        let prompt = StructuredPrompts::function_analysis("testFunc", "function test() {}");
        
        assert!(prompt.contains("testFunc"));
        assert!(prompt.contains("EXACTLY the JSON structure"));
        assert!(prompt.contains("ONLY valid JSON"));
    }

    #[test]
    fn test_change_risk_analysis_prompt() {
        let functions = vec!["func1".to_string(), "func2".to_string()];
        let prompt = StructuredPrompts::change_risk_analysis("test.ts", &functions);
        
        assert!(prompt.contains("test.ts"));
        assert!(prompt.contains("func1, func2"));
        assert!(prompt.contains("risk_level must be"));
    }

    #[test]
    fn test_json_validation() {
        let valid_json = r#"{"answer": "yes", "confidence": 0.85}"#;
        let invalid_json = r#"{"answer": "yes", "confidence":}"#;
        
        assert!(StructuredPrompts::validate_json_response(valid_json).is_ok());
        assert!(StructuredPrompts::validate_json_response(invalid_json).is_err());
    }

    #[test]
    fn test_json_extraction() {
        let messy_response = r#"Here is the analysis: {"answer": "yes", "confidence": 0.85} and some extra text"#;
        let clean_response = r#"{"answer": "yes", "confidence": 0.85}"#;
        
        let extracted = StructuredPrompts::extract_json_from_response(messy_response).unwrap();
        assert_eq!(extracted, clean_response);
    }

    #[test]
    fn test_simple_classification_prompt() {
        let prompt = StructuredPrompts::simple_classification("Is this secure?", "auth code");
        
        assert!(prompt.contains("Is this secure?"));
        assert!(prompt.contains("answer must be: \"yes\", \"no\", or \"unclear\""));
        assert!(prompt.contains("ONLY valid JSON"));
    }

    #[test]
    fn test_token_optimization_prompt() {
        let files = vec!["file1.ts".to_string(), "file2.ts".to_string()];
        let prompt = StructuredPrompts::token_optimization("fix bug", &files, 5000);
        
        assert!(prompt.contains("fix bug"));
        assert!(prompt.contains("5000"));
        assert!(prompt.contains("total_estimated must be <= token budget"));
    }

    #[test]
    fn test_semantic_search_prompt() {
        let prompt = StructuredPrompts::semantic_search("authentication", 5);
        
        assert!(prompt.contains("authentication"));
        assert!(prompt.contains("Max 5 results"));
        assert!(prompt.contains("relevance must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_pattern_detection_prompt() {
        let prompt = StructuredPrompts::pattern_detection("class BigClass { ... }");
        
        assert!(prompt.contains("BigClass"));
        assert!(prompt.contains("severity must be"));
        assert!(prompt.contains("Max 3 items per array"));
    }
}