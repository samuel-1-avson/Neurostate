/**
 * AI Router - Smart routing of tasks to appropriate LLM providers
 * Routes code/text to Claude, image/video to Gemini, with Ollama fallback
 */

import { 
  TaskType, 
  GenerateConfig, 
  GenerateResult,
  getAllProviders,
  getProvider,
  getImageProvider,
  getVideoProvider
} from './llmProvider';

// Import providers to register them
import './claudeProvider';
import './geminiProvider';
import './ollamaProvider';

interface RouteConfig {
  primary: string;
  fallbacks: string[];
}

const ROUTING_TABLE: Record<TaskType, RouteConfig> = {
  'CODE_GENERATION':   { primary: 'claude', fallbacks: ['ollama', 'gemini'] },
  'GRAPH_CREATION':    { primary: 'claude', fallbacks: ['ollama', 'gemini'] },
  'VALIDATION':        { primary: 'claude', fallbacks: ['ollama', 'gemini'] },
  'CHAT':              { primary: 'claude', fallbacks: ['ollama', 'gemini'] },
  'VOICE':             { primary: 'claude', fallbacks: ['gemini'] },
  'TRANSCRIPTION':     { primary: 'claude', fallbacks: ['gemini'] },
  'IMAGE_GENERATION':  { primary: 'gemini', fallbacks: [] },
  'VIDEO_GENERATION':  { primary: 'gemini', fallbacks: [] }
};

interface RetryConfig {
  maxRetries: number;
  baseDelayMs: number;
  maxDelayMs: number;
}

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  baseDelayMs: 1000,
  maxDelayMs: 16000
};

/**
 * Sleep for specified milliseconds with jitter
 */
async function sleep(ms: number): Promise<void> {
  const jitter = Math.random() * 0.1 * ms; // 10% jitter
  return new Promise(resolve => setTimeout(resolve, ms + jitter));
}

/**
 * Calculate exponential backoff delay
 */
function getBackoffDelay(attempt: number, config: RetryConfig): number {
  const delay = config.baseDelayMs * Math.pow(2, attempt);
  return Math.min(delay, config.maxDelayMs);
}

/**
 * Execute function with retry logic
 */
async function withRetry<T>(
  fn: () => Promise<T>,
  config: RetryConfig = DEFAULT_RETRY_CONFIG
): Promise<T> {
  let lastError: Error | null = null;
  
  for (let attempt = 0; attempt <= config.maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      console.error(`Attempt ${attempt + 1} failed:`, lastError.message);
      
      if (attempt < config.maxRetries) {
        const delay = getBackoffDelay(attempt, config);
        console.log(`Retrying in ${delay}ms...`);
        await sleep(delay);
      }
    }
  }
  
  throw lastError || new Error('All retries failed');
}

export class AIRouter {
  private static instance: AIRouter;
  
  private constructor() {}
  
  static getInstance(): AIRouter {
    if (!AIRouter.instance) {
      AIRouter.instance = new AIRouter();
    }
    return AIRouter.instance;
  }
  
  /**
   * Get the best available provider for a task type
   */
  async getBestProvider(taskType: TaskType): Promise<string> {
    const route = ROUTING_TABLE[taskType];
    
    // Check primary
    const primary = getProvider(route.primary);
    if (primary && await primary.isAvailable()) {
      return route.primary;
    }
    
    // Check fallbacks
    for (const fallback of route.fallbacks) {
      const provider = getProvider(fallback);
      if (provider && await provider.isAvailable()) {
        return fallback;
      }
    }
    
    throw new Error(`No available provider for task type: ${taskType}`);
  }
  
  /**
   * Generate text with automatic provider selection and retry
   */
  async generate(
    taskType: TaskType,
    prompt: string,
    config?: GenerateConfig
  ): Promise<GenerateResult> {
    const route = ROUTING_TABLE[taskType];
    const providerOrder = [route.primary, ...route.fallbacks];
    
    let lastError: Error | null = null;
    
    for (const providerName of providerOrder) {
      const provider = getProvider(providerName);
      if (!provider) continue;
      
      try {
        const isAvailable = await provider.isAvailable();
        if (!isAvailable) {
          console.log(`Provider ${providerName} is not available, trying next...`);
          continue;
        }
        
        // Try with retry logic
        return await withRetry(() => provider.generate(prompt, config));
      } catch (error) {
        lastError = error as Error;
        console.error(`Provider ${providerName} failed:`, lastError.message);
      }
    }
    
    throw lastError || new Error(`All providers failed for task: ${taskType}`);
  }
  
  /**
   * Generate JSON with automatic provider selection and retry
   */
  async generateJSON<T>(
    taskType: TaskType,
    prompt: string,
    schema: object,
    config?: GenerateConfig
  ): Promise<T> {
    const route = ROUTING_TABLE[taskType];
    const providerOrder = [route.primary, ...route.fallbacks];
    
    let lastError: Error | null = null;
    
    for (const providerName of providerOrder) {
      const provider = getProvider(providerName);
      if (!provider) continue;
      
      try {
        const isAvailable = await provider.isAvailable();
        if (!isAvailable) continue;
        
        return await withRetry(() => provider.generateJSON<T>(prompt, schema, config));
      } catch (error) {
        lastError = error as Error;
        console.error(`Provider ${providerName} JSON failed:`, lastError.message);
      }
    }
    
    throw lastError || new Error(`All providers failed for JSON task: ${taskType}`);
  }
  
  /**
   * Generate image (Gemini only)
   */
  async generateImage(prompt: string): Promise<string> {
    const provider = getImageProvider('gemini');
    if (!provider) {
      throw new Error('Image generation provider not available');
    }
    
    if (!await provider.isAvailable()) {
      throw new Error('Gemini is not available for image generation');
    }
    
    return withRetry(() => provider.generateImage(prompt));
  }
  
  /**
   * Generate video (Gemini/Veo only)
   */
  async generateVideo(prompt: string, aspectRatio?: '16:9' | '9:16'): Promise<string> {
    const provider = getVideoProvider('gemini');
    if (!provider) {
      throw new Error('Video generation provider not available');
    }
    
    if (!await provider.isAvailable()) {
      throw new Error('Gemini is not available for video generation');
    }
    
    return withRetry(() => provider.generateVideo(prompt, { aspectRatio }));
  }
  
  /**
   * Get provider health status
   */
  async getHealthStatus(): Promise<Record<string, boolean>> {
    const providers = getAllProviders();
    const status: Record<string, boolean> = {};
    
    for (const provider of providers) {
      status[provider.name] = await provider.isAvailable();
    }
    
    return status;
  }
}

// Export singleton instance
export const aiRouter = AIRouter.getInstance();
