
import { GoogleGenAI, Type } from "@google/genai";
import { Node, Edge } from "reactflow";
import { GhostIssue, ChatEntry, ValidationReport, ResourceMetrics } from "../types";

// Ensure API Key is available
const apiKey = process.env.GEMINI_API_KEY || process.env.API_KEY || '';
const ai = apiKey ? new GoogleGenAI({ apiKey }) : null;

export const geminiService = {

  // --- REVERSE ENGINEERING ---

  async reverseEngineerCode(code: string): Promise<{ nodes: Node[], edges: Edge[] }> {
    if (!ai) throw new Error("API Key missing");

    const prompt = `
      You are a Reverse Engineering Expert for Embedded Systems.
      Your task is to analyze the following C/C++ source code and reconstruct the Finite State Machine (FSM) into a JSON graph.

      SOURCE CODE:
      ${code.substring(0, 15000)} // Limit context window

      INSTRUCTIONS:
      1. Analyze the code to identify:
         - States (enums, #defines, or constants).
         - Transitions (switch-case statements, if-else blocks checking state).
         - Logic (actions performed on entry, exit, or during the state).
      2. Reconstruct this into a JSON object with 'nodes' and 'edges'.
      3. For Nodes:
         - 'id': unique string.
         - 'data.label': State name.
         - 'data.type': Infer type ('input' for initial, 'process' for normal, 'error' for fault handlers).
         - 'data.entryAction': Extract relevant C++ code that runs when entering or being in this state.
      4. For Edges:
         - 'source': ID of the source state.
         - 'target': ID of the target state.
         - 'label': The condition or event triggering the transition (e.g. "BTN_PRESSED", "x > 10").
      5. Layout: Assign intelligent X/Y positions so the graph flows logically (left-to-right or top-to-down). Do not stack all nodes at 0,0.

      OUTPUT FORMAT:
      Return ONLY raw JSON. No markdown formatting.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: {
          responseMimeType: 'application/json',
          thinkingConfig: { thinkingBudget: 32768 } // Deep thinking to parse logic
        },
        contents: prompt
      });

      const text = response.text || "{}";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      const result = JSON.parse(cleanJson);

      if (!Array.isArray(result.nodes) || !Array.isArray(result.edges)) {
        throw new Error("Invalid graph structure returned");
      }
      return result;
    } catch (error) {
      console.error("Reverse Engineering Error:", error);
      throw error;
    }
  },

  async generateCode(nodes: Node[], edges: Edge[], language: 'verilog' | 'cpp' | 'python' | 'rust'): Promise<string> {
    if (!ai) return "// Error: API Key missing.";

    // Include actions in the payload so the LLM can translate/comment them
    const graphRepresentation = JSON.stringify({
      nodes: (nodes || []).map(n => ({
        id: n.id,
        label: n.data.label,
        type: n.data.type,
        entryAction: n.data.entryAction,
        exitAction: n.data.exitAction
      })),
      edges: (edges || []).map(e => ({ source: e.source, target: e.target, event: e.label }))
    }, null, 2);

    let systemInstruction = "";
    let prompt = "";
    // Upgrade to Gemini 3 Pro for complex code generation
    const modelName = 'gemini-3-pro-preview';

    switch (language) {
      case 'verilog':
        systemInstruction = "You are an expert FPGA Engineer specializing in Verilog HDL. You prioritize robust state encoding and synchronous resets.";
        prompt = `
                Generate a robust, syntactically correct Verilog module for the following Finite State Machine (FSM).
                
                Requirements:
                1. Module Name: FSM_Controller
                2. Inputs: clk, rst_n (active low reset), events (input wire [3:0])
                3. Outputs: current_state (output reg [3:0]), status_leds (output reg [3:0])
                4. Architecture: Use the standard 3-process methodology (State Register, Next State Logic, Output Logic).
                5. Encoding: Use 'localparam' for One-Hot or Binary state encoding.
                6. Logic: 
                   - The 'events' input triggers transitions based on the edge labels. Map event strings to specific bit masks (e.g., event 'START' = 4'b0001) and document this mapping in the header.
                   - Reset should force the FSM to the node marked 'type: input'.
                7. Actions: The JSON contains 'entryAction' and 'exitAction' which are high-level logic. Insert these as COMMENTS inside the Verilog code where appropriate to guide the hardware implementer.
                
                Output Format: Return ONLY raw Verilog code. Do not wrap in markdown blocks.

                JSON Graph:
                ${graphRepresentation}
            `;
        break;
      case 'cpp':
        systemInstruction = "You are an expert Embedded Systems Engineer specializing in C++ for Microcontrollers (Arduino/ESP32). You prioritize volatile correctness and ISR safety.";
        prompt = `
                Generate a production-ready C++ class for the following Finite State Machine.

                Requirements:
                1. Class Name: NeuroFSM
                2. Structure: Use 'enum class State' for state definitions.
                3. Methods: 
                   - 'void init()' to setup initial state.
                   - 'void dispatch(String eventName)' to handle transitions.
                   - 'void update()' to be called in the main loop (handle entry/exit logic if needed).
                4. Logic:
                   - Implement string-based event matching.
                   - Translate the 'entryAction' and 'exitAction' fields (which are JavaScript) into C++ logic where possible, or add them as TODO comments if they rely on external libraries.
                   - Use 'Serial.println' for logging state changes.
                
                Output Format: Return ONLY raw C++ code (Header and Implementation combined). Do not wrap in markdown blocks.

                JSON Graph:
                ${graphRepresentation}
            `;
        break;
      case 'python':
        systemInstruction = "You are a Senior Python Developer specializing in Embedded Linux and IoT. You write clean, typed, and documented code.";
        prompt = `
                Generate a clean, Pythonic FSM class for the following graph.

                Requirements:
                1. Class Name: StateMachine
                2. Pattern: Use the State Pattern or a clean Dictionary-based dispatch table.
                3. Features:
                   - Support event-driven transitions.
                   - Implement 'on_enter' and 'on_exit' callbacks for states.
                   - Translate the provided 'entryAction'/'exitAction' JavaScript snippets into valid Python code.
                   - Include a '__main__' block demonstrating usage.
                
                Output Format: Return ONLY raw Python code. Do not wrap in markdown blocks.

                JSON Graph:
                ${graphRepresentation}
            `;
        break;
      case 'rust':
        systemInstruction = "You are a Rust Systems Programmer specializing in safe, zero-cost abstractions. You use enums and match patterns effectively.";
        prompt = `
                Generate a safe, idiomatic Rust implementation of this FSM.

                Requirements:
                1. Use 'enum' for States.
                2. Use 'match' expressions for state transitions.
                3. Struct 'StateMachine' holding current state and context.
                4. Logic:
                   - Method 'process_event(&mut self, event: &str)'.
                   - Convert the JavaScript 'entryAction'/'exitAction' into comments or Rust code where obvious (e.g., print statements).
                5. Ensure no 'unsafe' code blocks.
                
                Output Format: Return ONLY raw Rust code. Do not wrap in markdown blocks.

                JSON Graph:
                ${graphRepresentation}
            `;
        break;
    }

    try {
      const response = await ai.models.generateContent({
        model: modelName,
        config: {
          systemInstruction,
          thinkingConfig: { thinkingBudget: 32768 } // Use Max Reasoning for Code Gen
        },
        contents: prompt,
      });

      let code = response.text || "";
      code = code.replace(/^```[a-z]*\n/, '').replace(/\n```$/, '');
      return code.trim();

    } catch (error) {
      console.error("Gemini Generation Error:", error);
      return `// Error generating code: ${(error as Error).message}`;
    }
  },

  // --- UNIT TEST GENERATION ---

  async generateUnitTests(nodes: Node[], edges: Edge[]): Promise<string> {
    if (!ai) return "// Error: API Key missing.";

    const graph = JSON.stringify({
      nodes: nodes.map(n => ({ id: n.id, label: n.data.label })),
      edges: edges.map(e => ({ source: e.source, target: e.target, trigger: e.label, condition: e.data?.condition }))
    }, null, 2);

    const prompt = `
      Act as a QA Automation Engineer for Embedded Systems.
      Generate a GoogleTest (GTest) C++ test suite for this Finite State Machine.

      FSM Structure:
      ${graph}

      Requirements:
      1. Create a Test Fixture class 'FSMTest'.
      2. For EVERY edge in the graph, generate a test case:
         - Name: 'Transition_<Source>_To_<Target>'
         - Logic: Initialize FSM, Force state to <Source>, Trigger <Event>, Assert State == <Target>.
      3. If the edge has a condition (Guard), generate two tests:
         - One where condition is TRUE (Expect Transition).
         - One where condition is FALSE (Expect No Transition).
      4. Mock the Context/HAL if needed to satisfy guards.
      
      Output:
      Raw C++ code (GTest format). No markdown.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: { thinkingConfig: { thinkingBudget: 16000 } },
        contents: prompt
      });
      let text = response.text || "";
      return text.replace(/^```c\+\+\n/, '').replace(/^```cpp\n/, '').replace(/\n```$/, '').trim();
    } catch (e) {
      return "// Failed to generate tests.";
    }
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
      return { code: `// Error generating logic: ${(e as Error).message}`, reasoning: "Error" };
    }
  },

  // --- NEW EMBEDDED ENGINEERING METHODS ---

  async generateRegisterMap(nodes: Node[]): Promise<string> {
    if (!ai) return "// Error: API Key missing.";

    // Extract variables used in actions
    const allCode = (nodes || []).map(n => (n.data.entryAction || '') + (n.data.exitAction || '')).join('\n');

    const prompt = `
      Act as a Senior Firmware Architect.
      Analyze the Javascript-like logic below used in a Finite State Machine context.
      Identify all variables accessed via 'ctx.' (e.g., ctx.control_reg, ctx.status_flag, ctx.isEnabled).
      
      Task:
      Generate a professional C-style header file ('registers.h') that defines a memory-mapped structure for these variables.
      
      Requirements:
      1. Define a struct 'SystemRegisters_t'.
      2. Use <stdint.h> types (uint32_t, uint8_t, etc.).
      3. Mark all fields as 'volatile' to prevent compiler optimization (crucial for embedded).
      4. Intelligence:
         - If a variable is a boolean (true/false), pack it into a bitfield inside a 'FLAGS' or 'STATUS' register.
         - If a variable seems to be a counter or data value, give it a full u32 or u16.
         - Group configuration variables into a 'CR' (Control Register).
         - Group status variables into a 'SR' (Status Register).
      5. Add Doxygen-style comments (/** ... */) explaining the inferred purpose of each register.
      6. Wrap in #ifndef FIRMWARE_REGISTERS_H guard.
      
      Code Context:
      ${allCode}

      Output:
      Raw C header code only. No markdown.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: { thinkingConfig: { thinkingBudget: 16000 } }, // Use reasoning to structure the struct
        contents: prompt
      });
      let text = response.text || "";
      return text.replace(/^```c\n/, '').replace(/\n```$/, '').trim();
    } catch (e) {
      return "// Failed to generate register map.";
    }
  },

  async optimizeForLowPower(nodes: Node[], edges: Edge[]): Promise<string> {
    if (!ai) return "AI Offline";

    const graph = JSON.stringify({ nodes: (nodes || []).map(n => n.data.label), edges: (edges || []).map(e => ({ from: e.source, to: e.target, trig: e.label })) });
    const prompt = `
      Analyze this FSM for Low Power optimization opportunities in an embedded context (Cortex-M or similar).
      
      FSM Structure: ${graph}
      
      Identify:
      1. States where the CPU could enter WFI (Wait For Interrupt) or Sleep.
      2. Peripheral clock gating opportunities (e.g. if ADC is only used in one state).
      3. Potential race conditions in sleep entry.
      
      Output a concise bulleted list of recommendations.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: { thinkingConfig: { thinkingBudget: 16000 } },
        contents: prompt
      });
      return response.text || "No recommendations.";
    } catch (e) { return "Analysis failed."; }
  },

  async analyzeDatasheet(datasheetText: string): Promise<string> {
    if (!ai) return "AI Offline";

    const prompt = `
        You are a Datasheet Analysis Agent.
        Read the following text excerpt from a microcontroller or sensor datasheet.
        Extract any TIMING CONSTRAINTS (e.g., "Wait 5ms after reset") or REGISTER CONFIGURATION rules.
        Format them as a checklist for the FSM designer.
        
        Datasheet Excerpt:
        ${datasheetText.substring(0, 5000)}
      `;

    try {
      const response = await ai.models.generateContent({ model: 'gemini-3-pro-preview', contents: prompt });
      return response.text || "No analysis generated.";
    } catch (e) { return "Analysis failed."; }
  },

  // --- VEO VIDEO GENERATION ---

  async generateVeoVideo(prompt: string, imageBase64: string, mimeType: string, aspectRatio: '16:9' | '9:16'): Promise<string> {
    const freshApiKey = process.env.GEMINI_API_KEY || process.env.API_KEY;
    if (!freshApiKey) throw new Error("API Key missing");

    // Create new instance to ensure we use the selected key for Veo
    const videoAi = new GoogleGenAI({ apiKey: freshApiKey });

    try {
      let operation = await videoAi.models.generateVideos({
        model: 'veo-3.1-fast-generate-preview',
        prompt: prompt,
        image: {
          imageBytes: imageBase64,
          mimeType: mimeType,
        },
        config: {
          numberOfVideos: 1,
          resolution: '720p',
          aspectRatio: aspectRatio
        }
      });

      // Poll for completion
      while (!operation.done) {
        await new Promise(resolve => setTimeout(resolve, 5000)); // Wait 5 seconds
        operation = await videoAi.operations.getVideosOperation({ operation: operation });
      }

      const downloadLink = operation.response?.generatedVideos?.[0]?.video?.uri;
      if (!downloadLink) throw new Error("Video generation failed: No download link");

      // Fetch the actual video bytes
      const response = await fetch(`${downloadLink}&key=${freshApiKey}`);
      const blob = await response.blob();
      return URL.createObjectURL(blob);

    } catch (error) {
      console.error("Veo Generation Error:", error);
      throw error;
    }
  },

  // --- INTERACTIVE GRAPH MODIFICATION ---

  async classifyIntent(text: string): Promise<'CREATE' | 'MODIFY' | 'CHAT'> {
    if (!ai) return 'CHAT';

    const prompt = `
      Classify the user's intent based on this command: "${text}"
      
      Categories:
      1. CREATE: User wants to build a NEW system from scratch (e.g., "Design a traffic light", "Create a USB stack", "Make an FSM for...", "Build this flowchart").
      2. MODIFY: User wants to CHANGE the current graph or FIX errors (e.g., "Add a node", "Connect A to B", "Fix the dead end", "Fix errors", "Delete the start node", "Add an LED node", "Add an interrupt").
      3. CHAT: General question or request for explanation (e.g., "How does I2C work?", "Explain this state", "Optimize for power", "What is the thinking budget?").
      
      Output ONLY the category name.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-2.5-flash',
        contents: prompt
      });
      const intent = (response.text || '').trim().toUpperCase();
      if (['CREATE', 'MODIFY', 'CHAT'].includes(intent)) return intent as any;
      return 'CHAT';
    } catch (e) {
      return 'CHAT';
    }
  },

  async chatWithAssistant(
    history: ChatEntry[],
    nodes: Node[],
    edges: Edge[],
    issues: GhostIssue[],
    newMessage: string,
    attachment?: { base64: string, mimeType: string }
  ): Promise<string> {
    if (!ai) return "I am offline. Please configure the API Key.";

    const graphContext = `
      Current FSM Structure:
      Nodes: ${(nodes || []).map(n => `[${n.id}] ${n.data.label} (${n.data.type})`).join(', ')}
      Edges: ${(edges || []).map(e => `${e.source} -> ${e.target} on "${e.label}"`).join(', ')}

      Detected Design Issues (Ghost Engineer):
      ${issues.length > 0 ? JSON.stringify(issues, null, 2) : "None detected."}
    `;

    try {
      const contents: any[] = [
        { text: `Context: ${graphContext}\nPrevious Conversation:\n${(history || []).slice(-6).map(m => `${m.role.toUpperCase()}: ${m.content}`).join('\n')}\nUser Query: ${newMessage}` }
      ];

      // Add attachment if present
      if (attachment) {
        contents.push({ inlineData: { mimeType: attachment.mimeType, data: attachment.base64 } });
      }

      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: {
          systemInstruction: 'You are the "NeuroState" Embedded AI Assistant. Help the user design FSMs for firmware. Be technical (C/C++, Registers, ISRs). You can analyze the graph structure. Think deeply about potential race conditions or hardware constraints before answering. If an image is provided, analyze it in the context of embedded systems.',
          thinkingConfig: { thinkingBudget: 16000 }
        },
        contents: contents
      });
      return response.text || "No response.";
    } catch (error) {
      return "Connection interrupted.";
    }
  },

  async createGraphFromPrompt(prompt: string, attachment?: { base64: string, mimeType: string }): Promise<{ nodes: Node[], edges: Edge[] } | null> {
    if (!ai) throw new Error("API Key missing");

    const systemInstruction = `
      You are the NeuroState FSM Generator for Embedded Systems.
      You will think through the requirements, identify the necessary states, hardware interactions, and transitions, and then output the JSON.
      CREATE a new Finite State Machine based on the user's firmware requirement.
      If an image is provided (e.g. a flowchart, diagram, or whiteboard sketch), analyze it thoroughly and translate it into the FSM graph.
      
      Rules:
      1. Return JSON with 'nodes' and 'edges'.
      2. Nodes must have unique IDs, positions, and data object.
      3. data object must have: label, type ('input'|'process'|'output'|'error'|'interrupt'|'timer'), entryAction, exitAction.
      4. Use embedded-friendly variable names in actions (e.g., ctx.ISR_Flag, ctx.GPIO_Pin, HAL.readPin()).
      5. Layout nodes sensibly (e.g. Input at top/left, flow downwards/right).
      
      Response Format: JSON only. No markdown.
    `;

    const contents: any[] = [{ text: `Create an FSM for: "${prompt}"` }];
    if (attachment) {
      contents.push({ inlineData: { mimeType: attachment.mimeType, data: attachment.base64 } });
    }

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        config: {
          systemInstruction,
          responseMimeType: 'application/json',
          thinkingConfig: { thinkingBudget: 32768 } // Max budget for creation
        },
        contents: contents
      });

      const text = response.text || "";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      return JSON.parse(cleanJson);
    } catch (error) {
      console.error("Generation Error:", error);
      throw error;
    }
  },

  async modifyGraph(
    nodes: Node[],
    edges: Edge[],
    userInstruction: string,
    issues: GhostIssue[] = []
  ): Promise<{ nodes: Node[], edges: Edge[] } | null> {
    if (!ai) throw new Error("API Key missing");

    const graphState = JSON.stringify({
      nodes: (nodes || []).map(n => ({ id: n.id, label: n.data.label, type: n.data.type, position: n.position, data: n.data })),
      edges: (edges || []).map(e => ({ id: e.id, source: e.source, target: e.target, label: e.label, data: e.data }))
    }, null, 2);

    const prompt = `
      You are the NeuroState Canvas Operator.
      Your task is to MODIFY the existing FSM Graph based on the USER COMMAND.
      Think step-by-step about how the user's request impacts the existing logic, state flow, and hardware interactions.

      CURRENT GRAPH JSON:
      ${graphState}

      DETECTED ISSUES (Context):
      ${JSON.stringify(issues, null, 2)}

      USER COMMAND:
      "${userInstruction}"

      INSTRUCTIONS:
      1. Parse the command to understand the intent (Add Node, Remove Node, Connect, Edit Actions, Fix Issue, Rename).
      2. If the user asks to "Fix errors" or "Fix mistakes", prioritize resolving the items in DETECTED ISSUES (e.g. connect dead ends to a valid state, remove unreachable nodes).
      3. Modify the "nodes" and "edges" arrays accordingly.
      4. CRITICAL: PRESERVE existing data (IDs, positions, logic) unless explicitly asked to change or move them.
      5. If adding new nodes:
         - Invent a reasonable "position" {x, y} so they don't exactly overlap existing ones (e.g., offset by +200x or +100y from the last node).
         - Ensure unique IDs (use timestamp or random string).
      6. If the command implies embedded logic (e.g. "turn on LED" or "add interrupt"), update the node type and logic:
         - "Add interrupt": type='interrupt', entryAction='HAL.enableInterrupt(pin); ctx.isr=true;'
         - "Add timer": type='timer', entryAction='dispatch("TIMEOUT", 1000);'
         - "Hardware/LED": type='hardware', entryAction='HAL.writePin(1, true);'
      7. Return the COMPLETE updated JSON object containing "nodes" and "edges".
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          thinkingConfig: { thinkingBudget: 32768 } // Max budget for modification
        }
      });
      const text = response.text || "";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      const result = JSON.parse(cleanJson);

      if (Array.isArray(result.nodes) && Array.isArray(result.edges)) {
        return result;
      }
      throw new Error("Invalid JSON structure returned");
    } catch (error) {
      console.error("AI Modify Error:", error);
      throw error;
    }
  },

  async generateValidationReport(nodes: Node[], edges: Edge[]): Promise<ValidationReport> {
    if (!ai) throw new Error("API Key missing");

    const graphState = JSON.stringify({
      nodes: (nodes || []).map(n => ({ label: n.data.label, type: n.data.type, entry: n.data.entryAction })),
      edges: (edges || []).map(e => ({ source: e.source, target: e.target, event: e.label }))
    }, null, 2);

    const prompt = `
      Analyze this Embedded FSM.
      GRAPH DATA: ${graphState}
      TASK:
      1. Critique: Look for deadlocks, race conditions, missing error handlers (Watchdog).
      2. Suggestions: Improve robustness.
      3. Test Cases: Generate 3 test scenarios.
      OUTPUT JSON: { "critique": [], "suggestions": [], "testCases": [] }
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          thinkingConfig: { thinkingBudget: 16000 }
        }
      });
      const text = response.text || "";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      const report = JSON.parse(cleanJson);
      return {
        timestamp: Date.now(),
        critique: report.critique || [],
        suggestions: report.suggestions || [],
        testCases: report.testCases || []
      };
    } catch (error) {
      throw error;
    }
  },

  async estimateResources(nodes: Node[], edges: Edge[]): Promise<ResourceMetrics> {
    if (!ai) throw new Error("API Key missing");

    const graphState = JSON.stringify({
      nodesCount: (nodes || []).length,
      edgesCount: (edges || []).length,
      logicSample: (nodes || []).map(n => n.data.entryAction || '').join('\n').substring(0, 500)
    }, null, 2);

    const prompt = `
      Act as an Embedded Hardware Architect.
      Analyze this FSM complexity.
      Estimate metrics for an ARM Cortex-M4 or Artix-7 FPGA.
      CONTEXT: ${graphState}
      OUTPUT JSON Keys: lutUsage, ffUsage, memoryKB, powermW, maxFreqMHz, summary.
    `;

    try {
      const response = await ai.models.generateContent({
        model: 'gemini-3-pro-preview',
        contents: prompt,
        config: {
          responseMimeType: 'application/json',
          thinkingConfig: { thinkingBudget: 16000 }
        }
      });
      const text = response.text || "";
      const cleanJson = text.replace(/```json/g, '').replace(/```/g, '').trim();
      const metrics = JSON.parse(cleanJson);
      return {
        timestamp: Date.now(),
        lutUsage: metrics.lutUsage || 0,
        ffUsage: metrics.ffUsage || 0,
        memoryKB: metrics.memoryKB || 0,
        powermW: metrics.powermW || 0,
        maxFreqMHz: metrics.maxFreqMHz || 0,
        summary: metrics.summary || "Estimation complete."
      };
    } catch (error) {
      throw error;
    }
  },

  async transcribe(base64Audio: string, mimeType: string = 'audio/webm'): Promise<string> {
    if (!ai) throw new Error("API Key missing");
    try {
      const response = await ai.models.generateContent({
        model: 'gemini-2.5-flash',
        contents: {
          parts: [
            { inlineData: { mimeType: mimeType, data: base64Audio } },
            { text: "Transcribe the user's voice command exactly. No preamble." }
          ]
        }
      });
      return response.text?.trim() || "";
    } catch (error) {
      throw new Error("Voice processing failed.");
    }
  }
};
