// Project Management Commands

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// Encryption
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2;
use sha2::Sha256;
use hmac::Hmac;
use rand::{rngs::OsRng, RngCore};
use std::io::{Read, Write};

const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const ITERATIONS: u32 = 600_000;
const DEFAULT_APP_KEY: &str = "NEUROBENCH_INTERNAL_SECURE_KEY_2024_V1"; // Obfuscation key

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

/// Save project to disk (Plain Text)
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

/// Secure Save Check (Helper)
fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, ITERATIONS, &mut key)
        .expect("PBKDF2 failed");
    key
}

/// Secure Save Project (Encrypted)
#[tauri::command]
pub fn secure_save_project(
    project_json: String,
    path: String,
    password: Option<String>,
) -> Result<String, String> {
    // If password provided, use it. Otherwise use internal default key for transparent encryption.
    let pwd = password.unwrap_or_else(|| DEFAULT_APP_KEY.to_string());
    
    // Encrypt
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let key = derive_key(&pwd, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    
    let ciphertext = cipher
        .encrypt(nonce, project_json.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
        
    // Layout: [SALT] [NONCE] [CIPHERTEXT]
    let mut file_content = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + ciphertext.len());
    file_content.extend_from_slice(&salt);
    file_content.extend_from_slice(&nonce_bytes);
    file_content.extend_from_slice(&ciphertext);
    
    // Create folder structure if needed
    if let Some(parent) = std::path::Path::new(&path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    std::fs::write(&path, file_content).map_err(|e| format!("Failed to write file: {}", e))?;
    log::info!("Saved encrypted project to: {}", path);

    Ok(path)
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

/// Secure Load Project (Decrypted)
#[tauri::command]
pub fn secure_load_project(path: String, password: Option<String>) -> Result<String, String> {
    let file_bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Check if it looks encrypted (min size)
    if file_bytes.len() < SALT_SIZE + NONCE_SIZE {
        // Likely plaintext or empty
        return String::from_utf8(file_bytes.clone())
            .map_err(|_| "TRAP_PLAINTEXT: File is not valid UTF-8 and too short to be encrypted".to_string());
    }

    let salt = &file_bytes[0..SALT_SIZE];
    let nonce_bytes = &file_bytes[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let ciphertext = &file_bytes[SALT_SIZE + NONCE_SIZE..];
    let nonce = Nonce::from_slice(nonce_bytes);

    // Helper closure to try decryption
    let try_decrypt = |pwd: &str| -> Result<String, String> {
        let key = derive_key(pwd, salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed".to_string())?;
            
        String::from_utf8(plaintext).map_err(|e| e.to_string())
    };

    // Strategy:
    // 1. If password provided, try it.
    // 2. If no password provided, try DEFAULT_APP_KEY.
    // 3. If DEFAULT_APP_KEY fails, return "TRAP_PASSWORD_REQUIRED".
    
    if let Some(pwd) = password {
        // User provided specific password
        return try_decrypt(&pwd).map_err(|_| "TRAP_WRONG_PASSWORD".to_string());
    } else {
        // Try default key first (Automatic Decryption)
        match try_decrypt(DEFAULT_APP_KEY) {
            Ok(json) => return Ok(json),
            Err(_) => {
                // Default key failed. This file implies it has a user password.
                return Err("TRAP_PASSWORD_REQUIRED".to_string());
            }
        }
    }
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

