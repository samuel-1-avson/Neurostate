//! Prompt Manager - Template Management & Versioning
//!
//! Provides production prompt management:
//! - Template storage with variable substitution
//! - Version control for prompts
//! - A/B testing support
//! - Prompt analytics

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// TYPES
// =============================================================================

/// A prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Unique template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template content with {{variable}} placeholders
    pub content: String,
    /// Version number
    pub version: u32,
    /// Category/tag
    pub category: String,
    /// Variables used in the template
    pub variables: Vec<TemplateVariable>,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// Who created this version
    pub created_by: Option<String>,
    /// Whether this is the active version
    pub is_active: bool,
    /// Usage count
    pub usage_count: u64,
    /// Average rating (if collected)
    pub avg_rating: Option<f32>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PromptTemplate {
    pub fn new(id: &str, name: &str, content: &str) -> Self {
        let variables = extract_variables(content);
        
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            content: content.to_string(),
            version: 1,
            category: "general".to_string(),
            variables,
            created_at: Utc::now(),
            created_by: None,
            is_active: true,
            usage_count: 0,
            avg_rating: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    /// Render the template with given variables
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, PromptError> {
        let mut result = self.content.clone();
        
        for var in &self.variables {
            let placeholder = format!("{{{{{}}}}}", var.name);
            
            if let Some(value) = vars.get(&var.name) {
                result = result.replace(&placeholder, value);
            } else if var.required {
                if let Some(default) = &var.default_value {
                    result = result.replace(&placeholder, default);
                } else {
                    return Err(PromptError::MissingVariable(var.name.clone()));
                }
            } else if let Some(default) = &var.default_value {
                result = result.replace(&placeholder, default);
            } else {
                result = result.replace(&placeholder, "");
            }
        }
        
        Ok(result)
    }

    /// Create a new version of this template
    pub fn new_version(&self, content: &str, created_by: Option<&str>) -> Self {
        let variables = extract_variables(content);
        
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            content: content.to_string(),
            version: self.version + 1,
            category: self.category.clone(),
            variables,
            created_at: Utc::now(),
            created_by: created_by.map(|s| s.to_string()),
            is_active: true, // New version becomes active
            usage_count: 0,
            avg_rating: None,
            metadata: self.metadata.clone(),
        }
    }
}

/// A variable in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,
    /// Description of what this variable is for
    pub description: Option<String>,
    /// Whether this variable is required
    pub required: bool,
    /// Default value if not provided
    pub default_value: Option<String>,
    /// Type hint (string, number, json, etc.)
    pub var_type: String,
}

/// A/B test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    /// Test ID
    pub id: String,
    /// Test name
    pub name: String,
    /// Template ID being tested
    pub template_id: String,
    /// Variant A (control) version
    pub variant_a: u32,
    /// Variant B (treatment) version
    pub variant_b: u32,
    /// Traffic split (0.0 to 1.0 for A)
    pub split: f32,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time (if completed)
    pub ended_at: Option<DateTime<Utc>>,
    /// Results for variant A
    pub results_a: ABTestResults,
    /// Results for variant B
    pub results_b: ABTestResults,
    /// Whether the test is active
    pub is_active: bool,
}

impl ABTest {
    pub fn new(id: &str, name: &str, template_id: &str, variant_a: u32, variant_b: u32, split: f32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            template_id: template_id.to_string(),
            variant_a,
            variant_b,
            split: split.clamp(0.0, 1.0),
            started_at: Utc::now(),
            ended_at: None,
            results_a: ABTestResults::default(),
            results_b: ABTestResults::default(),
            is_active: true,
        }
    }

    /// Select a variant (returns version number)
    pub fn select_variant(&self) -> u32 {
        if rand_f32() < self.split {
            self.variant_a
        } else {
            self.variant_b
        }
    }

    /// Record a result
    pub fn record_result(&mut self, variant: u32, success: bool, rating: Option<f32>) {
        let results = if variant == self.variant_a {
            &mut self.results_a
        } else {
            &mut self.results_b
        };

        results.total_uses += 1;
        if success {
            results.successes += 1;
        }
        if let Some(r) = rating {
            results.total_rating += r;
            results.rating_count += 1;
        }
    }

    /// Get statistical significance
    pub fn significance(&self) -> f32 {
        let n_a = self.results_a.total_uses as f32;
        let n_b = self.results_b.total_uses as f32;
        
        if n_a < 30.0 || n_b < 30.0 {
            return 0.0; // Not enough data
        }

        let p_a = self.results_a.success_rate();
        let p_b = self.results_b.success_rate();
        
        let pooled_p = (self.results_a.successes + self.results_b.successes) as f32 
            / (n_a + n_b);
        
        let se = (pooled_p * (1.0 - pooled_p) * (1.0/n_a + 1.0/n_b)).sqrt();
        
        if se == 0.0 {
            return 0.0;
        }

        let z = (p_a - p_b).abs() / se;
        
        // Approximate significance (z > 1.96 is ~95% confidence)
        (z / 1.96).min(1.0)
    }

    /// Get the winning variant
    pub fn winner(&self) -> Option<u32> {
        if self.significance() < 0.95 {
            return None;
        }

        if self.results_a.success_rate() > self.results_b.success_rate() {
            Some(self.variant_a)
        } else {
            Some(self.variant_b)
        }
    }
}

/// Results for an A/B test variant
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ABTestResults {
    pub total_uses: u64,
    pub successes: u64,
    pub total_rating: f32,
    pub rating_count: u64,
}

impl ABTestResults {
    pub fn success_rate(&self) -> f32 {
        if self.total_uses == 0 {
            return 0.0;
        }
        self.successes as f32 / self.total_uses as f32
    }

    pub fn avg_rating(&self) -> Option<f32> {
        if self.rating_count == 0 {
            return None;
        }
        Some(self.total_rating / self.rating_count as f32)
    }
}

// =============================================================================
// PROMPT MANAGER
// =============================================================================

/// Prompt manager for template storage and versioning
pub struct PromptManager {
    /// Templates by ID (all versions)
    templates: Arc<RwLock<HashMap<String, Vec<PromptTemplate>>>>,
    /// Active A/B tests
    ab_tests: Arc<RwLock<HashMap<String, ABTest>>>,
    /// Usage analytics
    analytics: Arc<RwLock<HashMap<String, PromptAnalytics>>>,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            ab_tests: Arc::new(RwLock::new(HashMap::new())),
            analytics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new template
    pub async fn register(&self, template: PromptTemplate) {
        let mut templates = self.templates.write().await;
        let versions = templates.entry(template.id.clone()).or_default();
        versions.push(template);
    }

    /// Get the active version of a template
    pub async fn get(&self, id: &str) -> Option<PromptTemplate> {
        // Check for A/B test first
        {
            let tests = self.ab_tests.read().await;
            for test in tests.values() {
                if test.template_id == id && test.is_active {
                    let version = test.select_variant();
                    return self.get_version(id, version).await;
                }
            }
        }

        // Return active version
        let templates = self.templates.read().await;
        templates.get(id)?
            .iter()
            .find(|t| t.is_active)
            .cloned()
    }

    /// Get a specific version
    pub async fn get_version(&self, id: &str, version: u32) -> Option<PromptTemplate> {
        let templates = self.templates.read().await;
        templates.get(id)?
            .iter()
            .find(|t| t.version == version)
            .cloned()
    }

    /// Get all versions of a template
    pub async fn get_all_versions(&self, id: &str) -> Vec<PromptTemplate> {
        let templates = self.templates.read().await;
        templates.get(id).cloned().unwrap_or_default()
    }

    /// Update a template (creates new version)
    pub async fn update(&self, id: &str, content: &str, created_by: Option<&str>) -> Option<PromptTemplate> {
        let mut templates = self.templates.write().await;
        
        if let Some(versions) = templates.get_mut(id) {
            // Deactivate current active version
            for t in versions.iter_mut() {
                t.is_active = false;
            }

            // Get latest version to base new one on
            if let Some(latest) = versions.last() {
                let new_version = latest.new_version(content, created_by);
                let result = new_version.clone();
                versions.push(new_version);
                return Some(result);
            }
        }
        
        None
    }

    /// Render a template with variables
    pub async fn render(&self, id: &str, vars: &HashMap<String, String>) -> Result<String, PromptError> {
        let template = self.get(id).await
            .ok_or_else(|| PromptError::TemplateNotFound(id.to_string()))?;

        // Track usage
        self.track_usage(id, template.version).await;

        template.render(vars)
    }

    /// Render a specific version
    pub async fn render_version(&self, id: &str, version: u32, vars: &HashMap<String, String>) -> Result<String, PromptError> {
        let template = self.get_version(id, version).await
            .ok_or_else(|| PromptError::VersionNotFound(id.to_string(), version))?;

        self.track_usage(id, version).await;

        template.render(vars)
    }

    /// Start an A/B test
    pub async fn start_ab_test(&self, test: ABTest) {
        let mut tests = self.ab_tests.write().await;
        tests.insert(test.id.clone(), test);
    }

    /// End an A/B test
    pub async fn end_ab_test(&self, test_id: &str) -> Option<ABTest> {
        let mut tests = self.ab_tests.write().await;
        if let Some(test) = tests.get_mut(test_id) {
            test.is_active = false;
            test.ended_at = Some(Utc::now());
            return Some(test.clone());
        }
        None
    }

    /// Record A/B test result
    pub async fn record_ab_result(&self, test_id: &str, variant: u32, success: bool, rating: Option<f32>) {
        let mut tests = self.ab_tests.write().await;
        if let Some(test) = tests.get_mut(test_id) {
            test.record_result(variant, success, rating);
        }
    }

    /// Get A/B test results
    pub async fn get_ab_test(&self, test_id: &str) -> Option<ABTest> {
        let tests = self.ab_tests.read().await;
        tests.get(test_id).cloned()
    }

    /// List all A/B tests
    pub async fn list_ab_tests(&self) -> Vec<ABTest> {
        let tests = self.ab_tests.read().await;
        tests.values().cloned().collect()
    }

    /// Track usage
    async fn track_usage(&self, id: &str, version: u32) {
        let mut analytics = self.analytics.write().await;
        let entry = analytics.entry(id.to_string()).or_insert(PromptAnalytics::default());
        entry.total_uses += 1;
        *entry.version_uses.entry(version).or_default() += 1;
        entry.last_used = Some(Utc::now());
    }

    /// Get analytics for a template
    pub async fn get_analytics(&self, id: &str) -> Option<PromptAnalytics> {
        let analytics = self.analytics.read().await;
        analytics.get(id).cloned()
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<PromptTemplate> {
        let templates = self.templates.read().await;
        templates.values()
            .filter_map(|versions| versions.iter().find(|t| t.is_active).cloned())
            .collect()
    }

    /// Search templates by category
    pub async fn search_by_category(&self, category: &str) -> Vec<PromptTemplate> {
        let templates = self.templates.read().await;
        templates.values()
            .filter_map(|versions| {
                versions.iter()
                    .find(|t| t.is_active && t.category == category)
                    .cloned()
            })
            .collect()
    }

    /// Delete a template (all versions)
    pub async fn delete(&self, id: &str) -> bool {
        let mut templates = self.templates.write().await;
        templates.remove(id).is_some()
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Analytics for a prompt template
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptAnalytics {
    pub total_uses: u64,
    pub version_uses: HashMap<u32, u64>,
    pub last_used: Option<DateTime<Utc>>,
}

// =============================================================================
// BUILT-IN TEMPLATES
// =============================================================================

/// Create default templates for embedded systems
pub fn create_default_templates() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate::new(
            "fsm_design",
            "FSM Design",
            r#"You are an expert embedded systems engineer designing a Finite State Machine.

## Requirements
{{requirements}}

## Target MCU
{{mcu}}

## Constraints
{{constraints}}

Design a robust FSM that:
1. Has clear, well-named states
2. Handles all edge cases
3. Is memory efficient for embedded systems
4. Includes error handling states

Return the design as JSON."#
        ).with_category("fsm").with_description("Template for designing FSMs"),

        PromptTemplate::new(
            "code_review",
            "Code Review",
            r#"Review this {{language}} code for an embedded system.

## Code
```{{language}}
{{code}}
```

## Focus Areas
{{focus_areas}}

Provide feedback on:
1. Safety (memory, null checks, bounds)
2. Efficiency (memory usage, CPU cycles)
3. Maintainability
4. Embedded-specific concerns"#
        ).with_category("code").with_description("Template for code review"),

        PromptTemplate::new(
            "debug_assist",
            "Debug Assistant",
            r#"Help debug this issue in embedded firmware.

## Error/Symptom
{{error}}

## Context
{{context}}

## Relevant Code
```c
{{code}}
```

Analyze the issue and suggest:
1. Root cause hypothesis
2. Debugging steps
3. Potential fixes"#
        ).with_category("debug").with_description("Template for debugging assistance"),

        PromptTemplate::new(
            "generate_driver",
            "Driver Generation",
            r#"Generate a driver for {{peripheral}} on {{mcu}}.

## Configuration
{{config}}

## Special Requirements
{{requirements}}

Generate production-ready C code that:
1. Uses proper error handling
2. Is thread-safe (if applicable)
3. Has clear documentation
4. Follows embedded best practices"#
        ).with_category("hardware").with_description("Template for driver generation"),
    ]
}

// =============================================================================
// HELPERS
// =============================================================================

/// Extract variables from template content
fn extract_variables(content: &str) -> Vec<TemplateVariable> {
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    let mut seen = std::collections::HashSet::new();
    let mut variables = Vec::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            variables.push(TemplateVariable {
                name,
                description: None,
                required: true,
                default_value: None,
                var_type: "string".to_string(),
            });
        }
    }

    variables
}

/// Simple random float (0.0 to 1.0)
fn rand_f32() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}

// =============================================================================
// ERRORS
// =============================================================================

/// Prompt errors
#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Version not found: {0} v{1}")]
    VersionNotFound(String, u32),

    #[error("Missing required variable: {0}")]
    MissingVariable(String),

    #[error("Render error: {0}")]
    RenderError(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = PromptTemplate::new("test", "Test Template", "Hello {{name}}!");
        assert_eq!(template.id, "test");
        assert_eq!(template.variables.len(), 1);
        assert_eq!(template.variables[0].name, "name");
    }

    #[test]
    fn test_template_render() {
        let template = PromptTemplate::new("test", "Test", "Hello {{name}}, you are {{age}} years old.");
        
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }

    #[test]
    fn test_template_missing_required() {
        let template = PromptTemplate::new("test", "Test", "Hello {{name}}!");
        
        let vars = HashMap::new();
        let result = template.render(&vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_versioning() {
        let v1 = PromptTemplate::new("test", "Test", "Version 1");
        let v2 = v1.new_version("Version 2", Some("user"));
        
        assert_eq!(v2.version, 2);
        assert_eq!(v2.content, "Version 2");
    }

    #[tokio::test]
    async fn test_prompt_manager() {
        let manager = PromptManager::new();
        
        let template = PromptTemplate::new("greeting", "Greeting", "Hello {{name}}!");
        manager.register(template).await;

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = manager.render("greeting", &vars).await.unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[tokio::test]
    async fn test_prompt_update() {
        let manager = PromptManager::new();
        
        let template = PromptTemplate::new("test", "Test", "Version 1");
        manager.register(template).await;

        let updated = manager.update("test", "Version 2", None).await.unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.content, "Version 2");

        let active = manager.get("test").await.unwrap();
        assert_eq!(active.version, 2);
    }

    #[test]
    fn test_ab_test() {
        let mut test = ABTest::new("test", "Test AB", "template", 1, 2, 0.5);
        
        test.record_result(1, true, Some(5.0));
        test.record_result(1, true, Some(4.0));
        test.record_result(2, false, Some(2.0));

        assert_eq!(test.results_a.total_uses, 2);
        assert_eq!(test.results_a.success_rate(), 1.0);
        assert_eq!(test.results_b.total_uses, 1);
        assert_eq!(test.results_b.success_rate(), 0.0);
    }

    #[test]
    fn test_extract_variables() {
        let content = "Hello {{name}}, welcome to {{place}}. {{name}} is great!";
        let vars = extract_variables(content);
        
        assert_eq!(vars.len(), 2);
        assert!(vars.iter().any(|v| v.name == "name"));
        assert!(vars.iter().any(|v| v.name == "place"));
    }
}
