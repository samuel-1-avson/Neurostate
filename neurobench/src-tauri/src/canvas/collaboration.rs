//! Collaboration - Real-time multi-user collaboration for canvas
//!
//! Provides cursor tracking, presence awareness, and conflict resolution.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Unique identifier for a user
pub type UserId = String;

/// Collaboration state for real-time multi-user editing
#[derive(Debug, Clone, Default)]
pub struct CollaborationState {
    /// User cursors indexed by user ID
    cursors: HashMap<UserId, UserCursor>,
    /// User selections indexed by user ID
    selections: HashMap<UserId, Vec<String>>,
    /// User presence information
    presence: HashMap<UserId, UserPresence>,
    /// Local user ID
    local_user_id: Option<UserId>,
    /// Cursor timeout (users are considered gone after this duration)
    cursor_timeout: Duration,
}

/// Real-time user cursor position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCursor {
    pub user_id: UserId,
    pub display_name: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    #[serde(skip)]
    last_seen: Option<Instant>,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: UserId,
    pub display_name: String,
    pub color: String,
    pub status: PresenceStatus,
    #[serde(skip)]
    pub joined_at: Option<Instant>,
    #[serde(skip)]
    pub last_activity: Option<Instant>,
}

/// User activity status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PresenceStatus {
    Active,
    Idle,
    Away,
}

impl Default for PresenceStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl CollaborationState {
    pub fn new() -> Self {
        Self {
            cursors: HashMap::new(),
            selections: HashMap::new(),
            presence: HashMap::new(),
            local_user_id: None,
            cursor_timeout: Duration::from_secs(30),
        }
    }
    
    /// Set the local user identity
    pub fn set_local_user(&mut self, user_id: UserId, display_name: String, color: String) {
        self.local_user_id = Some(user_id.clone());
        self.presence.insert(user_id.clone(), UserPresence {
            user_id: user_id.clone(),
            display_name,
            color,
            status: PresenceStatus::Active,
            joined_at: Some(Instant::now()),
            last_activity: Some(Instant::now()),
        });
    }
    
    /// Update a user's cursor position
    pub fn update_cursor(&mut self, user_id: UserId, x: f64, y: f64) {
        if let Some(cursor) = self.cursors.get_mut(&user_id) {
            cursor.x = x;
            cursor.y = y;
            cursor.last_seen = Some(Instant::now());
        } else if let Some(presence) = self.presence.get(&user_id) {
            self.cursors.insert(user_id.clone(), UserCursor {
                user_id: user_id.clone(),
                display_name: presence.display_name.clone(),
                color: presence.color.clone(),
                x,
                y,
                last_seen: Some(Instant::now()),
            });
        }
        
        // Update presence activity
        if let Some(presence) = self.presence.get_mut(&user_id) {
            presence.last_activity = Some(Instant::now());
            presence.status = PresenceStatus::Active;
        }
    }
    
    /// Update a user's selection
    pub fn update_selection(&mut self, user_id: UserId, selection: Vec<String>) {
        self.selections.insert(user_id, selection);
    }
    
    /// User joined the session
    pub fn user_joined(&mut self, user_id: UserId, display_name: String, color: String) {
        self.presence.insert(user_id.clone(), UserPresence {
            user_id: user_id.clone(),
            display_name,
            color,
            status: PresenceStatus::Active,
            joined_at: Some(Instant::now()),
            last_activity: Some(Instant::now()),
        });
    }
    
    /// User left the session
    pub fn user_left(&mut self, user_id: &str) {
        self.cursors.remove(user_id);
        self.selections.remove(user_id);
        self.presence.remove(user_id);
    }
    
    /// Get all visible cursors (excluding local user)
    pub fn get_remote_cursors(&self) -> Vec<&UserCursor> {
        let now = Instant::now();
        self.cursors.values()
            .filter(|c| {
                // Filter out local user
                if let Some(ref local_id) = self.local_user_id {
                    if &c.user_id == local_id {
                        return false;
                    }
                }
                // Filter out stale cursors
                if let Some(last_seen) = c.last_seen {
                    now.duration_since(last_seen) < self.cursor_timeout
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// Get all active users
    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.presence.values().collect()
    }
    
    /// Get user's selection
    pub fn get_user_selection(&self, user_id: &str) -> Option<&Vec<String>> {
        self.selections.get(user_id)
    }
    
    /// Get all selections from other users (for rendering)
    pub fn get_remote_selections(&self) -> HashMap<&UserId, &Vec<String>> {
        self.selections.iter()
            .filter(|(id, _)| {
                if let Some(ref local_id) = self.local_user_id {
                    return *id != local_id;
                }
                true
            })
            .collect()
    }
    
    /// Clean up stale cursors and mark users as idle/away
    pub fn cleanup_stale(&mut self) {
        let now = Instant::now();
        let idle_threshold = Duration::from_secs(60);
        let away_threshold = Duration::from_secs(300);
        
        // Remove stale cursors
        self.cursors.retain(|_, cursor| {
            cursor.last_seen
                .map(|t| now.duration_since(t) < self.cursor_timeout)
                .unwrap_or(false)
        });
        
        // Update presence status
        for presence in self.presence.values_mut() {
            if let Some(last_activity) = presence.last_activity {
                let since = now.duration_since(last_activity);
                presence.status = if since > away_threshold {
                    PresenceStatus::Away
                } else if since > idle_threshold {
                    PresenceStatus::Idle
                } else {
                    PresenceStatus::Active
                };
            }
        }
    }
    
    /// Get count of active users
    pub fn active_user_count(&self) -> usize {
        self.presence.values()
            .filter(|p| p.status == PresenceStatus::Active)
            .count()
    }
    
    /// Check if there are any remote users
    pub fn has_remote_users(&self) -> bool {
        if let Some(ref local_id) = self.local_user_id {
            self.presence.len() > 1 || !self.presence.contains_key(local_id)
        } else {
            !self.presence.is_empty()
        }
    }
}

/// Message types for collaboration sync
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollaborationMessage {
    /// User moved their cursor
    CursorMove { user_id: UserId, x: f64, y: f64 },
    /// User changed their selection
    SelectionChange { user_id: UserId, selection: Vec<String> },
    /// User joined the session
    UserJoined { user_id: UserId, display_name: String, color: String },
    /// User left the session
    UserLeft { user_id: UserId },
    /// Node was added
    NodeAdded { user_id: UserId, node_id: String },
    /// Node was moved
    NodeMoved { user_id: UserId, node_id: String, x: f64, y: f64 },
    /// Node was deleted
    NodeDeleted { user_id: UserId, node_id: String },
    /// Edge was added
    EdgeAdded { user_id: UserId, edge_id: String, source: String, target: String },
    /// Edge was deleted
    EdgeDeleted { user_id: UserId, edge_id: String },
}

impl CollaborationMessage {
    pub fn cursor_move(user_id: impl Into<String>, x: f64, y: f64) -> Self {
        Self::CursorMove { user_id: user_id.into(), x, y }
    }
    
    pub fn user_joined(user_id: impl Into<String>, display_name: impl Into<String>, color: impl Into<String>) -> Self {
        Self::UserJoined { 
            user_id: user_id.into(), 
            display_name: display_name.into(), 
            color: color.into() 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaboration_state() {
        let mut state = CollaborationState::new();
        
        // Add local user
        state.set_local_user("user1".into(), "Alice".into(), "#FF0000".into());
        
        // Add remote user
        state.user_joined("user2".into(), "Bob".into(), "#00FF00".into());
        state.update_cursor("user2".into(), 100.0, 200.0);
        
        let cursors = state.get_remote_cursors();
        assert_eq!(cursors.len(), 1);
        assert_eq!(cursors[0].display_name, "Bob");
    }
    
    #[test]
    fn test_presence_status() {
        let mut state = CollaborationState::new();
        state.user_joined("user1".into(), "Alice".into(), "#FF0000".into());
        
        assert_eq!(state.active_user_count(), 1);
    }
    
    #[test]
    fn test_selection_tracking() {
        let mut state = CollaborationState::new();
        state.set_local_user("local".into(), "Me".into(), "#0000FF".into());
        state.user_joined("remote".into(), "Other".into(), "#FF0000".into());
        
        state.update_selection("remote".into(), vec!["n1".into(), "n2".into()]);
        
        let sel = state.get_user_selection("remote");
        assert!(sel.is_some());
        assert_eq!(sel.unwrap().len(), 2);
    }
}
