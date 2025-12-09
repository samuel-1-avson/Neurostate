
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
  
  // State Callback
  onStateChange: null as ((state: AgentState) => void) | null,
  
  // Tool Callback
  onToolCall: null as ((name: string, args: any) => Promise<any>) | null,

  async connect(
    onStateChange: (state: AgentState) => void,
    onToolCall: (name: string, args: any) => Promise<any>
  ) {
    if (this.isConnected) return;
    
    this.onStateChange = onStateChange;
    this.onToolCall = onToolCall;

    const apiKey = process.env.API_KEY || '';
    if (!apiKey) {
        console.error("LiveService: API Key missing");
        return;
    }

    const ai = new GoogleGenAI({ apiKey });
    
    // Initialize Audio Contexts
    this.inputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 16000 });
    this.outputContext = new (window.AudioContext || (window as any).webkitAudioContext)({ sampleRate: 24000 });
    
    // CRITICAL: Ensure contexts are resumed (browser autoplay policy)
    if (this.inputContext.state === 'suspended') await this.inputContext.resume();
    if (this.outputContext.state === 'suspended') await this.outputContext.resume();

    this.outputNode = this.outputContext.createGain();
    this.outputNode.connect(this.outputContext.destination);
    
    // Sync timing
    this.nextStartTime = this.outputContext.currentTime + 0.1;

    try {
      this.onStateChange('LISTENING');
      
      // Request Mic Access
      this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
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
            // 1. Handle Tool Calls (Function Calling)
            if (message.toolCall) {
               this.onStateChange?.('MODIFYING'); // Visual feedback
               for (const fc of message.toolCall.functionCalls) {
                  console.log("Neo Tool Call:", fc.name, fc.args);
                  
                  try {
                    // Execute the tool logic in App.tsx
                    let result = "Done";
                    if (this.onToolCall) {
                       result = await this.onToolCall(fc.name, fc.args);
                    }

                    // Send response back to Gemini
                    sessionPromise.then((session) => {
                       session.sendToolResponse({
                          functionResponses: {
                             id: fc.id,
                             name: fc.name,
                             response: { result: result } // Simple confirmation
                          }
                       });
                    });
                  } catch (e) {
                     console.error("Tool Execution Failed", e);
                     // Send error back
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
               // Resume listening state after tool handling
               this.onStateChange?.('LISTENING');
            }

            // 2. Handle Audio Output
            // Use optional chaining carefully and handle potential empty/text-only parts
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
               // Simple debounce to return to listening state after speaking
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
            console.log("LiveService: Closed");
            this.disconnect();
          },
          onerror: (err) => {
            console.error("LiveService: Error", err);
            this.disconnect();
          }
        }
      });
      
      this.activeSession = sessionPromise;

    } catch (err) {
      console.error("LiveService Connection Failed:", err);
      this.disconnect();
    }
  },

  startInputStream(sessionPromise: Promise<any>) {
    if (!this.inputContext || !this.mediaStream) return;
    
    this.source = this.inputContext.createMediaStreamSource(this.mediaStream);
    this.processor = this.inputContext.createScriptProcessor(4096, 1, 1);
    
    this.processor.onaudioprocess = (e) => {
      if (!this.isConnected) return;
      const inputData = e.inputBuffer.getChannelData(0);
      const pcmBlob = createBlob(inputData);
      
      sessionPromise.then((session) => {
        session.sendRealtimeInput({ media: pcmBlob });
      });
    };
    
    this.source.connect(this.processor);
    this.processor.connect(this.inputContext.destination);
  },

  async playAudioChunk(base64Audio: string) {
    if (!this.outputContext || !this.outputNode) return;
    
    // Safety check if audio context got suspended
    if (this.outputContext.state === 'suspended') {
        await this.outputContext.resume();
    }

    // Ensure schedule is valid
    const now = this.outputContext.currentTime;
    if (this.nextStartTime < now) {
        this.nextStartTime = now + 0.05; // Buffer slightly if late
    }

    const audioBuffer = await decodeAudioData(
       decode(base64Audio), 
       this.outputContext,
       24000,
       1
    );

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
  },

  stopAudioPlayback() {
     this.sources.forEach(s => s.stop());
     this.sources.clear();
     this.nextStartTime = 0;
  },

  disconnect() {
    this.isConnected = false;
    this.onStateChange?.('IDLE');
    
    // Cleanup Input
    if (this.mediaStream) {
        this.mediaStream.getTracks().forEach(track => track.stop());
        this.mediaStream = null;
    }
    if (this.processor) { this.processor.disconnect(); this.processor = null; }
    if (this.source) { this.source.disconnect(); this.source = null; }
    if (this.inputContext) { this.inputContext.close(); this.inputContext = null; }

    // Cleanup Output
    this.stopAudioPlayback();
    if (this.outputContext) { this.outputContext.close(); this.outputContext = null; }
    
    this.activeSession = null;
  }
};
