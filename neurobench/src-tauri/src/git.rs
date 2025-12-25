// Git Integration Module
// Provides version control for generated projects

use git2::{
    Commit, DiffOptions, Error, IndexAddOption, ObjectType, Oid, Repository, 
    Signature, StatusOptions, StatusShow, Time,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// File status in the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,           // "new", "modified", "deleted", "renamed", "untracked"
    pub staged: bool,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub short_id: String,
}

/// Diff information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffInfo {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub patches: Vec<FilePatch>,
}

/// Patch for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: String,
    pub status: String,
    pub hunks: Vec<String>,
}

/// Repository status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub files: Vec<FileStatus>,
    pub staged_count: usize,
    pub modified_count: usize,
    pub untracked_count: usize,
}

/// Initialize a new Git repository
pub fn init_repo(path: &str) -> Result<String, String> {
    let repo = Repository::init(path)
        .map_err(|e| format!("Failed to init repo: {}", e))?;
    
    Ok(repo.path().to_string_lossy().to_string())
}

/// Get the status of a repository
pub fn get_status(path: &str) -> Result<RepoStatus, String> {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return Ok(RepoStatus {
            is_repo: false,
            branch: None,
            files: vec![],
            staged_count: 0,
            modified_count: 0,
            untracked_count: 0,
        }),
    };

    // Get current branch
    let branch = repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    // Get file statuses
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .show(StatusShow::IndexAndWorkdir);

    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| format!("Failed to get status: {}", e))?;

    let mut files = Vec::new();
    let mut staged_count = 0;
    let mut modified_count = 0;
    let mut untracked_count = 0;

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        
        let (status_str, staged) = if status.is_index_new() {
            staged_count += 1;
            ("new", true)
        } else if status.is_index_modified() {
            staged_count += 1;
            ("modified", true)
        } else if status.is_index_deleted() {
            staged_count += 1;
            ("deleted", true)
        } else if status.is_wt_new() {
            untracked_count += 1;
            ("untracked", false)
        } else if status.is_wt_modified() {
            modified_count += 1;
            ("modified", false)
        } else if status.is_wt_deleted() {
            modified_count += 1;
            ("deleted", false)
        } else {
            continue;
        };

        files.push(FileStatus {
            path,
            status: status_str.to_string(),
            staged,
        });
    }

    Ok(RepoStatus {
        is_repo: true,
        branch,
        files,
        staged_count,
        modified_count,
        untracked_count,
    })
}

/// Stage files for commit
pub fn stage_files(path: &str, files: &[&str]) -> Result<usize, String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    for file in files {
        index.add_path(Path::new(file))
            .map_err(|e| format!("Failed to stage {}: {}", file, e))?;
    }

    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(files.len())
}

/// Stage all changes
pub fn stage_all(path: &str) -> Result<usize, String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to add all: {}", e))?;

    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(index.len())
}

/// Create a commit
pub fn commit(path: &str, message: &str, author_name: &str, author_email: &str) -> Result<CommitInfo, String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    let tree_oid = index.write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;

    let tree = repo.find_tree(tree_oid)
        .map_err(|e| format!("Failed to find tree: {}", e))?;

    let sig = Signature::now(author_name, author_email)
        .map_err(|e| format!("Failed to create signature: {}", e))?;

    // Get parent commit (if exists)
    let parent = repo.head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok());

    let parents: Vec<&Commit> = parent.as_ref().map(|c| vec![c]).unwrap_or_default();

    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parents,
    ).map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(CommitInfo {
        id: oid.to_string(),
        short_id: oid.to_string()[..7].to_string(),
        message: message.to_string(),
        author: author_name.to_string(),
        email: author_email.to_string(),
        time: sig.when().seconds(),
    })
}

/// Get commit history
pub fn get_history(path: &str, limit: usize) -> Result<Vec<CommitInfo>, String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    revwalk.push_head()
        .map_err(|e| format!("Failed to push head: {}", e))?;

    let mut commits = Vec::new();

    for (i, oid_result) in revwalk.enumerate() {
        if i >= limit {
            break;
        }

        let oid = oid_result.map_err(|e| format!("Failed to get oid: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let author = commit.author();
        
        commits.push(CommitInfo {
            id: oid.to_string(),
            short_id: oid.to_string()[..7].to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("").to_string(),
            email: author.email().unwrap_or("").to_string(),
            time: author.when().seconds(),
        });
    }

    Ok(commits)
}

/// Get diff between working tree and HEAD
pub fn get_diff(path: &str) -> Result<DiffInfo, String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let head = repo.head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let mut opts = DiffOptions::new();
    opts.include_untracked(true);

    let diff = repo.diff_tree_to_workdir(head.as_ref(), Some(&mut opts))
        .map_err(|e| format!("Failed to get diff: {}", e))?;

    let stats = diff.stats()
        .map_err(|e| format!("Failed to get diff stats: {}", e))?;

    let mut patches = Vec::new();

    for (i, delta) in diff.deltas().enumerate() {
        let path = delta.new_file().path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let status = match delta.status() {
            git2::Delta::Added => "added",
            git2::Delta::Deleted => "deleted",
            git2::Delta::Modified => "modified",
            git2::Delta::Renamed => "renamed",
            _ => "unknown",
        };

        patches.push(FilePatch {
            path,
            status: status.to_string(),
            hunks: vec![], // Full hunks omitted for brevity
        });
    }

    Ok(DiffInfo {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
        patches,
    })
}

/// Unstage a file
pub fn unstage_file(path: &str, file: &str) -> Result<(), String> {
    let repo = Repository::open(path)
        .map_err(|e| format!("Failed to open repo: {}", e))?;

    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?
        .peel_to_commit()
        .map_err(|e| format!("Failed to get commit: {}", e))?;

    repo.reset_default(Some(head.as_object()), [Path::new(file)])
        .map_err(|e| format!("Failed to unstage: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_repo() {
        let dir = tempdir().unwrap();
        let result = init_repo(dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_status_non_repo() {
        let dir = tempdir().unwrap();
        let status = get_status(dir.path().to_str().unwrap()).unwrap();
        assert!(!status.is_repo);
    }

    #[test]
    fn test_get_status_repo() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        init_repo(path).unwrap();
        
        let status = get_status(path).unwrap();
        assert!(status.is_repo);
    }
}
