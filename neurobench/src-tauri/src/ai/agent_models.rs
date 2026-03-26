//! Agent Model Configuration
//! 
//! Maps each agent to its optimal LLM model with fallback support.
//! Uses hybrid strategy: powerful models for complex tasks, fast models for simple tasks.

use super::providers::{ModelConfig, ModelProvider, ModelManager, OpenAIModel, OllamaModel, ClaudeModel};
use crate::ai::GeminiModel;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Agent-specific model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelConfig {
    pub agent_id: String,
    pub primary_provider: ModelProvider,
    pub primary_model: String,
    pub fallback_provider: Option<ModelProvider>,
    pub fallback_model: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    /// Whether this agent supports function/tool calling
    pub supports_tools: bool,
}

impl AgentModelConfig {
    /// Create config for Director Agent (needs best reasoning)
    pub fn director() -> Self {
        Self {
            agent_id: "director".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-5.2".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-2.0-pro".to_string()),
            temperature: 0.7,
            max_tokens: 8192,
            supports_tools: true,
        }
    }
    
    /// Create config for Code Agent (needs best code generation)
    pub fn code() -> Self {
        Self {
            agent_id: "code".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-4o".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-1.5-pro".to_string()),
            temperature: 0.3, // Lower for more deterministic code
            max_tokens: 16384, // Large for code generation
            supports_tools: true,
        }
    }
    
    /// Create config for Debug Agent (needs deep analysis)
    pub fn debug() -> Self {
        Self {
            agent_id: "debug".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-4o".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-1.5-pro".to_string()),
            temperature: 0.5,
            max_tokens: 8192,
            supports_tools: true,
        }
    }
    
    /// Create config for Hardware Agent (needs technical knowledge)
    pub fn hardware() -> Self {
        Self {
            agent_id: "hardware".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-4o".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-1.5-pro".to_string()),
            temperature: 0.4,
            max_tokens: 8192,
            supports_tools: true,
        }
    }
    
    /// Create config for FSM Agent (design tasks)
    pub fn fsm() -> Self {
        Self {
            agent_id: "fsm".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-4o".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-1.5-flash".to_string()),
            temperature: 0.6,
            max_tokens: 8192,
            supports_tools: true,
        }
    }
    
    /// Create config for Canvas Agent (fast, structured)
    pub fn canvas() -> Self {
        Self {
            agent_id: "canvas".to_string(),
            primary_provider: ModelProvider::Gemini,
            primary_model: "gemini-3.0-flash".to_string(),
            fallback_provider: Some(ModelProvider::Custom),
            fallback_model: Some("nanobanana".to_string()),
            temperature: 0.5,
            max_tokens: 4096,
            supports_tools: true,
        }
    }
    
    /// Create config for Build Agent (compilation tasks)
    pub fn build() -> Self {
        Self {
            agent_id: "build".to_string(),
            primary_provider: ModelProvider::Custom,
            primary_model: "claude-4.5-opus".to_string(),
            fallback_provider: Some(ModelProvider::OpenAI),
            fallback_model: Some("gpt-5.1".to_string()),
            temperature: 0.3,
            max_tokens: 4096,
            supports_tools: true,
        }
    }
    
    /// Create config for Deploy Agent (flashing & deployment)
    pub fn deploy() -> Self {
        Self {
            agent_id: "deploy".to_string(),
            primary_provider: ModelProvider::Custom,
            primary_model: "claude-4.5-sonnet".to_string(),
            fallback_provider: Some(ModelProvider::OpenAI),
            fallback_model: Some("gpt-5.0".to_string()),
            temperature: 0.3,
            max_tokens: 2048,
            supports_tools: true,
        }
    }
    
    /// Create config for Docs Agent (documentation)
    pub fn docs() -> Self {
        Self {
            agent_id: "docs".to_string(),
            primary_provider: ModelProvider::Gemini,
            primary_model: "gemini-1.5-flash".to_string(),
            fallback_provider: Some(ModelProvider::Ollama),
            fallback_model: Some("llama3.2".to_string()),
            temperature: 0.5,
            max_tokens: 8192,
            supports_tools: false,
        }
    }
    
    /// Create config for Voice Assistant Agent (Grok-powered)
    pub fn voice() -> Self {
        Self {
            agent_id: "voice".to_string(),
            primary_provider: ModelProvider::Custom,
            primary_model: "grok-2".to_string(),
            fallback_provider: Some(ModelProvider::OpenAI),
            fallback_model: Some("gpt-4o".to_string()),
            temperature: 0.8,  // More creative for conversation
            max_tokens: 4096,
            supports_tools: true,
        }
    }
    
    /// Create config for Super AI Agent (Nexus)
    pub fn nexus() -> Self {
        Self {
            agent_id: "nexus".to_string(),
            primary_provider: ModelProvider::OpenAI,
            primary_model: "gpt-4o".to_string(),
            fallback_provider: Some(ModelProvider::Gemini),
            fallback_model: Some("gemini-2.0-pro".to_string()),
            temperature: 0.3,  // Precise for parsing
            max_tokens: 8192,
            supports_tools: true,
        }
    }
}

/// Registry of agent model configurations
pub struct AgentModelRegistry {
    configs: HashMap<String, AgentModelConfig>,
}

impl AgentModelRegistry {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Register all agent configs
        let all_configs = vec![
            AgentModelConfig::director(),
            AgentModelConfig::code(),
            AgentModelConfig::debug(),
            AgentModelConfig::hardware(),
            AgentModelConfig::fsm(),
            AgentModelConfig::canvas(),
            AgentModelConfig::build(),
            AgentModelConfig::deploy(),
            AgentModelConfig::docs(),
            AgentModelConfig::voice(),
            AgentModelConfig::nexus(),
        ];
        
        for config in all_configs {
            configs.insert(config.agent_id.clone(), config);
        }
        
        Self { configs }
    }
    
    /// Get config for an agent
    pub fn get(&self, agent_id: &str) -> Option<&AgentModelConfig> {
        self.configs.get(agent_id)
    }
    
    /// Update config for an agent
    pub fn update(&mut self, config: AgentModelConfig) {
        self.configs.insert(config.agent_id.clone(), config);
    }
    
    /// List all configurations
    pub fn list(&self) -> Vec<&AgentModelConfig> {
        self.configs.values().collect()
    }
    
    /// Create a ModelManager for the given agent
    pub fn create_model_manager(&self, agent_id: &str) -> Option<ModelManager> {
        let config = self.get(agent_id)?;
        
        // Create primary model
        let primary: Box<dyn super::providers::AIModel> = match config.primary_provider {
            ModelProvider::Claude => {
                let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
                Box::new(ClaudeModel::new(ModelConfig {
                    provider: ModelProvider::Claude,
                    model_name: config.primary_model.clone(),
                    api_key,
                    base_url: Some("https://api.anthropic.com".to_string()),
                    temperature: config.temperature,
                    max_tokens: config.max_tokens,
                    timeout_secs: 120,
                }))
            }
            ModelProvider::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").ok();
                Box::new(OpenAIModel::new(ModelConfig {
                    provider: ModelProvider::OpenAI,
                    model_name: config.primary_model.clone(),
                    api_key,
                    base_url: Some("https://api.openai.com/v1".to_string()),
                    temperature: config.temperature,
                    max_tokens: config.max_tokens,
                    timeout_secs: 120,
                }))
            }
            ModelProvider::Gemini => {
                let api_key = std::env::var("GEMINI_API_KEY").ok();
                Box::new(GeminiModel::new(ModelConfig {
                    provider: ModelProvider::Gemini,
                    model_name: config.primary_model.clone(),
                    api_key,
                    base_url: None,
                    temperature: config.temperature,
                    max_tokens: config.max_tokens,
                    timeout_secs: 120,
                }))
            }
            ModelProvider::Ollama => {
                Box::new(OllamaModel::new(ModelConfig {
                    provider: ModelProvider::Ollama,
                    model_name: config.primary_model.clone(),
                    api_key: None,
                    base_url: Some("http://localhost:11434".to_string()),
                    temperature: config.temperature,
                    max_tokens: config.max_tokens,
                    timeout_secs: 180,
                }))
            }
            ModelProvider::Custom => {
                // Default to Gemini for custom
                let api_key = std::env::var("GEMINI_API_KEY").ok();
                Box::new(GeminiModel::new(ModelConfig {
                    provider: ModelProvider::Gemini,
                    model_name: "gemini-1.5-flash".to_string(),
                    api_key,
                    ..Default::default()
                }))
            }
        };
        
        let mut manager = ModelManager::new(primary);
        
        // Add fallback if configured
        if let (Some(fallback_provider), Some(fallback_model)) = 
            (&config.fallback_provider, &config.fallback_model) 
        {
            let fallback: Box<dyn super::providers::AIModel> = match fallback_provider {
                ModelProvider::Claude => {
                    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
                    Box::new(ClaudeModel::new(ModelConfig {
                        provider: ModelProvider::Claude,
                        model_name: fallback_model.clone(),
                        api_key,
                        base_url: Some("https://api.anthropic.com".to_string()),
                        temperature: config.temperature,
                        max_tokens: config.max_tokens,
                        timeout_secs: 120,
                    }))
                }
                ModelProvider::OpenAI => {
                    let api_key = std::env::var("OPENAI_API_KEY").ok();
                    Box::new(OpenAIModel::new(ModelConfig {
                        provider: ModelProvider::OpenAI,
                        model_name: fallback_model.clone(),
                        api_key,
                        base_url: Some("https://api.openai.com/v1".to_string()),
                        temperature: config.temperature,
                        max_tokens: config.max_tokens,
                        timeout_secs: 120,
                    }))
                }
                ModelProvider::Gemini => {
                    let api_key = std::env::var("GEMINI_API_KEY").ok();
                    Box::new(GeminiModel::new(ModelConfig {
                        provider: ModelProvider::Gemini,
                        model_name: fallback_model.clone(),
                        api_key,
                        base_url: None,
                        temperature: config.temperature,
                        max_tokens: config.max_tokens,
                        timeout_secs: 120,
                    }))
                }
                ModelProvider::Ollama => {
                    Box::new(OllamaModel::new(ModelConfig {
                        provider: ModelProvider::Ollama,
                        model_name: fallback_model.clone(),
                        api_key: None,
                        base_url: Some("http://localhost:11434".to_string()),
                        temperature: config.temperature,
                        max_tokens: config.max_tokens,
                        timeout_secs: 180,
                    }))
                }
                ModelProvider::Custom => {
                    let api_key = std::env::var("GEMINI_API_KEY").ok();
                    Box::new(GeminiModel::new(ModelConfig {
                        provider: ModelProvider::Gemini,
                        model_name: "gemini-1.5-flash".to_string(),
                        api_key,
                        ..Default::default()
                    }))
                }
            };
            
            manager = manager.with_fallback(fallback);
        }
        
        Some(manager)
    }
}

impl Default for AgentModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of agent model assignments for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelSummary {
    pub agent_id: String,
    pub agent_name: String,
    pub primary: String,
    pub fallback: Option<String>,
    pub purpose: String,
}

impl AgentModelSummary {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                agent_id: "director".to_string(),
                agent_name: "Director Agent".to_string(),
                primary: "GPT-5.2".to_string(),
                fallback: Some("Gemini 2.0 Pro".to_string()),
                purpose: "Task planning, intent understanding, agent coordination".to_string(),
            },
            Self {
                agent_id: "code".to_string(),
                agent_name: "Code Agent".to_string(),
                primary: "GPT-4o".to_string(),
                fallback: Some("Gemini 1.5 Pro".to_string()),
                purpose: "Code generation, review, optimization".to_string(),
            },
            Self {
                agent_id: "debug".to_string(),
                agent_name: "Debug Agent".to_string(),
                primary: "GPT-4o".to_string(),
                fallback: Some("Gemini 1.5 Pro".to_string()),
                purpose: "Error analysis, debugging, troubleshooting".to_string(),
            },
            Self {
                agent_id: "hardware".to_string(),
                agent_name: "Hardware Agent".to_string(),
                primary: "GPT-4o".to_string(),
                fallback: Some("Gemini 1.5 Pro".to_string()),
                purpose: "MCU configuration, peripheral setup".to_string(),
            },
            Self {
                agent_id: "fsm".to_string(),
                agent_name: "FSM Agent".to_string(),
                primary: "GPT-4o".to_string(),
                fallback: Some("Gemini 1.5 Flash".to_string()),
                purpose: "State machine design, analysis".to_string(),
            },
            Self {
                agent_id: "canvas".to_string(),
                agent_name: "Canvas Agent".to_string(),
                primary: "Gemini Flash 3".to_string(),
                fallback: Some("NanoBanana".to_string()),
                purpose: "Canvas manipulation, layout, visualization".to_string(),
            },
            Self {
                agent_id: "build".to_string(),
                agent_name: "Build Agent".to_string(),
                primary: "Claude 4.5 Opus".to_string(),
                fallback: Some("GPT-5.1".to_string()),
                purpose: "Build automation, error fixing".to_string(),
            },
            Self {
                agent_id: "deploy".to_string(),
                agent_name: "Deploy Agent".to_string(),
                primary: "Claude 4.5 Sonnet".to_string(),
                fallback: Some("GPT-5.0".to_string()),
                purpose: "Flashing, deployment, device monitoring".to_string(),
            },
            Self {
                agent_id: "docs".to_string(),
                agent_name: "Docs Agent".to_string(),
                primary: "Gemini 1.5 Flash".to_string(),
                fallback: Some("Ollama (llama3.2)".to_string()),
                purpose: "Documentation generation, help".to_string(),
            },
            Self {
                agent_id: "voice".to_string(),
                agent_name: "Voice Assistant".to_string(),
                primary: "Grok-2".to_string(),
                fallback: Some("GPT-4o".to_string()),
                purpose: "Conversational AI, voice commands, idea refinement".to_string(),
            },
            Self {
                agent_id: "nexus".to_string(),
                agent_name: "Nexus (Super AI)".to_string(),
                primary: "GPT-4o".to_string(),
                fallback: Some("Gemini 2.0 Pro".to_string()),
                purpose: "Input parsing, task decomposition, multi-agent coordination".to_string(),
            },
        ]
    }
}
