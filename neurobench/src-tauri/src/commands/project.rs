// Project Management Commands

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Create a new project
#[tauri::command]
pub fn create_project(name: String, target_mcu: Option<String>) -> Result<FSMProject, String> {
    let mut project = FSMProject::new(&name);
    project.target_mcu = target_mcu;
    
    // Add default start and end nodes
    let start = FSMNode::new("START", NodeType::Input)
        .with_position(200.0, 100.0);
    let end = FSMNode::new("END", NodeType::Output)
        .with_position(200.0, 400.0);
    
    let start_id = start.id;
    let end_id = end.id;
    
    project.nodes.push(start);
    project.nodes.push(end);
    
    // Add initial edge
    project.edges.push(
        FSMEdge::new(start_id, end_id).with_label("START")
    );
    
    log::info!("Created new project: {}", name);
    Ok(project)
}

/// Save project to disk
#[tauri::command]
pub fn save_project(project: FSMProject, path: Option<String>) -> Result<String, String> {
    let save_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            // Default to user's documents folder
            let filename = format!("{}.neurobench.json", project.name.replace(" ", "_"));
            dirs::document_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(filename)
        }
    };
    
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    std::fs::write(&save_path, json)
        .map_err(|e| format!("Failed to save: {}", e))?;
    
    log::info!("Saved project to: {:?}", save_path);
    Ok(save_path.to_string_lossy().to_string())
}

/// Load project from disk
#[tauri::command]
pub fn load_project(path: String) -> Result<FSMProject, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let project: FSMProject = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project: {}", e))?;
    
    log::info!("Loaded project: {} from {}", project.name, path);
    Ok(project)
}

/// List saved projects in a directory
#[tauri::command]
pub fn list_projects(directory: Option<String>) -> Result<Vec<ProjectInfo>, String> {
    let dir = match directory {
        Some(d) => PathBuf::from(d),
        None => dirs::document_dir().unwrap_or_else(|| PathBuf::from(".")),
    };
    
    let mut projects = vec![];
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") 
                && path.file_name().map_or(false, |n| n.to_string_lossy().contains(".neurobench")) 
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(project) = serde_json::from_str::<FSMProject>(&content) {
                        projects.push(ProjectInfo {
                            id: project.id,
                            name: project.name,
                            path: path.to_string_lossy().to_string(),
                            updated_at: project.updated_at.to_rfc3339(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(projects)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub updated_at: String,
}
