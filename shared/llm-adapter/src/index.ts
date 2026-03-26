/**
 * LLM Adapter Layer for Neurostate Agent Swarm
 * 
 * Provides unified interface for multiple LLM providers:
 * - Ollama (local models - FREE)
 * - LangChain (unified API with multiple provider support)
 * - Future: HuggingFace Inference API (FREE tier)
 * 
 * Designed for cost-effective, privacy-focused AI integration
 */

import { z } from 'zod';

// ============ Type Definitions ============

export type LLMProvider = 'ollama' | 'langchain-ollama' | 'huggingface';

export interface LLMConfig {
  provider: LLMProvider;
  model: string;
  baseUrl?: string; // For Ollama: http://localhost:11434
  apiKey?: string; // For HuggingFace
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  stopSequences?: string[];
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface LLMResponse {
  content: string;
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  model: string;
  provider: LLMProvider;
}

export interface IntentClassificationResult {
  intent: string;
  confidence: number;
  entities?: Record<string, any>;
  suggestedWorkflow?: string;
}

export interface TaskDecompositionResult {
  tasks: Array<{
    id: string;
    description: string;
    agentType: string;
    priority: number;
    dependencies?: string[];
    estimatedComplexity: 'low' | 'medium' | 'high';
  }>;
  workflowName: string;
  totalTasks: number;
}

// ============ Schema Definitions ============

export const IntentClassificationSchema = z.object({
  intent: z.string().describe('The classified intent type'),
  confidence: z.number().min(0).max(1).describe('Confidence score 0-1'),
  entities: z.record(z.any()).optional().describe('Extracted entities'),
  suggestedWorkflow: z.string().optional().describe('Suggested workflow name')
});

export const TaskDecompositionSchema = z.object({
  tasks: z.array(z.object({
    id: z.string(),
    description: z.string(),
    agentType: z.string(),
    priority: z.number().int().min(1).max(10),
    dependencies: z.array(z.string()).optional(),
    estimatedComplexity: z.enum(['low', 'medium', 'high'])
  })),
  workflowName: z.string(),
  totalTasks: z.number().int().min(1)
});

// ============ Base LLM Adapter Interface ============

export interface LLMAdapter {
  /**
   * Send a chat completion request
   */
  chat(messages: ChatMessage[], schema?: z.ZodSchema): Promise<LLMResponse>;
  
  /**
   * Classify user intent
   */
  classifyIntent(userRequest: string): Promise<IntentClassificationResult>;
  
  /**
   * Decompose request into tasks
   */
  decomposeTasks(intent: string, userRequest: string, availableAgents: string[]): Promise<TaskDecompositionResult>;
  
  /**
   * Check if the adapter is healthy/ready
   */
  healthCheck(): Promise<boolean>;
  
  /**
   * Get current configuration
   */
  getConfig(): LLMConfig;
}

// ============ Ollama Adapter (Native) ============

export class OllamaAdapter implements LLMAdapter {
  private config: LLMConfig;
  private ollamaClient: any;

  constructor(config: LLMConfig) {
    this.config = {
      ...config,
      baseUrl: config.baseUrl || 'http://localhost:11434',
      temperature: config.temperature ?? 0.7,
      maxTokens: config.maxTokens ?? 2048
    };
    
    // Lazy import to avoid issues in non-Node environments
    this.initializeClient();
  }

  private async initializeClient() {
    try {
      const { Ollama } = await import('ollama');
      this.ollamaClient = new Ollama({ 
        host: this.config.baseUrl 
      });
    } catch (error) {
      console.error('[OllamaAdapter] Failed to initialize client:', error);
      throw new Error('Ollama package not available. Run: npm install ollama');
    }
  }

  async chat(messages: ChatMessage[], schema?: z.ZodSchema): Promise<LLMResponse> {
    if (!this.ollamaClient) {
      await this.initializeClient();
    }

    const systemMessage = messages.find(m => m.role === 'system');
    const userMessages = messages.filter(m => m.role !== 'system');
    
    // Format prompt for Ollama
    let fullPrompt = '';
    if (systemMessage) {
      fullPrompt += `System: ${systemMessage.content}\n\n`;
    }
    
    userMessages.forEach(msg => {
      fullPrompt += `${msg.role === 'user' ? 'User' : 'Assistant'}: ${msg.content}\n\n`;
    });

    // Add schema instruction if provided
    if (schema) {
      fullPrompt += `\nRespond ONLY with valid JSON matching this schema. Do not include any other text.\nSchema: ${JSON.stringify(schema.describe('response schema'))}`;
    }

    const response = await this.ollamaClient.chat({
      model: this.config.model,
      messages: [
        ...(systemMessage ? [{ role: 'system' as const, content: systemMessage.content }] : []),
        ...userMessages.map(m => ({ role: m.role as 'user' | 'assistant', content: m.content }))
      ],
      format: schema ? 'json' : undefined,
      options: {
        temperature: this.config.temperature,
        num_predict: this.config.maxTokens,
        top_p: this.config.topP ?? 0.9,
        stop: this.config.stopSequences
      }
    });

    let content = response.message?.content || '';
    
    // If schema provided, validate and parse JSON
    if (schema && content.trim()) {
      try {
        // Extract JSON from response if wrapped in markdown
        const jsonMatch = content.match(/```(?:json)?\s*([\s\S]*?)\s*```/) || 
                         content.match(/\{[\s\S]*\}/);
        if (jsonMatch) {
          content = jsonMatch[1] || jsonMatch[0];
        }
        
        const parsed = JSON.parse(content.trim());
        const validated = schema.parse(parsed);
        content = JSON.stringify(validated);
      } catch (error) {
        console.error('[OllamaAdapter] JSON parsing/validation failed:', error);
        throw new Error(`Invalid JSON response: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    }

    return {
      content,
      usage: {
        promptTokens: response.prompt_eval_count || 0,
        completionTokens: response.eval_count || 0,
        totalTokens: (response.prompt_eval_count || 0) + (response.eval_count || 0)
      },
      model: this.config.model,
      provider: 'ollama'
    };
  }

  async classifyIntent(userRequest: string): Promise<IntentClassificationResult> {
    const systemPrompt = `You are an intent classifier for a software development AI agent swarm.
Classify the user's request into one of these intents:
- code_generation: Writing new code, drivers, or modules
- code_refactoring: Improving existing code structure
- testing: Creating or running tests
- debugging: Fixing bugs or issues
- architecture: System design or architectural changes
- documentation: Writing docs or comments
- deployment: Deploying or configuring CI/CD
- security: Security analysis or fixes
- hardware: Hardware-related tasks (pinout, electrical, etc.)
- optimization: Performance or efficiency improvements
- analysis: Code review or static analysis
- configuration: Config files or environment setup

Respond with JSON: {"intent": "string", "confidence": number, "entities": {}, "suggestedWorkflow": "string"}
Be concise and accurate.`;

    const response = await this.chat([
      { role: 'system', content: systemPrompt },
      { role: 'user', content: userRequest }
    ], IntentClassificationSchema);

    return JSON.parse(response.content) as IntentClassificationResult;
  }

  async decomposeTasks(intent: string, userRequest: string, availableAgents: string[]): Promise<TaskDecompositionResult> {
    const systemPrompt = `You are a task decomposition engine for a software development AI agent swarm.
Available agents: ${availableAgents.join(', ')}

Break down the user's request into specific, actionable tasks.
Each task should:
- Have a clear description
- Be assigned to an appropriate agent
- Have a priority (1-10, 10 being highest)
- List dependencies if any
- Estimate complexity (low/medium/high)

Respond with JSON matching the schema.`;

    const response = await this.chat([
      { role: 'system', content: systemPrompt },
      { role: 'user', content: `Intent: ${intent}\n\nRequest: ${userRequest}` }
    ], TaskDecompositionSchema);

    return JSON.parse(response.content) as TaskDecompositionResult;
  }

  async healthCheck(): Promise<boolean> {
    try {
      if (!this.ollamaClient) {
        await this.initializeClient();
      }
      
      const { Ollama } = await import('ollama');
      const testClient = new Ollama({ host: this.config.baseUrl });
      await testClient.list();
      return true;
    } catch (error) {
      console.error('[OllamaAdapter] Health check failed:', error);
      return false;
    }
  }

  getConfig(): LLMConfig {
    return { ...this.config };
  }
}

// ============ LangChain Ollama Adapter ============

export class LangChainOllamaAdapter implements LLMAdapter {
  private config: LLMConfig;
  private chainInstance: any;

  constructor(config: LLMConfig) {
    this.config = {
      ...config,
      baseUrl: config.baseUrl || 'http://localhost:11434',
      temperature: config.temperature ?? 0.7,
      maxTokens: config.maxTokens ?? 2048
    };
  }

  private async initializeChain() {
    try {
      const { ChatOllama } = await import('@langchain/ollama');
      const { HumanMessage, SystemMessage } = await import('@langchain/core/messages');
      
      this.chainInstance = {
        ChatOllama,
        HumanMessage,
        SystemMessage,
        model: new ChatOllama({
          model: this.config.model,
          baseUrl: this.config.baseUrl,
          temperature: this.config.temperature,
          maxTokens: this.config.maxTokens
        })
      };
    } catch (error) {
      console.error('[LangChainOllamaAdapter] Failed to initialize:', error);
      throw new Error('LangChain packages not available. Run: npm install @langchain/core @langchain/ollama');
    }
  }

  async chat(messages: ChatMessage[], schema?: z.ZodSchema): Promise<LLMResponse> {
    if (!this.chainInstance) {
      await this.initializeChain();
    }

    const { HumanMessage, SystemMessage, model } = this.chainInstance;
    
    const langchainMessages = messages.map(msg => {
      if (msg.role === 'system') {
        return new SystemMessage(msg.content);
      } else if (msg.role === 'user') {
        return new HumanMessage(msg.content);
      } else {
        return new HumanMessage(msg.content); // Treat assistant as human for simplicity
      }
    });

    const response = await model.invoke(langchainMessages);
    
    let content = typeof response.content === 'string' 
      ? response.content 
      : JSON.stringify(response.content);

    // Validate against schema if provided
    if (schema && content.trim()) {
      try {
        const jsonMatch = content.match(/```(?:json)?\s*([\s\S]*?)\s*```/) || 
                         content.match(/\{[\s\S]*\}/);
        if (jsonMatch) {
          content = jsonMatch[1] || jsonMatch[0];
        }
        
        const parsed = JSON.parse(content.trim());
        const validated = schema.parse(parsed);
        content = JSON.stringify(validated);
      } catch (error) {
        console.error('[LangChainOllamaAdapter] JSON validation failed:', error);
      }
    }

    return {
      content,
      usage: {
        promptTokens: 0, // LangChain doesn't always provide token counts
        completionTokens: 0,
        totalTokens: 0
      },
      model: this.config.model,
      provider: 'langchain-ollama'
    };
  }

  async classifyIntent(userRequest: string): Promise<IntentClassificationResult> {
    const systemPrompt = `You are an intent classifier for a software development AI agent swarm.
Classify the user's request into one of these intents:
- code_generation, code_refactoring, testing, debugging, architecture, 
  documentation, deployment, security, hardware, optimization, analysis, configuration

Respond with JSON: {"intent": "string", "confidence": number, "entities": {}, "suggestedWorkflow": "string"}`;

    const response = await this.chat([
      { role: 'system', content: systemPrompt },
      { role: 'user', content: userRequest }
    ], IntentClassificationSchema);

    return JSON.parse(response.content) as IntentClassificationResult;
  }

  async decomposeTasks(intent: string, userRequest: string, availableAgents: string[]): Promise<TaskDecompositionResult> {
    const systemPrompt = `You are a task decomposition engine.
Available agents: ${availableAgents.join(', ')}

Break down the request into specific tasks with JSON output.`;

    const response = await this.chat([
      { role: 'system', content: systemPrompt },
      { role: 'user', content: `Intent: ${intent}\n\nRequest: ${userRequest}` }
    ], TaskDecompositionSchema);

    return JSON.parse(response.content) as TaskDecompositionResult;
  }

  async healthCheck(): Promise<boolean> {
    try {
      await this.initializeChain();
      const { ChatOllama } = await import('@langchain/ollama');
      const testModel = new ChatOllama({
        model: this.config.model,
        baseUrl: this.config.baseUrl
      });
      await testModel.invoke('Hello');
      return true;
    } catch (error) {
      console.error('[LangChainOllamaAdapter] Health check failed:', error);
      return false;
    }
  }

  getConfig(): LLMConfig {
    return { ...this.config };
  }
}

// ============ Factory Function ============

export function createLLMAdapter(config: LLMConfig): LLMAdapter {
  switch (config.provider) {
    case 'ollama':
      return new OllamaAdapter(config);
    case 'langchain-ollama':
      return new LangChainOllamaAdapter(config);
    default:
      throw new Error(`Unsupported LLM provider: ${config.provider}`);
  }
}

// ============ Default Export with Pre-configured Adapters ============

export const defaultConfigs = {
  ollama: {
    provider: 'ollama' as LLMProvider,
    model: 'qwen2.5:7b', // Default to a good general-purpose model
    baseUrl: 'http://localhost:11434',
    temperature: 0.7,
    maxTokens: 2048
  },
  ollamaLarge: {
    provider: 'ollama' as LLMProvider,
    model: 'qwen2.5:14b', // For complex reasoning
    baseUrl: 'http://localhost:11434',
    temperature: 0.5,
    maxTokens: 4096
  },
  ollamaFast: {
    provider: 'ollama' as LLMProvider,
    model: 'qwen2.5:3b', // For quick classifications
    baseUrl: 'http://localhost:11434',
    temperature: 0.3,
    maxTokens: 512
  },
  langchainOllama: {
    provider: 'langchain-ollama' as LLMProvider,
    model: 'qwen2.5:7b',
    baseUrl: 'http://localhost:11434',
    temperature: 0.7,
    maxTokens: 2048
  }
};

export default createLLMAdapter;
