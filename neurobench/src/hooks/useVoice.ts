// Voice Interface Hook
// Provides speech-to-text and text-to-speech for hands-free agent interaction

import { createSignal, onMount, onCleanup } from "solid-js";

// Check if speech recognition is available
const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
const speechSynthesis = window.speechSynthesis;

export interface VoiceConfig {
  language?: string;
  continuous?: boolean;
  interimResults?: boolean;
  voiceRate?: number;
  voicePitch?: number;
  voiceName?: string;
}

export interface VoiceState {
  isListening: boolean;
  isSupported: boolean;
  isSpeaking: boolean;
  transcript: string;
  interimTranscript: string;
  error: string | null;
}

const defaultConfig: VoiceConfig = {
  language: "en-US",
  continuous: false,
  interimResults: true,
  voiceRate: 1.0,
  voicePitch: 1.0,
};

export function useVoice(config: VoiceConfig = {}) {
  const mergedConfig = { ...defaultConfig, ...config };
  
  const [isListening, setIsListening] = createSignal(false);
  const [isSpeaking, setIsSpeaking] = createSignal(false);
  const [transcript, setTranscript] = createSignal("");
  const [interimTranscript, setInterimTranscript] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);
  const [voices, setVoices] = createSignal<SpeechSynthesisVoice[]>([]);
  
  let recognition: any = null;
  
  const isSupported = !!SpeechRecognition && !!speechSynthesis;
  
  // Initialize speech recognition
  onMount(() => {
    if (SpeechRecognition) {
      recognition = new SpeechRecognition();
      recognition.lang = mergedConfig.language;
      recognition.continuous = mergedConfig.continuous;
      recognition.interimResults = mergedConfig.interimResults;
      
      recognition.onresult = (event: any) => {
        let interim = "";
        let final = "";
        
        for (let i = event.resultIndex; i < event.results.length; i++) {
          const result = event.results[i];
          if (result.isFinal) {
            final += result[0].transcript;
          } else {
            interim += result[0].transcript;
          }
        }
        
        if (final) {
          setTranscript(prev => prev + final);
        }
        setInterimTranscript(interim);
      };
      
      recognition.onerror = (event: any) => {
        setError(event.error);
        setIsListening(false);
      };
      
      recognition.onend = () => {
        setIsListening(false);
      };
    }
    
    // Load available voices
    if (speechSynthesis) {
      const loadVoices = () => {
        const availableVoices = speechSynthesis.getVoices();
        setVoices(availableVoices);
      };
      
      loadVoices();
      speechSynthesis.onvoiceschanged = loadVoices;
    }
  });
  
  onCleanup(() => {
    if (recognition) {
      recognition.abort();
    }
    if (speechSynthesis) {
      speechSynthesis.cancel();
    }
  });
  
  // Start listening for speech
  function startListening() {
    if (!recognition) {
      setError("Speech recognition not supported");
      return;
    }
    
    setError(null);
    setTranscript("");
    setInterimTranscript("");
    
    try {
      recognition.start();
      setIsListening(true);
    } catch (e) {
      setError("Failed to start speech recognition");
    }
  }
  
  // Stop listening
  function stopListening() {
    if (recognition) {
      recognition.stop();
      setIsListening(false);
    }
  }
  
  // Toggle listening
  function toggleListening() {
    if (isListening()) {
      stopListening();
    } else {
      startListening();
    }
  }
  
  // Speak text
  function speak(text: string, options?: { voice?: string; rate?: number; pitch?: number }) {
    if (!speechSynthesis) {
      setError("Speech synthesis not supported");
      return;
    }
    
    // Cancel any ongoing speech
    speechSynthesis.cancel();
    
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = options?.rate ?? mergedConfig.voiceRate ?? 1.0;
    utterance.pitch = options?.pitch ?? mergedConfig.voicePitch ?? 1.0;
    
    // Find and set voice
    const voiceName = options?.voice ?? mergedConfig.voiceName;
    if (voiceName) {
      const voice = voices().find(v => v.name.includes(voiceName));
      if (voice) {
        utterance.voice = voice;
      }
    }
    
    utterance.onstart = () => setIsSpeaking(true);
    utterance.onend = () => setIsSpeaking(false);
    utterance.onerror = () => {
      setIsSpeaking(false);
      setError("Speech synthesis failed");
    };
    
    speechSynthesis.speak(utterance);
  }
  
  // Stop speaking
  function stopSpeaking() {
    if (speechSynthesis) {
      speechSynthesis.cancel();
      setIsSpeaking(false);
    }
  }
  
  // Get state
  function getState(): VoiceState {
    return {
      isListening: isListening(),
      isSupported,
      isSpeaking: isSpeaking(),
      transcript: transcript(),
      interimTranscript: interimTranscript(),
      error: error(),
    };
  }
  
  return {
    // State
    isListening,
    isSpeaking,
    isSupported,
    transcript,
    interimTranscript,
    error,
    voices,
    
    // Actions
    startListening,
    stopListening,
    toggleListening,
    speak,
    stopSpeaking,
    getState,
    
    // Clear
    clearTranscript: () => {
      setTranscript("");
      setInterimTranscript("");
    },
    clearError: () => setError(null),
  };
}

// Voice command detection
export interface VoiceCommand {
  trigger: string | RegExp;
  action: (match: RegExpMatchArray | null, transcript: string) => void;
}

export function createVoiceCommands(commands: VoiceCommand[]) {
  return {
    process(transcript: string): boolean {
      const lowerTranscript = transcript.toLowerCase().trim();
      
      for (const command of commands) {
        if (typeof command.trigger === "string") {
          if (lowerTranscript.includes(command.trigger.toLowerCase())) {
            command.action(null, transcript);
            return true;
          }
        } else {
          const match = lowerTranscript.match(command.trigger);
          if (match) {
            command.action(match, transcript);
            return true;
          }
        }
      }
      
      return false;
    }
  };
}

// Common voice commands for NeuroBench
export const defaultCommands: VoiceCommand[] = [
  {
    trigger: "add node",
    action: (_, transcript) => {
      console.log("Voice: Add node command", transcript);
    }
  },
  {
    trigger: "connect",
    action: (_, transcript) => {
      console.log("Voice: Connect command", transcript);
    }
  },
  {
    trigger: "auto layout",
    action: () => {
      console.log("Voice: Auto layout command");
    }
  },
  {
    trigger: "generate code",
    action: () => {
      console.log("Voice: Generate code command");
    }
  },
  {
    trigger: "build project",
    action: () => {
      console.log("Voice: Build project command");
    }
  },
  {
    trigger: "flash device",
    action: () => {
      console.log("Voice: Flash device command");
    }
  },
  {
    trigger: /switch to (\w+) agent/,
    action: (match) => {
      if (match) {
        console.log("Voice: Switch to agent", match[1]);
      }
    }
  },
];

export default useVoice;
