/**
 * Claude Provider - Primary LLM for Neurostate AI Engine
 * Handles code generation, graph creation, validation, and chat
 */

import Anthropic from '@anthropic-ai/sdk';
import { 
  LLMProvider, 
  GenerateConfig, 
  GenerateResult, 
  TaskType,
  VoiceProvider,
  registerProvider,
  registerVoiceProvider
} from './llmProvider';

const CLAUDE_MODEL = 'claude-sonnet-4-20250514';
const CLAUDE_MODEL_FAST = 'claude-sonnet-4-20250514';

class ClaudeProvider implements LLMProvider, VoiceProvider {
  readonly name = 'claude';
  readonly supportedTasks: TaskType[] = [
    'CODE_GENERATION',
    'GRAPH_CREATION', 
    'VALIDATION',
    'CHAT',
    'VOICE',
    'TRANSCRIPTION'
  ];
  
  private client: Anthropic | null = null;
  private lastHealthCheck: number = 0;
  private isHealthy: boolean = false;
  
  constructor() {
    this.initClient();
  }
  
  private initClient(): void {
    const apiKey = process.env.ANTHROPIC_API_KEY;
    if (apiKey) {
      this.client = new Anthropic({ apiKey });
    }
  }
  
  async isAvailable(): Promise<boolean> {
    if (!this.client) {
      this.initClient();
      if (!this.client) return false;
    }
    
    // Cache health check for 30 seconds
    const now = Date.now();
    if (now - this.lastHealthCheck < 30000) {
      return this.isHealthy;
    }
    
    try {
      // Simple ping to check availability
      await this.client.messages.create({
        model: CLAUDE_MODEL_FAST,
        max_tokens: 10,
        messages: [{ role: 'user', content: 'ping' }]
      });
      this.isHealthy = true;
      this.lastHealthCheck = now;
      return true;
    } catch (error) {
      console.error('Claude health check failed:', error);
      this.isHealthy = false;
      this.lastHealthCheck = now;
      return false;
    }
  }
  
  async generate(prompt: string, config?: GenerateConfig): Promise<GenerateResult> {
    if (!this.client) {
      throw new Error('Claude API key not configured');
    }
    
    const systemPrompt = config?.systemInstruction || 
      'You are an expert embedded systems AI assistant for the NeuroState platform.';
    
    try {
      const response = await this.client.messages.create({
        model: config?.thinkingBudget ? CLAUDE_MODEL : CLAUDE_MODEL_FAST,
        max_tokens: config?.maxTokens || 8192,
        system: systemPrompt,
        messages: [{ role: 'user', content: prompt }]
      });
      
      const textContent = response.content.find(c => c.type === 'text');
      const text = textContent?.type === 'text' ? textContent.text : '';
      
      return {
        text,
        model: response.model,
        usage: {
          inputTokens: response.usage.input_tokens,
          outputTokens: response.usage.output_tokens
        }
      };
    } catch (error) {
      console.error('Claude generation error:', error);
      throw error;
    }
  }
  
  async generateJSON<T>(prompt: string, schema: object, config?: GenerateConfig): Promise<T> {
    const jsonPrompt = `${prompt}

IMPORTANT: Respond with ONLY valid JSON matching this schema:
${JSON.stringify(schema, null, 2)}

Do not include any markdown formatting or explanation. Only raw JSON.`;
    
    const result = await this.generate(jsonPrompt, {
      ...config,
      responseFormat: 'json'
    });
    
    try {
      // Clean potential markdown formatting
      let cleanJson = result.text
        .replace(/```json\n?/g, '')
        .replace(/```\n?/g, '')
        .trim();
      
      return JSON.parse(cleanJson) as T;
    } catch (error) {
      console.error('Failed to parse Claude JSON response:', result.text);
      throw new Error('Invalid JSON response from Claude');
    }
  }
  
  async generateStream(
    prompt: string, 
    config?: GenerateConfig, 
    onChunk?: (chunk: string) => void
  ): Promise<GenerateResult> {
    if (!this.client) {
      throw new Error('Claude API key not configured');
    }
    
    const systemPrompt = config?.systemInstruction || 
      'You are an expert embedded systems AI assistant for the NeuroState platform.';
    
    let fullText = '';
    let inputTokens = 0;
    let outputTokens = 0;
    
    const stream = this.client.messages.stream({
      model: CLAUDE_MODEL,
      max_tokens: config?.maxTokens || 8192,
      system: systemPrompt,
      messages: [{ role: 'user', content: prompt }]
    });
    
    for await (const event of stream) {
      if (event.type === 'content_block_delta') {
        const delta = event.delta;
        if ('text' in delta) {
          fullText += delta.text;
          onChunk?.(delta.text);
        }
      } else if (event.type === 'message_delta') {
        if (event.usage) {
          outputTokens = event.usage.output_tokens;
        }
      }
    }
    
    const finalMessage = await stream.finalMessage();
    inputTokens = finalMessage.usage.input_tokens;
    
    return {
      text: fullText,
      model: CLAUDE_MODEL,
      usage: { inputTokens, outputTokens }
    };
  }
  
  // Voice Provider Implementation
  async transcribe(audioBase64: string, mimeType: string = 'audio/webm'): Promise<string> {
    if (!this.client) {
      throw new Error('Claude API key not configured');
    }
    
    try {
      const response = await this.client.messages.create({
        model: CLAUDE_MODEL_FAST,
        max_tokens: 4096,
        messages: [{
          role: 'user',
          content: [
            {
              type: 'text',
              text: 'Transcribe the following audio exactly. Return only the transcription, no preamble or explanation.'
            }
          ]
        }]
      });
      
      const textContent = response.content.find(c => c.type === 'text');
      return textContent?.type === 'text' ? textContent.text.trim() : '';
    } catch (error) {
      console.error('Claude transcription error:', error);
      throw error;
    }
  }
}

// Create singleton instance and register
const claudeProvider = new ClaudeProvider();
registerProvider(claudeProvider);
registerVoiceProvider(claudeProvider);

export { claudeProvider, ClaudeProvider };
