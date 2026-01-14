/**
 * Ollama Provider - Local LLM for Offline Mode
 * Fallback when cloud providers are unavailable
 */

import { 
  LLMProvider, 
  GenerateConfig, 
  GenerateResult, 
  TaskType,
  registerProvider
} from './llmProvider';

const OLLAMA_BASE_URL = process.env.OLLAMA_URL || 'http://localhost:11434';
const DEFAULT_MODEL = 'codellama:13b'; // Good for code generation
const FAST_MODEL = 'llama3.2:3b'; // Fast for simple tasks

interface OllamaResponse {
  model: string;
  response: string;
  done: boolean;
  context?: number[];
  total_duration?: number;
  load_duration?: number;
  prompt_eval_count?: number;
  eval_count?: number;
}

class OllamaProvider implements LLMProvider {
  readonly name = 'ollama';
  readonly supportedTasks: TaskType[] = [
    'CODE_GENERATION',
    'GRAPH_CREATION',
    'VALIDATION',
    'CHAT'
  ];
  
  private lastHealthCheck: number = 0;
  private isHealthy: boolean = false;
  private availableModels: string[] = [];
  
  async isAvailable(): Promise<boolean> {
    // Cache health check for 30 seconds
    const now = Date.now();
    if (now - this.lastHealthCheck < 30000) {
      return this.isHealthy;
    }
    
    try {
      const response = await fetch(`${OLLAMA_BASE_URL}/api/tags`, {
        method: 'GET',
        signal: AbortSignal.timeout(5000)
      });
      
      if (!response.ok) {
        this.isHealthy = false;
        this.lastHealthCheck = now;
        return false;
      }
      
      const data = await response.json();
      this.availableModels = (data.models || []).map((m: any) => m.name);
      this.isHealthy = this.availableModels.length > 0;
      this.lastHealthCheck = now;
      
      return this.isHealthy;
    } catch (error) {
      console.warn('Ollama not available:', (error as Error).message);
      this.isHealthy = false;
      this.lastHealthCheck = now;
      return false;
    }
  }
  
  private selectModel(config?: GenerateConfig): string {
    // Prefer code-oriented model for code generation
    const preferCodeModel = config?.systemInstruction?.toLowerCase().includes('code') ||
                           config?.systemInstruction?.toLowerCase().includes('embedded');
    
    if (preferCodeModel && this.availableModels.includes(DEFAULT_MODEL)) {
      return DEFAULT_MODEL;
    }
    
    // Fallback to any available model
    if (this.availableModels.includes(FAST_MODEL)) {
      return FAST_MODEL;
    }
    
    return this.availableModels[0] || DEFAULT_MODEL;
  }
  
  async generate(prompt: string, config?: GenerateConfig): Promise<GenerateResult> {
    const model = this.selectModel(config);
    
    const systemPrompt = config?.systemInstruction || 
      'You are an expert embedded systems AI assistant.';
    
    const fullPrompt = `${systemPrompt}\n\n${prompt}`;
    
    try {
      const response = await fetch(`${OLLAMA_BASE_URL}/api/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model,
          prompt: fullPrompt,
          stream: false,
          options: {
            temperature: config?.temperature || 0.7,
            num_predict: config?.maxTokens || 4096
          }
        }),
        signal: AbortSignal.timeout(120000) // 2 minute timeout for generation
      });
      
      if (!response.ok) {
        throw new Error(`Ollama API error: ${response.status}`);
      }
      
      const data: OllamaResponse = await response.json();
      
      return {
        text: data.response,
        model: data.model,
        usage: {
          inputTokens: data.prompt_eval_count || 0,
          outputTokens: data.eval_count || 0
        }
      };
    } catch (error) {
      console.error('Ollama generation error:', error);
      throw error;
    }
  }
  
  async generateJSON<T>(prompt: string, schema: object, config?: GenerateConfig): Promise<T> {
    const jsonPrompt = `${prompt}

IMPORTANT: Respond with ONLY valid JSON matching this schema:
${JSON.stringify(schema, null, 2)}

Do not include any markdown formatting, explanation, or text outside the JSON. Only raw JSON.`;
    
    const result = await this.generate(jsonPrompt, config);
    
    try {
      // Clean potential markdown formatting
      let cleanJson = result.text
        .replace(/```json\n?/g, '')
        .replace(/```\n?/g, '')
        .trim();
      
      // Try to extract JSON if there's extra text
      const jsonMatch = cleanJson.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        cleanJson = jsonMatch[0];
      }
      
      return JSON.parse(cleanJson) as T;
    } catch (error) {
      console.error('Failed to parse Ollama JSON response:', result.text);
      throw new Error('Invalid JSON response from Ollama');
    }
  }
  
  async generateStream(
    prompt: string, 
    config?: GenerateConfig, 
    onChunk?: (chunk: string) => void
  ): Promise<GenerateResult> {
    const model = this.selectModel(config);
    
    const systemPrompt = config?.systemInstruction || 
      'You are an expert embedded systems AI assistant.';
    
    const fullPrompt = `${systemPrompt}\n\n${prompt}`;
    
    try {
      const response = await fetch(`${OLLAMA_BASE_URL}/api/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model,
          prompt: fullPrompt,
          stream: true,
          options: {
            temperature: config?.temperature || 0.7,
            num_predict: config?.maxTokens || 4096
          }
        })
      });
      
      if (!response.ok) {
        throw new Error(`Ollama API error: ${response.status}`);
      }
      
      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('No response body');
      }
      
      const decoder = new TextDecoder();
      let fullText = '';
      let inputTokens = 0;
      let outputTokens = 0;
      
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        
        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n').filter(line => line.trim());
        
        for (const line of lines) {
          try {
            const data: OllamaResponse = JSON.parse(line);
            if (data.response) {
              fullText += data.response;
              onChunk?.(data.response);
            }
            if (data.done) {
              inputTokens = data.prompt_eval_count || 0;
              outputTokens = data.eval_count || 0;
            }
          } catch (e) {
            // Ignore parse errors for incomplete chunks
          }
        }
      }
      
      return {
        text: fullText,
        model,
        usage: { inputTokens, outputTokens }
      };
    } catch (error) {
      console.error('Ollama streaming error:', error);
      throw error;
    }
  }
  
  /**
   * Pull a model if not already available
   */
  async pullModel(modelName: string): Promise<void> {
    try {
      const response = await fetch(`${OLLAMA_BASE_URL}/api/pull`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: modelName })
      });
      
      if (!response.ok) {
        throw new Error(`Failed to pull model: ${response.status}`);
      }
      
      // Stream the progress
      const reader = response.body?.getReader();
      if (reader) {
        const decoder = new TextDecoder();
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          console.log('Pull progress:', decoder.decode(value));
        }
      }
      
      // Refresh available models
      await this.isAvailable();
    } catch (error) {
      console.error('Failed to pull model:', error);
      throw error;
    }
  }
}

// Create singleton instance and register
const ollamaProvider = new OllamaProvider();
registerProvider(ollamaProvider);

export { ollamaProvider, OllamaProvider };
