
import { GoogleGenAI, LiveServerMessage, Modality, Type, FunctionDeclaration } from "@google/genai";
import { AgentState } from "../types";

// Helper: Decode Base64 to ArrayBuffer
function decode(base64: string) {
  const binaryString = atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

// Helper: Encode Uint8Array to Base64
function encode(bytes: Uint8Array) {
  let binary = '';
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

// Helper: Create PCM Blob for Gemini Input
function createBlob(data: Float32Array): { data: string, mimeType: string } {
  const l = data.length;
  const int16 = new Int16Array(l);
  for (let i = 0; i < l; i++) {
    // Convert Float32 (-1.0 to 1.0) to Int16
    // Clamp to prevent wrapping: 1.0 * 32768 = 32768 (overflows to -32768) -> use 32767
    const val = data[i];
    int16[i] = val < 0 ? val * 32768 : val * 32767;
  }
  return {
    data: encode(new Uint8Array(int16.buffer)),
    mimeType: 'audio/pcm;rate=16000',
  };
}

// Helper: Decode PCM Audio Data from Gemini
async function decodeAudioData(
  data: Uint8Array,
  ctx: AudioContext,
  sampleRate: number = 24000,
  numChannels: number = 1,
): Promise<AudioBuffer> {
  // Ensure even byte length for Int16Array
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

// --- TOOL DEFINITIONS ---
const tools: { functionDeclarations: FunctionDeclaration[] }[] = [
  {
    functionDeclarations: [
      {
        name: "create_design",
        description: "Create a new Finite State Machine (FSM) firmware design on the canvas based on a verbal description.",
        parameters: {
          type: Type.OBJECT,
          properties: {
            description: { 
              type: Type.STRING, 
              description: "The description of the system to design (e.g., 'traffic light', 'usb stack', 'blinking led')." 
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
               description: "The modification instruction (e.g., 'add an error state', 'connect node A to B')." 
             }
          },
          required: ["instruction"]
        }
      }
    ]
  }
];

export const liveService = {
  activeSession: null as any,
  isConnected: false,
  
  // Audio Contexts
  inputContext: null as AudioContext | null,
  outputContext: null as AudioContext | null,
  
  // Input Stream
  mediaStream: null as MediaStream | null,
  processor: null as ScriptProcessorNode | null,
  source: null as MediaStreamAudioSourceNode | null,
  
  // Output Queue
  outputNode: null as GainNode | null,
  nextStartTime: 0,
  sources: new Set<AudioBufferSourceNode>(),
  
  // Callbacks
  onStateChange: null as ((state: AgentState) => void) | null,
  onToolCall: null as ((name: string, args: any) => Promise<any>) | null,
  onClose: null as (() => void) | null,

  // Helper: Get Media Stream with Retry Logic
  async getStreamWithRetry(constraints: MediaStreamConstraints, retries = 3): Promise<MediaStream> {
    for (let i = 0; i < retries; i++) {
      try {
        return await navigator.mediaDevices.getUserMedia(constraints);
      } catch (e: any) {
        if (i === retries - 1) throw e; // Last attempt failed, throw
        
        // If permission denied, don't retry, fail immediately
        if (e.name === 'NotAllowedError' || e.name === 'PermissionDeniedError') throw e;
        
        // For NotFound or NotReadable (busy), wait and retry
        console.warn(`LiveService: Mic busy/not found (Attempt ${i+1}/${retries}), retrying in 500ms...`, e.name);
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }
    throw new Error("Failed to acquire microphone after retries");
  },

  async connect(
    onStateChange: (state: AgentState) => void,
    onToolCall: (name: string, args: any) => Promise<any>,
    onClose?: () => void
  ) {
    if (this.isConnected) return;
    
    this.onStateChange = onStateChange;
    this.onToolCall = onToolCall;
    this.onClose = onClose || null;

    const apiKey = process.env.API_KEY || '';
    if (!apiKey) {
        console.error("LiveService: API Key missing");
        this.onClose?.();
        return;
    }

    try {
      this.onStateChange('LISTENING');
      
      // Request Mic Access with Retry Strategy
      try {
        // Try advanced constraints first
        this.mediaStream = await this.getStreamWithRetry({ 
          audio: {
            channelCount: 1,
            echoCancellation: true,
            autoGainControl: true,
            noiseSuppression: true,
          } 
        });
      } catch (e) {
        console.warn("LiveService: Advanced audio constraints failed. Falling back to basic.", e);
        try {
            // Fallback to basic if advanced failed
            this.mediaStream = await this.getStreamWithRetry({ audio: true });
        } catch (e2: any) {
            console.error("LiveService: Basic audio failed:", e2);
            throw new Error(`Microphone unavailable: ${e2.name || e2.message}`);
        }
      }

      // Initialize Audio Contexts after stream is ready to avoid context limits
      this.inputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 16000 });
      this.outputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 24000 });
      
      if (this.inputContext.state === 'suspended') await this.inputContext.resume();
      if (this.outputContext.state === 'suspended') await this.outputContext.resume();

      this.outputNode = this.outputContext.createGain();
      this.outputNode.connect(this.outputContext.destination);
      this.nextStartTime = this.outputContext.currentTime + 0.1;

      const ai = new GoogleGenAI({ apiKey });
      
      // Connect to Gemini Live
      const sessionPromise = ai.live.connect({
        model: 'gemini-2.5-flash-native-audio-preview-09-2025',
        config: {
          responseModalities: [Modality.AUDIO],
          tools: tools,
          speechConfig: {
            voiceConfig: { prebuiltVoiceConfig: { voiceName: 'Zephyr' } },
          },
          systemInstruction: 'You are Neo, an advanced embedded systems architect. You are running inside the NeuroState IDE. You have direct control over the canvas. When asked to design or modify systems, use the available tools. Be concise, technical, and helpful.',
        },
        callbacks: {
          onopen: () => {
            console.log("LiveService: Connected");
            this.isConnected = true;
            this.startInputStream(sessionPromise);
          },
          onmessage: async (message: LiveServerMessage) => {
            // 1. Handle Tool Calls
            if (message.toolCall) {
               this.onStateChange?.('MODIFYING');
               for (const fc of message.toolCall.functionCalls) {
                  try {
                    let result = "Done";
                    if (this.onToolCall) {
                       result = await this.onToolCall(fc.name, fc.args);
                    }
                    sessionPromise.then((session) => {
                       session.sendToolResponse({
                          functionResponses: {
                             id: fc.id,
                             name: fc.name,
                             response: { result: result }
                          }
                       });
                    });
                  } catch (e) {
                     sessionPromise.then((session) => {
                        session.sendToolResponse({
                           functionResponses: {
                              id: fc.id,
                              name: fc.name,
                              response: { error: (e as Error).message }
                           }
                        });
                     });
                  }
               }
               this.onStateChange?.('LISTENING');
            }

            // 2. Handle Audio Output
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
                   if(this.isConnected && this.sources.size === 0 && this.activeSession) this.onStateChange?.('LISTENING');
               }, 2000); 
            }

            // 3. Handle Interruption
            if (message.serverContent?.interrupted) {
              this.stopAudioPlayback();
              this.onStateChange?.('LISTENING');
            }
          },
          onclose: () => {
            console.log("LiveService: Closed by Server");
            this.disconnect();
          },
          onerror: (err) => {
            console.error("LiveService: Error", err);
            this.disconnect();
          }
        }
      });
      
      this.activeSession = sessionPromise;

    } catch (err: any) {
      console.error("LiveService Connection Failed:", err);
      this.disconnect();
      // Ensure specific errors bubble up/are logged
      if (this.onClose) { 
          // We can't pass args to onClose easily, but disconnect handles the state reset
          console.log("LiveService: Notifying UI of failure via close callback");
      }
    }
  },

  startInputStream(sessionPromise: Promise<any>) {
    if (!this.inputContext || !this.mediaStream) return;
    
    this.source = this.inputContext.createMediaStreamSource(this.mediaStream);
    this.processor = this.inputContext.createScriptProcessor(2048, 1, 1);
    
    this.processor.onaudioprocess = (e) => {
      if (!this.isConnected) return;
      
      try {
          const inputData = e.inputBuffer.getChannelData(0);
          const pcmBlob = createBlob(inputData);
          
          sessionPromise.then((session) => {
            if(this.isConnected) {
                session.sendRealtimeInput({ media: pcmBlob });
            }
          }).catch(err => {
              console.error("Stream send failed (Session likely closed):", err);
              // If we can't send, we should probably disconnect to avoid freezing UI
              this.disconnect();
          });
      } catch (err) {
          console.error("Audio processing error:", err);
      }
    };
    
    this.source.connect(this.processor);
    this.processor.connect(this.inputContext.destination);
  },

  async playAudioChunk(base64Audio: string) {
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
         if(this.sources.size === 0) this.onStateChange?.('LISTENING');
      });
      source.start(this.nextStartTime);
      this.nextStartTime += audioBuffer.duration;
      this.sources.add(source);
    } catch (e) { console.error("Audio Decode Error", e); }
  },

  stopAudioPlayback() {
     this.sources.forEach(s => { try { s.stop(); } catch(e) {} });
     this.sources.clear();
     if (this.outputContext) this.nextStartTime = this.outputContext.currentTime;
     else this.nextStartTime = 0;
  },

  disconnect() {
    this.isConnected = false;
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
    if (this.onClose) { this.onClose(); this.onClose = null; }
  }
};
