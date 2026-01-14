/**
 * LLM Provider Abstraction Layer
 * Unified interface for multiple AI providers (Claude, Gemini, Ollama)
 */

export type TaskType = 
  | 'CODE_GENERATION'
  | 'GRAPH_CREATION'
  | 'VALIDATION'
  | 'CHAT'
  | 'VOICE'
  | 'TRANSCRIPTION'
  | 'IMAGE_GENERATION'
  | 'VIDEO_GENERATION';

export interface GenerateConfig {
  systemInstruction?: string;
  maxTokens?: number;
  temperature?: number;
  thinkingBudget?: number;
  responseFormat?: 'text' | 'json';
  schema?: object;
}

export interface GenerateResult {
  text: string;
  model: string;
  usage?: {
    inputTokens: number;
    outputTokens: number;
  };
}

export interface LLMProvider {
  readonly name: string;
  readonly supportedTasks: TaskType[];
  
  /**
   * Check if the provider is available and healthy
   */
  isAvailable(): Promise<boolean>;
  
  /**
   * Generate text response
   */
  generate(prompt: string, config?: GenerateConfig): Promise<GenerateResult>;
  
  /**
   * Generate structured JSON response
   */
  generateJSON<T>(prompt: string, schema: object, config?: GenerateConfig): Promise<T>;
  
  /**
   * Stream text response
   */
  generateStream?(prompt: string, config?: GenerateConfig, onChunk?: (chunk: string) => void): Promise<GenerateResult>;
}

export interface ImageGenerationConfig {
  size?: '256x256' | '512x512' | '1024x1024';
  quality?: 'standard' | 'hd';
  style?: 'natural' | 'vivid';
}

export interface ImageProvider {
  readonly name: string;
  
  isAvailable(): Promise<boolean>;
  
  generateImage(prompt: string, config?: ImageGenerationConfig): Promise<string>; // Returns base64 or URL
}

export interface VideoGenerationConfig {
  aspectRatio?: '16:9' | '9:16';
  duration?: number;
  resolution?: '720p' | '1080p';
}

export interface VideoProvider {
  readonly name: string;
  
  isAvailable(): Promise<boolean>;
  
  generateVideo(prompt: string, config?: VideoGenerationConfig): Promise<string>; // Returns URL
}

export interface VoiceConfig {
  voice?: string;
  sampleRate?: number;
}

export interface VoiceProvider {
  readonly name: string;
  
  isAvailable(): Promise<boolean>;
  
  transcribe(audioBase64: string, mimeType?: string): Promise<string>;
  
  synthesize?(text: string, config?: VoiceConfig): Promise<string>; // Returns audio base64
}

// Provider registry for dependency injection
const providers: Map<string, LLMProvider> = new Map();
const imageProviders: Map<string, ImageProvider> = new Map();
const videoProviders: Map<string, VideoProvider> = new Map();
const voiceProviders: Map<string, VoiceProvider> = new Map();

export function registerProvider(provider: LLMProvider): void {
  providers.set(provider.name, provider);
}

export function registerImageProvider(provider: ImageProvider): void {
  imageProviders.set(provider.name, provider);
}

export function registerVideoProvider(provider: VideoProvider): void {
  videoProviders.set(provider.name, provider);
}

export function registerVoiceProvider(provider: VoiceProvider): void {
  voiceProviders.set(provider.name, provider);
}

export function getProvider(name: string): LLMProvider | undefined {
  return providers.get(name);
}

export function getAllProviders(): LLMProvider[] {
  return Array.from(providers.values());
}

export function getImageProvider(name: string): ImageProvider | undefined {
  return imageProviders.get(name);
}

export function getVideoProvider(name: string): VideoProvider | undefined {
  return videoProviders.get(name);
}

export function getVoiceProvider(name: string): VoiceProvider | undefined {
  return voiceProviders.get(name);
}
