/**
 * Gemini Provider - Secondary LLM for Image/Video Generation
 * Handles image generation and Veo video generation
 */

import { GoogleGenAI, Type } from "@google/genai";
import { 
  LLMProvider, 
  GenerateConfig, 
  GenerateResult, 
  TaskType,
  ImageProvider,
  VideoProvider,
  ImageGenerationConfig,
  VideoGenerationConfig,
  VoiceProvider,
  registerProvider,
  registerImageProvider,
  registerVideoProvider,
  registerVoiceProvider
} from './llmProvider';

const GEMINI_MODEL = 'gemini-2.5-flash';
const GEMINI_PRO_MODEL = 'gemini-2.5-pro-preview-06-05';
const VEO_MODEL = 'veo-3.1-fast-generate-preview';

class GeminiProvider implements LLMProvider, ImageProvider, VideoProvider, VoiceProvider {
  readonly name = 'gemini';
  readonly supportedTasks: TaskType[] = [
    'IMAGE_GENERATION',
    'VIDEO_GENERATION',
    'TRANSCRIPTION',
    // Fallback capabilities
    'CODE_GENERATION',
    'GRAPH_CREATION',
    'VALIDATION',
    'CHAT'
  ];
  
  private client: GoogleGenAI | null = null;
  private lastHealthCheck: number = 0;
  private isHealthy: boolean = false;
  
  constructor() {
    this.initClient();
  }
  
  private initClient(): void {
    const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY;
    if (apiKey) {
      this.client = new GoogleGenAI({ apiKey });
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
      await this.client.models.generateContent({
        model: GEMINI_MODEL,
        contents: 'ping'
      });
      this.isHealthy = true;
      this.lastHealthCheck = now;
      return true;
    } catch (error) {
      console.error('Gemini health check failed:', error);
      this.isHealthy = false;
      this.lastHealthCheck = now;
      return false;
    }
  }
  
  async generate(prompt: string, config?: GenerateConfig): Promise<GenerateResult> {
    if (!this.client) {
      throw new Error('Gemini API key not configured');
    }
    
    try {
      const modelConfig: any = {};
      
      if (config?.systemInstruction) {
        modelConfig.systemInstruction = config.systemInstruction;
      }
      
      if (config?.thinkingBudget) {
        modelConfig.thinkingConfig = { thinkingBudget: config.thinkingBudget };
      }
      
      if (config?.responseFormat === 'json') {
        modelConfig.responseMimeType = 'application/json';
      }
      
      const response = await this.client.models.generateContent({
        model: config?.thinkingBudget ? GEMINI_PRO_MODEL : GEMINI_MODEL,
        config: modelConfig,
        contents: prompt
      });
      
      return {
        text: response.text || '',
        model: GEMINI_MODEL
      };
    } catch (error) {
      console.error('Gemini generation error:', error);
      throw error;
    }
  }
  
  async generateJSON<T>(prompt: string, schema: object, config?: GenerateConfig): Promise<T> {
    if (!this.client) {
      throw new Error('Gemini API key not configured');
    }
    
    try {
      const response = await this.client.models.generateContent({
        model: GEMINI_MODEL,
        config: {
          systemInstruction: config?.systemInstruction,
          responseMimeType: 'application/json',
          responseSchema: schema as any
        },
        contents: prompt
      });
      
      const text = response.text || '{}';
      const cleanJson = text.replace(/```json\n?/g, '').replace(/```\n?/g, '').trim();
      return JSON.parse(cleanJson) as T;
    } catch (error) {
      console.error('Gemini JSON generation error:', error);
      throw error;
    }
  }
  
  // Image Provider Implementation
  async generateImage(prompt: string, config?: ImageGenerationConfig): Promise<string> {
    if (!this.client) {
      throw new Error('Gemini API key not configured');
    }
    
    try {
      const response = await this.client.models.generateContent({
        model: 'gemini-2.0-flash-exp-image-generation',
        contents: prompt,
        config: {
          responseModalities: ['IMAGE', 'TEXT']
        }
      });
      
      // Extract image from response
      const parts = response.candidates?.[0]?.content?.parts || [];
      for (const part of parts) {
        if ('inlineData' in part && part.inlineData) {
          return part.inlineData.data || '';
        }
      }
      
      throw new Error('No image generated');
    } catch (error) {
      console.error('Gemini image generation error:', error);
      throw error;
    }
  }
  
  // Video Provider Implementation
  async generateVideo(prompt: string, config?: VideoGenerationConfig): Promise<string> {
    if (!this.client) {
      throw new Error('Gemini API key not configured');
    }
    
    try {
      let operation = await this.client.models.generateVideos({
        model: VEO_MODEL,
        prompt: prompt,
        config: {
          numberOfVideos: 1,
          resolution: config?.resolution || '720p',
          aspectRatio: config?.aspectRatio || '16:9'
        }
      });
      
      // Poll for completion
      while (!operation.done) {
        await new Promise(resolve => setTimeout(resolve, 5000));
        operation = await this.client.operations.getVideosOperation({ operation });
      }
      
      const downloadLink = operation.response?.generatedVideos?.[0]?.video?.uri;
      if (!downloadLink) {
        throw new Error('Video generation failed: No download link');
      }
      
      // Fetch video and return as blob URL
      const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY;
      const response = await fetch(`${downloadLink}&key=${apiKey}`);
      const blob = await response.blob();
      return URL.createObjectURL(blob);
    } catch (error) {
      console.error('Gemini video generation error:', error);
      throw error;
    }
  }
  
  // Voice Provider Implementation
  async transcribe(audioBase64: string, mimeType: string = 'audio/webm'): Promise<string> {
    if (!this.client) {
      throw new Error('Gemini API key not configured');
    }
    
    try {
      const response = await this.client.models.generateContent({
        model: GEMINI_MODEL,
        contents: {
          parts: [
            { inlineData: { mimeType, data: audioBase64 } },
            { text: "Transcribe the user's voice command exactly. No preamble." }
          ]
        }
      });
      
      return response.text?.trim() || '';
    } catch (error) {
      console.error('Gemini transcription error:', error);
      throw error;
    }
  }
}

// Create singleton instance and register
const geminiProvider = new GeminiProvider();
registerProvider(geminiProvider);
registerImageProvider(geminiProvider);
registerVideoProvider(geminiProvider);
registerVoiceProvider(geminiProvider);

export { geminiProvider, GeminiProvider };
