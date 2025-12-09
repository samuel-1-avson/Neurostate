
import { useState, useEffect, useRef, useCallback } from 'react';

export function useWakeWord(enabled: boolean, onWake: () => void) {
  const [isListening, setIsListening] = useState(false);
  
  // Refs for state access in callbacks to avoid dependency cycles
  const enabledRef = useRef(enabled);
  const onWakeRef = useRef(onWake);
  const recognitionRef = useRef<any>(null);
  const retryTimeoutRef = useRef<any>(null);

  useEffect(() => { enabledRef.current = enabled; }, [enabled]);
  useEffect(() => { onWakeRef.current = onWake; }, [onWake]);

  const startListening = useCallback(() => {
    // Cleanup existing retry timer
    if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
        retryTimeoutRef.current = null;
    }

    if (!enabledRef.current) return;
    if (recognitionRef.current) return; // Already running

    // Browser support check
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    try {
      const recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang = 'en-US';

      recognition.onstart = () => {
        setIsListening(true);
      };

      recognition.onend = () => {
        setIsListening(false);
        recognitionRef.current = null;
        
        // Auto-restart if still enabled
        // We use a small delay to prevent tight loops if errors occur frequently
        if (enabledRef.current) {
            retryTimeoutRef.current = setTimeout(startListening, 500); 
        }
      };

      recognition.onerror = (event: any) => {
        // 'no-speech' is common and not fatal, just means silence
        if (event.error !== 'no-speech') {
            console.debug("WakeWord Error:", event.error);
        }
        // If permission is denied, stop the loop to prevent spamming
        if (event.error === 'not-allowed') {
            enabledRef.current = false; 
        }
      };

      recognition.onresult = (event: any) => {
        for (let i = event.resultIndex; i < event.results.length; ++i) {
            // Use interim results for faster reaction
            const transcript = event.results[i][0].transcript.trim().toLowerCase();
            // Check for Neo matches (phonetic variations included)
            if (transcript.match(/\b(neo|nio|neil|neon|leo|mio)\b/i)) {
                recognition.abort(); // Stop listening
                onWakeRef.current(); // Trigger callback
                return;
            }
        }
      };

      recognition.start();
      recognitionRef.current = recognition;

    } catch (e) {
      console.error("WakeWord Start Error:", e);
      // Retry logic
      retryTimeoutRef.current = setTimeout(startListening, 1000);
    }
  }, []);

  useEffect(() => {
    if (enabled) {
        startListening();
    } else {
        // Cleanup if disabled
        if (recognitionRef.current) {
            recognitionRef.current.onend = null;
            recognitionRef.current.abort();
            recognitionRef.current = null;
        }
        setIsListening(false);
        if (retryTimeoutRef.current) clearTimeout(retryTimeoutRef.current);
    }

    // Visibility handler to recover from background throttling
    // We don't abort on hide (allowing background use), but we force check on show
    const handleVisibility = () => {
        if (!document.hidden && enabledRef.current && !recognitionRef.current) {
            console.log("Tab visible, recovering wake word listener...");
            startListening();
        }
    };
    
    document.addEventListener('visibilitychange', handleVisibility);
    return () => {
        document.removeEventListener('visibilitychange', handleVisibility);
        if (retryTimeoutRef.current) clearTimeout(retryTimeoutRef.current);
        if (recognitionRef.current) {
            recognitionRef.current.onend = null;
            recognitionRef.current.abort();
            recognitionRef.current = null;
        }
    };
  }, [enabled, startListening]);

  return isListening;
}
