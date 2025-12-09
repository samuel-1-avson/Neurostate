import { geminiService } from './geminiService';

export const voiceService = {
  listen: (): Promise<string> => {
    return new Promise(async (resolve, reject) => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        
        // Determine supported mime type
        const mimeType = MediaRecorder.isTypeSupported("audio/webm") ? "audio/webm" : "audio/mp4";
        const mediaRecorder = new MediaRecorder(stream, { mimeType });
        const audioChunks: Blob[] = [];

        mediaRecorder.ondataavailable = (event) => {
          if (event.data.size > 0) {
            audioChunks.push(event.data);
          }
        };

        mediaRecorder.onstop = async () => {
          // Cleanup tracks
          stream.getTracks().forEach(track => track.stop());

          const audioBlob = new Blob(audioChunks, { type: mimeType });
          const reader = new FileReader();
          
          reader.readAsDataURL(audioBlob);
          reader.onloadend = async () => {
            if (reader.result) {
              const base64String = (reader.result as string).split(',')[1];
              try {
                const text = await geminiService.transcribe(base64String, mimeType);
                resolve(text);
              } catch (error) {
                reject(error);
              }
            } else {
              reject(new Error("Failed to process audio data."));
            }
          };
        };

        mediaRecorder.start();

        // Record for 3.5 seconds - optimized for short commands like "Start", "Connect A to B"
        setTimeout(() => {
          if (mediaRecorder.state !== 'inactive') {
            mediaRecorder.stop();
          }
        }, 3500);

      } catch (err) {
        console.error("Microphone Access Error:", err);
        reject(new Error("Microphone access denied or unavailable."));
      }
    });
  }
};