//! Agent Collaboration System
//!
//! Enables agents to communicate with each other, share context,
//! and collaborate on complex multi-step problems.

use super::{AgentInfo, AgentResponse, Agent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// A message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: MessageType,
    pub content: String,
    pub context: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<String>,
}

/// Types of inter-agent messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Request help with a task
    HelpRequest,
    /// Provide assistance
    HelpResponse,
    /// Share information/context
    InfoShare,
    /// Delegate a subtask
    Delegate,
    /// Report completion of delegated task
    DelegateResult,
    /// Ask a question
    Query,
    /// Answer a query
    Answer,
    /// General collaboration message
    Collaborate,
}

impl AgentMessage {
    pub fn new(from: &str, to: &str, msg_type: MessageType, content: &str) -> Self {
        Self {
            id: format!("msg_{}_{}", from, Utc::now().timestamp_millis()),
            from_agent: from.to_string(),
            to_agent: to.to_string(),
            message_type: msg_type,
            content: content.to_string(),
            context: None,
            timestamp: Utc::now(),
            reply_to: None,
        }
    }
    
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn as_reply_to(mut self, msg_id: &str) -> Self {
        self.reply_to = Some(msg_id.to_string());
        self
    }
}

/// Collaboration session between multiple agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: String,
    pub initiator: String,
    pub participants: Vec<String>,
    pub goal: String,
    pub messages: Vec<AgentMessage>,
    pub status: CollaborationStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationStatus {
    Active,
    Waiting,
    Completed,
    Failed,
}

impl CollaborationSession {
    pub fn new(initiator: &str, participants: Vec<&str>, goal: &str) -> Self {
        Self {
            id: format!("collab_{}", Utc::now().timestamp_millis()),
            initiator: initiator.to_string(),
            participants: participants.into_iter().map(|s| s.to_string()).collect(),
            goal: goal.to_string(),
            messages: Vec::new(),
            status: CollaborationStatus::Active,
            created_at: Utc::now(),
            completed_at: None,
            result: None,
        }
    }
    
    pub fn add_message(&mut self, message: AgentMessage) {
        self.messages.push(message);
    }
    
    pub fn complete(&mut self, result: &str) {
        self.status = CollaborationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result.to_string());
    }
    
    pub fn fail(&mut self, reason: &str) {
        self.status = CollaborationStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.result = Some(format!("Failed: {}", reason));
    }
    
    /// Get conversation history as context string
    pub fn to_context_string(&self) -> String {
        let mut lines = vec![format!("## Collaboration: {}", self.goal)];
        lines.push(format!("Participants: {}", self.participants.join(", ")));
        lines.push(String::new());
        
        for msg in &self.messages {
            lines.push(format!(
                "[{}] -> [{}] ({}): {}",
                msg.from_agent,
                msg.to_agent,
                format!("{:?}", msg.message_type),
                if msg.content.len() > 100 {
                    format!("{}...", &msg.content[..100])
                } else {
                    msg.content.clone()
                }
            ));
        }
        
        lines.join("\n")
    }
}

/// Manages agent collaboration
pub struct CollaborationManager {
    active_sessions: HashMap<String, CollaborationSession>,
    message_queue: HashMap<String, Vec<AgentMessage>>,  // agent_id -> pending messages
}

impl CollaborationManager {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            message_queue: HashMap::new(),
        }
    }
    
    /// Start a new collaboration session
    pub fn start_session(&mut self, initiator: &str, participants: Vec<&str>, goal: &str) -> String {
        let session = CollaborationSession::new(initiator, participants, goal);
        let id = session.id.clone();
        self.active_sessions.insert(id.clone(), session);
        id
    }
    
    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&CollaborationSession> {
        self.active_sessions.get(session_id)
    }
    
    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut CollaborationSession> {
        self.active_sessions.get_mut(session_id)
    }
    
    /// Send a message in a session
    pub fn send_message(&mut self, session_id: &str, message: AgentMessage) -> Result<(), String> {
        let session = self.active_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        
        // Add to session history
        session.add_message(message.clone());
        
        // Queue for recipient
        self.message_queue
            .entry(message.to_agent.clone())
            .or_default()
            .push(message);
        
        Ok(())
    }
    
    /// Get pending messages for an agent
    pub fn get_pending_messages(&mut self, agent_id: &str) -> Vec<AgentMessage> {
        self.message_queue.remove(agent_id).unwrap_or_default()
    }
    
    /// Check if an agent has pending messages
    pub fn has_pending_messages(&self, agent_id: &str) -> bool {
        self.message_queue
            .get(agent_id)
            .map(|msgs| !msgs.is_empty())
            .unwrap_or(false)
    }
    
    /// Complete a session
    pub fn complete_session(&mut self, session_id: &str, result: &str) -> Result<(), String> {
        let session = self.active_sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        
        session.complete(result);
        Ok(())
    }
    
    /// List active sessions
    pub fn list_active(&self) -> Vec<&CollaborationSession> {
        self.active_sessions
            .values()
            .filter(|s| s.status == CollaborationStatus::Active)
            .collect()
    }
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Collaboration protocol for agents
pub struct CollaborationProtocol;

impl CollaborationProtocol {
    /// Create a help request message
    pub fn request_help(from: &str, to: &str, problem: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::HelpRequest, problem)
    }
    
    /// Create a help response message
    pub fn respond_help(from: &str, to: &str, solution: &str, reply_to: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::HelpResponse, solution)
            .as_reply_to(reply_to)
    }
    
    /// Create a delegation message
    pub fn delegate_task(from: &str, to: &str, task: &str, context: serde_json::Value) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::Delegate, task)
            .with_context(context)
    }
    
    /// Create a delegation result message
    pub fn delegation_result(from: &str, to: &str, result: &str, reply_to: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::DelegateResult, result)
            .as_reply_to(reply_to)
    }
    
    /// Create an info sharing message
    pub fn share_info(from: &str, to: &str, info: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::InfoShare, info)
    }
    
    /// Create a query message
    pub fn query(from: &str, to: &str, question: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::Query, question)
    }
    
    /// Create an answer message
    pub fn answer(from: &str, to: &str, answer: &str, reply_to: &str) -> AgentMessage {
        AgentMessage::new(from, to, MessageType::Answer, answer)
            .as_reply_to(reply_to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_collaboration_session() {
        let mut session = CollaborationSession::new(
            "director",
            vec!["code", "debug"],
            "Fix compilation error"
        );
        
        assert_eq!(session.participants.len(), 2);
        assert_eq!(session.status, CollaborationStatus::Active);
        
        // Add messages
        session.add_message(CollaborationProtocol::delegate_task(
            "director",
            "debug",
            "Analyze this error",
            serde_json::json!({"error": "undefined reference"})
        ));
        
        assert_eq!(session.messages.len(), 1);
        
        // Complete
        session.complete("Error fixed by adding missing include");
        assert_eq!(session.status, CollaborationStatus::Completed);
    }
    
    #[test]
    fn test_collaboration_manager() {
        let mut manager = CollaborationManager::new();
        
        let session_id = manager.start_session(
            "director",
            vec!["code", "hardware"],
            "Configure GPIO and generate code"
        );
        
        // Send message
        let msg = CollaborationProtocol::delegate_task(
            "director",
            "hardware",
            "Configure PA0 as output",
            serde_json::json!({})
        );
        
        manager.send_message(&session_id, msg).unwrap();
        
        // Check pending
        assert!(manager.has_pending_messages("hardware"));
        
        let pending = manager.get_pending_messages("hardware");
        assert_eq!(pending.len(), 1);
    }
}
