//! Self-Reflection - Agent Self-Improvement & Critique
//!
//! Provides self-improvement capabilities for agents:
//! - Self-critique of generated outputs
//! - Iterative refinement loops
//! - Quality scoring and improvement suggestions
//! - Learning from past performance

use crate::ai::providers::{AIModel, ChatMessage, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// TYPES
// =============================================================================

/// Result of self-reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// Quality score (0.0 to 1.0)
    pub score: f32,
    /// Issues identified
    pub issues: Vec<Issue>,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Strengths identified
    pub strengths: Vec<String>,
    /// Overall assessment
    pub assessment: String,
    /// Recommendation: accept, revise, or reject
    pub recommendation: Recommendation,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl Reflection {
    pub fn new(score: f32) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
            issues: Vec::new(),
            improvements: Vec::new(),
            strengths: Vec::new(),
            assessment: String::new(),
            recommendation: if score >= 0.8 {
                Recommendation::Accept
            } else if score >= 0.5 {
                Recommendation::Revise
            } else {
                Recommendation::Reject
            },
            timestamp: Utc::now(),
        }
    }

    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }

    pub fn with_improvements(mut self, improvements: Vec<String>) -> Self {
        self.improvements = improvements;
        self
    }

    pub fn with_assessment(mut self, assessment: &str) -> Self {
        self.assessment = assessment.to_string();
        self
    }

    /// Check if output should be refined
    pub fn needs_refinement(&self) -> bool {
        matches!(self.recommendation, Recommendation::Revise)
    }
}

/// An issue found during reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
    /// Location hint (line number, section, etc.)
    pub location: Option<String>,
}

impl Issue {
    pub fn new(severity: IssueSeverity, category: IssueCategory, description: &str) -> Self {
        Self {
            severity,
            category,
            description: description.to_string(),
            suggested_fix: None,
            location: None,
        }
    }

    pub fn with_fix(mut self, fix: &str) -> Self {
        self.suggested_fix = Some(fix.to_string());
        self
    }

    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Minor issue, doesn't affect functionality
    Minor,
    /// Moderate issue, should be addressed
    Moderate,
    /// Major issue, must be fixed
    Major,
    /// Critical issue, blocks acceptance
    Critical,
}

/// Issue categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// Logical/correctness issue
    Logic,
    /// Incomplete implementation
    Completeness,
    /// Code style/clarity
    Style,
    /// Performance concern
    Performance,
    /// Security vulnerability
    Security,
    /// Documentation missing
    Documentation,
    /// Best practice violation
    BestPractice,
    /// Embedded systems specific
    EmbeddedConstraint,
    /// Other
    Other(String),
}

/// Recommendation after reflection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recommendation {
    /// Good enough to use as-is
    Accept,
    /// Needs improvement before use
    Revise,
    /// Too flawed, start over
    Reject,
}

/// Output after refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedOutput {
    /// Original output
    pub original: String,
    /// Refined output
    pub refined: String,
    /// Reflection history
    pub reflection_history: Vec<Reflection>,
    /// Number of refinement iterations
    pub iterations: usize,
    /// Final quality score
    pub final_score: f32,
    /// Whether refinement was successful
    pub success: bool,
}

// =============================================================================
// SELF-REFLECTION ENGINE
// =============================================================================

/// Configuration for self-reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionConfig {
    /// Minimum acceptable score (0.0 to 1.0)
    pub min_score: f32,
    /// Maximum refinement iterations
    pub max_iterations: usize,
    /// Score improvement threshold to continue
    pub improvement_threshold: f32,
    /// Enable detailed critique
    pub detailed_critique: bool,
    /// Criteria to evaluate
    pub criteria: Vec<String>,
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            min_score: 0.75,
            max_iterations: 3,
            improvement_threshold: 0.05,
            detailed_critique: true,
            criteria: vec![
                "correctness".to_string(),
                "completeness".to_string(),
                "clarity".to_string(),
                "efficiency".to_string(),
            ],
        }
    }
}

/// Self-reflection engine for agent self-improvement
pub struct SelfReflection {
    /// Configuration
    config: ReflectionConfig,
}

impl SelfReflection {
    pub fn new() -> Self {
        Self {
            config: ReflectionConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ReflectionConfig) -> Self {
        self.config = config;
        self
    }

    /// Reflect on an output and provide critique
    pub async fn reflect<M: AIModel + ?Sized>(
        &self,
        output: &str,
        goal: &str,
        context: &str,
        model: &M,
    ) -> Result<Reflection, ReflectionError> {
        let criteria_str = self.criteria_prompt();

        let prompt = format!(
            r#"You are a critical reviewer. Evaluate this output against the goal.

## Goal
{}

## Context
{}

## Output to Evaluate
{}

## Evaluation Criteria
{}

Provide your evaluation as JSON:
```json
{{
  "score": 0.0-1.0,
  "issues": [
    {{"severity": "minor|moderate|major|critical", "category": "logic|completeness|style|performance|security|documentation|best_practice|embedded_constraint", "description": "...", "suggested_fix": "..."}}
  ],
  "improvements": ["suggestion 1", "suggestion 2"],
  "strengths": ["strength 1", "strength 2"],
  "assessment": "overall assessment paragraph"
}}
```

Be critical but fair. Focus on actionable feedback."#,
            goal, context, output, criteria_str
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a meticulous code/output reviewer with expertise in embedded systems.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = model.chat(&messages).await
            .map_err(|e| ReflectionError::AIError(e.to_string()))?;

        self.parse_reflection(&response.content)
    }

    /// Refine output based on reflection
    pub async fn refine<M: AIModel + ?Sized>(
        &self,
        output: &str,
        reflection: &Reflection,
        goal: &str,
        model: &M,
    ) -> Result<String, ReflectionError> {
        let issues_str = reflection.issues.iter()
            .map(|i| format!("- [{:?}] {}: {}", i.severity, i.description, 
                i.suggested_fix.as_deref().unwrap_or("No fix suggested")))
            .collect::<Vec<_>>()
            .join("\n");

        let improvements_str = reflection.improvements.join("\n- ");

        let prompt = format!(
            r#"Improve this output based on the feedback.

## Original Goal
{}

## Current Output
{}

## Issues to Address
{}

## Suggested Improvements
- {}

## Instructions
1. Address all major and critical issues
2. Implement suggested improvements where applicable
3. Maintain the core functionality
4. Return the improved output only, no explanations"#,
            goal, output, issues_str, improvements_str
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are an expert at improving code and outputs based on feedback.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = model.chat(&messages).await
            .map_err(|e| ReflectionError::AIError(e.to_string()))?;

        Ok(response.content)
    }

    /// Iteratively refine output until acceptable or max iterations
    pub async fn iterative_refine<M: AIModel + ?Sized>(
        &self,
        initial_output: &str,
        goal: &str,
        context: &str,
        model: &M,
    ) -> Result<RefinedOutput, ReflectionError> {
        let mut current_output = initial_output.to_string();
        let mut reflection_history = Vec::new();
        let mut iterations = 0;

        loop {
            // Reflect on current output
            let reflection = self.reflect(&current_output, goal, context, model).await?;
            let current_score = reflection.score;
            
            reflection_history.push(reflection.clone());
            iterations += 1;

            // Check if we've reached acceptable quality
            if current_score >= self.config.min_score {
                return Ok(RefinedOutput {
                    original: initial_output.to_string(),
                    refined: current_output,
                    reflection_history,
                    iterations,
                    final_score: current_score,
                    success: true,
                });
            }

            // Check if we've hit max iterations
            if iterations >= self.config.max_iterations {
                return Ok(RefinedOutput {
                    original: initial_output.to_string(),
                    refined: current_output,
                    reflection_history,
                    iterations,
                    final_score: current_score,
                    success: false,
                });
            }

            // Check if improvement is stalling
            if reflection_history.len() >= 2 {
                let prev_score = reflection_history[reflection_history.len() - 2].score;
                if current_score - prev_score < self.config.improvement_threshold {
                    // Not improving enough, stop
                    return Ok(RefinedOutput {
                        original: initial_output.to_string(),
                        refined: current_output,
                        reflection_history,
                        iterations,
                        final_score: current_score,
                        success: current_score >= self.config.min_score,
                    });
                }
            }

            // Refine based on reflection
            if reflection.needs_refinement() {
                current_output = self.refine(&current_output, &reflection, goal, model).await?;
            } else if reflection.recommendation == Recommendation::Reject {
                // If rejected, we can't improve further without more context
                return Ok(RefinedOutput {
                    original: initial_output.to_string(),
                    refined: current_output,
                    reflection_history,
                    iterations,
                    final_score: current_score,
                    success: false,
                });
            }
        }
    }

    /// Quick quality check without detailed critique
    pub async fn quick_score<M: AIModel + ?Sized>(
        &self,
        output: &str,
        goal: &str,
        model: &M,
    ) -> Result<f32, ReflectionError> {
        let prompt = format!(
            r#"Rate this output's quality from 0.0 to 1.0 based on how well it achieves the goal.

Goal: {}

Output:
{}

Return ONLY a number between 0.0 and 1.0."#,
            goal, output
        );

        let messages = vec![
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = model.chat(&messages).await
            .map_err(|e| ReflectionError::AIError(e.to_string()))?;

        response.content.trim()
            .parse::<f32>()
            .map(|s| s.clamp(0.0, 1.0))
            .map_err(|_| ReflectionError::ParseError("Could not parse score".to_string()))
    }

    /// Compare two outputs and pick the better one
    pub async fn compare<M: AIModel + ?Sized>(
        &self,
        output_a: &str,
        output_b: &str,
        goal: &str,
        model: &M,
    ) -> Result<ComparisonResult, ReflectionError> {
        let prompt = format!(
            r#"Compare these two outputs and determine which better achieves the goal.

## Goal
{}

## Output A
{}

## Output B
{}

Return JSON:
```json
{{
  "winner": "a" or "b",
  "score_a": 0.0-1.0,
  "score_b": 0.0-1.0,
  "reason": "brief explanation"
}}
```"#,
            goal, output_a, output_b
        );

        let messages = vec![
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = model.chat(&messages).await
            .map_err(|e| ReflectionError::AIError(e.to_string()))?;

        self.parse_comparison(&response.content)
    }

    /// Generate criteria prompt
    fn criteria_prompt(&self) -> String {
        self.config.criteria.iter()
            .enumerate()
            .map(|(i, c)| format!("{}. {}", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse reflection from AI response
    fn parse_reflection(&self, response: &str) -> Result<Reflection, ReflectionError> {
        let json_str = extract_json(response);
        
        #[derive(Deserialize)]
        struct ReflectionJson {
            score: f32,
            #[serde(default)]
            issues: Vec<IssueJson>,
            #[serde(default)]
            improvements: Vec<String>,
            #[serde(default)]
            strengths: Vec<String>,
            #[serde(default)]
            assessment: String,
        }

        #[derive(Deserialize)]
        struct IssueJson {
            severity: String,
            category: String,
            description: String,
            #[serde(default)]
            suggested_fix: Option<String>,
        }

        let parsed: ReflectionJson = serde_json::from_str(json_str)
            .map_err(|e| ReflectionError::ParseError(format!("Failed to parse reflection: {}. Response: {}", e, json_str)))?;

        let issues: Vec<Issue> = parsed.issues.into_iter()
            .map(|i| {
                let severity = match i.severity.to_lowercase().as_str() {
                    "minor" => IssueSeverity::Minor,
                    "moderate" => IssueSeverity::Moderate,
                    "major" => IssueSeverity::Major,
                    "critical" => IssueSeverity::Critical,
                    _ => IssueSeverity::Moderate,
                };

                let category = match i.category.to_lowercase().as_str() {
                    "logic" => IssueCategory::Logic,
                    "completeness" => IssueCategory::Completeness,
                    "style" => IssueCategory::Style,
                    "performance" => IssueCategory::Performance,
                    "security" => IssueCategory::Security,
                    "documentation" => IssueCategory::Documentation,
                    "best_practice" => IssueCategory::BestPractice,
                    "embedded_constraint" => IssueCategory::EmbeddedConstraint,
                    other => IssueCategory::Other(other.to_string()),
                };

                Issue::new(severity, category, &i.description)
                    .with_fix(i.suggested_fix.as_deref().unwrap_or(""))
            })
            .collect();

        Ok(Reflection::new(parsed.score)
            .with_issues(issues)
            .with_improvements(parsed.improvements)
            .with_assessment(&parsed.assessment))
    }

    /// Parse comparison result
    fn parse_comparison(&self, response: &str) -> Result<ComparisonResult, ReflectionError> {
        let json_str = extract_json(response);

        #[derive(Deserialize)]
        struct ComparisonJson {
            winner: String,
            score_a: f32,
            score_b: f32,
            reason: String,
        }

        let parsed: ComparisonJson = serde_json::from_str(json_str)
            .map_err(|e| ReflectionError::ParseError(format!("Failed to parse comparison: {}", e)))?;

        Ok(ComparisonResult {
            winner: if parsed.winner.to_lowercase() == "a" { Winner::A } else { Winner::B },
            score_a: parsed.score_a,
            score_b: parsed.score_b,
            reason: parsed.reason,
        })
    }
}

impl Default for SelfReflection {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// HELPER TYPES
// =============================================================================

/// Result of comparing two outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub winner: Winner,
    pub score_a: f32,
    pub score_b: f32,
    pub reason: String,
}

/// Winner of comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Winner {
    A,
    B,
}

/// Reflection errors
#[derive(Debug, thiserror::Error)]
pub enum ReflectionError {
    #[error("AI error: {0}")]
    AIError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Refinement failed: {0}")]
    RefinementFailed(String),
}

/// Extract JSON from content
fn extract_json(content: &str) -> &str {
    if let Some(start) = content.find("```json") {
        let start = start + 7;
        if let Some(end) = content[start..].find("```") {
            return content[start..start + end].trim();
        }
    }
    
    if let Some(start) = content.find("```") {
        let start = start + 3;
        let start = content[start..].find('\n').map(|n| start + n + 1).unwrap_or(start);
        if let Some(end) = content[start..].find("```") {
            return content[start..start + end].trim();
        }
    }

    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            return &content[start..=end];
        }
    }

    content.trim()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_creation() {
        let reflection = Reflection::new(0.85)
            .with_issues(vec![
                Issue::new(IssueSeverity::Minor, IssueCategory::Style, "Could improve variable naming"),
            ])
            .with_improvements(vec!["Add more comments".to_string()])
            .with_assessment("Good overall, minor improvements needed");

        assert_eq!(reflection.score, 0.85);
        assert_eq!(reflection.issues.len(), 1);
        assert_eq!(reflection.recommendation, Recommendation::Accept);
        assert!(!reflection.needs_refinement());
    }

    #[test]
    fn test_recommendation_thresholds() {
        let high = Reflection::new(0.9);
        assert_eq!(high.recommendation, Recommendation::Accept);

        let medium = Reflection::new(0.6);
        assert_eq!(medium.recommendation, Recommendation::Revise);
        assert!(medium.needs_refinement());

        let low = Reflection::new(0.3);
        assert_eq!(low.recommendation, Recommendation::Reject);
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(IssueSeverity::Major, IssueCategory::Logic, "Missing null check")
            .with_fix("Add if (ptr != NULL) before using")
            .with_location("line 42");

        assert_eq!(issue.severity, IssueSeverity::Major);
        assert!(issue.suggested_fix.is_some());
        assert!(issue.location.is_some());
    }

    #[test]
    fn test_json_extraction() {
        let markdown = "Here's the result:\n```json\n{\"score\": 0.8}\n```\nDone!";
        let extracted = extract_json(markdown);
        assert!(extracted.contains("score"));
    }

    #[test]
    fn test_config_defaults() {
        let config = ReflectionConfig::default();
        assert_eq!(config.max_iterations, 3);
        assert_eq!(config.min_score, 0.75);
        assert!(!config.criteria.is_empty());
    }
}
