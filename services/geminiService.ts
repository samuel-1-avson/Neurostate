
import { GoogleGenAI, Type } from "@google/genai";
import { Node, Edge } from "reactflow";
import { GhostIssue, ChatEntry, ValidationReport, ResourceMetrics } from "../types";

// Ensure API Key is available
const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY || '';
const ai = apiKey ? new GoogleGenAI({ apiKey }) : null;

export const geminiService = {

  // ... [Other methods unchanged] ...
  async reverseEngineerCode(code: string): Promise<{ nodes: Node[], edges: Edge[] }> {
    if (!ai) throw new Error("API Key missing");
    const prompt = `You are a Reverse Engineering Expert. Analyze C/C++ code and output FSM JSON. Code: ${code.substring(0, 15000)}`;
    try {
      const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents: prompt, config: { responseMimeType: 'application/json' } });
      const text = response.text || "{}";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      return JSON.parse(cleanJson);
    } catch (error) { throw error; }
  },

  async generateCode(nodes: Node[], edges: Edge[], language: 'verilog' | 'cpp' | 'python' | 'rust'): Promise<string> {
    if (!ai) return "// Error: API Key missing.";
    const graphRepresentation = JSON.stringify({ nodes: nodes.map(n => ({ id: n.id, label: n.data.label, type: n.data.type, entry: n.data.entryAction })), edges: edges.map(e => ({ from: e.source, to: e.target, event: e.label })) });
    const prompt = `Generate ${language} code for this FSM: ${graphRepresentation}`;
    try {
      const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents: prompt });
      let code = response.text || "";
      return code.replace(/^```[a-z]*\n/, '').replace(/\n```$/, '').trim();
    } catch (error) { return `// Error: ${(error as Error).message}`; }
  },

  async generateUnitTests(nodes: Node[], edges: Edge[]): Promise<string> {
    return "// Tests";
  },

  // --- SMART NODE LOGIC GENERATION ---

  async generateNodeScript(nodeLabel: string, nodeType: string, userDescription: string, existingContextKeys: string[]): Promise<{ code: string, reasoning: string }> {
    if (!ai) return { code: "// AI Offline", reasoning: "Could not connect to AI service." };

    const prompt = `
      Act as an Embedded Javascript Generator for the NeuroState FSM Engine.
      The user wants logic for a single node.
      
      Node Label: "${nodeLabel}"
      Node Type: "${nodeType}"
      User Description: "${userDescription}"
      Existing Context Variables: ${(existingContextKeys || []).join(', ')}
      
      Environment:
      - "ctx" is the global context object (read/write).
      - "HAL" is the hardware abstraction layer. Available methods:
        - HAL.readPin(pin: number): boolean
        - HAL.writePin(pin: number, value: boolean)
        - HAL.getADC(channel: number): number
        - HAL.setPWM(channel: number, duty: number)
        - HAL.UART_Transmit(str: string)
        - HAL.UART_Receive(): string | null
      - "dispatch(eventName, delayMs)" to trigger transitions.
      
      Task:
      1. Write the JAVASCRIPT code for the 'entryAction' of this node.
      2. Provide a brief explanation of the logic and intent (reasoning).
      
      Rules:
      1. Keep code concise (1-5 lines).
      2. Prefer using existing context variables if they match the intent.
      3. Use 'ctx' to store state.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-2.5-flash',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          responseSchema: {
            type: Type.OBJECT,
            properties: {
              code: { type: Type.STRING, description: "The raw JavaScript code for the node." },
              reasoning: { type: Type.STRING, description: "Explanation of why this code was generated and how it works." }
            },
            required: ["code", "reasoning"]
          }
        }
      });

      const json = JSON.parse(response.text || "{}");
      return {
        code: json.code || "// No code generated",
        reasoning: json.reasoning || "No reasoning provided."
      };
    } catch (e) {
      console.error("AI Script Generation Failed", e);
      return { code: `// Error generating logic: ${(e as Error).message}`, reasoning: "Error" };
    }
  },

  async generateRegisterMap(nodes: Node[]): Promise<string> {
     return "// Register Map";
  },

  async optimizeForLowPower(nodes: Node[], edges: Edge[]): Promise<string> {
     return "Low power suggestions";
  },

  async analyzeDatasheet(datasheetText: string): Promise<string> {
     return "Datasheet Analysis";
  },

  // ... [Other methods unchanged] ...
  async generateVeoVideo(prompt: string, imageBase64: string, mimeType: string, aspectRatio: '16:9' | '9:16'): Promise<string> {
    const freshApiKey = process.env.GEMINI_API_KEY || process.env.API_KEY;
    if (!freshApiKey) throw new Error("API Key missing");
    const videoAi = new GoogleGenAI({ apiKey: freshApiKey });
    try {
      let operation = await videoAi.models.generateVideos({ model: 'veo-3.1-fast-generate-preview', prompt: prompt, image: { imageBytes: imageBase64, mimeType: mimeType }, config: { numberOfVideos: 1, resolution: '720p', aspectRatio: aspectRatio } });
      while (!operation.done) { await new Promise(resolve => setTimeout(resolve, 5000)); operation = await videoAi.operations.getVideosOperation({ operation: operation }); }
      const downloadLink = operation.response?.generatedVideos?.[0]?.video?.uri;
      if (!downloadLink) throw new Error("Video generation failed: No download link");
      const response = await fetch(`${downloadLink}&key=${freshApiKey}`);
      const blob = await response.blob();
      return URL.createObjectURL(blob);
    } catch (error) { throw error; }
  },

  async classifyIntent(text: string): Promise<'CREATE' | 'MODIFY' | 'CHAT'> {
    if (!ai) return 'CHAT';
    try { const response = await ai.models.generateContent({ model: 'gemini-2.5-flash', contents: `Classify intent: ${text} -> CREATE/MODIFY/CHAT` }); return (response.text || 'CHAT').trim() as any; } catch(e) { return 'CHAT'; }
  },

  async chatWithAssistant(history: ChatEntry[], nodes: Node[], edges: Edge[], issues: GhostIssue[], newMessage: string, attachment?: { base64: string, mimeType: string }): Promise<string> {
    if (!ai) return "Offline";
    try {
      const contents: any[] = [{ text: `History: ${JSON.stringify(history.slice(-3))}. Graph: ${nodes.length} nodes. User: ${newMessage}` }];
      if (attachment) contents.push({ inlineData: { mimeType: attachment.mimeType, data: attachment.base64 } });
      const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents });
      return response.text || "No response";
    } catch(e) { return "Error"; }
  },

  async createGraphFromPrompt(prompt: string, attachment?: { base64: string, mimeType: string }): Promise<{ nodes: Node[], edges: Edge[] } | null> {
    if (!ai) return null;
    try {
        const contents: any[] = [{ text: `Create FSM JSON for: ${prompt}` }];
        if (attachment) contents.push({ inlineData: { mimeType: attachment.mimeType, data: attachment.base64 } });
        const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents, config: { responseMimeType: 'application/json' } });
        return JSON.parse(response.text || "{}");
    } catch(e) { return null; }
  },

  async modifyGraph(nodes: Node[], edges: Edge[], userInstruction: string, issues: GhostIssue[] = []): Promise<{ nodes: Node[], edges: Edge[] } | null> {
    if (!ai) return null;
    try {
        const prompt = `Modify FSM. Nodes: ${JSON.stringify(nodes.map(n=>({id:n.id, label:n.data.label})))}. Instruction: ${userInstruction}. Return {nodes, edges}.`;
        const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents: prompt, config: { responseMimeType: 'application/json' } });
        return JSON.parse(response.text || "{}");
    } catch(e) { return null; }
  },

  async generateValidationReport(nodes: Node[], edges: Edge[]): Promise<ValidationReport> {
      return { timestamp: Date.now(), critique: [], suggestions: [], testCases: [] };
  },

  async estimateResources(nodes: Node[], edges: Edge[]): Promise<ResourceMetrics> {
      return { timestamp: Date.now(), lutUsage: 0, ffUsage: 0, memoryKB: 0, powermW: 0, maxFreqMHz: 0, summary: "N/A" };
  },

  async transcribe(base64Audio: string, mimeType: string = 'audio/webm'): Promise<string> {
    if (!ai) throw new Error("API Key missing");
    try {
      const response = await ai.models.generateContent({
        model: 'gemini-2.5-flash',
        contents: {
          parts: [
            { inlineData: { mimeType: mimeType, data: base64Audio } },
            { text: "Transcribe audio." }
          ]
        }
      });
      return response.text?.trim() || "";
    } catch (error) { throw new Error("Voice failed."); }
  }
};
