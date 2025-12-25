// Diff Engine - Structured patches for safe, reviewable changes
//
// Uses:
// - JSON Patch (RFC 6902) for FSM/peripheral configs
// - Text diffs for generated code files
// 
// All patches go through: propose → review → apply → audit

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// A patch that can be applied to project resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub target: PatchTarget,
    pub operations: PatchOperations,
    pub reversible: bool,
}

impl Patch {
    pub fn new(description: impl Into<String>, target: PatchTarget, operations: PatchOperations) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description: description.into(),
            target,
            operations,
            reversible: true,
        }
    }
    
    /// Create a JSON patch for FSM/config changes
    pub fn json_patch(
        description: impl Into<String>,
        target: PatchTarget,
        ops: Vec<JsonPatchOp>,
    ) -> Self {
        Self::new(description, target, PatchOperations::Json(ops))
    }
    
    /// Create a text diff patch for generated code
    pub fn text_diff(
        description: impl Into<String>,
        file_path: PathBuf,
        hunks: Vec<DiffHunk>,
    ) -> Self {
        Self::new(
            description,
            PatchTarget::GeneratedFile { path: file_path },
            PatchOperations::TextDiff(hunks),
        )
    }
}

/// What the patch targets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PatchTarget {
    /// FSM state (JSON Patch)
    FsmState { state_id: String },
    /// FSM transition (JSON Patch)
    FsmTransition { edge_id: String },
    /// Entire FSM graph (JSON Patch)
    FsmGraph,
    /// Peripheral configuration (JSON Patch)
    PeripheralConfig { peripheral: String, instance: String },
    /// Project settings (JSON Patch)
    ProjectSettings,
    /// Generated code file (Text Diff)
    GeneratedFile { path: PathBuf },
}

/// The operations in the patch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum PatchOperations {
    /// RFC 6902 JSON Patch operations
    Json(Vec<JsonPatchOp>),
    /// Text diff hunks
    TextDiff(Vec<DiffHunk>),
}

// ==================== JSON Patch (RFC 6902) ====================

/// RFC 6902 JSON Patch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum JsonPatchOp {
    Add { path: String, value: Value },
    Remove { path: String },
    Replace { path: String, value: Value },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: Value },
}

impl JsonPatchOp {
    pub fn add(path: impl Into<String>, value: Value) -> Self {
        Self::Add { path: path.into(), value }
    }
    
    pub fn remove(path: impl Into<String>) -> Self {
        Self::Remove { path: path.into() }
    }
    
    pub fn replace(path: impl Into<String>, value: Value) -> Self {
        Self::Replace { path: path.into(), value }
    }
    
    /// Apply this operation to a JSON value
    pub fn apply(&self, doc: &mut Value) -> Result<(), PatchError> {
        match self {
            JsonPatchOp::Add { path, value } => {
                json_pointer_set(doc, path, value.clone())
            }
            JsonPatchOp::Remove { path } => {
                json_pointer_remove(doc, path)
            }
            JsonPatchOp::Replace { path, value } => {
                json_pointer_set(doc, path, value.clone())
            }
            JsonPatchOp::Move { from, path } => {
                let value = json_pointer_get(doc, from)?
                    .ok_or_else(|| PatchError::PathNotFound(from.clone()))?
                    .clone();
                json_pointer_remove(doc, from)?;
                json_pointer_set(doc, path, value)
            }
            JsonPatchOp::Copy { from, path } => {
                let value = json_pointer_get(doc, from)?
                    .ok_or_else(|| PatchError::PathNotFound(from.clone()))?
                    .clone();
                json_pointer_set(doc, path, value)
            }
            JsonPatchOp::Test { path, value } => {
                let current = json_pointer_get(doc, path)?
                    .ok_or_else(|| PatchError::PathNotFound(path.clone()))?;
                if current != value {
                    return Err(PatchError::TestFailed(path.clone()));
                }
                Ok(())
            }
        }
    }
    
    /// Create inverse operation for undo
    pub fn inverse(&self, original: &Value) -> Option<JsonPatchOp> {
        match self {
            JsonPatchOp::Add { path, .. } => {
                Some(JsonPatchOp::remove(path))
            }
            JsonPatchOp::Remove { path } => {
                json_pointer_get(original, path).ok().flatten().map(|v| {
                    JsonPatchOp::add(path, v.clone())
                })
            }
            JsonPatchOp::Replace { path, .. } => {
                json_pointer_get(original, path).ok().flatten().map(|v| {
                    JsonPatchOp::replace(path, v.clone())
                })
            }
            _ => None,
        }
    }
}

// ==================== Text Diff ====================

/// A hunk in a text diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

impl DiffHunk {
    /// Create a simple replacement hunk
    pub fn replace(old_start: usize, old_lines: Vec<String>, new_lines: Vec<String>) -> Self {
        Self {
            old_start,
            old_count: old_lines.len(),
            new_start: old_start,
            new_count: new_lines.len(),
            old_lines,
            new_lines,
            context_before: vec![],
            context_after: vec![],
        }
    }
    
    /// Create an insertion hunk
    pub fn insert(at_line: usize, new_lines: Vec<String>) -> Self {
        Self {
            old_start: at_line,
            old_count: 0,
            new_start: at_line,
            new_count: new_lines.len(),
            old_lines: vec![],
            new_lines,
            context_before: vec![],
            context_after: vec![],
        }
    }
    
    /// Create a deletion hunk
    pub fn delete(old_start: usize, old_lines: Vec<String>) -> Self {
        Self {
            old_start,
            old_count: old_lines.len(),
            new_start: old_start,
            new_count: 0,
            old_lines,
            new_lines: vec![],
            context_before: vec![],
            context_after: vec![],
        }
    }
    
    /// Format as unified diff
    pub fn to_unified(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            self.old_start, self.old_count,
            self.new_start, self.new_count
        ));
        for line in &self.context_before {
            out.push_str(&format!(" {}\n", line));
        }
        for line in &self.old_lines {
            out.push_str(&format!("-{}\n", line));
        }
        for line in &self.new_lines {
            out.push_str(&format!("+{}\n", line));
        }
        for line in &self.context_after {
            out.push_str(&format!(" {}\n", line));
        }
        out
    }
}

/// Create text diff between two strings
pub fn create_text_diff(old: &str, new: &str) -> Vec<DiffHunk> {
    // Simple line-by-line diff using LCS
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    
    let mut hunks = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < old_lines.len() || j < new_lines.len() {
        // Skip matching lines
        while i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
            i += 1;
            j += 1;
        }
        
        if i >= old_lines.len() && j >= new_lines.len() {
            break;
        }
        
        // Collect differing lines
        let old_start = i + 1; // 1-indexed
        let new_start = j + 1;
        let mut old_chunk = Vec::new();
        let mut new_chunk = Vec::new();
        
        // Find next matching point
        while i < old_lines.len() && (j >= new_lines.len() || !old_lines[i..].contains(&new_lines.get(j).unwrap_or(&""))) {
            old_chunk.push(old_lines[i].to_string());
            i += 1;
        }
        while j < new_lines.len() && (i >= old_lines.len() || !new_lines[j..].contains(&old_lines.get(i).unwrap_or(&""))) {
            new_chunk.push(new_lines[j].to_string());
            j += 1;
        }
        
        if !old_chunk.is_empty() || !new_chunk.is_empty() {
            hunks.push(DiffHunk {
                old_start,
                old_count: old_chunk.len(),
                new_start,
                new_count: new_chunk.len(),
                old_lines: old_chunk,
                new_lines: new_chunk,
                context_before: vec![],
                context_after: vec![],
            });
        }
    }
    
    hunks
}

// ==================== Patch Application ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchError {
    PathNotFound(String),
    InvalidPath(String),
    TestFailed(String),
    TypeMismatch(String),
    ApplicationFailed(String),
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchError::PathNotFound(p) => write!(f, "Path not found: {}", p),
            PatchError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
            PatchError::TestFailed(p) => write!(f, "Test failed at: {}", p),
            PatchError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            PatchError::ApplicationFailed(msg) => write!(f, "Application failed: {}", msg),
        }
    }
}

impl std::error::Error for PatchError {}

/// Apply a patch to a value
pub fn apply_patch(patch: &Patch, doc: &mut Value) -> Result<(), PatchError> {
    match &patch.operations {
        PatchOperations::Json(ops) => {
            for op in ops {
                op.apply(doc)?;
            }
            Ok(())
        }
        PatchOperations::TextDiff(_) => {
            Err(PatchError::TypeMismatch(
                "Cannot apply text diff to JSON value".to_string()
            ))
        }
    }
}

/// Apply a text diff patch to file content
pub fn apply_text_patch(hunks: &[DiffHunk], content: &str) -> Result<String, PatchError> {
    let mut lines: Vec<&str> = content.lines().collect();
    let mut offset: i64 = 0;
    
    for hunk in hunks {
        let adjusted_start = (hunk.old_start as i64 + offset - 1) as usize;
        
        // Verify old content matches
        for (i, old_line) in hunk.old_lines.iter().enumerate() {
            if let Some(current) = lines.get(adjusted_start + i) {
                if *current != old_line {
                    return Err(PatchError::ApplicationFailed(format!(
                        "Line {} mismatch", hunk.old_start + i
                    )));
                }
            }
        }
        
        // Remove old lines
        for _ in 0..hunk.old_count {
            if adjusted_start < lines.len() {
                lines.remove(adjusted_start);
            }
        }
        
        // Insert new lines
        for (i, new_line) in hunk.new_lines.iter().enumerate() {
            lines.insert(adjusted_start + i, new_line);
        }
        
        offset += hunk.new_count as i64 - hunk.old_count as i64;
    }
    
    Ok(lines.join("\n"))
}

// ==================== Audit Log ====================

/// Entry in the patch audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub action: AuditAction,
    pub patch: Patch,
    pub status: AuditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Proposed,
    Applied,
    Rejected,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Pending,
    Approved,
    Rejected,
    Applied,
    RolledBack,
}

/// Audit log for tracking all patches
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
    
    pub fn record_proposal(&mut self, agent_id: impl Into<String>, patch: Patch) -> String {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            agent_id: agent_id.into(),
            action: AuditAction::Proposed,
            patch,
            status: AuditStatus::Pending,
        };
        let id = entry.id.clone();
        self.entries.push(entry);
        id
    }
    
    pub fn record_applied(&mut self, entry_id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == entry_id) {
            entry.action = AuditAction::Applied;
            entry.status = AuditStatus::Applied;
        }
    }
    
    pub fn record_rejected(&mut self, entry_id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == entry_id) {
            entry.action = AuditAction::Rejected;
            entry.status = AuditStatus::Rejected;
        }
    }
    
    pub fn get_pending(&self) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| matches!(e.status, AuditStatus::Pending))
            .collect()
    }
    
    pub fn get_by_agent(&self, agent_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== JSON Pointer Helpers ====================

fn parse_json_pointer(path: &str) -> Result<Vec<String>, PatchError> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    if !path.starts_with('/') {
        return Err(PatchError::InvalidPath(format!(
            "JSON pointer must start with '/': {}", path
        )));
    }
    
    Ok(path[1..]
        .split('/')
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn json_pointer_get<'a>(doc: &'a Value, path: &str) -> Result<Option<&'a Value>, PatchError> {
    let parts = parse_json_pointer(path)?;
    let mut current = doc;
    
    for part in parts {
        current = match current {
            Value::Object(map) => map.get(&part),
            Value::Array(arr) => {
                let idx: usize = part.parse()
                    .map_err(|_| PatchError::InvalidPath(format!("Invalid array index: {}", part)))?;
                arr.get(idx)
            }
            _ => None,
        }.ok_or_else(|| PatchError::PathNotFound(path.to_string()))?;
    }
    
    Ok(Some(current))
}

fn json_pointer_set(doc: &mut Value, path: &str, value: Value) -> Result<(), PatchError> {
    let parts = parse_json_pointer(path)?;
    
    if parts.is_empty() {
        *doc = value;
        return Ok(());
    }
    
    let mut current = doc;
    
    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        
        current = match current {
            Value::Object(map) => {
                if is_last {
                    map.insert(part.clone(), value.clone());
                    return Ok(());
                }
                map.entry(part.clone()).or_insert(Value::Null)
            }
            Value::Array(arr) => {
                let idx: usize = if part == "-" {
                    if is_last {
                        arr.push(value.clone());
                        return Ok(());
                    }
                    arr.len()
                } else {
                    part.parse().map_err(|_| PatchError::InvalidPath(format!("Invalid index: {}", part)))?
                };
                
                if is_last {
                    if idx < arr.len() {
                        arr[idx] = value.clone();
                    } else {
                        arr.push(value.clone());
                    }
                    return Ok(());
                }
                
                arr.get_mut(idx).ok_or_else(|| PatchError::PathNotFound(path.to_string()))?
            }
            _ => return Err(PatchError::TypeMismatch(format!("Cannot traverse into: {}", part))),
        };
    }
    
    Ok(())
}

fn json_pointer_remove(doc: &mut Value, path: &str) -> Result<(), PatchError> {
    let parts = parse_json_pointer(path)?;
    
    if parts.is_empty() {
        return Err(PatchError::InvalidPath("Cannot remove root".to_string()));
    }
    
    let parent_path: Vec<&str> = parts[..parts.len()-1].iter().map(|s| s.as_str()).collect();
    let last = parts.last().unwrap();
    
    let parent = if parent_path.is_empty() {
        doc
    } else {
        let parent_ptr = format!("/{}", parent_path.join("/"));
        // Get mutable reference to parent
        let mut current = &mut *doc;
        for part in &parent_path {
            current = match current {
                Value::Object(map) => map.get_mut(*part),
                Value::Array(arr) => {
                    let idx: usize = part.parse().map_err(|_| PatchError::InvalidPath(part.to_string()))?;
                    arr.get_mut(idx)
                }
                _ => None,
            }.ok_or_else(|| PatchError::PathNotFound(parent_ptr.clone()))?;
        }
        current
    };
    
    match parent {
        Value::Object(map) => {
            map.remove(last).ok_or_else(|| PatchError::PathNotFound(path.to_string()))?;
        }
        Value::Array(arr) => {
            let idx: usize = last.parse()
                .map_err(|_| PatchError::InvalidPath(format!("Invalid array index: {}", last)))?;
            if idx < arr.len() {
                arr.remove(idx);
            } else {
                return Err(PatchError::PathNotFound(path.to_string()));
            }
        }
        _ => return Err(PatchError::TypeMismatch("Parent is not object or array".to_string())),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_patch_add() {
        let mut doc = serde_json::json!({"foo": "bar"});
        let op = JsonPatchOp::add("/baz", serde_json::json!("qux"));
        op.apply(&mut doc).unwrap();
        assert_eq!(doc, serde_json::json!({"foo": "bar", "baz": "qux"}));
    }
    
    #[test]
    fn test_json_patch_replace() {
        let mut doc = serde_json::json!({"foo": "bar"});
        let op = JsonPatchOp::replace("/foo", serde_json::json!("baz"));
        op.apply(&mut doc).unwrap();
        assert_eq!(doc, serde_json::json!({"foo": "baz"}));
    }
    
    #[test]
    fn test_json_patch_remove() {
        let mut doc = serde_json::json!({"foo": "bar", "baz": "qux"});
        let op = JsonPatchOp::remove("/baz");
        op.apply(&mut doc).unwrap();
        assert_eq!(doc, serde_json::json!({"foo": "bar"}));
    }
    
    #[test]
    fn test_text_diff() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";
        
        let hunks = create_text_diff(old, new);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_lines, vec!["line2"]);
        assert_eq!(hunks[0].new_lines, vec!["modified"]);
    }
    
    #[test]
    fn test_apply_text_patch() {
        let content = "line1\nline2\nline3";
        let hunks = vec![DiffHunk::replace(2, vec!["line2".to_string()], vec!["modified".to_string()])];
        
        let result = apply_text_patch(&hunks, content).unwrap();
        assert_eq!(result, "line1\nmodified\nline3");
    }
    
    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new();
        
        let patch = Patch::json_patch(
            "Add new state",
            PatchTarget::FsmGraph,
            vec![JsonPatchOp::add("/states/-", serde_json::json!({"id": "test"}))],
        );
        
        let id = log.record_proposal("test_agent", patch);
        assert_eq!(log.get_pending().len(), 1);
        
        log.record_applied(&id);
        assert_eq!(log.get_pending().len(), 0);
    }
}
