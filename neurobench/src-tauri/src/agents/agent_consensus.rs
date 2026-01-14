//! Agent Consensus - Multi-Agent Voting & Decision Making
//!
//! Provides consensus protocols for multi-agent collaboration:
//! - Majority voting among agents
//! - Weighted voting based on expertise
//! - Quality-based selection using evaluators
//! - Unanimous agreement requirements

use crate::ai::providers::{AIModel, ChatMessage, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// =============================================================================
// TYPES
// =============================================================================

/// An opinion from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opinion {
    /// Agent that provided the opinion
    pub agent_id: String,
    /// The opinion/answer content
    pub content: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Reasoning behind the opinion
    pub reasoning: Option<String>,
    /// When this opinion was given
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Opinion {
    pub fn new(agent_id: &str, content: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            content: content.to_string(),
            confidence: 0.5,
            reasoning: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_reasoning(mut self, reasoning: &str) -> Self {
        self.reasoning = Some(reasoning.to_string());
        self
    }
}

/// A decision reached through consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// The final decision/answer
    pub content: String,
    /// How the decision was reached
    pub method: ConsensusMethod,
    /// Was consensus achieved?
    pub consensus_reached: bool,
    /// Overall confidence in the decision
    pub confidence: f32,
    /// All opinions considered
    pub opinions: Vec<Opinion>,
    /// Vote counts (for voting methods)
    pub vote_counts: HashMap<String, usize>,
    /// Winner's vote percentage
    pub vote_percentage: f32,
    /// Timestamp
    pub decided_at: DateTime<Utc>,
}

/// Consensus method to use
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusMethod {
    /// Simple majority wins
    MajorityVote,
    /// Weighted by agent expertise
    WeightedVote {
        weights: HashMap<String, f32>,
    },
    /// Super majority required (e.g., 2/3)
    SuperMajority {
        threshold: f32,
    },
    /// All agents must agree
    Unanimity,
    /// Best answer selected by quality evaluation
    BestOf,
    /// First response wins (no consensus needed)
    FirstResponse,
    /// Average/merge all opinions
    Aggregate,
}

impl Default for ConsensusMethod {
    fn default() -> Self {
        Self::MajorityVote
    }
}

/// Configuration for consensus protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus method to use
    pub method: ConsensusMethod,
    /// Minimum number of opinions required
    pub min_opinions: usize,
    /// Timeout for gathering opinions (seconds)
    pub timeout_secs: u64,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Allow abstentions
    pub allow_abstain: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            method: ConsensusMethod::MajorityVote,
            min_opinions: 2,
            timeout_secs: 30,
            min_confidence: 0.3,
            allow_abstain: true,
        }
    }
}

// =============================================================================
// CONSENSUS PROTOCOL
// =============================================================================

/// Consensus protocol for multi-agent decision making
pub struct ConsensusProtocol {
    /// Configuration
    config: ConsensusConfig,
    /// Agent weights for weighted voting
    weights: HashMap<String, f32>,
    /// Quality evaluator (optional)
    evaluator: Option<Arc<dyn QualityEvaluator + Send + Sync>>,
}

impl ConsensusProtocol {
    pub fn new() -> Self {
        Self {
            config: ConsensusConfig::default(),
            weights: HashMap::new(),
            evaluator: None,
        }
    }

    pub fn with_config(mut self, config: ConsensusConfig) -> Self {
        // Extract weights if weighted voting
        if let ConsensusMethod::WeightedVote { ref weights } = config.method {
            self.weights = weights.clone();
        }
        self.config = config;
        self
    }

    pub fn with_evaluator(mut self, evaluator: Arc<dyn QualityEvaluator + Send + Sync>) -> Self {
        self.evaluator = Some(evaluator);
        self
    }

    /// Set weight for an agent
    pub fn set_weight(&mut self, agent_id: &str, weight: f32) {
        self.weights.insert(agent_id.to_string(), weight.clamp(0.0, 1.0));
    }

    /// Reach consensus from a set of opinions
    pub async fn reach_consensus(&self, opinions: Vec<Opinion>) -> Result<Decision, ConsensusError> {
        if opinions.len() < self.config.min_opinions {
            return Err(ConsensusError::InsufficientOpinions {
                required: self.config.min_opinions,
                received: opinions.len(),
            });
        }

        // Filter by minimum confidence
        let valid_opinions: Vec<_> = opinions.iter()
            .filter(|o| o.confidence >= self.config.min_confidence)
            .cloned()
            .collect();

        if valid_opinions.is_empty() {
            return Err(ConsensusError::NoValidOpinions);
        }

        match &self.config.method {
            ConsensusMethod::MajorityVote => self.majority_vote(&valid_opinions),
            ConsensusMethod::WeightedVote { weights } => self.weighted_vote(&valid_opinions, weights),
            ConsensusMethod::SuperMajority { threshold } => self.super_majority(&valid_opinions, *threshold),
            ConsensusMethod::Unanimity => self.unanimity(&valid_opinions),
            ConsensusMethod::BestOf => self.best_of(&valid_opinions).await,
            ConsensusMethod::FirstResponse => self.first_response(&valid_opinions),
            ConsensusMethod::Aggregate => self.aggregate(&valid_opinions),
        }
    }

    /// Simple majority voting
    fn majority_vote(&self, opinions: &[Opinion]) -> Result<Decision, ConsensusError> {
        let vote_counts = self.count_votes(opinions);
        
        let total_votes: usize = vote_counts.values().sum();
        let (winner, winner_count) = vote_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(k, v)| (k.clone(), *v))
            .ok_or(ConsensusError::NoValidOpinions)?;

        let vote_percentage = winner_count as f32 / total_votes as f32;
        let consensus_reached = vote_percentage > 0.5;

        // Find the winning opinion
        let winning_opinion = opinions.iter()
            .find(|o| self.normalize_content(&o.content) == winner)
            .cloned()
            .unwrap();

        Ok(Decision {
            content: winning_opinion.content,
            method: ConsensusMethod::MajorityVote,
            consensus_reached,
            confidence: vote_percentage * winning_opinion.confidence,
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage,
            decided_at: Utc::now(),
        })
    }

    /// Weighted voting based on agent expertise
    fn weighted_vote(&self, opinions: &[Opinion], weights: &HashMap<String, f32>) -> Result<Decision, ConsensusError> {
        let mut weighted_votes: HashMap<String, f32> = HashMap::new();
        let mut total_weight = 0.0f32;

        for opinion in opinions {
            let weight = weights.get(&opinion.agent_id).copied().unwrap_or(1.0);
            let normalized = self.normalize_content(&opinion.content);
            *weighted_votes.entry(normalized).or_default() += weight * opinion.confidence;
            total_weight += weight;
        }

        let (winner, winner_weight) = weighted_votes.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
            .ok_or(ConsensusError::NoValidOpinions)?;

        let vote_percentage = winner_weight / total_weight;
        let consensus_reached = vote_percentage > 0.5;

        let winning_opinion = opinions.iter()
            .find(|o| self.normalize_content(&o.content) == winner)
            .cloned()
            .unwrap();

        // Convert weighted votes to counts for display
        let vote_counts: HashMap<String, usize> = weighted_votes.iter()
            .map(|(k, v)| (k.clone(), (v * 100.0) as usize))
            .collect();

        Ok(Decision {
            content: winning_opinion.content,
            method: ConsensusMethod::WeightedVote { weights: weights.clone() },
            consensus_reached,
            confidence: vote_percentage,
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage,
            decided_at: Utc::now(),
        })
    }

    /// Super majority (e.g., 2/3 agreement)
    fn super_majority(&self, opinions: &[Opinion], threshold: f32) -> Result<Decision, ConsensusError> {
        let vote_counts = self.count_votes(opinions);
        let total_votes: usize = vote_counts.values().sum();

        let (winner, winner_count) = vote_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(k, v)| (k.clone(), *v))
            .ok_or(ConsensusError::NoValidOpinions)?;

        let vote_percentage = winner_count as f32 / total_votes as f32;
        let consensus_reached = vote_percentage >= threshold;

        let winning_opinion = opinions.iter()
            .find(|o| self.normalize_content(&o.content) == winner)
            .cloned()
            .unwrap();

        Ok(Decision {
            content: winning_opinion.content,
            method: ConsensusMethod::SuperMajority { threshold },
            consensus_reached,
            confidence: if consensus_reached { vote_percentage } else { 0.0 },
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage,
            decided_at: Utc::now(),
        })
    }

    /// Require unanimous agreement
    fn unanimity(&self, opinions: &[Opinion]) -> Result<Decision, ConsensusError> {
        let vote_counts = self.count_votes(opinions);
        
        let consensus_reached = vote_counts.len() == 1;
        let (winner, winner_count) = vote_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(k, v)| (k.clone(), *v))
            .ok_or(ConsensusError::NoValidOpinions)?;

        let vote_percentage = if consensus_reached { 1.0 } else { winner_count as f32 / opinions.len() as f32 };

        let winning_opinion = opinions.iter()
            .find(|o| self.normalize_content(&o.content) == winner)
            .cloned()
            .unwrap();

        Ok(Decision {
            content: winning_opinion.content,
            method: ConsensusMethod::Unanimity,
            consensus_reached,
            confidence: if consensus_reached { 
                opinions.iter().map(|o| o.confidence).sum::<f32>() / opinions.len() as f32 
            } else { 
                0.0 
            },
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage,
            decided_at: Utc::now(),
        })
    }

    /// Select best answer using quality evaluation
    async fn best_of(&self, opinions: &[Opinion]) -> Result<Decision, ConsensusError> {
        let evaluator = self.evaluator.as_ref()
            .ok_or(ConsensusError::NoEvaluator)?;

        let mut best_opinion: Option<(Opinion, f32)> = None;

        for opinion in opinions {
            let score = evaluator.evaluate(&opinion.content).await;
            
            if best_opinion.is_none() || score > best_opinion.as_ref().unwrap().1 {
                best_opinion = Some((opinion.clone(), score));
            }
        }

        let (winner, score) = best_opinion.ok_or(ConsensusError::NoValidOpinions)?;

        let mut vote_counts = HashMap::new();
        vote_counts.insert(winner.content.clone(), 1);

        Ok(Decision {
            content: winner.content.clone(),
            method: ConsensusMethod::BestOf,
            consensus_reached: true, // BestOf always reaches consensus
            confidence: score,
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage: 1.0,
            decided_at: Utc::now(),
        })
    }

    /// First response wins
    fn first_response(&self, opinions: &[Opinion]) -> Result<Decision, ConsensusError> {
        let first = opinions.first()
            .ok_or(ConsensusError::NoValidOpinions)?;

        let mut vote_counts = HashMap::new();
        vote_counts.insert(first.content.clone(), 1);

        Ok(Decision {
            content: first.content.clone(),
            method: ConsensusMethod::FirstResponse,
            consensus_reached: true,
            confidence: first.confidence,
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage: 1.0,
            decided_at: Utc::now(),
        })
    }

    /// Aggregate/merge all opinions
    fn aggregate(&self, opinions: &[Opinion]) -> Result<Decision, ConsensusError> {
        // For aggregation, we combine all opinions
        let combined_content: Vec<String> = opinions.iter()
            .map(|o| format!("[{}] {}", o.agent_id, o.content))
            .collect();

        let avg_confidence = opinions.iter()
            .map(|o| o.confidence)
            .sum::<f32>() / opinions.len() as f32;

        let vote_counts: HashMap<String, usize> = opinions.iter()
            .map(|o| (o.agent_id.clone(), 1))
            .collect();

        Ok(Decision {
            content: combined_content.join("\n\n"),
            method: ConsensusMethod::Aggregate,
            consensus_reached: true,
            confidence: avg_confidence,
            opinions: opinions.to_vec(),
            vote_counts,
            vote_percentage: 1.0,
            decided_at: Utc::now(),
        })
    }

    /// Count votes by normalizing content
    fn count_votes(&self, opinions: &[Opinion]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for opinion in opinions {
            let normalized = self.normalize_content(&opinion.content);
            *counts.entry(normalized).or_default() += 1;
        }
        
        counts
    }

    /// Normalize content for comparison
    fn normalize_content(&self, content: &str) -> String {
        // Simple normalization: lowercase, trim, remove extra whitespace
        content.to_lowercase()
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for ConsensusProtocol {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// QUALITY EVALUATOR
// =============================================================================

/// Trait for evaluating the quality of an opinion/answer
#[async_trait::async_trait]
pub trait QualityEvaluator: Send + Sync {
    /// Evaluate the quality of content (0.0 to 1.0)
    async fn evaluate(&self, content: &str) -> f32;
}

/// AI-based quality evaluator
pub struct AIQualityEvaluator<M: AIModel + Send + Sync> {
    model: Arc<M>,
    criteria: Vec<String>,
}

impl<M: AIModel + Send + Sync> AIQualityEvaluator<M> {
    pub fn new(model: Arc<M>) -> Self {
        Self {
            model,
            criteria: vec![
                "correctness".to_string(),
                "completeness".to_string(),
                "clarity".to_string(),
                "relevance".to_string(),
            ],
        }
    }

    pub fn with_criteria(mut self, criteria: Vec<String>) -> Self {
        self.criteria = criteria;
        self
    }
}

#[async_trait::async_trait]
impl<M: AIModel + Send + Sync> QualityEvaluator for AIQualityEvaluator<M> {
    async fn evaluate(&self, content: &str) -> f32 {
        let criteria_str = self.criteria.join(", ");
        
        let prompt = format!(
            r#"Evaluate this response on a scale of 0.0 to 1.0 based on: {}

Response to evaluate:
{}

Return ONLY a number between 0.0 and 1.0, nothing else."#,
            criteria_str, content
        );

        let messages = vec![
            ChatMessage {
                role: Role::System,
                content: "You are a quality evaluator. Return only a number between 0.0 and 1.0.".to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: prompt,
            },
        ];

        match self.model.chat(&messages).await {
            Ok(response) => {
                response.content.trim()
                    .parse::<f32>()
                    .unwrap_or(0.5)
                    .clamp(0.0, 1.0)
            }
            Err(_) => 0.5, // Default on error
        }
    }
}

/// Simple heuristic-based evaluator (no AI required)
pub struct HeuristicEvaluator {
    min_length: usize,
    max_length: usize,
    required_keywords: Vec<String>,
}

impl HeuristicEvaluator {
    pub fn new() -> Self {
        Self {
            min_length: 10,
            max_length: 10000,
            required_keywords: Vec::new(),
        }
    }

    pub fn with_length_bounds(mut self, min: usize, max: usize) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }

    pub fn with_required_keywords(mut self, keywords: Vec<String>) -> Self {
        self.required_keywords = keywords;
        self
    }
}

impl Default for HeuristicEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl QualityEvaluator for HeuristicEvaluator {
    async fn evaluate(&self, content: &str) -> f32 {
        let mut score = 1.0f32;
        let len = content.len();

        // Length penalty
        if len < self.min_length {
            score *= len as f32 / self.min_length as f32;
        } else if len > self.max_length {
            score *= self.max_length as f32 / len as f32;
        }

        // Keyword bonus
        if !self.required_keywords.is_empty() {
            let content_lower = content.to_lowercase();
            let matches = self.required_keywords.iter()
                .filter(|k| content_lower.contains(&k.to_lowercase()))
                .count();
            let keyword_score = matches as f32 / self.required_keywords.len() as f32;
            score *= 0.5 + (keyword_score * 0.5); // 50% weight to keywords
        }

        score.clamp(0.0, 1.0)
    }
}

// =============================================================================
// ERRORS
// =============================================================================

/// Consensus errors
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Insufficient opinions: required {required}, received {received}")]
    InsufficientOpinions { required: usize, received: usize },

    #[error("No valid opinions after filtering")]
    NoValidOpinions,

    #[error("No evaluator configured for BestOf method")]
    NoEvaluator,

    #[error("Timeout waiting for opinions")]
    Timeout,

    #[error("Agent error: {0}")]
    AgentError(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opinion_creation() {
        let opinion = Opinion::new("agent1", "Yes, this is correct")
            .with_confidence(0.9)
            .with_reasoning("Based on the documentation");

        assert_eq!(opinion.agent_id, "agent1");
        assert_eq!(opinion.confidence, 0.9);
        assert!(opinion.reasoning.is_some());
    }

    #[tokio::test]
    async fn test_majority_vote() {
        let protocol = ConsensusProtocol::new();

        let opinions = vec![
            Opinion::new("agent1", "Yes").with_confidence(0.8),
            Opinion::new("agent2", "Yes").with_confidence(0.7),
            Opinion::new("agent3", "No").with_confidence(0.6),
        ];

        let decision = protocol.reach_consensus(opinions).await.unwrap();
        
        assert!(decision.consensus_reached);
        assert_eq!(decision.content.to_lowercase(), "yes");
        assert!(decision.vote_percentage > 0.5);
    }

    #[tokio::test]
    async fn test_unanimity_fails() {
        let protocol = ConsensusProtocol::new()
            .with_config(ConsensusConfig {
                method: ConsensusMethod::Unanimity,
                min_opinions: 2,
                ..Default::default()
            });

        let opinions = vec![
            Opinion::new("agent1", "Yes").with_confidence(0.8),
            Opinion::new("agent2", "No").with_confidence(0.7),
        ];

        let decision = protocol.reach_consensus(opinions).await.unwrap();
        
        assert!(!decision.consensus_reached);
    }

    #[tokio::test]
    async fn test_unanimity_succeeds() {
        let protocol = ConsensusProtocol::new()
            .with_config(ConsensusConfig {
                method: ConsensusMethod::Unanimity,
                min_opinions: 2,
                ..Default::default()
            });

        let opinions = vec![
            Opinion::new("agent1", "Yes").with_confidence(0.8),
            Opinion::new("agent2", "Yes").with_confidence(0.9),
        ];

        let decision = protocol.reach_consensus(opinions).await.unwrap();
        
        assert!(decision.consensus_reached);
    }

    #[tokio::test]
    async fn test_aggregate() {
        let protocol = ConsensusProtocol::new()
            .with_config(ConsensusConfig {
                method: ConsensusMethod::Aggregate,
                min_opinions: 2,
                ..Default::default()
            });

        let opinions = vec![
            Opinion::new("agent1", "First opinion").with_confidence(0.8),
            Opinion::new("agent2", "Second opinion").with_confidence(0.7),
        ];

        let decision = protocol.reach_consensus(opinions).await.unwrap();
        
        assert!(decision.consensus_reached);
        assert!(decision.content.contains("First opinion"));
        assert!(decision.content.contains("Second opinion"));
    }

    #[tokio::test]
    async fn test_heuristic_evaluator() {
        let evaluator = HeuristicEvaluator::new()
            .with_length_bounds(10, 1000)
            .with_required_keywords(vec!["important".to_string(), "relevant".to_string()]);

        // Good content
        let good_score = evaluator.evaluate("This is important and relevant content").await;
        assert!(good_score > 0.7);

        // Too short
        let short_score = evaluator.evaluate("Short").await;
        assert!(short_score < 0.5);

        // Missing keywords
        let no_keywords = evaluator.evaluate("This is some content without the expected words").await;
        assert!(no_keywords < 0.8);
    }
}
