/**
 * Voice Service - Real-time voice AI using Gemini Native Audio
 * Refactored to use provider abstraction layer
 */

import { GoogleGenAI, LiveServerMessage, Modality, Type, FunctionDeclaration } from "@google/genai";
import { AgentState } from "../types";

// --- AUDIO HELPERS ---

function decode(base64: string): Uint8Array {
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

function encode(bytes: Uint8Array): string {
  let binary = '';
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function createPCMBlob(data: Float32Array): { data: string, mimeType: string } {
  const l = data.length;
  const int16 = new Int16Array(l);
  for (let i = 0; i < l; i++) {
    const val = data[i];
    int16[i] = val < 0 ? val * 32768 : val * 32767;
  }
  return {
    data: encode(new Uint8Array(int16.buffer)),
    mimeType: 'audio/pcm;rate=16000',
  };
}

async function decodeAudioData(
  data: Uint8Array,
  ctx: AudioContext,
  sampleRate: number = 24000,
  numChannels: number = 1,
): Promise<AudioBuffer> {
  let safeData = data;
  if (data.byteLength % 2 !== 0) {
    const newBytes = new Uint8Array(data.byteLength + 1);
    newBytes.set(data);
    safeData = newBytes;
  }

  const dataInt16 = new Int16Array(safeData.buffer);
  const frameCount = dataInt16.length / numChannels;
  const buffer = ctx.createBuffer(numChannels, frameCount, sampleRate);

  for (let channel = 0; channel < numChannels; channel++) {
    const channelData = buffer.getChannelData(channel);
    for (let i = 0; i < frameCount; i++) {
      channelData[i] = dataInt16[i * numChannels + channel] / 32768.0;
    }
  }
  return buffer;
}

// --- VOICE CONFIGURATION ---

export interface VoiceServiceConfig {
  model?: string;
  voice?: string;
  systemInstruction?: string;
}

const DEFAULT_CONFIG: VoiceServiceConfig = {
  model: 'gemini-2.5-flash-native-audio-preview-09-2025',
  voice: 'Zephyr',
  systemInstruction: 'You are Neo, an advanced embedded systems architect. You have direct control over the canvas. Call "read_design" to see the graph. Use "create_design" and "modify_design" to edit. Be concise and technical.'
};

// --- TOOL DEFINITIONS ---

const tools: { functionDeclarations: FunctionDeclaration[] }[] = [
  {
    functionDeclarations: [
      {
        name: "read_design",
        description: "Read the current structure of the FSM design (nodes, edges, and logic). Call this to understand the user's work.",
        parameters: {
          type: Type.OBJECT,
          properties: {}, 
        }
      },
      {
        name: "create_design",
        description: "Create a new FSM design on the canvas based on a verbal description.",
        parameters: {
          type: Type.OBJECT,
          properties: {
            description: { 
              type: Type.STRING, 
              description: "The description of the system to design." 
            }
          },
          required: ["description"]
        }
      },
      {
        name: "modify_design",
        description: "Modify the existing FSM design (add nodes, connect edges, fix issues, change logic).",
        parameters: {
          type: Type.OBJECT,
          properties: {
             instruction: { 
               type: Type.STRING, 
               description: "The modification instruction." 
             }
          },
          required: ["instruction"]
        }
      },
      {
        name: "analyze_design",
        description: "Run static analysis on the current FSM to find issues like dead ends, race conditions, or missing error handlers.",
        parameters: {
          type: Type.OBJECT,
          properties: {}, 
        }
      },
      {
        name: "generate_code",
        description: "Generate code from the current FSM design in a specified language.",
        parameters: {
          type: Type.OBJECT,
          properties: {
            language: { 
              type: Type.STRING, 
              description: "The target language: 'cpp', 'python', 'rust', or 'verilog'" 
            }
          },
          required: ["language"]
        }
      },
      {
        name: "disconnect_session",
        description: "Disconnect and close the voice session immediately.",
        parameters: {
          type: Type.OBJECT,
          properties: {}, 
        }
      }
    ]
  }
];

// --- VOICE SERVICE CLASS ---

class VoiceService {
  private activeSession: any = null;
  private isConnected: boolean = false;
  private currentSessionId: string | null = null;
  
  private inputContext: AudioContext | null = null;
  private outputContext: AudioContext | null = null;
  
  private mediaStream: MediaStream | null = null;
  private processor: ScriptProcessorNode | null = null;
  private source: MediaStreamAudioSourceNode | null = null;
  
  private outputNode: GainNode | null = null;
  private nextStartTime: number = 0;
  private sources: Set<AudioBufferSourceNode> = new Set();
  
  private onStateChange: ((state: AgentState) => void) | null = null;
  private onToolCall: ((name: string, args: any) => Promise<any>) | null = null;
  private onClose: (() => void) | null = null;
  private onLog: ((msg: string) => void) | null = null;
  
  private config: VoiceServiceConfig = DEFAULT_CONFIG;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 3;

  /**
   * Check if Gemini voice API is available
   */
  async isAvailable(): Promise<boolean> {
    const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY;
    if (!apiKey) return false;
    
    try {
      const ai = new GoogleGenAI({ apiKey });
      // Just check if we can create the client
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get microphone stream with retries
   */
  private async getStreamWithRetry(constraints: MediaStreamConstraints, retries = 3): Promise<MediaStream> {
    for (let i = 0; i < retries; i++) {
      try {
        return await navigator.mediaDevices.getUserMedia(constraints);
      } catch (e: any) {
        if (i === retries - 1) throw e;
        if (e.name === 'NotAllowedError' || e.name === 'PermissionDeniedError') throw e;
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
    throw new Error("Failed to acquire microphone");
  }

  /**
   * Connect to voice session
   */
  async connect(
    onStateChange: (state: AgentState) => void,
    onToolCall: (name: string, args: any) => Promise<any>,
    onClose?: () => void,
    onLog?: (msg: string) => void,
    config?: VoiceServiceConfig
  ): Promise<void> {
    if (this.isConnected) return;
    
    const sessionId = Math.random().toString(36).substring(7);
    this.currentSessionId = sessionId;
    this.config = { ...DEFAULT_CONFIG, ...config };

    this.onStateChange = onStateChange;
    this.onToolCall = onToolCall;
    this.onClose = onClose || null;
    this.onLog = onLog || null;

    const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY || '';
    if (!apiKey) {
      console.error("VoiceService: API Key missing");
      this.onLog?.("Error: Gemini API key not configured");
      this.onClose?.();
      return;
    }

    try {
      this.onStateChange('LISTENING');
      this.onLog?.("Acquiring microphone...");
      
      // Get microphone access
      try {
        this.mediaStream = await this.getStreamWithRetry({ 
          audio: {
            channelCount: 1,
            echoCancellation: true,
            autoGainControl: true,
            noiseSuppression: true,
          } 
        });
      } catch (e) {
        try {
          this.mediaStream = await this.getStreamWithRetry({ audio: true });
        } catch (e2: any) {
          throw new Error(`Microphone unavailable: ${e2.name || e2.message}`);
        }
      }

      if (this.currentSessionId !== sessionId) return;
      
      this.onLog?.("Microphone acquired, connecting to Neo...");

      // Setup audio contexts
      this.inputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 16000 });
      this.outputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 24000 });
      
      if (this.inputContext.state === 'suspended') await this.inputContext.resume();
      if (this.outputContext.state === 'suspended') await this.outputContext.resume();

      this.outputNode = this.outputContext.createGain();
      this.outputNode.connect(this.outputContext.destination);
      this.nextStartTime = this.outputContext.currentTime + 0.1;

      // Connect to Gemini Live
      const ai = new GoogleGenAI({ apiKey });
      
      const sessionPromise = ai.live.connect({
        model: this.config.model!,
        config: {
          responseModalities: [Modality.AUDIO],
          tools: tools,
          speechConfig: {
            voiceConfig: { prebuiltVoiceConfig: { voiceName: this.config.voice! } },
          },
          systemInstruction: this.config.systemInstruction,
        },
        callbacks: {
          onopen: () => {
            if (this.currentSessionId !== sessionId) return;
            console.log("VoiceService: Connected");
            this.isConnected = true;
            this.reconnectAttempts = 0;
            this.startInputStream(sessionPromise);
            this.onLog?.("Neo Connected");
          },
          onmessage: async (message: LiveServerMessage) => {
            if (this.currentSessionId !== sessionId) return;
            await this.handleMessage(message, sessionPromise);
          },
          onclose: () => {
            if (this.currentSessionId !== sessionId) return;
            console.log("VoiceService: Closed by Server");
            this.handleDisconnect();
          },
          onerror: (err) => {
            if (this.currentSessionId !== sessionId) return;
            this.handleError(err, sessionId);
          }
        }
      });
      
      this.activeSession = sessionPromise;

    } catch (err: any) {
      if (this.currentSessionId !== sessionId) return;
      console.error("VoiceService Connection Failed:", err);
      this.onLog?.(`Connection failed: ${err.message}`);
      this.internalCleanup();
      this.onClose?.();
    }
  }

  /**
   * Handle incoming messages
   */
  private async handleMessage(message: LiveServerMessage, sessionPromise: Promise<any>): Promise<void> {
    // Handle tool calls
    if (message.toolCall) {
      this.onStateChange?.('MODIFYING');
      const functionCalls = message.toolCall.functionCalls;
      const responses = [];

      for (const fc of functionCalls) {
        if (fc.name === 'disconnect_session') {
          this.disconnect();
          return;
        }

        this.onLog?.(`Executing: ${fc.name}`);
        
        try {
          let result = "Done";
          if (this.onToolCall) {
            result = await this.onToolCall(fc.name, fc.args);
          }
          responses.push({
            id: fc.id,
            name: fc.name,
            response: { result: result } 
          });
        } catch (e) {
          console.error(`Tool execution failed for ${fc.name}:`, e);
          responses.push({
            id: fc.id,
            name: fc.name,
            response: { error: (e as Error).message }
          });
        }
      }

      if (responses.length > 0) {
        sessionPromise.then((session) => {
          if (this.isConnected) {
            try {
              session.sendToolResponse({
                functionResponses: responses
              });
            } catch(sendErr) {
              console.error("Failed to send tool response", sendErr);
            }
          }
        });
      }
      
      this.onStateChange?.('LISTENING');
    }

    // Handle audio response
    const parts = message.serverContent?.modelTurn?.parts || [];
    let hasAudio = false;
    for (const part of parts) {
      const base64Audio = part.inlineData?.data;
      if (base64Audio) {
        hasAudio = true;
        this.onStateChange?.('SPEAKING');
        await this.playAudioChunk(base64Audio);
      }
    }
    if (hasAudio) {
      setTimeout(() => {
        if (this.isConnected && this.sources.size === 0 && this.activeSession) {
          this.onStateChange?.('LISTENING');
        }
      }, 2000); 
    }

    // Handle interruption
    if (message.serverContent?.interrupted) {
      this.stopAudioPlayback();
      this.onStateChange?.('LISTENING');
    }
  }

  /**
   * Handle disconnection with optional reconnect
   */
  private handleDisconnect(): void {
    if (this.onClose) {
      this.onClose();
      this.onClose = null;
    }
    this.internalCleanup();
  }

  /**
   * Handle errors with retry logic
   */
  private handleError(err: any, sessionId: string): void {
    console.error("VoiceService: Error", err);
    
    // Ignore generic Network Errors unless persistent
    if (err instanceof Error && err.message.includes('Network')) {
      console.warn("Network glitch detected. Attempting to stay alive...");
      this.reconnectAttempts++;
      
      if (this.reconnectAttempts >= this.maxReconnectAttempts) {
        this.onLog?.("Connection lost after multiple retries");
        this.handleDisconnect();
      }
    }
  }

  /**
   * Start audio input stream
   */
  private startInputStream(sessionPromise: Promise<any>): void {
    if (!this.inputContext || !this.mediaStream) return;
    
    this.source = this.inputContext.createMediaStreamSource(this.mediaStream);
    this.processor = this.inputContext.createScriptProcessor(2048, 1, 1);
    
    this.processor.onaudioprocess = (e) => {
      if (!this.isConnected) return;
      
      try {
        const inputData = e.inputBuffer.getChannelData(0);
        const pcmBlob = createPCMBlob(inputData);
        
        sessionPromise.then((session) => {
          if (this.isConnected) {
            session.sendRealtimeInput({ media: pcmBlob });
          }
        }).catch(() => {
          // Suppress streaming errors during disconnects
        });
      } catch {
        // Suppress processing errors
      }
    };
    
    this.source.connect(this.processor);
    this.processor.connect(this.inputContext.destination);
  }

  /**
   * Play audio chunk from response
   */
  private async playAudioChunk(base64Audio: string): Promise<void> {
    if (!this.outputContext || !this.outputNode) return;
    if (this.outputContext.state === 'suspended') await this.outputContext.resume();

    const now = this.outputContext.currentTime;
    if (this.nextStartTime < now) this.nextStartTime = now + 0.05;

    try {
      const audioBuffer = await decodeAudioData(decode(base64Audio), this.outputContext, 24000, 1);
      const source = this.outputContext.createBufferSource();
      source.buffer = audioBuffer;
      source.connect(this.outputNode);
      source.addEventListener('ended', () => {
        this.sources.delete(source);
        if (this.sources.size === 0) this.onStateChange?.('LISTENING');
      });
      source.start(this.nextStartTime);
      this.nextStartTime += audioBuffer.duration;
      this.sources.add(source);
    } catch (e) { 
      console.error("Audio Decode Error", e); 
    }
  }

  /**
   * Stop audio playback
   */
  private stopAudioPlayback(): void {
    this.sources.forEach(s => { try { s.stop(); } catch {} });
    this.sources.clear();
    if (this.outputContext) this.nextStartTime = this.outputContext.currentTime;
    else this.nextStartTime = 0;
  }

  /**
   * Disconnect voice session
   */
  disconnect(): void {
    this.onClose = null;
    this.currentSessionId = null;
    this.internalCleanup();
  }

  /**
   * Internal cleanup
   */
  private internalCleanup(): void {
    this.isConnected = false;
    this.currentSessionId = null;
    this.onStateChange?.('IDLE');
    
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach(track => track.stop());
      this.mediaStream = null;
    }
    if (this.processor) { 
      this.processor.disconnect(); 
      this.processor.onaudioprocess = null;
      this.processor = null; 
    }
    if (this.source) { this.source.disconnect(); this.source = null; }
    if (this.inputContext) { this.inputContext.close(); this.inputContext = null; }

    this.stopAudioPlayback();
    if (this.outputContext) { this.outputContext.close(); this.outputContext = null; }
    
    this.activeSession = null;
  }

  /**
   * Check if currently connected
   */
  getIsConnected(): boolean {
    return this.isConnected;
  }
}

// Export singleton and class
const voiceService = new VoiceService();

// Backwards compatible export
export const liveService = {
  isConnected: false,
  
  async connect(
    onStateChange: (state: AgentState) => void,
    onToolCall: (name: string, args: any) => Promise<any>,
    onClose?: () => void,
    onLog?: (msg: string) => void
  ) {
    await voiceService.connect(onStateChange, onToolCall, onClose, onLog);
    this.isConnected = voiceService.getIsConnected();
  },
  
  disconnect() {
    voiceService.disconnect();
    this.isConnected = false;
  }
};

export { voiceService, VoiceService };
