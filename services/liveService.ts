import { GoogleGenAI } from "@google/genai";

// Audio Context Global References
let audioContext: AudioContext | null = null;
let mediaStream: MediaStream | null = null;
let processor: ScriptProcessorNode | null = null;
let source: MediaStreamAudioSourceNode | null = null;

// PCM Helpers
function floatTo16BitPCM(input: Float32Array): Int16Array {
  const output = new Int16Array(input.length);
  for (let i = 0; i < input.length; i++) {
    const s = Math.max(-1, Math.min(1, input[i]));
    output[i] = s < 0 ? s * 0x8000 : s * 0x7FFF;
  }
  return output;
}

function base64EncodeAudio(float32Array: Float32Array): string {
  const int16Array = floatTo16BitPCM(float32Array);
  const buffer = new ArrayBuffer(int16Array.byteLength);
  const view = new DataView(buffer);
  for (let i = 0; i < int16Array.length; i++) {
    view.setInt16(i * 2, int16Array[i], true); // Little endian
  }
  let binary = '';
  const bytes = new Uint8Array(buffer);
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

export const liveService = {
  activeSession: null as any,
  isConnected: false,

  async connect(apiKey: string, onAudioData: (base64: string) => void) {
    if (!apiKey) throw new Error("API Key required for Live Service");

    const client = new GoogleGenAI({ apiKey });

    // Initialize Audio Input (Microphone)
    audioContext = new (window.AudioContext || (window as any).webkitAudioContext)({
      sampleRate: 16000, // Gemini expects 16kHz for input
    });

    try {
      mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
      source = audioContext.createMediaStreamSource(mediaStream);
      processor = audioContext.createScriptProcessor(512, 1, 1);

      processor.onaudioprocess = (e) => {
        const inputData = e.inputBuffer.getChannelData(0);
        const base64PCM = base64EncodeAudio(inputData);
        // Send to Gemini if session active
        if (this.isConnected && this.activeSession) {
             // In a real implementation using the WebSocket API directly:
             // this.activeSession.send({ realtime_input: { media_chunks: [{ data: base64PCM, mime_type: "audio/pcm" }] } });
             
             // Since the SDK handles this differently, we simulate the hook here for the UI
             onAudioData(base64PCM);
        }
      };

      source.connect(processor);
      processor.connect(audioContext.destination);
      
      this.isConnected = true;
      console.log("Live Service: Audio Stream Connected");

    } catch (err) {
      console.error("Live Service Error:", err);
      this.disconnect();
      throw err;
    }
  },

  disconnect() {
    this.isConnected = false;
    if (mediaStream) mediaStream.getTracks().forEach(track => track.stop());
    if (processor) processor.disconnect();
    if (source) source.disconnect();
    if (audioContext) audioContext.close();
    
    mediaStream = null;
    processor = null;
    source = null;
    audioContext = null;
    this.activeSession = null;
    console.log("Live Service: Disconnected");
  }
};