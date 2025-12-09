
import React, { useState, useCallback, useRef, useEffect, useMemo } from 'react';
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  BackgroundVariant,
  Panel as FlowPanel,
  MarkerType,
  Handle,
  Position,
  MiniMap,
  useReactFlow,
  ReactFlowProvider,
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  EdgeProps,
  NodeToolbar,
  ConnectionMode
} from 'reactflow';
import { Play, Square, Wand2, AlertTriangle, Save, Upload, Undo, Redo, Mic, Cpu, MessageSquare, GitBranch, Zap, FileJson, FileCode, Bot, Menu, ChevronDown, CheckCircle, Terminal, Layers, Plus, X, Variable, Activity, MousePointerClick, Copy, Info, Sparkles, Send, PanelRightClose, PanelRightOpen, PanelBottomClose, PanelBottomOpen, LayoutTemplate, Bug, Microscope, FlaskConical, BarChart3, Gauge, Trash2, Edit3, Target, ZoomIn, ZoomOut, Maximize, Move, Box, GripVertical, Sidebar, CircuitBoard, Layout, Monitor, Grid, Search, FilePlus, Settings2, Clock, FastForward, Pause, ArrowRightLeft, Ear, Hash, ToggleLeft, Disc, Battery, Shield, Split, Database, Cable, HardDrive, LayoutDashboard, FolderOpen, BookOpen, Download, Command, ChevronRight, LogOut, TableProperties, Wrench, Hourglass, Loader2, Group, Code2, TestTube, Waves, Volume2, MicOff, Book, AlignJustify, Paperclip, Image as ImageIcon, Film, Camera, Lock, ShieldAlert, Calculator, Wifi, Globe, Thermometer, FileText } from 'lucide-react';
import { clsx } from 'clsx';
import { toPng } from 'html-to-image';

import { Button, Panel, Input, Label, Toast, ToastMessage, VirtualLED, VirtualSwitch, VirtualDisplay, ProgressBar, MetricCard, LogicAnalyzer } from './components/RetroUI';
import { useShortcuts } from './hooks/useShortcuts';
import { FSMExecutor, VisualEventType } from './services/fsmEngine';
import { GhostEngineer } from './services/ghostEngineer';
import { geminiService } from './services/geminiService';
import { fileManager } from './services/fileManager';
import { hardwareBridge } from './services/hardwareBridge';
import { useHistory } from './hooks/useHistory';
import { usePersistence } from './hooks/usePersistence';
import { liveService } from './services/liveService';
import { useWakeWord } from './hooks/useWakeWord';
import { GhostIssue, LogEntry, SimulationStatus, FSMProject, ChatEntry, ValidationReport, ResourceMetrics, WorkspaceTemplate, FSMNodeData, SimTelemetry, McuDefinition, AgentState } from './types';
import { TEMPLATES, FSMTemplate } from './services/templates';
import { MCU_REGISTRY } from './services/deviceRegistry';
import { HAL, HalSnapshot } from './services/hal';

// --- DOCUMENTATION CONTENT ---
const DOCS_CONTENT = [
  {
    id: 'overview',
    title: '1. System Architecture',
    content: `
      # NeuroState System Bible
      
      NeuroState is a holistic Integrated Development Environment (IDE) designed to bridge the gap between human intent (Analog) and rigorous firmware logic (Digital).
      
      ### Core Architecture
      The application is built on four pillars:
      
      1.  **FSM Runtime Engine**: An asynchronous event-driven executor that runs the state machine logic in the browser. It manages the global context (\`ctx\`), handles event dispatching, and synchronizes with the visual graph.
      
      2.  **Hardware Abstraction Layer (HAL)**: A singleton service that simulates physical microcontroller peripherals. It allows the FSM to interact with "Virtual Silicon" (GPIO, UART, ADC) without real hardware.
      
      3.  **Visual Graph Interface**: Powered by React Flow, this layer handles the rendering of Nodes, Edges, and Animations. It translates user interactions into logical graph mutations.
      
      4.  **Gemini Intelligence Core**: Deep integration with Google's Gemini 3 Pro and Live API. It provides code generation, real-time voice assistance (Neo), and static analysis (Ghost Engineer).
    `
  },
  {
    id: 'components',
    title: '2. Node & Graph Components',
    content: `
      # Graph Primitives
      
      The visual language consists of specialized components that define behavior.
      
      ### Node Types
      - **Input** (Blue): The dedicated entry point. Initializes system clock and context variables.
      - **Process** (White): The workhorse of the FSM. Executes logic via \`entryAction\` and \`exitAction\`.
      - **Output** (Green): Final states or major milestones (e.g., "DEPLOYED").
      - **Decision** (Amber): Visual branching points. *Note: Actual logic resides in the Edge Guards.*
      - **Hardware** (Cyan): Nodes explicitly designed for HAL interaction (e.g., toggling pins).
      - **UART** (Purple): Serial communication blocks (TX/RX handling).
      - **Listener** (Indigo): Blocking states that wait for external events or interrupts.
      - **Error** (Red): Fault handling states. Used by the Ghost Engineer to route failures.
      
      ### Edges & Guards
      - **Transitions**: Connections between nodes representing state flow.
      - **Guards**: JavaScript expressions (e.g., \`ctx.voltage > 3.3\`) attached to edges. The FSM Engine evaluates these before traversing.
      - **Packet Animation**: Visual feedback showing data traversing the edge during execution.
    `
  },
  {
    id: 'processing',
    title: '3. Simulation & Processing',
    content: `
      # The Simulation Lifecycle
      
      The \`FSMExecutor\` class drives the simulation loop.
      
      ### The Processing Loop
      1.  **Node Entry**: The engine enters a node and executes the \`entryAction\` JavaScript code. This code can manipulate \`ctx\` or call \`HAL\` methods.
      2.  **Idle/Wait**: The engine enters a suspended state, waiting for an **Event**.
      3.  **Dispatch**: Events are triggered via \`dispatch("EVENT_NAME", delay)\` within node logic or by external UI interactions.
      4.  **Guard Evaluation**: When an event fires, the engine evaluates the conditions of all outgoing edges.
      5.  **Transition**: If a guard passes, the engine executes the current node's \`exitAction\`, triggers the edge animation, and moves to the target node.
      
      ### Shadow Mode (Digital Twin)
      In Shadow Mode, the simulator disconnects its internal clock. It acts as a passive visualizer, waiting for **Hardware Sync** events from a connected physical device (via WebSerial) to update the current state.
    `
  },
  {
    id: 'hal_reference',
    title: '4. HAL & Virtual I/O',
    content: `
      # Hardware Abstraction Layer
      
      The \`HAL\` object is globally available in all node scripts.
      
      ### API Reference
      - **GPIO**: 
        - \`HAL.writePin(pin: number, value: boolean)\`
        - \`HAL.readPin(pin: number): boolean\`
      - **ADC**: 
        - \`HAL.getADC(channel: number)\` (Returns simulated 12-bit value 0-4095)
      - **PWM**: 
        - \`HAL.setPWM(channel: number, duty: number)\`
      - **UART**: 
        - \`HAL.UART_Transmit(string)\`
        - \`HAL.UART_Receive(): string | null\`
      
      ### Virtual Tools
      - **I/O Panel**: A UI overlay that automatically generates Switches and LEDs for any \`ctx\` variable named \`btn_*\` or \`led_*\`.
      - **Logic Analyzer**: A real-time canvas rendering digital waveforms of GPIO and PWM signals history.
    `
  },
  {
    id: 'ai_engine',
    title: '5. AI & Neo Companion',
    content: `
      # Gemini 3 Pro Integration
      
      NeuroState uses a multimodal AI pipeline for advanced capabilities.
      
      ### Neo (Live Companion)
      - **Architecture**: Uses the Gemini Multimodal Live API via WebRTC.
      - **Capabilities**: Real-time voice interaction, context-aware graph manipulation.
      - **Tools**: Neo has write access to the graph via \`create_design\` and \`modify_design\` function calls.
      
      ### Ghost Engineer
      - **Static Analysis**: Scans the node graph for topological issues (dead ends, race conditions, unreachable nodes).
      - **Auto-Fix**: Can autonomously modify the graph to resolve detected issues.
      
      ### Code Generation
      - **Transpiler**: Converts the JSON graph into production-ready code for:
        - **C++** (Arduino/STM32)
        - **Verilog** (FPGA)
        - **Python** (Embedded Linux)
        - **Rust** (Safety-critical)
        - **GoogleTest** (Unit Testing)
    `
  },
  {
    id: 'interface_tools',
    title: '6. Interface & Tools',
    content: `
      # Workbench Features
      
      - **Workspaces**:
        - *Architect*: Focus on high-level design.
        - *Firmware Engineer*: Debugger and variable inspection focus.
        - *Hardware Lab*: I/O and Logic Analyzer focus.
      - **Device Manager**: Bridges the browser to physical hardware using the Web Serial API for flashing and telemetry.
      - **Diagnostic Panel**: Low-level view of HAL state (Registers, Buffers).
      - **Persistence**: Automatic LocalStorage saving with JSON Import/Export capabilities.
    `
  }
];

// --- CUSTOM NODE COMPONENT (ADVANCED VISUALS) ---
const RetroNode = ({ data, id, selected }: { data: FSMNodeData, id: string, selected: boolean }) => {
  const isInput = data.type === 'input' || data.label === 'START';
  const isOutput = data.type === 'output' || data.label === 'END';
  const isError = data.type === 'error';
  const isListener = data.type === 'listener';
  const isDecision = data.type === 'decision';
  const isHardware = data.type === 'hardware';
  const isUART = data.type === 'uart';
  const isInterrupt = data.type === 'interrupt';
  const isTimer = data.type === 'timer';
  const isPeripheral = data.type === 'peripheral';
  // New Types
  const isQueue = data.type === 'queue';
  const isMutex = data.type === 'mutex';
  const isCritical = data.type === 'critical';
  const isMath = data.type === 'math';
  const isWireless = data.type === 'wireless';
  const isStorage = data.type === 'storage';
  const isLogger = data.type === 'logger';
  const isDisplay = data.type === 'display';
  const isNetwork = data.type === 'network';
  const isSensor = data.type === 'sensor';
  
  // Custom Check for Code Analysis Node
  const isCodeAnalysis = data.label === 'CODE_ANALYSIS';
  
  const { setNodes, setEdges } = useReactFlow();
  
  const onDelete = () => {
    setNodes((nodes) => nodes.filter((n) => n.id !== id));
    setEdges((edges) => edges.filter((e) => e.source !== id && e.target !== id));
  };
  
  const onClone = () => {
     setNodes((nodes) => {
        const node = nodes.find(n => n.id === id);
        if(!node) return nodes;
        const newNode = {
           ...node,
           id: `node_${Date.now()}`,
           position: { x: node.position.x + 50, y: node.position.y + 50 },
           data: { ...node.data, label: `${node.data.label}_COPY` },
           selected: true
        };
        return [...nodes.map(n => ({...n, selected: false})), newNode];
     });
  };

  const onToggleBreakpoint = () => {
     setNodes((nodes) => nodes.map(n => n.id === id ? { ...n, data: { ...n.data, isBreakpoint: !n.data.isBreakpoint } } : n));
  };

  return (
    <>
      <NodeToolbar isVisible={selected} position={Position.Top} className="flex gap-1 mb-2">
         <button onClick={onDelete} className="bg-white text-red-600 border border-neuro-dim p-1 rounded shadow-sm hover:bg-red-50" title="Delete"><Trash2 size={12}/></button>
         <button onClick={onClone} className="bg-white text-neuro-primary border border-neuro-dim p-1 rounded shadow-sm hover:bg-gray-50" title="Duplicate"><Copy size={12}/></button>
         <button onClick={onToggleBreakpoint} className={clsx("border border-neuro-dim p-1 rounded shadow-sm hover:bg-gray-50", data.isBreakpoint ? "bg-red-100 text-red-600" : "bg-white text-neuro-primary")} title="Toggle Breakpoint"><Disc size={12}/></button>
      </NodeToolbar>

      {/* Execution Monitor Overlay */}
      {(data.executionState === 'entry' || data.executionState === 'exit') && data.executionLog && (
         <div className="absolute -top-12 left-1/2 -translate-x-1/2 z-50 pointer-events-none">
            <div className={clsx("px-2 py-1 rounded text-[9px] font-mono border shadow-xl flex items-center gap-2 whitespace-nowrap animate-in zoom-in-95 duration-200", 
               data.executionState === 'entry' ? "bg-blue-900 text-blue-100 border-blue-500" : "bg-purple-900 text-purple-100 border-purple-500"
            )}>
               {data.executionState === 'entry' ? <span className="font-bold text-blue-400">ENTRY &gt;</span> : <span className="font-bold text-purple-400">EXIT &gt;</span>}
               <span className="opacity-90">{data.executionLog.substring(0, 30)}{data.executionLog.length > 30 ? '...' : ''}</span>
            </div>
            {/* Connecting line */}
            <div className={clsx("w-0.5 h-3 mx-auto", data.executionState === 'entry' ? "bg-blue-500" : "bg-purple-500")}></div>
         </div>
      )}

      <div className={clsx(
        "min-w-[160px] bg-white border transition-all duration-300 relative group flex flex-col rounded-sm overflow-hidden font-mono",
        selected && !data.active ? "border-neuro-primary shadow-lg scale-[1.02] z-20" : "border-neuro-dim hover:border-neuro-primary hover:shadow-md",
        (isError || data.error) && "border-red-500 ring-1 ring-red-500 bg-red-50/10",
        isListener && !selected && !data.active && "border-indigo-300 bg-indigo-50/10",
        isDecision && !selected && !data.active && "border-amber-400 bg-amber-50/10",
        isHardware && !selected && !data.active && "border-cyan-500 bg-cyan-50/10",
        isUART && !selected && !data.active && "border-purple-500 bg-purple-50/10",
        isInterrupt && !selected && !data.active && "border-purple-600 bg-purple-50/10",
        isTimer && !selected && !data.active && "border-orange-500 bg-orange-50/10",
        isPeripheral && !selected && !data.active && "border-teal-500 bg-teal-50/10",
        
        isQueue && !selected && !data.active && "border-pink-500 bg-pink-50/10",
        isMutex && !selected && !data.active && "border-slate-500 bg-slate-50/10",
        isCritical && !selected && !data.active && "border-rose-600 bg-rose-50/10",
        isMath && !selected && !data.active && "border-blue-400 bg-blue-50/10",
        isWireless && !selected && !data.active && "border-sky-500 bg-sky-50/10",
        isStorage && !selected && !data.active && "border-amber-600 bg-amber-50/10",
        isLogger && !selected && !data.active && "border-gray-500 bg-gray-50/10",
        isDisplay && !selected && !data.active && "border-fuchsia-500 bg-fuchsia-50/10",
        isNetwork && !selected && !data.active && "border-indigo-600 bg-indigo-50/10",
        isSensor && !selected && !data.active && "border-emerald-500 bg-emerald-50/10",
        isCodeAnalysis && !selected && !data.active && "border-blue-600 bg-blue-50/10",

        data.active && "!border-green-600 !shadow-[0_0_30px_rgba(34,197,94,0.4)] !ring-2 !ring-green-400 !bg-green-50 z-30 !scale-105",
        data.executionState === 'entry' && "!border-blue-500 !shadow-[0_0_20px_rgba(59,130,246,0.6)] !bg-blue-50 z-40",
        data.executionState === 'exit' && "!border-purple-500 !shadow-[0_0_20px_rgba(168,85,247,0.6)] !bg-purple-50 z-40"
      )}>
        
        <div className={clsx(
           "h-1 w-full transition-colors duration-300",
           isInput ? "bg-neuro-primary" : 
           isOutput ? "bg-neuro-accent" : 
           isError ? "bg-red-500" :
           isListener ? "bg-indigo-500" :
           isDecision ? "bg-amber-500" :
           isHardware ? "bg-cyan-600" :
           isUART ? "bg-purple-600" :
           isInterrupt ? "bg-purple-600" :
           isTimer ? "bg-orange-500" :
           isPeripheral ? "bg-teal-500" :
           isQueue ? "bg-pink-500" :
           isMutex ? "bg-slate-500" :
           isCritical ? "bg-rose-600" :
           isMath ? "bg-blue-400" :
           isWireless ? "bg-sky-500" :
           isStorage ? "bg-amber-600" :
           isLogger ? "bg-gray-500" :
           isDisplay ? "bg-fuchsia-500" :
           isNetwork ? "bg-indigo-600" :
           isSensor ? "bg-emerald-500" :
           isCodeAnalysis ? "bg-blue-600" :
           data.active ? "bg-green-600" : 
           data.executionState === 'entry' ? "bg-blue-500" :
           data.executionState === 'exit' ? "bg-purple-500" :
           "bg-gray-200"
        )}></div>

        {data.isBreakpoint && (
           <div className="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full animate-pulse shadow-sm" title="Breakpoint Active" />
        )}

        <div className="p-3 flex flex-col gap-2">
          <div className="flex items-center gap-2">
             <div className={clsx("p-1 rounded-sm shrink-0 text-white transition-colors duration-300", 
                isInput ? "bg-neuro-primary" : isOutput ? "bg-neuro-accent" : 
                isError ? "bg-red-500" : 
                isListener ? "bg-indigo-500" :
                isDecision ? "bg-amber-500" :
                isHardware ? "bg-cyan-600" :
                isUART ? "bg-purple-600" :
                isInterrupt ? "bg-purple-600" :
                isTimer ? "bg-orange-500" :
                isPeripheral ? "bg-teal-500" :
                isQueue ? "bg-pink-500" :
                isMutex ? "bg-slate-500" :
                isCritical ? "bg-rose-600" :
                isMath ? "bg-blue-400" :
                isWireless ? "bg-sky-500" :
                isStorage ? "bg-amber-600" :
                isLogger ? "bg-gray-500" :
                isDisplay ? "bg-fuchsia-500" :
                isNetwork ? "bg-indigo-600" :
                isSensor ? "bg-emerald-500" :
                isCodeAnalysis ? "bg-blue-600" :
                data.executionState === 'entry' ? "bg-blue-500" :
                data.executionState === 'exit' ? "bg-purple-500" :
                "bg-gray-200 text-gray-500"
             )}>
                {data.executionState === 'entry' || data.executionState === 'exit' ? <Loader2 size={10} className="animate-spin"/> :
                 isInput ? <Play size={10} fill="currentColor"/> : 
                 isOutput ? <CheckCircle size={10}/> : 
                 isError ? <AlertTriangle size={10}/> :
                 isListener ? <Ear size={10}/> :
                 isDecision ? <Split size={10}/> :
                 isHardware ? <CircuitBoard size={10}/> :
                 isUART ? <Cable size={10}/> :
                 isInterrupt ? <Zap size={10}/> :
                 isTimer ? <Hourglass size={10}/> :
                 isPeripheral ? <Cpu size={10}/> :
                 isQueue ? <Layers size={10}/> :
                 isMutex ? <Lock size={10}/> :
                 isCritical ? <ShieldAlert size={10}/> :
                 isMath ? <Calculator size={10}/> :
                 isWireless ? <Wifi size={10}/> :
                 isStorage ? <Database size={10}/> :
                 isLogger ? <FileText size={10}/> :
                 isDisplay ? <Monitor size={10}/> :
                 isNetwork ? <Globe size={10}/> :
                 isSensor ? <Thermometer size={10}/> :
                 isCodeAnalysis ? <FileCode size={10}/> :
                 <Square size={10}/>}
             </div>
             <div className="flex-1 min-w-0">
               <div className={clsx("font-bold text-xs tracking-tight truncate transition-colors", 
                 data.active ? "text-green-800" : "text-neuro-primary"
               )}>
                 {data.label}
               </div>
               <div className="text-[9px] text-gray-400 truncate uppercase tracking-wider">{data.type}</div>
             </div>
          </div>

          {(data.entryAction || data.exitAction) && (
            <div className="bg-gray-50 border border-gray-100 rounded p-1.5 mt-1 overflow-hidden">
               <div className="text-[8px] text-gray-400 font-bold mb-0.5 flex items-center gap-1"><Terminal size={8}/> LOGIC</div>
               <div className="text-[9px] font-mono text-gray-600 truncate opacity-75">
                 {data.entryAction ? data.entryAction.split('\n')[0] : data.exitAction?.split('\n')[0]}
               </div>
            </div>
          )}

          {data.tags && data.tags.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-1">
              {data.tags.map(t => (
                <span key={t} className="px-1.5 py-0.5 bg-gray-100 text-gray-500 text-[8px] rounded-sm font-bold flex items-center gap-0.5">
                   <Hash size={6}/> {t}
                </span>
              ))}
            </div>
          )}
        </div>

        <Handle type="target" position={Position.Top} className="!w-2 !h-2 !rounded-full !border !border-neuro-dim !bg-white transition-all top-[-4px]" />
        <Handle type="source" position={Position.Bottom} className="!w-2 !h-2 !rounded-full !border !border-neuro-dim !bg-white transition-all bottom-[-4px]" />
      </div>
    </>
  );
};

// --- GROUP NODE FOR HIERARCHY ---
const GroupNode = ({ data, selected }: { data: FSMNodeData, selected: boolean }) => {
   return (
      <div className={clsx("w-full h-full border-2 border-dashed rounded-md p-4 transition-all -z-10 relative", selected ? "border-neuro-primary bg-neuro-primary/5" : "border-gray-300 bg-gray-50/50")}>
         <div className="absolute top-0 left-2 -translate-y-1/2 bg-white px-2 text-[10px] font-bold text-gray-500 flex items-center gap-1 border border-gray-200 rounded">
            <Group size={10} /> {data.label}
         </div>
      </div>
   );
};

// --- ADVANCED EDGE WITH PACKET ANIMATION ---
const RetroEdge = ({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, style = {}, markerEnd, label, selected, animated, data }: EdgeProps) => {
  const [edgePath, labelX, labelY] = getSmoothStepPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, borderRadius: 8 });
  const hasCondition = data && data.condition && data.condition.trim() !== '';
  const isTraversing = data?.isTraversing;
  const guardResult = data?.guardResult;

  return (
    <>
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={{ ...style, strokeWidth: selected || animated || isTraversing ? 2 : 1.5, stroke: isTraversing ? '#06b6d4' : selected ? '#111827' : (style.stroke || '#d1d5db'), transition: 'stroke 0.3s' }} />
      
      {/* PACKET ANIMATION */}
      {isTraversing && (
        <circle r="4" fill="#06b6d4">
          <animateMotion dur="0.8s" repeatCount="1" path={edgePath} rotate="auto" />
        </circle>
      )}

      {(label || hasCondition) && (
        <EdgeLabelRenderer>
           <div style={{ position: 'absolute', transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`, pointerEvents: 'all' }} className={clsx("px-1.5 py-0.5 text-[9px] font-bold font-mono tracking-wider border transition-all duration-300 bg-white shadow-sm select-none rounded-[2px] flex flex-col items-center gap-0.5", selected ? "border-neuro-primary text-neuro-primary z-20" : "border-gray-200 text-gray-400 z-10", animated && "border-green-500 text-green-700 bg-green-50", isTraversing && "!border-cyan-500 !text-cyan-600 !bg-cyan-50 scale-110 shadow-md")}>
             <div title={typeof label === 'string' ? label : undefined}>{label}</div>
             {hasCondition && (
                <div className={clsx("text-[7px] px-1 rounded-sm border flex items-center gap-1", 
                   guardResult === 'pass' ? "bg-green-100 text-green-800 border-green-200" : 
                   guardResult === 'fail' ? "bg-red-100 text-red-800 border-red-200" :
                   "bg-yellow-100 text-yellow-800 border-yellow-200"
                )} title={`Guard: ${data.condition}`}>
                   {guardResult === 'pass' ? <CheckCircle size={6}/> : guardResult === 'fail' ? <X size={6}/> : <span>ƒ(x)</span>}
                </div>
             )}
           </div>
           {selected && (
            <div style={{ position: 'absolute', transform: `translate(-50%, -120%) translate(${labelX}px,${labelY}px)`, pointerEvents: 'none' }} className="z-50 bg-neuro-primary text-white text-[10px] p-2 rounded shadow-xl min-w-[120px]">
               <div className="font-bold border-b border-gray-600 pb-1 mb-1">EDGE DETAILS</div>
               <div>Event: <span className="text-neuro-accent">{label}</span></div>
               {hasCondition && <div className="mt-1 text-yellow-300">Guard: {data.condition}</div>}
            </div>
           )}
        </EdgeLabelRenderer>
      )}
    </>
  );
};

// ... (Rest of components: ContextMenu, LayoutMenu, TemplateBrowser, DeviceManagerModal, DiagnosticPanel - kept largely the same)
const ContextMenu: React.FC<{ top: number; left: number; onClose: () => void; onAddNode: (type: any, x: number, y: number) => void; onGroupSelected?: () => void; onAiDefine?: () => void }> = ({ top, left, onClose, onAddNode, onGroupSelected, onAiDefine }) => {
  useEffect(() => { const h = () => onClose(); document.addEventListener('click', h); return () => document.removeEventListener('click', h); }, [onClose]);
  return (
    <div style={{ top, left }} className="absolute z-50 bg-white border border-neuro-dim shadow-lg rounded-sm min-w-[160px] py-1 animate-in fade-in zoom-in-95 duration-100 flex flex-col max-h-[400px]">
      <div className="px-3 py-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest border-b border-gray-100 mb-1 shrink-0">Add Node</div>
      <div className="overflow-y-auto custom-scrollbar flex-1">
        {['process', 'decision', 'hardware', 'uart', 'listener', 'input', 'output', 'queue', 'mutex', 'critical', 'math', 'wireless', 'storage', 'logger', 'display', 'network', 'sensor'].map(t => (
          <button key={t} onClick={() => onAddNode(t, left, top)} className="w-full text-left px-4 py-2 text-xs hover:bg-gray-50 hover:text-neuro-primary font-bold capitalize flex items-center gap-2 text-gray-600">
            <Plus size={12}/> {t}
          </button>
        ))}
        
        <div className="h-px bg-gray-100 my-1 mx-2"></div>
        <button onClick={() => onAddNode('code_analysis', left, top)} className="w-full text-left px-4 py-2 text-xs hover:bg-gray-50 hover:text-neuro-primary font-bold capitalize flex items-center gap-2 text-blue-600">
            <FileCode size={12}/> Code Analysis
        </button>
      </div>
      
      {(onGroupSelected || onAiDefine) && (
         <div className="shrink-0 border-t border-gray-100 mt-1 pt-1">
            {onGroupSelected && (
                <button onClick={onGroupSelected} className="w-full text-left px-4 py-2 text-xs hover:bg-gray-50 hover:text-neuro-primary font-bold flex items-center gap-2 text-neuro-primary">
                  <Group size={12}/> Group Selected
                </button>
            )}
            {onAiDefine && (
                <button onClick={onAiDefine} className="w-full text-left px-4 py-2 text-xs hover:bg-indigo-50 hover:text-indigo-600 font-bold flex items-center gap-2 text-indigo-500">
                  <Sparkles size={12}/> AI Define Logic
                </button>
            )}
         </div>
      )}
    </div>
  );
};

const LayoutMenu: React.FC<{ onClose: () => void; onSelect: (template: WorkspaceTemplate) => void; active: WorkspaceTemplate }> = ({ onClose, onSelect, active }) => {
  useEffect(() => { const h = () => onClose(); document.addEventListener('click', h); return () => document.removeEventListener('click', h); }, [onClose]);
  
  const options: { id: WorkspaceTemplate, label: string, icon: any, desc: string }[] = [
    { id: 'FULL_SUITE', label: 'Mission Control', icon: LayoutDashboard, desc: 'All Panels Open' },
    { id: 'ARCHITECT', label: 'Architect', icon: Box, desc: 'Design & Properties' },
    { id: 'ENGINEER', label: 'Firmware Eng.', icon: Cpu, desc: 'Simulate & Registers' },
    { id: 'DEBUG_FOCUS', label: 'Debug Focus', icon: Bug, desc: 'Logs & Hardware' },
    { id: 'HARDWARE_LAB', label: 'Hardware Lab', icon: Wrench, desc: 'IO & Diagnostics' },
    { id: 'AI_PAIR', label: 'AI Pair Prog.', icon: Bot, desc: 'Chat & Canvas' },
    { id: 'AUDITOR', label: 'QA / Auditor', icon: Shield, desc: 'Safety Validation' },
    { id: 'HACKER', label: 'Hacker', icon: Terminal, desc: 'Code & Exploits' },
    { id: 'ZEN', label: 'Zen Mode', icon: Maximize, desc: 'Canvas Only' },
  ];
  
  return (
    <div className="absolute top-10 right-4 z-50 bg-white border border-neuro-primary shadow-hard min-w-[200px] py-1 animate-in fade-in zoom-in-95 duration-100">
       <div className="px-3 py-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest border-b border-gray-100 mb-1">Workspace Layouts</div>
      {options.map(opt => (
        <button key={opt.id} onClick={(e) => { e.stopPropagation(); onSelect(opt.id); }} className={clsx("w-full text-left px-4 py-2 text-xs hover:bg-gray-50 hover:text-neuro-primary font-bold flex items-center gap-3", active === opt.id ? "bg-gray-100 text-neuro-primary" : "text-gray-600")}>
          <opt.icon size={14}/> <div><div className="leading-none">{opt.label}</div><div className="text-[9px] font-normal text-gray-400 mt-0.5 uppercase">{opt.desc}</div></div>
        </button>
      ))}
    </div>
  );
};

// ... (TemplateBrowser, VeoModal, DeviceManagerModal, etc. - kept largely the same)
// Re-inserting TemplateBrowser etc for full context if needed, but reducing bloat for brevity where possible.
// Assuming VeoModal etc are unchanged from previous successful fix.

const TemplateBrowser: React.FC<{ onSelect: (t: FSMTemplate) => void; onClose: () => void }> = ({ onSelect, onClose }) => {
  const [filter, setFilter] = useState('');
  const [category, setCategory] = useState<string>('ALL');
  const categories = ['ALL', ...Array.from(new Set(TEMPLATES.map(t => t.category)))];
  const filtered = TEMPLATES.filter(t => (t.name.toLowerCase().includes(filter.toLowerCase()) || t.description.toLowerCase().includes(filter.toLowerCase())) && (category === 'ALL' || t.category === category));

  return (
    <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
       <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-4xl h-[80vh] flex flex-col animate-in zoom-in-95 duration-150">
          <div className="bg-neuro-primary text-white p-4 flex justify-between items-center shrink-0">
             <div className="font-bold tracking-widest flex items-center gap-3"><Grid size={18}/> FIRMWARE TEMPLATES</div>
             <button onClick={onClose} className="hover:text-red-300"><X size={20}/></button>
          </div>
          <div className="p-4 border-b border-neuro-dim bg-gray-50 flex gap-4 shrink-0">
             <div className="relative flex-1">
               <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"/>
               <input className="w-full pl-9 pr-4 py-2 border border-neuro-dim text-sm outline-none focus:border-neuro-primary" placeholder="Search (e.g., 'USB', 'Bootloader')..." value={filter} onChange={e => setFilter(e.target.value)} autoFocus />
             </div>
             <select className="px-4 py-2 border border-neuro-dim text-sm outline-none focus:border-neuro-primary bg-white" value={category} onChange={e => setCategory(e.target.value)}>
               {categories.map(c => <option key={c} value={c}>{c}</option>)}
             </select>
          </div>
          <div className="flex-1 overflow-y-auto p-4 bg-neuro-bg custom-scrollbar">
             <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {filtered.map(t => (
                  <div key={t.id} onClick={() => onSelect(t)} className="bg-white border border-neuro-dim p-4 cursor-pointer hover:border-neuro-primary hover:shadow-md transition-all group relative overflow-hidden">
                     <div className="absolute top-0 right-0 bg-gray-100 text-[9px] px-2 py-1 font-bold text-gray-500 rounded-bl-sm group-hover:bg-neuro-primary group-hover:text-white transition-colors">{t.category}</div>
                     <h3 className="font-bold text-neuro-primary mb-1 flex items-center gap-2">{t.name}</h3>
                     <p className="text-xs text-gray-500 line-clamp-2 h-8">{t.description}</p>
                     <div className="mt-4 flex gap-2 text-[10px] text-gray-400">
                        <span className="bg-gray-50 px-1.5 py-0.5 border rounded flex items-center gap-1">{t.nodes.length} Nodes</span>
                        <span className="bg-gray-50 px-1.5 py-0.5 border rounded flex items-center gap-1">{t.edges.length} Edges</span>
                     </div>
                  </div>
                ))}
             </div>
          </div>
       </div>
    </div>
  );
};

const VeoModal: React.FC<{ onClose: () => void }> = ({ onClose }) => {
   const [prompt, setPrompt] = useState('A cinematic shot of this circuit board with blinking green LEDs, 4k highly detailed.');
   const [image, setImage] = useState<{base64: string, preview: string, mimeType: string} | null>(null);
   const [aspectRatio, setAspectRatio] = useState<'16:9'|'9:16'>('16:9');
   const [isGenerating, setIsGenerating] = useState(false);
   const [videoUrl, setVideoUrl] = useState<string | null>(null);
   const fileInputRef = useRef<HTMLInputElement>(null);

   const handleImageSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onloadend = () => {
         const result = reader.result as string;
         const [header, base64] = result.split(',');
         const mime = header.match(/:(.*?);/)?.[1] || 'image/png';
         setImage({ base64, preview: result, mimeType: mime });
      };
      reader.readAsDataURL(file);
   };

   // Auto-capture canvas on mount
   useEffect(() => {
      const flowEl = document.querySelector('.react-flow') as HTMLElement;
      if (flowEl) {
         setTimeout(() => {
             toPng(flowEl, { 
                backgroundColor: '#f9fafb',
                width: flowEl.offsetWidth,
                height: flowEl.offsetHeight,
                style: { transform: 'scale(1)', transformOrigin: 'top left' },
                skipFonts: true,
                filter: (node) => {
                    return !node.classList?.contains('react-flow__minimap') && !node.classList?.contains('react-flow__controls');
                }
             })
             .then((dataUrl) => {
                const [header, base64] = dataUrl.split(',');
                setImage({ base64, preview: dataUrl, mimeType: 'image/png' });
             })
             .catch((err) => {
                console.warn("Auto-capture failed (CORS/Permissions). User must upload manually.", err);
             });
         }, 1000);
      }
   }, []);

   const handleGenerate = async () => {
      if (!image) return;
      
      if (!(window as any).aistudio?.hasSelectedApiKey()) {
         try {
            await (window as any).aistudio?.openSelectKey();
         } catch(e) {
            console.error("Key selection failed", e);
            return;
         }
      }

      setIsGenerating(true);
      setVideoUrl(null);
      try {
         const url = await geminiService.generateVeoVideo(prompt, image.base64, image.mimeType, aspectRatio);
         setVideoUrl(url);
      } catch (e: any) {
         console.error(e);
         const errorMsg = e.toString() || "";
         if (errorMsg.includes("Requested entity was not found") || errorMsg.includes("404")) {
             try {
                 await (window as any).aistudio?.openSelectKey();
                 const url = await geminiService.generateVeoVideo(prompt, image.base64, image.mimeType, aspectRatio);
                 setVideoUrl(url);
             } catch (retryError: any) {
                 alert("Generation failed even after key selection. Please ensure your selected project has billing enabled for Veo.");
             }
         } else {
             alert("Video generation failed: " + e.message);
         }
      } finally {
         setIsGenerating(false);
      }
   };

   return (
      <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
         <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-2xl flex flex-col animate-in zoom-in-95 duration-150">
            <div className="bg-neuro-primary text-white p-3 flex justify-between items-center shrink-0">
               <div className="font-bold tracking-widest flex items-center gap-2"><Film size={16}/> VEO VISUALIZATION</div>
               <button onClick={onClose} className="hover:text-red-300"><X size={18}/></button>
            </div>
            <div className="p-6 flex flex-col gap-4">
               <div className="flex gap-4">
                  <div 
                     className="w-48 h-32 border-2 border-dashed border-gray-300 flex flex-col items-center justify-center cursor-pointer hover:border-neuro-primary hover:bg-gray-50 relative overflow-hidden bg-gray-100 group"
                     onClick={() => fileInputRef.current?.click()}
                  >
                     {image ? (
                        <>
                           <img src={image.preview} className="w-full h-full object-cover" />
                           <div className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity text-white text-[10px] font-bold">
                              CHANGE IMAGE
                           </div>
                        </>
                     ) : (
                        <>
                           <Camera size={24} className="text-gray-400 mb-2"/>
                           <span className="text-[10px] text-gray-500 uppercase font-bold text-center px-2">Capturing Canvas...<br/>(Or Click to Upload)</span>
                        </>
                     )}
                     <input type="file" ref={fileInputRef} className="hidden" accept="image/*" onChange={handleImageSelect} />
                  </div>
                  <div className="flex-1 flex flex-col gap-2">
                     <div>
                        <Label>Prompt</Label>
                        <textarea className="w-full h-20 text-xs p-2 border border-neuro-dim outline-none resize-none" value={prompt} onChange={e => setPrompt(e.target.value)} />
                     </div>
                     <div>
                        <Label>Aspect Ratio</Label>
                        <div className="flex gap-2">
                           {['16:9', '9:16'].map(r => (
                              <button key={r} onClick={() => setAspectRatio(r as any)} className={clsx("px-3 py-1 text-xs border", aspectRatio === r ? "bg-neuro-primary text-white" : "bg-white text-gray-500 hover:bg-gray-50")}>{r}</button>
                           ))}
                        </div>
                     </div>
                  </div>
               </div>

               <Button onClick={handleGenerate} disabled={!image || isGenerating} className={clsx("w-full py-3", isGenerating ? "bg-amber-50 text-amber-600 border-amber-200" : "")}>
                  {isGenerating ? <><Loader2 size={16} className="animate-spin"/> GENERATING VIDEO (This may take a minute)...</> : <><Film size={16}/> GENERATE PREVIEW</>}
               </Button>

               {videoUrl && (
                  <div className="mt-4 border border-neuro-dim p-2 bg-black">
                     <video src={videoUrl} controls autoPlay loop className="w-full max-h-[300px]" />
                     <div className="mt-2 flex justify-end">
                        <a href={videoUrl} download="neurostate_preview.mp4" className="text-white text-xs underline hover:text-neuro-accent">Download MP4</a>
                     </div>
                  </div>
               )}
            </div>
         </div>
      </div>
   );
};

const DeviceManagerModal: React.FC<{ onClose: () => void; onConnect: (target: McuDefinition) => void; isConnected: boolean }> = ({ onClose, onConnect, isConnected }) => {
   const [selectedId, setSelectedId] = useState<string>(MCU_REGISTRY[0].id);
   const currentMcu = MCU_REGISTRY.find(m => m.id === selectedId) || MCU_REGISTRY[0];

   return (
     <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
        <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-2xl flex flex-col animate-in zoom-in-95 duration-150">
           <div className="bg-neuro-primary text-white p-3 flex justify-between items-center shrink-0">
              <div className="font-bold tracking-widest flex items-center gap-2"><HardDrive size={16}/> DEVICE MANAGER</div>
              <button onClick={onClose} className="hover:text-red-300"><X size={18}/></button>
           </div>
           <div className="p-4 flex gap-4 h-[400px]">
              <div className="w-1/3 border-r border-neuro-dim pr-2 overflow-y-auto custom-scrollbar">
                 {MCU_REGISTRY.map(mcu => (
                    <div key={mcu.id} onClick={() => setSelectedId(mcu.id)} className={clsx("p-2 text-xs font-bold cursor-pointer border-b border-neuro-dim hover:bg-gray-50", selectedId === mcu.id ? "bg-neuro-primary text-white hover:bg-neuro-primary" : "text-gray-600")}>
                       <div className="truncate">{mcu.name}</div>
                       <div className="text-[9px] opacity-70 font-normal">{mcu.family}</div>
                    </div>
                 ))}
              </div>
              <div className="flex-1 flex flex-col">
                 <h3 className="text-lg font-bold text-neuro-primary">{currentMcu.name}</h3>
                 <div className="text-xs text-gray-500 mb-4 font-mono">{currentMcu.description}</div>
                 
                 <div className="grid grid-cols-2 gap-4 mb-6">
                    <MetricCard label="Architecture" value={currentMcu.arch} />
                    <MetricCard label="Flash Method" value={currentMcu.flashMethod} />
                    <MetricCard label="Flash Size" value={currentMcu.specs.flashKB} unit="KB" />
                    <MetricCard label="Frequency" value={currentMcu.specs.freqMHz} unit="MHz" />
                 </div>

                 <div className="mt-auto bg-gray-50 p-3 border border-neuro-dim text-[10px] text-gray-500 font-mono">
                    {currentMcu.flashMethod === 'WEB_SERIAL' && "Ready to connect via Web Serial API. Ensure drivers are installed."}
                    {currentMcu.flashMethod === 'USB_MSD' && "Device requires UF2 Drag-and-Drop flashing."}
                    {currentMcu.flashMethod === 'DOWNLOAD_BIN' && "Direct flashing not supported. Binary will be downloaded."}
                 </div>

                 <div className="mt-4 flex justify-end gap-2">
                    <Button variant="ghost" onClick={onClose}>Cancel</Button>
                    <Button onClick={() => onConnect(currentMcu)} disabled={isConnected && currentMcu.flashMethod === 'WEB_SERIAL'}>
                       {isConnected && currentMcu.flashMethod === 'WEB_SERIAL' ? "Connected" : "Select & Connect"}
                    </Button>
                 </div>
              </div>
           </div>
        </div>
     </div>
   );
};

// ... (DocumentationModal, AboutModal, DiagnosticPanel, SerialMonitor, CompanionOrb - kept same)
const DocumentationModal: React.FC<{ onClose: () => void }> = ({ onClose }) => {
   const [activeSection, setActiveSection] = useState(DOCS_CONTENT[0].id);
   const activeContent = DOCS_CONTENT.find(c => c.id === activeSection) || DOCS_CONTENT[0];

   return (
      <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
         <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-4xl h-[80vh] flex flex-col animate-in zoom-in-95 duration-150">
            <div className="bg-neuro-primary text-white p-3 flex justify-between items-center shrink-0">
               <div className="font-bold tracking-widest flex items-center gap-2"><Book size={16}/> NEUROSTATE MANUAL</div>
               <button onClick={onClose} className="hover:text-red-300"><X size={18}/></button>
            </div>
            <div className="flex-1 flex overflow-hidden">
               {/* Sidebar */}
               <div className="w-1/4 border-r border-neuro-dim bg-gray-50 p-2 overflow-y-auto">
                  {DOCS_CONTENT.map(section => (
                     <button
                        key={section.id}
                        onClick={() => setActiveSection(section.id)}
                        className={clsx("w-full text-left px-3 py-2 text-xs font-bold rounded-sm mb-1 transition-colors", activeSection === section.id ? "bg-neuro-primary text-white" : "text-gray-600 hover:bg-gray-200")}
                     >
                        {section.title}
                     </button>
                  ))}
               </div>
               {/* Content */}
               <div className="flex-1 p-6 overflow-y-auto bg-white custom-scrollbar prose prose-sm max-w-none">
                  {activeContent.content.split('\n').map((line, i) => {
                     const trimmed = line.trim();
                     if (trimmed.startsWith('# ')) return <h1 key={i} className="text-2xl font-bold mb-4 border-b pb-2">{trimmed.slice(2)}</h1>;
                     if (trimmed.startsWith('### ')) return <h3 key={i} className="text-lg font-bold mt-6 mb-2 text-neuro-primary">{trimmed.slice(4)}</h3>;
                     if (trimmed.startsWith('- ')) return <li key={i} className="ml-4 list-disc text-gray-700 mb-1">{trimmed.slice(2)}</li>;
                     if (trimmed.startsWith('```')) return null;
                     return <p key={i} className="mb-2 text-gray-600 leading-relaxed">{trimmed}</p>;
                  })}
               </div>
            </div>
         </div>
      </div>
   );
};

const AboutModal: React.FC<{ onClose: () => void }> = ({ onClose }) => {
   return (
      <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
         <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-md flex flex-col animate-in zoom-in-95 duration-150">
            <div className="bg-neuro-primary text-white p-3 flex justify-between items-center shrink-0">
               <div className="font-bold tracking-widest flex items-center gap-2"><Info size={16}/> ABOUT NEUROSTATE</div>
               <button onClick={onClose} className="hover:text-red-300"><X size={18}/></button>
            </div>
            <div className="p-6 text-center space-y-4">
               <div className="flex justify-center mb-2">
                  <div className="p-3 bg-gray-100 rounded-full border border-neuro-dim">
                     <CircuitBoard size={48} className="text-neuro-primary"/>
                  </div>
               </div>
               
               <div>
                  <h2 className="text-xl font-bold text-neuro-primary">NeuroState</h2>
                  <p className="text-gray-500 text-xs mt-1">v1.1 - Embedded Systems Intelligence</p>
               </div>

               <p className="text-xs text-gray-600 leading-relaxed max-w-[300px] mx-auto">
                  A multimodal bridge translating analog intent into digital logic. Designed for firmware engineers to architect, simulate, and verify Finite State Machines with the power of Gemini 3 Pro.
               </p>

               <div className="grid grid-cols-2 gap-2 text-[10px] text-gray-500 mt-4 border-t border-b border-gray-100 py-3">
                  <div className="flex flex-col gap-1">
                     <span className="font-bold text-gray-700">AI ENGINE</span>
                     <span>Gemini 3 Pro</span>
                  </div>
                  <div className="flex flex-col gap-1">
                     <span className="font-bold text-gray-700">ARCHITECTURE</span>
                     <span>React + WebSerial</span>
                  </div>
                  <div className="flex flex-col gap-1">
                     <span className="font-bold text-gray-700">SIMULATION</span>
                     <span>Async Executor</span>
                  </div>
                  <div className="flex flex-col gap-1">
                     <span className="font-bold text-gray-700">DESIGN</span>
                     <span>Minimal Retro</span>
                  </div>
               </div>

               <Button onClick={onClose} className="w-full mt-2">CLOSE</Button>
            </div>
         </div>
      </div>
   );
};

const DiagnosticPanel = ({ state }: { state: HalSnapshot }) => {
  return (
    <Panel title="HARDWARE DIAGNOSTICS" className="w-[300px] pointer-events-auto" actions={<Monitor size={12}/>}>
      <div className="bg-[#111] text-green-500 font-mono text-[10px] p-2 h-[300px] overflow-y-auto">
        <div className="mb-2 font-bold text-white border-b border-gray-700">GPIO STATE</div>
        <div className="grid grid-cols-4 gap-1 mb-3">
           {state.gpio && Object.keys(state.gpio).length === 0 && <span className="opacity-50 italic">No Pins Active</span>}
           {state.gpio && Object.entries(state.gpio).map(([pin, val]) => (
             <div key={pin} className={clsx("px-1 py-0.5 text-center border", val ? "bg-green-900 border-green-500 text-white" : "border-gray-700 text-gray-500")}>
                P{pin}:{val?'1':'0'}
             </div>
           ))}
        </div>

        <div className="mb-2 font-bold text-white border-b border-gray-700">PWM CHANNELS</div>
        <div className="space-y-1 mb-3">
           {state.pwm && Object.keys(state.pwm).length === 0 && <span className="opacity-50 italic">No PWM Active</span>}
           {state.pwm && Object.entries(state.pwm).map(([ch, val]) => (
             <div key={ch} className="flex items-center gap-2">
                <span className="w-6 text-gray-400">CH{ch}</span>
                <div className="flex-1 h-1.5 bg-gray-800"><div className="h-full bg-yellow-600" style={{width: `${val}%`}}></div></div>
                <span className="w-6 text-right text-yellow-500">{val}%</span>
             </div>
           ))}
        </div>

        <div className="mb-2 font-bold text-white border-b border-gray-700">UART BUFFERS</div>
        <div className="mb-1 text-gray-400">TX (Out):</div>
        <div className="bg-black border border-gray-700 p-1 mb-2 h-12 overflow-y-auto text-cyan-400 break-all whitespace-pre-wrap">
           {state.uartTx && state.uartTx.length ? state.uartTx.join('\n') : <span className="opacity-30">-- empty --</span>}
        </div>
        <div className="mb-1 text-gray-400">RX (In):</div>
        <div className="bg-black border border-gray-700 p-1 h-12 overflow-y-auto text-purple-400 break-all whitespace-pre-wrap">
           {state.uartRx && state.uartRx.length ? state.uartRx.join('\n') : <span className="opacity-30">-- empty --</span>}
        </div>
      </div>
    </Panel>
  );
};

const SerialMonitor = ({ state }: { state: HalSnapshot }) => {
   const [input, setInput] = useState('');
   const scrollRef = useRef<HTMLDivElement>(null);

   useEffect(() => {
      if(scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
   }, [state.uartTx, state.uartRx]);

   const handleSend = (e: React.FormEvent) => {
      e.preventDefault();
      if (!input.trim()) return;
      HAL.mockReceive(input);
      setInput('');
   };

   return (
      <div className="flex flex-col h-full bg-[#1e1e1e] font-mono text-xs">
         <div className="flex-1 overflow-y-auto p-2 space-y-1" ref={scrollRef}>
            {state.uartTx.map((msg, i) => (
               <div key={`tx-${i}`} className="text-cyan-400 flex"><span className="w-12 opacity-50 select-none text-right mr-2">TX &gt;</span> {msg}</div>
            ))}
            {state.uartRx.map((msg, i) => (
               <div key={`rx-${i}`} className="text-purple-400 flex"><span className="w-12 opacity-50 select-none text-right mr-2">RX &lt;</span> {msg}</div>
            ))}
            {state.uartTx.length === 0 && state.uartRx.length === 0 && <div className="text-gray-600 italic p-4 text-center">Serial buffer empty. Start simulation or send data.</div>}
         </div>
         <form onSubmit={handleSend} className="border-t border-gray-700 p-2 flex gap-2 bg-[#252526]">
            <input 
               className="flex-1 bg-[#3c3c3c] text-white px-2 py-1 outline-none border border-gray-600 focus:border-neuro-primary rounded-sm"
               placeholder="Send mock serial data (e.g., 'OK', 'ERROR')..."
               value={input}
               onChange={e => setInput(e.target.value)}
            />
            <Button type="submit" className="h-6 text-[10px] px-3 bg-neuro-primary text-white border-none hover:bg-gray-700">SEND</Button>
         </form>
      </div>
   );
};

const CompanionOrb: React.FC<{ 
   state: AgentState; 
   onMute: () => void;
   muted: boolean 
}> = ({ state, onMute, muted }) => {
   return (
      <div className="fixed bottom-8 right-8 z-[60] flex flex-col items-center gap-2">
         {/* Status Bubble */}
         {state !== 'IDLE' && (
            <div className={clsx("text-white text-[10px] px-3 py-1 rounded-full shadow-lg animate-in slide-in-from-bottom-2 uppercase font-bold tracking-wider",
               state === 'SPEAKING' ? "bg-green-500" : 
               state === 'MODIFYING' ? "bg-amber-500" : "bg-neuro-primary"
            )}>
               {state === 'SPEAKING' ? 'SPEAKING' : state === 'MODIFYING' ? 'BUILDING...' : 'LISTENING'}
            </div>
         )}
         
         <div className="relative group">
            {/* Animated Rings */}
            {state === 'LISTENING' && (
               <div className="absolute inset-0 rounded-full border-2 border-red-500 animate-ping opacity-75"></div>
            )}
            {state === 'THINKING' && (
               <div className="absolute inset-0 rounded-full border-t-2 border-blue-500 animate-spin"></div>
            )}
            {state === 'MODIFYING' && (
               <div className="absolute inset-0 rounded-full border-4 border-amber-400 animate-spin"></div>
            )}
            {state === 'SPEAKING' && (
               <div className="absolute inset-0 rounded-full border-4 border-green-500 animate-pulse"></div>
            )}

            {/* Main Button */}
            <button 
               onClick={onMute}
               className={clsx(
                  "w-14 h-14 rounded-full shadow-2xl flex items-center justify-center transition-all duration-300 transform active:scale-95 border-2",
                  state === 'IDLE' ? "bg-white border-neuro-dim hover:border-neuro-primary" :
                  state === 'LISTENING' ? "bg-red-500 border-red-600 text-white scale-110" :
                  state === 'THINKING' ? "bg-blue-600 border-blue-400 text-white scale-105" :
                  state === 'MODIFYING' ? "bg-amber-500 border-amber-400 text-white scale-110" :
                  state === 'SPEAKING' ? "bg-green-500 border-green-400 text-white scale-110" :
                  "bg-green-500 border-green-400 text-white"
               )}
            >
               {state === 'IDLE' ? <Mic size={20} className="text-gray-600 group-hover:text-neuro-primary"/> :
                state === 'LISTENING' ? <Waves size={24} className="animate-pulse"/> :
                state === 'THINKING' ? <Loader2 size={24} className="animate-spin"/> :
                state === 'MODIFYING' ? <Wand2 size={24} className="animate-bounce"/> :
                state === 'SPEAKING' ? <Volume2 size={24} className="animate-bounce"/> :
                <Sparkles size={24} className="animate-bounce"/>}
            </button>
         </div>
         
         <div className="text-[9px] font-bold text-gray-400 bg-white/80 px-2 py-0.5 rounded backdrop-blur-sm border border-gray-100 shadow-sm">
            LIVE AGENT
         </div>
      </div>
   );
};

const initialNodes: Node[] = [
  { id: 'start', type: 'input', position: { x: 250, y: 50 }, data: { label: 'PWR_ON_RESET', type: 'input', entryAction: '// Initialize Core Clock\nctx.sysclk = 16000000;\nctx.wdt_enable = true;\ndispatch("BOOT_OK", 1000);' } },
  { id: 'init', type: 'process', position: { x: 250, y: 200 }, data: { label: 'HAL_INIT', type: 'process', entryAction: 'HAL_Init();\nMX_GPIO_Init();\nctx.status = "OK";\ndispatch("SETUP_DONE", 1000);', exitAction: '' } },
  { id: 'loop', type: 'output', position: { x: 250, y: 350 }, data: { label: 'MAIN_LOOP', type: 'output', entryAction: 'console.log("Blinking...");\nHAL.writePin(13, !HAL.readPin(13));\ndispatch("TICK", 500);' } },
  { id: 'uart_tx', type: 'uart', position: { x: 50, y: 350 }, data: { label: 'UART_TX', type: 'uart', entryAction: 'HAL.UART_Transmit("Hello from Node!"); dispatch("TX_SENT", 500);' } }
];
const initialEdges: Edge[] = [
  { id: 'e1', source: 'start', target: 'init', label: 'BOOT_OK', type: 'retro', markerEnd: { type: MarkerType.ArrowClosed } },
  { id: 'e2', source: 'init', target: 'loop', label: 'SETUP_DONE', type: 'retro', markerEnd: { type: MarkerType.ArrowClosed } },
  { id: 'e3', source: 'loop', target: 'loop', label: 'TICK', type: 'retro', markerEnd: { type: MarkerType.ArrowClosed } }
];
const DEFAULT_PROJECT_ID = 'proj_fw_default';
const createDefaultProject = (): FSMProject => ({ id: DEFAULT_PROJECT_ID, name: 'STM32_Blinky', description: 'Basic Firmware Template', version: '0.1.0', nodes: initialNodes, edges: initialEdges, chatHistory: [], updatedAt: Date.now() });

export default function App() { return <ReactFlowProvider><AppContent /></ReactFlowProvider>; }

function AppContent() {
  const [activeLayout, setActiveLayout] = useState<WorkspaceTemplate>('ARCHITECT');
  const [showLeftPanel, setShowLeftPanel] = useState(true);
  const [showRightPanel, setShowRightPanel] = useState(true);
  const [showBottomPanel, setShowBottomPanel] = useState(false);
  const [showIOPanel, setShowIOPanel] = useState(false);
  const [showLayoutMenu, setShowLayoutMenu] = useState(false);
  const [showTemplateBrowser, setShowTemplateBrowser] = useState(false);
  const [showDatasheetModal, setShowDatasheetModal] = useState(false);
  const [showAboutModal, setShowAboutModal] = useState(false);
  const [showDocsModal, setShowDocsModal] = useState(false);
  const [showVeoModal, setShowVeoModal] = useState(false);
  const [datasheetInput, setDatasheetInput] = useState('');
  
  const [showDeviceManager, setShowDeviceManager] = useState(false);
  const [targetMcu, setTargetMcu] = useState<McuDefinition>(MCU_REGISTRY[0]);
  const [isDeviceConnected, setIsDeviceConnected] = useState(false);
  const [flashProgress, setFlashProgress] = useState(0);
  const [flashStatus, setFlashStatus] = useState('');
  const [isFlashing, setIsFlashing] = useState(false);

  const [showDiagnostics, setShowDiagnostics] = useState(false);
  const [halSnapshot, setHalSnapshot] = useState<HalSnapshot>(HAL.getSnapshot());
  const [smartPrompt, setSmartPrompt] = useState('');
  const smartPromptInputRef = useRef<HTMLTextAreaElement>(null);
  
  const [halHistory, setHalHistory] = useState<{ timestamp: number, signals: Record<string, number | boolean> }[]>([]);
  const [isShadowMode, setIsShadowMode] = useState(false);
  const [isCompanionMode, setIsCompanionMode] = useState(false);
  const [isCompanionMuted, setIsCompanionMuted] = useState(false);
  const [isStandbyMode, setIsStandbyMode] = useState(true); 
  const recognitionRef = useRef<any>(null); 

  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const cppInputRef = useRef<HTMLInputElement>(null);

  const [activeBottomTab, setActiveBottomTab] = useState<'OUTPUT' | 'PROBLEMS' | 'VALIDATION' | 'RESOURCES' | 'SERIAL' | 'LOGIC'>('OUTPUT');
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const [contextMenu, setContextMenu] = useState<{ top: number; left: number } | null>(null);

  const [rightPanelTab, setRightPanelTab] = useState<'DEBUG' | 'PROPS' | 'CHAT'>('CHAT');

  const { projects, setProjects, activeProjectId, setActiveProjectId, isLoaded } = usePersistence([createDefaultProject()], DEFAULT_PROJECT_ID);
  
  const activeProject = useMemo(() => (projects.find(p => p.id === activeProjectId) || projects[0]) as FSMProject, [projects, activeProjectId]);

  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);
  
  const selectedNode = useMemo(() => nodes.find(n => n.id === selectedNodeId), [nodes, selectedNodeId]);

  const [simStatus, setSimStatus] = useState<SimulationStatus>(SimulationStatus.IDLE);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [ghostIssues, setGhostIssues] = useState<GhostIssue[]>([]);
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null);
  const [resourceMetrics, setResourceMetrics] = useState<ResourceMetrics | null>(null);
  
  const [activeStateId, setActiveStateId] = useState<string | null>(null);
  const [simHistory, setSimHistory] = useState<string[]>([]);
  const [simContext, setSimContext] = useState<Record<string, any>>({});
  const [autoSimMode, setAutoSimMode] = useState(false);
  const [simSpeed, setSimSpeed] = useState(1000);
  const [simTelemetry, setSimTelemetry] = useState<SimTelemetry | null>(null);
  
  const [agentState, setAgentState] = useState<AgentState>('IDLE');
  
  const [isValidating, setIsValidating] = useState(false);
  const [isEstimating, setIsEstimating] = useState(false);
  const [isAiLoading, setIsAiLoading] = useState(false);
  const [aiQuery, setAiQuery] = useState('');
  
  const chatFileRef = useRef<HTMLInputElement>(null);
  const [chatAttachment, setChatAttachment] = useState<{base64: string, mimeType: string, preview: string} | null>(null);
  
  const { takeSnapshot, undo, redo, clear: clearHistory, canUndo, canRedo } = useHistory(initialNodes, initialEdges);
  const reactFlowInstance = useReactFlow();
  const executorRef = useRef<FSMExecutor | null>(null);
  const autoSimTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const chatScrollRef = useRef<HTMLDivElement>(null);

  const showToast = useCallback((message: string, type: ToastMessage['type'] = 'info') => {
    setToasts(prev => [...prev, { id: Math.random().toString(36), message, type }]);
  }, []);
  const closeToast = useCallback((id: string) => setToasts(prev => prev.filter(t => t.id !== id)), []);

  const addLog = useCallback((message: string, type: 'info' | 'error' | 'success' | 'warning' = 'info') => {
      setLogs(prev => [{
         id: Math.random().toString(36),
         timestamp: new Date().toLocaleTimeString(),
         source: 'SYSTEM' as const,
         message,
         type
      }, ...prev].slice(0, 100));
  }, []);

  useEffect(() => {
    const unsubscribe = HAL.subscribe((snapshot) => {
      const now = Date.now();
      const signals: Record<string, number | boolean> = {};
      Object.entries(snapshot.gpio).forEach(([pin, val]) => { signals[`GPIO_${pin}`] = val ? 1 : 0; });
      Object.entries(snapshot.pwm).forEach(([ch, val]) => { signals[`PWM_${ch}`] = val; });
      signals['ADC_0'] = HAL.getADC(0);
      signals['ADC_1'] = HAL.getADC(1);
      setHalSnapshot(snapshot); 
      setHalHistory(prev => {
        const newHistory = [...prev, { timestamp: now, signals }];
        if (newHistory.length > 200) return newHistory.slice(-200);
        return newHistory;
      });
    });
    return unsubscribe;
  }, []);

  useEffect(() => {
     if (simStatus === SimulationStatus.RUNNING) {
        setRightPanelTab('DEBUG');
        setShowRightPanel(true);
     }
  }, [simStatus]);

  useEffect(() => {
     if (selectedNodeId) {
        setRightPanelTab('PROPS');
        setShowRightPanel(true);
     }
  }, [selectedNodeId]);

  useEffect(() => {
     if (isCompanionMode) {
        setRightPanelTab('CHAT');
        setShowRightPanel(true);
     }
  }, [isCompanionMode]);

  const handleLiveToolCall = useCallback(async (name: string, args: any) => {
     if (name === 'create_design' && args.description) {
         try {
             const newGraph = await geminiService.createGraphFromPrompt(args.description);
             if (newGraph) {
                takeSnapshot(nodes, edges);
                setNodes(newGraph.nodes);
                setEdges(newGraph.edges);
                return "Design created successfully on the canvas.";
             }
         } catch (e) {
             throw new Error("Failed to create design: " + (e as Error).message);
         }
     }
     if (name === 'modify_design' && args.instruction) {
         try {
             const newGraph = await geminiService.modifyGraph(nodes, edges, args.instruction, ghostIssues);
             if (newGraph) {
                takeSnapshot(nodes, edges);
                setNodes(newGraph.nodes);
                setEdges(newGraph.edges);
                return "Modifications applied successfully.";
             }
         } catch (e) {
             throw new Error("Failed to modify design: " + (e as Error).message);
         }
     }
     return "Unknown tool";
  }, [nodes, edges, ghostIssues, takeSnapshot, setNodes, setEdges]);

  const handleLiveToolCallRef = useRef(handleLiveToolCall);
  useEffect(() => {
      handleLiveToolCallRef.current = handleLiveToolCall;
  }, [handleLiveToolCall]);

  useEffect(() => {
    let timeoutId: any;
    if (isCompanionMode) {
      timeoutId = setTimeout(() => {
          liveService.connect(
            (state) => setAgentState(state), 
            (name, args) => handleLiveToolCallRef.current(name, args),
            () => { 
               setIsCompanionMode(false); 
               setAgentState('IDLE');
               showToast("Neo Disconnected", "warning");
            }
          );
      }, 800); 
    } else {
      liveService.disconnect();
      setAgentState('IDLE');
    }
    return () => {
        if(timeoutId) clearTimeout(timeoutId);
        liveService.disconnect();
    };
  }, [isCompanionMode, showToast]);

  const handleWake = useCallback(() => {
        showToast("Neo Activated!", "success");
        setIsCompanionMode(true);
        setRightPanelTab('CHAT');
  }, [showToast]);

  const isWakeWordActive = useWakeWord(isStandbyMode && !isCompanionMode, handleWake);

  const nodeTypes = useMemo(() => ({ 
     input: RetroNode, process: RetroNode, output: RetroNode, error: RetroNode, 
     listener: RetroNode, decision: RetroNode, hardware: RetroNode, uart: RetroNode, 
     interrupt: RetroNode, timer: RetroNode, peripheral: RetroNode, 
     queue: RetroNode, mutex: RetroNode, critical: RetroNode, math: RetroNode,
     wireless: RetroNode, storage: RetroNode, logger: RetroNode, display: RetroNode,
     network: RetroNode, sensor: RetroNode,
     group: GroupNode, default: RetroNode 
  }), []);
  const edgeTypes = useMemo(() => ({ retro: RetroEdge, default: RetroEdge, smoothstep: RetroEdge }), []);

  const syncCurrentProject = useCallback(() => {
      if (!activeProjectId) return;
      setProjects(prev => prev.map(p => p.id === activeProjectId ? { 
          ...p, 
          nodes, 
          edges, 
          updatedAt: Date.now() 
      } : p));
  }, [activeProjectId, nodes, edges, setProjects]);

  const handleAttachmentSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      
      const reader = new FileReader();
      reader.onloadend = () => {
          const result = reader.result as string;
          const [header, base64] = result.split(',');
          const mime = header.match(/:(.*?);/)?.[1] || 'image/png';
          
          setChatAttachment({
              base64,
              mimeType: mime,
              preview: result
          });
      };
      reader.readAsDataURL(file);
      e.target.value = ''; 
  };

  const handleChatSend = async () => {
      if (!aiQuery.trim() && !chatAttachment) return;
      
      const userMsg = aiQuery.trim();
      const currentAttachment = chatAttachment;
      
      setAiQuery('');
      setChatAttachment(null);
      setIsAiLoading(true);

      const displayContent = userMsg + (currentAttachment ? `\n[Attached: ${currentAttachment.mimeType}]` : '');
      appendChatMessage('user', displayContent);

      try {
          let intent = 'CHAT';
          if (currentAttachment) {
              intent = await geminiService.classifyIntent(userMsg || "Analyze this image");
          } else if (userMsg) {
              intent = await geminiService.classifyIntent(userMsg);
          }

          if (intent === 'CREATE' && currentAttachment) {
              showToast("Analyzing Visual Design...", "info");
              const newGraph = await geminiService.createGraphFromPrompt(userMsg || "Create FSM from this image", currentAttachment);
              if (newGraph) {
                  takeSnapshot(nodes, edges);
                  setNodes(newGraph.nodes);
                  setEdges(newGraph.edges);
                  appendChatMessage('assistant', "I've generated the FSM design based on your image and requirements.");
                  showToast("Design Generated", "success");
              } else {
                  appendChatMessage('assistant', "I couldn't generate a valid graph from the input.");
              }
          } else {
              const response = await geminiService.chatWithAssistant(activeProject.chatHistory, nodes, edges, ghostIssues, userMsg, currentAttachment || undefined);
              appendChatMessage('assistant', response);
          }
      } catch (e) {
          appendChatMessage('assistant', `Error: ${(e as Error).message}`);
      } finally {
          setIsAiLoading(false);
      }
  };

  const handleVisualEvent = useCallback(async (event: VisualEventType, id: string, data?: any) => {
      if (event === 'node_entry') {
         setNodes(nds => nds.map(n => n.id === id ? { ...n, data: { ...n.data, executionState: 'entry', executionLog: data?.code ? data.code.split('\n')[0] : 'Executing...' } } : n));
      } else if (event === 'node_exit') {
         setNodes(nds => nds.map(n => n.id === id ? { ...n, data: { ...n.data, executionState: 'exit', executionLog: data?.code ? data.code.split('\n')[0] : 'Exiting...' } } : n));
      } else if (event === 'node_idle') {
         setNodes(nds => nds.map(n => n.id === id ? { ...n, data: { ...n.data, executionState: 'idle', executionLog: undefined } } : n));
      } else if (event === 'edge_traverse') {
         setEdges(eds => eds.map(e => e.id === id ? { ...e, data: { ...e.data, isTraversing: true } } : e));
         setTimeout(() => {
            setEdges(eds => eds.map(e => e.id === id ? { ...e, data: { ...e.data, isTraversing: false } } : e));
         }, 800); 
      } else if (event === 'guard_check') {
         setEdges(eds => eds.map(e => e.id === id ? { ...e, animated: true } : e));
      } else if (event === 'guard_result') {
         setEdges(eds => eds.map(e => e.id === id ? { ...e, animated: false, data: { ...e.data, guardResult: data?.passed ? 'pass' : 'fail' } } : e));
         setTimeout(() => {
            setEdges(eds => eds.map(e => e.id === id ? { ...e, data: { ...e.data, guardResult: null } } : e));
         }, 1500);
      }
  }, [setNodes, setEdges]);

  const stopSimulation = useCallback(() => {
    if (executorRef.current) executorRef.current.stop();
    setSimStatus(SimulationStatus.IDLE); setActiveStateId(null); setSimHistory([]); setSimTelemetry(null);
    setNodes(nds => nds.map(n => ({ ...n, data: { ...n.data, executionState: undefined, executionLog: undefined, active: false } })));
    setEdges(eds => eds.map(e => ({ ...e, animated: false, data: { ...e.data, isTraversing: false, guardResult: null } })));
    if (autoSimTimerRef.current) clearInterval(autoSimTimerRef.current);
    setAutoSimMode(false);
    showToast('Simulation Stopped', 'info');
  }, [setNodes, setEdges, showToast]);

  const startSimulation = async () => {
    if (simStatus !== SimulationStatus.IDLE) return;
    syncCurrentProject();
    const executor = new FSMExecutor(nodes, edges, (msg, type) => addLog(msg, type), (nodeId, history) => { setActiveStateId(nodeId); setSimHistory(history); }, (ctx) => setSimContext(ctx), (telemetry) => setSimTelemetry(telemetry), handleVisualEvent);
    executor.setSpeed(simSpeed);
    executor.setShadowMode(isShadowMode);
    executorRef.current = executor;
    try { await executor.start(); setSimStatus(SimulationStatus.RUNNING); } catch (e) { addLog(`Start Failed: ${(e as Error).message}`, 'error'); setSimStatus(SimulationStatus.ERROR); }
  };

  const createBlankProject = () => { if (simStatus !== SimulationStatus.IDLE) stopSimulation(); syncCurrentProject(); const newId = `proj_blank_${Date.now()}`; const newProject: FSMProject = { id: newId, name: 'Untitled', description: 'New Project', version: '0.1.0', nodes: [], edges: [], chatHistory: [], updatedAt: Date.now() }; setProjects(prev => [...prev, newProject]); setNodes(newProject.nodes); setEdges(newProject.edges); setActiveProjectId(newId); setSelectedNodeId(null); setSelectedEdgeId(null); clearHistory(); setValidationReport(null); setResourceMetrics(null); showToast('New Blank Project Created', 'success'); };
  const handleCreateProjectFromTemplate = (template: FSMTemplate) => { if (simStatus !== SimulationStatus.IDLE) stopSimulation(); syncCurrentProject(); const newId = `proj_${Date.now()}`; const newProject: FSMProject = { id: newId, name: template.name, description: template.description, version: '0.1.0', nodes: template.nodes, edges: template.edges, chatHistory: [], updatedAt: Date.now() }; setProjects(prev => [...prev, newProject]); setNodes(newProject.nodes); setEdges(newProject.edges); setActiveProjectId(newId); setSelectedNodeId(null); setSelectedEdgeId(null); clearHistory(); setValidationReport(null); setResourceMetrics(null); setShowTemplateBrowser(false); showToast('Template Instantiated', 'success'); };
  const handleImportProject = () => { fileInputRef.current?.click(); };
  const handleImportCpp = () => { cppInputRef.current?.click(); };
  
  const onCppLoad = async (e: React.ChangeEvent<HTMLInputElement>) => { const file = e.target.files?.[0]; if (!file) return; showToast("Reverse Engineering C++...", "info"); try { const text = await file.text(); const graph = await geminiService.reverseEngineerCode(text); if (graph) { if (simStatus !== SimulationStatus.IDLE) stopSimulation(); syncCurrentProject(); const newId = `proj_rev_${Date.now()}`; const newProject: FSMProject = { id: newId, name: file.name.replace('.cpp','').replace('.h',''), description: 'Reverse Engineered from C++ Source', version: '0.1.0', nodes: graph.nodes, edges: graph.edges, chatHistory: [], updatedAt: Date.now() }; setProjects(prev => [...prev, newProject]); setNodes(newProject.nodes); setEdges(newProject.edges); setActiveProjectId(newId); clearHistory(); showToast('Code Successfully Imported', 'success'); } } catch (err) { showToast("Reverse Engineering Failed", "error"); } finally { if (cppInputRef.current) cppInputRef.current.value = ''; } };
  const onFileLoad = async (e: React.ChangeEvent<HTMLInputElement>) => { const file = e.target.files?.[0]; if (!file) return; try { const projData = await fileManager.loadProject(file); if (simStatus !== SimulationStatus.IDLE) stopSimulation(); syncCurrentProject(); const newId = `proj_imp_${Date.now()}`; const newProject: FSMProject = { id: newId, name: projData.name || 'Imported Project', description: projData.description || 'Imported from JSON', version: projData.version || '1.0', nodes: projData.nodes || [], edges: projData.edges || [], chatHistory: projData.chatHistory || [], updatedAt: Date.now() }; setProjects(prev => [...prev, newProject]); setNodes(newProject.nodes); setEdges(newProject.edges); setActiveProjectId(newId); clearHistory(); showToast('Project Imported', 'success'); } catch (err) { showToast("Failed to load project file", "error"); } finally { if(fileInputRef.current) fileInputRef.current.value = ''; } };
  const handleExportCode = async (lang: 'cpp' | 'verilog' | 'python' | 'rust') => { setIsAiLoading(true); try { const code = await geminiService.generateCode(nodes, edges, lang); const ext = lang === 'verilog' ? 'v' : lang === 'python' ? 'py' : lang === 'rust' ? 'rs' : 'cpp'; fileManager.downloadCode(code, `fsm_export.${ext}`); showToast(`${lang.toUpperCase()} Exported`, 'success'); } catch(e) { showToast('Export Failed', 'error'); } finally { setIsAiLoading(false); } };
  const handleGenerateRegisterMap = async () => { showToast('Generating RegMap...', 'info'); const code = await geminiService.generateRegisterMap(nodes); fileManager.downloadCode(code, 'registers.h'); showToast('Header File Exported', 'success'); };
  const handlePowerAnalysis = async () => { showToast('Analyzing Power...', 'info'); const result = await geminiService.optimizeForLowPower(nodes, edges); appendChatMessage('assistant', result); setRightPanelTab('CHAT'); setShowRightPanel(true); showToast('Report in Chat', 'success'); };
  
  const handleSmartLogicGenerate = async () => { 
      if (!selectedNodeId || !smartPrompt) return; 
      const node = nodes.find(n => n.id === selectedNodeId); 
      if (!node) return; 
      setIsAiLoading(true); 
      try { 
          const result = await geminiService.generateNodeScript(node.data.label, node.data.type || 'process', smartPrompt, Object.keys(simContext)); 
          takeSnapshot(nodes, edges); 
          setNodes(nds => nds.map(n => n.id === selectedNodeId ? { ...n, data: { ...n.data, entryAction: result.code, aiReasoning: result.reasoning } } : n)); 
          setSmartPrompt(''); 
          showToast('Logic Generated!', 'success'); 
      } catch (e) { 
          showToast('Generation Failed', 'error'); 
      } finally { 
          setIsAiLoading(false); 
      } 
  };

  const handleConnectDevice = async (mcu: McuDefinition) => { setTargetMcu(mcu); if (mcu.flashMethod === 'WEB_SERIAL') { const connected = await hardwareBridge.requestConnection(); setIsDeviceConnected(connected); if (connected) showToast(`Connected to ${mcu.name}`, 'success'); } else { setIsDeviceConnected(true); showToast(`Target Set: ${mcu.name}`, 'info'); } setShowDeviceManager(false); };
  const handleFlashBoard = async () => { if (!isDeviceConnected) { setShowDeviceManager(true); return; } setIsFlashing(true); setFlashProgress(0); setFlashStatus('Initializing...'); showToast('Starting Flash Sequence...', 'info'); try { const msg = await hardwareBridge.flashDevice(targetMcu, (pct, status) => { setFlashProgress(pct); setFlashStatus(status); }); showToast(msg, 'success'); setFlashStatus('DONE'); } catch (e) { showToast('Flash Failed: ' + (e as Error).message, 'error'); setFlashStatus('ERROR'); } finally { setTimeout(() => setIsFlashing(false), 2000); } };
  const handleAnalyzeDatasheet = async () => { setIsAiLoading(true); try { const result = await geminiService.analyzeDatasheet(datasheetInput); appendChatMessage('assistant', `**Datasheet Analysis Checklist:**\n\n${result}`); setShowDatasheetModal(false); showToast('Checklist added to Chat', 'success'); } catch (e) { showToast('Analysis Failed', 'error'); } finally { setIsAiLoading(false); } };
  const appendChatMessage = (role: 'user' | 'assistant', content: string) => { setProjects(prev => prev.map(p => p.id === activeProjectId ? { ...p, chatHistory: [...p.chatHistory, { id: Date.now().toString(), role, content, timestamp: Date.now() }] } : p)); };
  
  const renderMessageContent = (content: string) => {
    const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
    const parts = [];
    let lastIndex = 0;
    let match;
    while ((match = codeBlockRegex.exec(content)) !== null) {
        if (match.index > lastIndex) { parts.push({ type: 'text', content: content.substring(lastIndex, match.index) }); }
        parts.push({ type: 'code', lang: match[1] || 'text', content: match[2] });
        lastIndex = codeBlockRegex.lastIndex;
    }
    if (lastIndex < content.length) { parts.push({ type: 'text', content: content.substring(lastIndex) }); }
    return parts.map((part, i) => {
        if (part.type === 'code') {
            return ( <div key={i} className="my-3 bg-[#1e1e1e] text-gray-200 p-3 rounded-md border border-gray-700 font-mono text-[11px] overflow-x-auto shadow-inner relative group"> {part.lang && <div className="text-[9px] text-gray-500 uppercase mb-1 font-bold select-none border-b border-gray-700 pb-1 flex justify-between"> <span>{part.lang}</span> <span className="opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer hover:text-white" onClick={() => { navigator.clipboard.writeText(part.content); showToast("Copied code", "info"); }}>COPY</span> </div>} <pre className="whitespace-pre">{part.content}</pre> </div> );
        } else {
            const lines = part.content.split('\n');
            return ( <div key={i} className="whitespace-pre-wrap leading-relaxed text-gray-700"> {lines.map((line, j) => { if (line.startsWith('### ')) { return <h4 key={j} className="font-bold text-neuro-primary mt-2 mb-1 uppercase text-[11px]">{line.replace('### ', '')}</h4>; } if (line.trim().startsWith('- ')) { return <div key={j} className="flex gap-2 ml-2"><span className="text-gray-400">•</span> <span>{formatInline(line.replace('- ', ''))}</span></div>; } if (/^\d+\.\s/.test(line.trim())) { return <div key={j} className="flex gap-2 ml-2"><span className="text-gray-400 font-mono text-[10px]">{line.trim().split('.')[0]}.</span> <span>{formatInline(line.replace(/^\d+\.\s/, ''))}</span></div>; } return <div key={j} className="min-h-[4px]">{formatInline(line)}</div>; })} </div> );
        }
    });
  };
  const formatInline = (text: string) => { return text.split(/(\*\*.*?\*\*|`[^`]+`)/g).map((subPart, j) => { if (subPart.startsWith('**') && subPart.endsWith('**')) { return <strong key={j} className="font-bold text-neuro-primary">{subPart.slice(2, -2)}</strong>; } if (subPart.startsWith('`') && subPart.endsWith('`')) { return <code key={j} className="bg-gray-100 text-purple-700 px-1 py-0.5 rounded text-[90%] font-mono border border-gray-200 mx-0.5">{subPart.slice(1, -1)}</code>; } return subPart; }); };
  const handleDeleteSelected = useCallback(() => { if (selectedNodeIds.length > 0) { setNodes(nds => nds.filter(n => !selectedNodeIds.includes(n.id))); setEdges(eds => eds.filter(e => !selectedNodeIds.includes(e.source) && !selectedNodeIds.includes(e.target))); setSelectedNodeId(null); setSelectedNodeIds([]); showToast('Selection Deleted', 'info'); } if (selectedEdgeId) { setEdges(eds => eds.filter(e => e.id !== selectedEdgeId)); setSelectedEdgeId(null); showToast('Edge Deleted', 'info'); } }, [selectedNodeIds, selectedEdgeId, setNodes, setEdges, showToast]);
  const handleGroupSelection = useCallback(() => { if (selectedNodeIds.length < 2) { showToast("Select at least 2 nodes to group", "warning"); return; } const selectedNodes = nodes.filter(n => selectedNodeIds.includes(n.id)); const minX = Math.min(...selectedNodes.map(n => n.position.x)); const minY = Math.min(...selectedNodes.map(n => n.position.y)); const maxX = Math.max(...selectedNodes.map(n => n.position.x + (n.width || 150))); const maxY = Math.max(...selectedNodes.map(n => n.position.y + (n.height || 100))); const padding = 40; const groupNode: Node = { id: `group_${Date.now()}`, type: 'group', position: { x: minX - padding, y: minY - padding }, style: { width: maxX - minX + padding*2, height: maxY - minY + padding*2 }, data: { label: 'New Superstate' } }; takeSnapshot(nodes, edges); setNodes(nds => [groupNode, ...nds]); showToast("Nodes Grouped", "success"); }, [nodes, selectedNodeIds, takeSnapshot, setNodes, showToast]);
  const applyLayout = useCallback((template: WorkspaceTemplate) => { setActiveLayout(template); setShowLayoutMenu(false); setShowLeftPanel(false); setShowRightPanel(false); setShowBottomPanel(false); setShowIOPanel(false); setShowDiagnostics(false); if (template === 'ARCHITECT') { setShowLeftPanel(true); setShowRightPanel(true); setRightPanelTab('PROPS'); } else if (template === 'ENGINEER') { setShowLeftPanel(true); setShowRightPanel(true); setShowBottomPanel(true); setActiveBottomTab('SERIAL'); setRightPanelTab('DEBUG'); } else if (template === 'HARDWARE_LAB') { setShowRightPanel(true); setShowBottomPanel(true); setShowIOPanel(true); setShowDiagnostics(true); setActiveBottomTab('LOGIC'); setRightPanelTab('DEBUG'); } else if (template === 'DEBUG_FOCUS') { setShowRightPanel(true); setShowBottomPanel(true); setActiveBottomTab('OUTPUT'); setRightPanelTab('DEBUG'); } else if (template === 'HACKER') { setShowBottomPanel(true); setActiveBottomTab('OUTPUT'); } else if (template === 'FULL_SUITE') { setShowLeftPanel(true); setShowRightPanel(true); setShowBottomPanel(true); } else if (template === 'AI_PAIR') { setShowRightPanel(true); setIsCompanionMode(true); setRightPanelTab('CHAT'); } if (template !== 'AI_PAIR') setIsCompanionMode(false); showToast(`Layout: ${template.replace('_', ' ')}`, 'info'); }, [showToast]);
  const switchProject = useCallback((id: string) => { if (simStatus !== SimulationStatus.IDLE) stopSimulation(); syncCurrentProject(); setActiveProjectId(id); clearHistory(); setValidationReport(null); setResourceMetrics(null); setSelectedNodeId(null); setSelectedEdgeId(null); }, [simStatus, stopSimulation, syncCurrentProject, setActiveProjectId, clearHistory]);
  const closeProject = useCallback((e: React.MouseEvent, id: string) => { e.stopPropagation(); if (projects.length <= 1) { showToast("Cannot close the last project.", "warning"); return; } const newProjects = projects.filter(p => p.id !== id); setProjects(newProjects); if (activeProjectId === id && newProjects.length > 0) { setActiveProjectId(newProjects[0].id); clearHistory(); } }, [projects, activeProjectId, setProjects, setActiveProjectId, clearHistory, showToast]);
  
  const handleAddNodeFromContext = useCallback((type: string, x: number, y: number) => { 
      const position = reactFlowInstance.screenToFlowPosition({ x, y });
      
      let newNode: Node;
      if (type === 'code_analysis') {
          // Special case for pre-configured Code Analysis node
          newNode = { 
            id: `node_${Date.now()}`, 
            type: 'process', 
            position, 
            data: { 
                label: 'CODE_ANALYSIS', 
                type: 'process',
                tags: ['analysis', 'static'],
                entryAction: `// --- CODE ANALYSIS ACTION ---\n// Placeholder for analysis logic\n\nconst snippet = ctx.codeBuffer || "";\nif (snippet.includes("unsafe")) {\n  console.warn("Unsafe pattern detected");\n  dispatch("RISK_HIGH");\n} else {\n  dispatch("ANALYSIS_PASS");\n}`,
                exitAction: '' 
            }, 
          };
      } else {
          newNode = { 
            id: `node_${Date.now()}`, 
            type: type === 'group' ? 'group' : (['input','output','process','decision','hardware','error'].includes(type) ? type : 'process'), 
            position, 
            data: { 
               label: `${type.toUpperCase()}_${Math.floor(Math.random()*100)}`, 
               type: type as any, 
               entryAction: '', 
               exitAction: '' 
            } 
         };
      }

      takeSnapshot(nodes, edges); 
      setNodes((nds) => nds.concat(newNode)); 
      setContextMenu(null); 
  }, [reactFlowInstance, nodes, edges, takeSnapshot, setNodes]);

  const onDrop = useCallback((event: React.DragEvent) => { event.preventDefault(); const type = event.dataTransfer.getData('application/reactflow'); if (!type) return; const position = reactFlowInstance.screenToFlowPosition({ x: event.clientX, y: event.clientY, }); const newNode: Node = { id: `node_${Date.now()}`, type, position, data: { label: `${type.toUpperCase()}_${Math.floor(Math.random()*100)}`, type: type as any, entryAction: '', exitAction: '' }, }; takeSnapshot(nodes, edges); setNodes((nds) => nds.concat(newNode)); }, [reactFlowInstance, nodes, edges, takeSnapshot, setNodes]);
  const onDragOver = useCallback((event: React.DragEvent) => { event.preventDefault(); event.dataTransfer.dropEffect = 'move'; }, []);
  const onDragStart = useCallback((event: React.DragEvent, nodeType: string) => { event.dataTransfer.setData('application/reactflow', nodeType); event.dataTransfer.effectAllowed = 'move'; }, []);
  const onPaneClick = useCallback(() => { setContextMenu(null); setShowLayoutMenu(false); setActiveMenu(null); }, []);
  const onPaneContextMenu = useCallback((event: React.MouseEvent) => { event.preventDefault(); setContextMenu({ top: event.clientY, left: event.clientX }); }, []);
  const onSelectionChange = useCallback(({ nodes: selectedNodes, edges: selectedEdges }: { nodes: Node[], edges: Edge[] }) => { setSelectedNodeIds(selectedNodes.map(n => n.id)); setSelectedNodeId(selectedNodes.length === 1 ? selectedNodes[0].id : null); setSelectedEdgeId(selectedEdges.length === 1 ? selectedEdges[0].id : null); if (selectedNodes.length === 1) { setRightPanelTab('PROPS'); setShowRightPanel(true); } }, []);
  const onConnect = useCallback((params: Connection) => { takeSnapshot(nodes, edges); setEdges((eds) => addEdge({ ...params, type: 'retro', animated: false }, eds)); }, [nodes, edges, takeSnapshot, setEdges]);
  const onNodesChangeWithHistory = useCallback((changes: any) => { onNodesChange(changes); }, [onNodesChange]);
  const onEdgesChangeWithHistory = useCallback((changes: any) => { onEdgesChange(changes); }, [onEdgesChange]);

  const handleAutoFix = async () => {
    if (ghostIssues.length === 0) return;
    setIsAiLoading(true);
    showToast("Attempting Auto-Fix...", "info");
    try {
        const issuesDesc = JSON.stringify(ghostIssues.map(i => i.title + ": " + i.description));
        const newGraph = await geminiService.modifyGraph(nodes, edges, `Fix the following detected issues: ${issuesDesc}`, ghostIssues);
        if (newGraph) {
            takeSnapshot(nodes, edges);
            setNodes(newGraph.nodes);
            setEdges(newGraph.edges);
            showToast("Auto-Fix Applied", "success");
            setGhostIssues(GhostEngineer.analyze(newGraph.nodes, newGraph.edges));
        }
    } catch (e) {
        showToast("Auto-Fix Failed: " + (e as Error).message, "error");
    } finally {
        setIsAiLoading(false);
    }
  };

  const handleRunValidationWrapper = async () => {
      setIsValidating(true);
      try {
          const report = await geminiService.generateValidationReport(nodes, edges);
          setValidationReport(report);
          showToast("Validation Report Ready", "success");
      } catch(e) {
          showToast("Validation Failed", "error");
      } finally {
          setIsValidating(false);
      }
  };

  const handleEstimateResourcesWrapper = async () => {
      setIsEstimating(true);
      try {
          const metrics = await geminiService.estimateResources(nodes, edges);
          setResourceMetrics(metrics);
          showToast("Estimation Complete", "success");
      } catch(e) {
          showToast("Estimation Failed", "error");
      } finally {
          setIsEstimating(false);
      }
  };

  const onNodeContextMenu = useCallback((event: React.MouseEvent, node: Node) => {
      event.preventDefault();
      // Select the node programmatically so the context menu options apply to it
      setNodes((nds) => nds.map(n => ({ ...n, selected: n.id === node.id })));
      // Update local selection state immediately to ensure UI responsiveness
      setSelectedNodeId(node.id);
      setSelectedNodeIds([node.id]);
      setContextMenu({ top: event.clientY, left: event.clientX });
  }, [setNodes]);

  useEffect(() => {
     const issues = GhostEngineer.analyze(nodes, edges);
     setGhostIssues(issues);
  }, [nodes, edges]);

  const MENU_ITEMS = useMemo(() => ({
    File: [
      { label: 'New Project', icon: FilePlus, action: createBlankProject, shortcut: 'Alt+N' },
      { label: 'Open JSON...', icon: FolderOpen, action: handleImportProject, shortcut: 'Ctrl+O' },
      { label: 'Save Project', icon: Save, action: () => fileManager.saveProject(activeProject), shortcut: 'Ctrl+S' },
      { separator: true },
      { label: 'Import C++', icon: FileCode, action: handleImportCpp },
      { label: 'Export C++', icon: Code2, action: () => handleExportCode('cpp') },
      { label: 'Export Verilog', icon: Cpu, action: () => handleExportCode('verilog') },
    ],
    Edit: [
      { label: 'Undo', icon: Undo, action: () => undo(nodes, edges), disabled: !canUndo, shortcut: 'Ctrl+Z' },
      { label: 'Redo', icon: Redo, action: () => redo(nodes, edges), disabled: !canRedo, shortcut: 'Ctrl+Y' },
      { separator: true },
      { label: 'Delete Selected', icon: Trash2, action: handleDeleteSelected, shortcut: 'Del' },
      { label: 'Group Selection', icon: Group, action: handleGroupSelection, shortcut: 'Ctrl+G' },
    ],
    View: [
      { label: 'Architect Layout', icon: Layout, action: () => applyLayout('ARCHITECT'), shortcut: 'Alt+2' },
      { label: 'Engineer Layout', icon: Cpu, action: () => applyLayout('ENGINEER'), shortcut: 'Alt+3' },
      { label: 'Hardware Lab', icon: Wrench, action: () => applyLayout('HARDWARE_LAB'), shortcut: 'Alt+4' },
      { label: 'Zen Mode', icon: Maximize, action: () => applyLayout('ZEN'), shortcut: 'Alt+5' },
      { separator: true },
      { label: 'Toggle Diagnostics', icon: Monitor, action: () => setShowDiagnostics(!showDiagnostics), checked: showDiagnostics },
    ],
    Help: [
      { label: 'Documentation', icon: Book, action: () => setShowDocsModal(true) },
      { label: 'About', icon: Info, action: () => setShowAboutModal(true) },
    ]
  }), [createBlankProject, handleImportProject, activeProject, handleImportCpp, handleExportCode, undo, canUndo, nodes, edges, redo, canRedo, handleDeleteSelected, handleGroupSelection, applyLayout, showDiagnostics]);

  useShortcuts([
    { key: '1', alt: true, action: () => applyLayout('FULL_SUITE') },
    { key: '2', alt: true, action: () => applyLayout('ARCHITECT') },
    { key: '3', alt: true, action: () => applyLayout('ENGINEER') },
    { key: '4', alt: true, action: () => applyLayout('HARDWARE_LAB') },
    { key: '5', alt: true, action: () => applyLayout('ZEN') },
    { key: 'g', ctrl: true, action: () => handleGroupSelection() },
  ]);

  return (
    <div className="flex flex-col h-[100dvh] bg-neuro-bg text-neuro-primary font-mono text-xs overflow-hidden min-h-0">
      <input type="file" ref={fileInputRef} className="hidden" accept=".json" onChange={onFileLoad} />
      <input type="file" ref={cppInputRef} className="hidden" accept=".cpp,.h,.c" onChange={onCppLoad} />
      <input type="file" ref={chatFileRef} className="hidden" accept="image/*,video/*,application/pdf" onChange={handleAttachmentSelect} />
      
      {activeMenu && <div className="fixed inset-0 z-40" onClick={() => setActiveMenu(null)}></div>}

      <div className="bg-gray-100 border-b border-neuro-dim px-2 flex items-center h-8 select-none shrink-0 relative z-50">
         <div className="flex items-center gap-1">
            <span className="font-bold mr-4 text-sm tracking-tight text-neuro-primary flex items-center gap-2"><CircuitBoard size={16}/> NeuroState</span>
            {Object.keys(MENU_ITEMS).map(m => (
               <div key={m} className="relative">
                  <button className={clsx("px-3 py-1 hover:bg-gray-200 text-gray-700 rounded-sm font-medium transition-colors", activeMenu === m && "bg-gray-200 text-neuro-primary")} onClick={() => setActiveMenu(activeMenu === m ? null : m)}>{m}</button>
                  {activeMenu === m && (
                     <div className="absolute top-full left-0 mt-1 bg-white border border-neuro-dim shadow-xl rounded-sm min-w-[220px] py-1 animate-in fade-in zoom-in-95 duration-75 flex flex-col">
                        {(MENU_ITEMS as any)[m].map((item: any, i: number) => (
                           item.separator ? <div key={i} className="h-px bg-gray-100 my-1 mx-2"></div> :
                           <button key={i} onClick={() => { item.action(); setActiveMenu(null); }} disabled={item.disabled} className="px-4 py-2 text-left hover:bg-gray-50 flex items-center gap-3 text-gray-700 disabled:opacity-40 disabled:cursor-not-allowed group">
                              <span className="text-gray-400 group-hover:text-neuro-primary">{item.icon && <item.icon size={14}/>}</span>
                              <span className="flex-1">{item.label}</span>
                              {item.checked !== undefined && (item.checked ? <CheckCircle size={12} className="text-neuro-primary"/> : <div className="w-3"/>)}
                              {item.shortcut && <span className="text-[9px] text-gray-400 ml-4 font-sans border border-gray-200 px-1 rounded bg-gray-50">{item.shortcut}</span>}
                           </button>
                        ))}
                     </div>
                  )}
               </div>
            ))}
         </div>
         <div className="flex-1"></div>
      </div>

      <div className="h-10 border-b border-neuro-dim bg-white flex items-center px-4 gap-2 justify-between shrink-0 z-40 shadow-sm relative">
        <div className="flex items-center gap-2">
           <Button onClick={createBlankProject} tooltip="New Blank Project"><FilePlus size={14}/></Button>
           <Button onClick={() => setShowTemplateBrowser(true)} tooltip="Browse Templates"><Grid size={14}/></Button>
           <Button onClick={() => fileManager.saveProject(activeProject)} tooltip="Save Project (Ctrl+S)"><Save size={14}/></Button>
           <div className="w-px h-6 bg-gray-200 mx-1"></div>
           <Button onClick={() => undo(nodes, edges)} disabled={!canUndo} tooltip="Undo (Ctrl+Z)"><Undo size={14}/></Button>
           <Button onClick={() => redo(nodes, edges)} disabled={!canRedo} tooltip="Redo (Ctrl+Y)"><Redo size={14}/></Button>
           <div className="w-px h-6 bg-gray-200 mx-1"></div>
           <Button onClick={() => setShowLayoutMenu(!showLayoutMenu)} tooltip="Layouts"><LayoutTemplate size={14}/></Button>
           <Button onClick={() => setShowBottomPanel(!showBottomPanel)} variant={showBottomPanel ? 'primary' : 'ghost'} tooltip="Toggle Bottom Panel (Logs, Validation)"><PanelBottomOpen size={14}/></Button>
           <Button onClick={() => setShowDiagnostics(!showDiagnostics)} variant={showDiagnostics ? 'primary' : 'ghost'} tooltip="Toggle Diagnostics"><Monitor size={14}/></Button>
           <Button onClick={() => setShowIOPanel(!showIOPanel)} variant={showIOPanel ? 'primary' : 'ghost'} tooltip="Toggle IO Panel"><ToggleLeft size={14}/></Button>
        </div>

        <div className="flex-1 flex justify-center overflow-hidden px-4">
           <div className="flex items-end gap-1 h-full pt-1 overflow-x-auto custom-scrollbar">
              {projects.map(p => (
                 <div key={p.id} onClick={() => switchProject(p.id)} className={clsx("px-3 py-1.5 border-t border-l border-r rounded-t-sm cursor-pointer flex items-center gap-2 min-w-[120px] max-w-[200px] transition-all", p.id === activeProjectId ? "bg-neuro-bg border-neuro-dim border-b-neuro-bg -mb-px font-bold z-10 text-neuro-primary" : "bg-gray-50 border-gray-200 text-gray-400 hover:bg-gray-100 hover:text-gray-600")}>
                    <span className="truncate flex-1">{p.name}</span>
                    <button onClick={(e) => closeProject(e, p.id)} className="hover:text-red-500 rounded-full p-0.5 hover:bg-red-50"><X size={10}/></button>
                 </div>
              ))}
           </div>
        </div>

        <div className="flex items-center gap-2">
           {!isCompanionMode && isStandbyMode && (
              <button 
                 onClick={() => setIsStandbyMode(!isStandbyMode)}
                 className={clsx("flex items-center gap-1 text-[9px] font-bold px-2 py-1 rounded border transition-colors hover:bg-gray-100", 
                    isWakeWordActive ? "bg-green-50 text-green-600 border-green-200 animate-pulse" : "bg-gray-50 text-gray-400 border-gray-200"
                 )} 
                 title="Click to toggle Standby Mode"
              >
                 <Ear size={10} className={isWakeWordActive ? "animate-bounce" : ""} />
                 {isWakeWordActive ? 'LISTENING' : 'STANDBY'}
              </button>
           )}
           <Button onClick={() => setShowVeoModal(true)} variant={showVeoModal ? 'primary' : 'ghost'} tooltip="Visualize with Veo (Video)">
              <Film size={14} className="text-purple-600"/>
           </Button>
           <Button onClick={() => { setIsCompanionMode(!isCompanionMode); setRightPanelTab('CHAT'); setShowRightPanel(true); }} variant={isCompanionMode ? 'primary' : 'ghost'} tooltip="Neo AI Companion (Live Voice)">
              <Waves size={14} className={isCompanionMode ? "text-purple-500 animate-pulse" : ""}/>
           </Button>
           <div className="flex items-center bg-gray-100 rounded-md px-1 border border-gray-200" title="Shadow Mode (Hardware-in-Loop)">
              <button 
                 onClick={() => setIsShadowMode(!isShadowMode)} 
                 className={clsx("p-1.5 rounded transition-all", isShadowMode ? "bg-neuro-primary text-white shadow-sm" : "text-gray-400 hover:text-gray-600")}
              >
                 <Activity size={14}/>
              </button>
           </div>
           <Button onClick={() => setShowDeviceManager(true)} variant={isDeviceConnected ? 'primary' : 'ghost'} tooltip="Device Manager"><CircuitBoard size={14}/></Button>
           <Button onClick={handleFlashBoard} disabled={isFlashing} tooltip="Flash Firmware"><Zap size={14} className={isFlashing ? "fill-yellow-400 text-yellow-500 animate-pulse" : ""}/></Button>
           <div className="w-px h-6 bg-gray-200 mx-1"></div>
           {simStatus === SimulationStatus.IDLE ? (
             <Button onClick={startSimulation} className="border-green-600 text-green-700 bg-green-50 hover:bg-green-100 shadow-sm" tooltip="Start Simulation"><Play size={14} fill="currentColor"/> {isShadowMode ? 'CONNECT' : 'SIMULATE'}</Button>
           ) : (
             <Button onClick={stopSimulation} className="border-red-600 text-red-600 bg-red-50 hover:bg-red-100 shadow-sm"><Square size={14} fill="currentColor"/> STOP</Button>
           )}
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden relative min-h-0">
        {showLeftPanel && (
          <div className="w-16 border-r border-neuro-dim bg-white flex flex-col items-center py-4 gap-4 z-10 shadow-sm shrink-0 overflow-y-auto custom-scrollbar">
             {['input', 'process', 'decision', 'output', 'error', 'hardware', 'uart', 'listener', 'interrupt', 'timer', 'peripheral', 'queue', 'mutex', 'critical', 'math', 'wireless', 'storage', 'logger', 'display', 'network', 'sensor'].map(type => (
               <div key={type} draggable onDragStart={(e) => onDragStart(e, type)} className="w-10 h-10 border border-neuro-dim bg-white hover:border-neuro-primary hover:shadow-md hover:scale-110 transition-all flex items-center justify-center cursor-grab active:cursor-grabbing rounded-sm group relative shrink-0">
                  {/* Icon Rendering Logic */}
                  {type==='input'?<Play size={18} fill="currentColor" className="text-neuro-primary"/> :
                   type==='output'?<CheckCircle size={18} className="text-neuro-accent"/> :
                   type==='error'?<AlertTriangle size={18} className="text-red-500"/> :
                   type==='listener'?<Ear size={18} className="text-indigo-500"/> :
                   type==='decision'?<Split size={18} className="text-amber-500"/> :
                   type==='hardware'?<CircuitBoard size={18} className="text-cyan-600"/> :
                   type==='uart'?<Cable size={18} className="text-purple-600"/> :
                   type==='interrupt'?<Zap size={18} className="text-purple-600"/> :
                   type==='timer'?<Hourglass size={18} className="text-orange-500"/> :
                   type==='peripheral'?<Cpu size={18} className="text-teal-500"/> :
                   type==='queue'?<Layers size={18} className="text-pink-500"/> :
                   type==='mutex'?<Lock size={18} className="text-slate-500"/> :
                   type==='critical'?<ShieldAlert size={18} className="text-rose-600"/> :
                   type==='math'?<Calculator size={18} className="text-blue-400"/> :
                   type==='wireless'?<Wifi size={18} className="text-sky-500"/> :
                   type==='storage'?<Database size={18} className="text-amber-600"/> :
                   type==='logger'?<FileText size={18} className="text-gray-500"/> :
                   type==='display'?<Monitor size={18} className="text-fuchsia-500"/> :
                   type==='network'?<Globe size={18} className="text-indigo-600"/> :
                   type==='sensor'?<Thermometer size={18} className="text-emerald-500"/> :
                   <Square size={18} className="text-gray-500"/>}
                  <div className="absolute left-full ml-2 bg-neuro-primary text-white text-[9px] px-2 py-1 rounded opacity-0 group-hover:opacity-100 pointer-events-none whitespace-nowrap z-50 capitalize font-bold tracking-wider shadow-lg transform translate-x-2 group-hover:translate-x-0 transition-all">{type}</div>
               </div>
             ))}
          </div>
        )}

        <div className="flex-1 relative bg-neuro-bg" onContextMenu={onPaneContextMenu} onClick={onPaneClick} onDragOver={onDragOver} onDrop={onDrop}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChangeWithHistory}
            onEdgesChange={onEdgesChangeWithHistory}
            onConnect={onConnect}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            onNodeClick={(_, node) => { setSelectedNodeId(node.id); setShowRightPanel(true); }}
            onEdgeClick={(_, edge) => { setSelectedEdgeId(edge.id); setShowRightPanel(true); }}
            onSelectionChange={onSelectionChange}
            onNodeContextMenu={onNodeContextMenu}
            fitView
            minZoom={0.1}
            snapToGrid={true}
            snapGrid={[20, 20]}
            connectionLineStyle={{ stroke: '#111827', strokeWidth: 1.5 }}
          >
            <Background gap={20} size={1} color="#e5e7eb" variant={BackgroundVariant.Dots} />
            <Controls className="!bg-white !border-neuro-dim !shadow-sm !rounded-sm !m-4" />
            <MiniMap className="!bg-white !border-neuro-dim !shadow-sm !rounded-sm !m-4" nodeColor={() => '#e5e7eb'} maskColor="rgba(240, 240, 240, 0.6)" />
          </ReactFlow>

          {isCompanionMode && (
             <CompanionOrb 
                state={agentState} 
                onMute={() => setIsCompanionMuted(!isCompanionMuted)}
                muted={isCompanionMuted}
             />
          )}

          {showDiagnostics && <div className="absolute bottom-4 left-4 z-40 animate-in slide-in-from-bottom-5 duration-300"><DiagnosticPanel state={halSnapshot} /></div>}
          {showIOPanel && (
             <div className="absolute top-4 right-4 z-40 animate-in slide-in-from-right-5 duration-300 bg-neuro-surface border border-neuro-primary shadow-hard p-0 flex flex-col w-[200px]">
                <div className="bg-gray-100 p-2 text-[10px] font-bold border-b border-neuro-dim flex justify-between">
                   <span>VIRTUAL I/O BOARD</span>
                   <Monitor size={12}/>
                </div>
                <div className="p-3 grid grid-cols-2 gap-3 max-h-[300px] overflow-y-auto custom-scrollbar">
                   {Object.keys(simContext || {}).filter(k=>k.startsWith('led_')||k.startsWith('btn_')||k.startsWith('dsp_')).length === 0 && (
                      <div className="col-span-2 text-center text-gray-400 italic text-[10px] py-4">
                         No IO variables (led_*, btn_*, dsp_*) detected.
                      </div>
                   )}
                   {Object.entries(simContext || {}).filter(([k]) => k.startsWith('led_')).map(([k, v]) => (
                      <VirtualLED key={k} label={k.replace('led_','')} active={!!v} color={k.includes('red')?'red':k.includes('green')?'green':k.includes('blue')?'blue':'yellow'} />
                   ))}
                   {Object.entries(simContext || {}).filter(([k]) => k.startsWith('btn_')).map(([k, v]) => (
                      <VirtualSwitch key={k} label={k.replace('btn_','')} active={!!v} onChange={(val) => {
                         setSimContext(prev => ({...prev, [k]: val}));
                         if(executorRef.current) executorRef.current.triggerEvent(val ? 'BTN_PRESS' : 'BTN_RELEASE');
                      }} />
                   ))}
                   {Object.entries(simContext || {}).filter(([k]) => k.startsWith('dsp_')).map(([k, v]) => (
                      <VirtualDisplay key={k} label={k.replace('dsp_','')} value={v as any} />
                   ))}
                </div>
             </div>
          )}
          {showLayoutMenu && <LayoutMenu active={activeLayout} onClose={() => setShowLayoutMenu(false)} onSelect={applyLayout} />}
          {contextMenu && (
             <ContextMenu 
                top={contextMenu.top} 
                left={contextMenu.left} 
                onClose={() => setContextMenu(null)} 
                onAddNode={handleAddNodeFromContext} 
                onGroupSelected={selectedNodeIds.length > 1 ? handleGroupSelection : undefined}
                onAiDefine={selectedNodeIds.length === 1 ? () => {
                   setRightPanelTab('PROPS');
                   setShowRightPanel(true);
                   setContextMenu(null);
                   setTimeout(() => smartPromptInputRef.current?.focus(), 100);
                } : undefined}
             />
          )}
        </div>

        {showRightPanel && (
          <div className="w-80 border-l border-neuro-dim bg-white flex flex-col z-20 shadow-xl shrink-0">
             <div className="flex border-b border-neuro-dim bg-gray-50">
               <button 
                  onClick={() => setRightPanelTab('PROPS')} 
                  className={clsx("flex-1 py-2 text-[10px] font-bold border-r border-neuro-dim hover:bg-white transition-colors flex justify-center items-center gap-2", rightPanelTab === 'PROPS' ? "bg-white border-b-2 border-b-neuro-primary text-neuro-primary" : "text-gray-400")}
                  title="Properties"
               >
                  <Edit3 size={12}/> PROPS
               </button>
               <button 
                  onClick={() => setRightPanelTab('DEBUG')} 
                  className={clsx("flex-1 py-2 text-[10px] font-bold border-r border-neuro-dim hover:bg-white transition-colors flex justify-center items-center gap-2", rightPanelTab === 'DEBUG' ? "bg-white border-b-2 border-b-neuro-primary text-neuro-primary" : "text-gray-400")}
                  title="Debugger"
               >
                  <Cpu size={12}/> DEBUG
               </button>
               <button 
                  onClick={() => setRightPanelTab('CHAT')} 
                  className={clsx("flex-1 py-2 text-[10px] font-bold hover:bg-white transition-colors flex justify-center items-center gap-2", rightPanelTab === 'CHAT' ? "bg-white border-b-2 border-b-neuro-primary text-neuro-primary" : "text-gray-400", isCompanionMode && "animate-pulse text-purple-600")}
                  title="Neo AI Chat"
               >
                  <Waves size={12}/> NEO AI
               </button>
             </div>

             <div className="flex-1 overflow-hidden relative">
               {rightPanelTab === 'DEBUG' && (
                  <Panel title="SIMULATION DEBUGGER" className="h-full border-0">
                     <div className="p-4 space-y-6">
                        {simStatus === SimulationStatus.IDLE && (
                           <div className="text-center text-gray-400 p-4 border border-dashed rounded-sm">
                              <Play size={24} className="mx-auto mb-2 opacity-50"/>
                              <div className="text-xs">Simulation Idle</div>
                              <Button onClick={startSimulation} className="mt-2 w-full text-[10px]">START SIM</Button>
                           </div>
                        )}
                        {simStatus !== SimulationStatus.IDLE && (
                           <>
                              <div className={clsx("p-3 border rounded-sm", isShadowMode ? "bg-purple-50 border-purple-200" : "bg-green-50 border-green-200")}>
                                 <div className={clsx("text-[10px] font-bold mb-1", isShadowMode ? "text-purple-800" : "text-green-800")}>
                                    {isShadowMode ? "DIGITAL TWIN (HIL)" : "CURRENT STATE"}
                                 </div>
                                 <div className={clsx("text-xl font-bold font-mono", isShadowMode ? "text-purple-700" : "text-green-700")}>
                                    {nodes.find(n=>n.id===activeStateId)?.data.label || 'Unknown'}
                                 </div>
                                 <div className={clsx("text-[10px] mt-1 flex gap-2", isShadowMode ? "text-purple-600" : "text-green-600")}>
                                    <span>Transitions: {simHistory.length}</span>
                                    <span>Time: {((Date.now() - (executorRef.current as any)?.startTime)/1000).toFixed(1)}s</span>
                                 </div>
                              </div>
                              
                              {simTelemetry && (
                                 <div className="grid grid-cols-2 gap-2">
                                    <MetricCard label="CPU Load" value={Math.round(simTelemetry.cpuLoad)} unit="%" />
                                    <MetricCard label="Power" value={Math.round(simTelemetry.powerDrawMW)} unit="mW" />
                                    <div className="col-span-2">
                                       <ProgressBar value={simTelemetry.ramUsageBytes} max={8192} label="RAM Usage (8KB)" color="bg-purple-500"/>
                                    </div>
                                 </div>
                              )}
                              
                              <div>
                                 <Label>Active Variables (Context)</Label>
                                 <div className="border border-neuro-dim rounded-sm overflow-hidden text-xs">
                                    <table className="w-full">
                                       <tbody className="bg-gray-50">
                                          {Object.entries(simContext || {}).length === 0 && <tr><td className="p-2 text-gray-400 italic text-center">No variables</td></tr>}
                                          {Object.entries(simContext || {}).map(([k, v]) => (
                                             <tr key={k} className="border-b border-neuro-dim last:border-0">
                                                <td className="p-2 font-bold text-gray-600 border-r border-neuro-dim w-1/3">{k}</td>
                                                <td className="p-2 font-mono text-neuro-primary bg-white">{typeof v === 'object' ? JSON.stringify(v) : String(v)}</td>
                                             </tr>
                                          ))}
                                       </tbody>
                                    </table>
                                 </div>
                              </div>
                              <div className="p-3 bg-gray-50 border border-neuro-dim rounded-sm">
                                 <div className="flex justify-between items-center mb-2">
                                    <Label>Simulation Control</Label>
                                    <span className="text-[9px] font-bold text-neuro-accent">{autoSimMode ? 'AUTO' : 'MANUAL'}</span>
                                 </div>
                                 <div className="flex gap-2 mb-3">
                                    <Button className="flex-1" onClick={() => { setAutoSimMode(!autoSimMode); }} variant={autoSimMode ? 'primary' : 'ghost'} tooltip="Toggle Auto-Step">
                                       {autoSimMode ? <Pause size={12}/> : <FastForward size={12}/>} {autoSimMode ? 'PAUSE' : 'AUTO-RUN'}
                                    </Button>
                                    <Button onClick={() => { if(executorRef.current) executorRef.current.triggerEvent('TICK'); }} tooltip="Manual Tick"><Clock size={12}/></Button>
                                 </div>
                                 {autoSimMode && (
                                    <div className="px-1">
                                       <input type="range" min="100" max="2000" step="100" value={simSpeed} onChange={(e) => setSimSpeed(Number(e.target.value))} className="w-full accent-neuro-primary h-1 bg-gray-300 rounded-lg appearance-none cursor-pointer"/>
                                       <div className="flex justify-between text-[9px] text-gray-400 mt-1"><span>Fast (100ms)</span><span>Slow (2s)</span></div>
                                    </div>
                                 )}
                              </div>
                           </>
                        )}
                     </div>
                  </Panel>
               )}

               {rightPanelTab === 'PROPS' && (
                  <Panel title="NODE PROPERTIES" className="h-full border-0">
                     <div className="p-4 space-y-4">
                        {!selectedNode && (
                           <div className="text-center text-gray-400 p-8">
                              <MousePointerClick size={32} className="mx-auto mb-2 opacity-30"/>
                              <div>Select a node to edit properties</div>
                           </div>
                        )}
                        {selectedNode && (
                           <>
                              <div>
                                 <Label>Label</Label>
                                 <Input value={selectedNode.data.label} onChange={(e) => {
                                    const val = e.target.value;
                                    setNodes(nds => nds.map(n => n.id === selectedNodeId ? { ...n, data: { ...n.data, label: val } } : n));
                                 }} />
                              </div>
                              
                              <div>
                                 <Label>Type</Label>
                                 <select className="w-full bg-white border border-neuro-dim text-xs px-2 py-2 outline-none font-mono" value={selectedNode.data.type} onChange={(e) => {
                                    const val = e.target.value;
                                    setNodes(nds => nds.map(n => n.id === selectedNodeId ? { ...n, type: val as any, data: { ...n.data, type: val as any } } : n));
                                 }}>
                                    {['input', 'process', 'decision', 'output', 'error', 'listener', 'hardware', 'uart', 'interrupt', 'timer', 'peripheral'].map(t => <option key={t} value={t}>{t.toUpperCase()}</option>)}
                                 </select>
                              </div>

                              <div className="p-3 bg-indigo-50 border border-indigo-100 rounded-sm shadow-sm relative">
                                 <div className="flex justify-between items-center mb-2">
                                    <div className="text-[10px] font-bold text-indigo-800 flex items-center gap-1"><Sparkles size={10}/> DEFINE NODE LOGIC</div>
                                 </div>
                                 <textarea 
                                    ref={smartPromptInputRef}
                                    className="w-full h-20 text-xs p-2 border border-indigo-200 rounded-sm outline-none resize-none mb-2 font-mono text-indigo-900 placeholder:text-indigo-300 focus:border-indigo-400 transition-colors" 
                                    placeholder="e.g. 'Read ADC on pin 1, check if > 2000, then switch to High State'"
                                    value={smartPrompt}
                                    onChange={(e) => setSmartPrompt(e.target.value)}
                                 />
                                 <Button onClick={handleSmartLogicGenerate} disabled={isAiLoading || !smartPrompt} className="w-full border-indigo-300 text-indigo-700 bg-white hover:bg-indigo-50 shadow-sm">
                                    {isAiLoading ? <><Loader2 size={10} className="animate-spin"/> GENERATING...</> : 'GENERATE SCRIPT'}
                                 </Button>
                              </div>

                              {selectedNode.data.aiReasoning && (
                                 <div className="bg-yellow-50 border border-yellow-200 p-2 text-[10px] text-yellow-800 rounded-sm leading-relaxed">
                                    <strong className="block mb-1 opacity-70">AI REASONING:</strong>
                                    {selectedNode.data.aiReasoning}
                                 </div>
                              )}

                              <div className="space-y-2">
                                 <Label>Entry Action (JS)</Label>
                                 <textarea className="w-full h-24 bg-gray-50 border border-neuro-dim text-[10px] font-mono p-2 outline-none focus:border-neuro-primary resize-y" 
                                    value={selectedNode.data.entryAction || ''}
                                    onChange={(e) => setNodes(nds => nds.map(n => n.id === selectedNodeId ? { ...n, data: { ...n.data, entryAction: e.target.value } } : n))}
                                    placeholder="// e.g. ctx.count++"
                                 />
                              </div>
                              
                              <div className="space-y-2">
                                 <Label>Exit Action (JS)</Label>
                                 <textarea className="w-full h-24 bg-gray-50 border border-neuro-dim text-[10px] font-mono p-2 outline-none focus:border-neuro-primary resize-y" 
                                    value={selectedNode.data.exitAction || ''}
                                    onChange={(e) => setNodes(nds => nds.map(n => n.id === selectedNodeId ? { ...n, data: { ...n.data, exitAction: e.target.value } } : n))}
                                    placeholder="// Cleanup code"
                                 />
                              </div>

                              <div className="pt-4 border-t border-neuro-dim">
                                 <Button onClick={handleGenerateRegisterMap} className="w-full mb-2">Generate registers.h</Button>
                                 <div className="text-[9px] text-gray-400 text-center">AI analyzes 'ctx' variables</div>
                              </div>
                           </>
                        )}
                     </div>
                  </Panel>
               )}

               {rightPanelTab === 'CHAT' && (
                  <Panel title="AI ASSISTANT (NEO)" className="h-full border-0 flex flex-col">
                     <div className="flex flex-col h-full relative">
                        {isCompanionMode && (
                           <div className="absolute top-0 left-0 right-0 bg-purple-50 text-purple-700 text-[10px] p-2 text-center border-b border-purple-100 flex items-center justify-center gap-2 animate-in slide-in-from-top-2 z-10">
                              <Waves size={12} className="animate-pulse"/> Voice Agent Active. You can also text below.
                           </div>
                        )}
                        <div className="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar pb-20" ref={chatScrollRef}>
                           {activeProject.chatHistory.length === 0 && <div className="text-gray-400 text-center italic mt-10">Ask me anything or attach an image to build a graph...</div>}
                           {activeProject.chatHistory.map(msg => (
                              <div key={msg.id} className={clsx("p-3 rounded-lg text-xs leading-relaxed break-words shadow-sm", msg.role === 'user' ? "bg-neuro-primary text-white ml-8" : "bg-white text-gray-800 mr-8 border border-gray-200")}>
                                 <div className="font-bold mb-1 opacity-70 text-[9px] uppercase tracking-wider">{msg.role}</div>
                                 {renderMessageContent(msg.content)}
                              </div>
                           ))}
                           {isAiLoading && (
                              <div className="flex justify-center p-4">
                                 <div className="flex items-center gap-2 text-gray-400 text-xs animate-pulse">
                                    <Loader2 size={14} className="animate-spin"/> Thinking...
                                 </div>
                              </div>
                           )}
                        </div>
                        <div className="p-3 border-t border-neuro-dim bg-gray-50 absolute bottom-0 left-0 right-0">
                           {chatAttachment && (
                              <div className="mb-2 p-2 bg-gray-100 border border-gray-200 rounded flex items-center justify-between">
                                 <div className="flex items-center gap-2">
                                    {chatAttachment.mimeType.startsWith('image/') ? 
                                       <img src={chatAttachment.preview} className="w-8 h-8 object-cover rounded border" alt="attachment" /> : 
                                       <div className="w-8 h-8 bg-gray-200 flex items-center justify-center rounded"><FileCode size={16}/></div>
                                    }
                                    <div className="text-[10px] text-gray-600 truncate max-w-[150px]">{chatAttachment.mimeType}</div>
                                 </div>
                                 <button onClick={() => setChatAttachment(null)} className="text-gray-400 hover:text-red-500"><X size={14}/></button>
                              </div>
                           )}
                           
                           <div className="flex gap-2">
                              <button onClick={() => chatFileRef.current?.click()} className="p-2 text-gray-400 hover:text-neuro-primary hover:bg-gray-100 rounded border border-transparent hover:border-neuro-dim transition-all" title="Attach Image/Video">
                                 <Paperclip size={14}/>
                              </button>
                              <textarea 
                                 className="flex-1 min-h-[40px] max-h-[100px] border border-neuro-dim p-2 text-xs outline-none focus:border-neuro-primary rounded-sm resize-none"
                                 placeholder={isCompanionMode ? "Type to Neo..." : "Type query or attach flowchart..."}
                                 value={aiQuery}
                                 onChange={e => setAiQuery(e.target.value)}
                                 onKeyDown={e => { if(e.key==='Enter' && !e.shiftKey) { e.preventDefault(); handleChatSend(); } }}
                              />
                              <Button onClick={handleChatSend} disabled={isAiLoading || (!aiQuery.trim() && !chatAttachment)}><Send size={14}/></Button>
                           </div>
                           <div className="mt-2 flex justify-between">
                              <div className="text-[9px] text-gray-400 flex gap-2">
                                 <button className="hover:text-neuro-primary underline decoration-dotted" onClick={() => setAiQuery("Find dead ends in the graph.")}>Find Issues</button>
                                 <button className="hover:text-neuro-primary underline decoration-dotted" onClick={() => setAiQuery("Optimize this for low power.")}>Optimize</button>
                              </div>
                              <button className="text-[9px] text-neuro-primary font-bold hover:underline" onClick={handlePowerAnalysis}>POWER REPORT</button>
                           </div>
                        </div>
                     </div>
                  </Panel>
               )}
             </div>
          </div>
        )}
      </div>

      {showBottomPanel && (
        <div className="h-48 border-t border-neuro-dim bg-white flex flex-col shrink-0">
           <div className="flex border-b border-neuro-dim">
              {['OUTPUT', 'PROBLEMS', 'VALIDATION', 'RESOURCES', 'SERIAL', 'LOGIC'].map(tab => (
                 <button key={tab} onClick={() => setActiveBottomTab(tab as any)} className={clsx("px-4 py-1.5 text-[10px] font-bold tracking-wider hover:bg-gray-50 border-r border-neuro-dim", activeBottomTab === tab ? "bg-gray-100 text-neuro-primary border-b-2 border-b-neuro-primary" : "text-gray-500")}>
                    {tab} {tab==='PROBLEMS' && ghostIssues.length > 0 && `(${ghostIssues.length})`}
                 </button>
              ))}
              <div className="flex-1 bg-gray-50"></div>
              <button onClick={() => setShowBottomPanel(false)} className="px-3 hover:bg-red-50 hover:text-red-500 text-gray-400"><X size={14}/></button>
           </div>
           
           <div className="flex-1 overflow-auto p-0 custom-scrollbar font-mono">
              {activeBottomTab === 'OUTPUT' && (
                 <div className="p-2 space-y-1">
                    {logs.length === 0 && <div className="text-gray-400 italic p-2">System logs will appear here...</div>}
                    {logs.map(log => (
                       <div key={log.id} className={clsx("text-[11px] flex gap-2 font-mono border-b border-gray-50 pb-0.5", log.type === 'error' ? "text-red-600" : log.type === 'warning' ? "text-orange-600" : log.type === 'success' ? "text-green-600" : "text-gray-600")}>
                          <span className="text-gray-400 w-16 shrink-0">{log.timestamp}</span>
                          <span className="font-bold w-12 shrink-0">[{log.source}]</span>
                          <span>{log.message}</span>
                       </div>
                    ))}
                 </div>
              )}
              {activeBottomTab === 'PROBLEMS' && (
                 <div className="p-0">
                    <table className="w-full text-left border-collapse">
                       <thead className="bg-gray-50 text-gray-500 font-bold sticky top-0">
                          <tr><th className="p-2 border-b">Severity</th><th className="p-2 border-b">Issue</th><th className="p-2 border-b">Location</th></tr>
                       </thead>
                       <tbody>
                          {ghostIssues.map(issue => (
                             <tr key={issue.id} className="hover:bg-gray-50 border-b border-gray-100 cursor-pointer" onClick={() => { if(issue.nodeId) { setSelectedNodeId(issue.nodeId); reactFlowInstance.fitView({ nodes: [{id: issue.nodeId} as any], duration: 500, minZoom: 1 }); } }}>
                                <td className="p-2"><span className={clsx("px-1.5 py-0.5 rounded text-[9px] font-bold border", issue.severity === 'CRITICAL' ? "bg-red-50 text-red-600 border-red-200" : "bg-yellow-50 text-yellow-600 border-yellow-200")}>{issue.severity}</span></td>
                                <td className="p-2">
                                   <div className="font-bold text-neuro-primary">{issue.title}</div>
                                   <div className="text-gray-500">{issue.description}</div>
                                </td>
                                <td className="p-2 text-gray-400 font-mono">{issue.nodeId || 'Graph'}</td>
                             </tr>
                          ))}
                          {ghostIssues.length === 0 && <tr><td colSpan={3} className="p-8 text-center text-gray-400 italic">No design issues detected. Good job!</td></tr>}
                       </tbody>
                    </table>
                    {ghostIssues.length > 0 && (
                       <div className="p-2 flex justify-end">
                          <Button onClick={handleAutoFix} variant="primary" className="text-xs">
                             <Wand2 size={12}/> Auto-Fix All Issues
                          </Button>
                       </div>
                    )}
                 </div>
              )}
              {activeBottomTab === 'VALIDATION' && (
                 <div className="p-4">
                    {!validationReport && (
                       <div className="flex flex-col items-center justify-center h-full gap-3 opacity-50">
                          <Shield size={32}/>
                          <Button onClick={handleRunValidationWrapper} disabled={isValidating}>{isValidating ? 'ANALYZING...' : 'RUN DEEP ANALYSIS'}</Button>
                       </div>
                    )}
                    {validationReport && (
                       <div className="grid grid-cols-2 gap-6">
                          <div>
                             <h4 className="font-bold text-neuro-primary mb-2 flex items-center gap-2"><Bug size={14}/> AI CRITIQUE</h4>
                             <ul className="list-disc pl-4 space-y-1 text-gray-600">
                                {validationReport.critique.map((c, i) => <li key={i}>{c}</li>)}
                             </ul>
                             <h4 className="font-bold text-neuro-primary mt-4 mb-2 flex items-center gap-2"><Sparkles size={14}/> SUGGESTIONS</h4>
                             <ul className="list-disc pl-4 space-y-1 text-green-700">
                                {validationReport.suggestions.map((s, i) => <li key={i}>{s}</li>)}
                             </ul>
                          </div>
                          <div>
                             <h4 className="font-bold text-neuro-primary mb-2 flex items-center gap-2"><FlaskConical size={14}/> GENERATED TEST CASES</h4>
                             <div className="space-y-2">
                                {validationReport.testCases.map((tc, i) => (
                                   <div key={i} className="bg-gray-50 border border-neuro-dim p-2 rounded-sm">
                                      <div className="font-bold text-xs">{tc.name}</div>
                                      <div className="text-[10px] text-gray-500 mt-1">Seq: {(tc.sequence || []).join(' -> ')}</div>
                                      <div className="text-[10px] text-gray-500">Expect: {tc.expectedState}</div>
                                      <Button className="w-full mt-2 h-6 text-[9px]" onClick={() => { showToast('Auto-running test case...', 'info'); }}>RUN TEST</Button>
                                   </div>
                                ))}
                             </div>
                          </div>
                       </div>
                    )}
                 </div>
              )}
              {activeBottomTab === 'RESOURCES' && (
                 <div className="p-4">
                    {!resourceMetrics && (
                       <div className="flex flex-col items-center justify-center h-full gap-3 opacity-50">
                          <Gauge size={32}/>
                          <Button onClick={handleEstimateResourcesWrapper} disabled={isEstimating}>{isEstimating ? 'CALCULATING...' : 'ESTIMATE HARDWARE USAGE'}</Button>
                       </div>
                    )}
                    {resourceMetrics && (
                       <div>
                          <div className="grid grid-cols-4 gap-4 mb-6">
                             <MetricCard label="Logic Cells (LUT)" value={resourceMetrics.lutUsage} unit="%" />
                             <MetricCard label="Flip-Flops" value={resourceMetrics.ffUsage} />
                             <MetricCard label="Memory" value={resourceMetrics.memoryKB} unit="KB" />
                             <MetricCard label="Est. Power" value={resourceMetrics.powermW} unit="mW" />
                          </div>
                          <div className="p-3 bg-blue-50 border border-blue-200 text-blue-800 rounded-sm">
                             <div className="font-bold mb-1">AI SUMMARY</div>
                             {resourceMetrics.summary}
                          </div>
                       </div>
                    )}
                 </div>
              )}
              {activeBottomTab === 'SERIAL' && (
                 <SerialMonitor state={halSnapshot} />
              )}
              {activeBottomTab === 'LOGIC' && (
                 <div className="w-full h-full p-2 bg-[#111] overflow-hidden">
                    <LogicAnalyzer 
                       history={halHistory} 
                       channels={[
                          ...Object.keys(halHistory[halHistory.length-1]?.signals || {}).filter(k=>k.includes('GPIO')),
                          ...Object.keys(halHistory[halHistory.length-1]?.signals || {}).filter(k=>k.includes('PWM')),
                          'ADC_0',
                          'ADC_1'
                       ]} 
                       height={160}
                    />
                 </div>
              )}
           </div>
        </div>
      )}

      <div className="h-7 bg-[#111827] text-gray-400 text-[10px] flex items-center px-2 justify-between select-none shrink-0 border-t border-gray-800 z-50 font-medium">
         <div className="flex items-center gap-4">
            <div className="flex items-center gap-1.5 hover:text-white cursor-pointer transition-colors" title="Git Branch">
                <GitBranch size={12}/> 
                <span>main*</span>
            </div>
            <div className="flex items-center gap-1.5 hover:text-white cursor-pointer transition-colors" title="Device Target">
                <CircuitBoard size={12}/> 
                <span>{targetMcu.name}</span>
            </div>
            <div className={clsx("flex items-center gap-1.5 transition-colors", isDeviceConnected ? "text-green-500" : "text-gray-500")} title="Connection Status">
                <div className={clsx("w-1.5 h-1.5 rounded-full", isDeviceConnected ? "bg-green-500 animate-pulse" : "bg-gray-500")}></div>
                {isDeviceConnected ? "CONNECTED" : "OFFLINE"}
            </div>
         </div>

         <div className="flex items-center gap-4">
             <div className={clsx("flex items-center gap-1.5 px-2 py-0.5 rounded", simStatus === SimulationStatus.RUNNING ? "bg-green-900/30 text-green-400" : "")}>
                <Activity size={12}/>
                <span>{isShadowMode ? "SHADOW LINK" : simStatus}</span>
             </div>
         </div>

         <div className="flex items-center gap-4">
            <div className="flex items-center gap-3">
                <span title="Total Nodes">N: {nodes.length}</span>
                <span title="Total Edges">E: {edges.length}</span>
                {simStatus === SimulationStatus.RUNNING && (
                    <span className="text-blue-400" title="Active Nodes">
                        Active: {nodes.filter(n => n.data.executionState === 'entry' || n.data.executionState === 'exit').length}
                    </span>
                )}
            </div>
            
            <div className={clsx("flex items-center gap-1.5 px-2 py-0.5 rounded transition-all", isWakeWordActive ? "bg-green-500/20 text-green-400 animate-pulse" : "text-gray-500")}>
                <Ear size={12} className={isWakeWordActive ? "animate-bounce" : ""} />
                <span>{isWakeWordActive ? 'LISTENING' : 'STANDBY'}</span>
            </div>

            <div className={clsx("flex items-center gap-1.5 px-2 py-0.5 rounded transition-all", agentState !== 'IDLE' ? "bg-neuro-accent text-white" : "text-gray-500")}>
                {agentState !== 'IDLE' ? <Waves size={12} className="animate-pulse"/> : <Bot size={12}/>}
                <span>{agentState === 'IDLE' ? 'AI READY' : agentState}</span>
            </div>

            <div className="pl-2 border-l border-gray-700">
                {activeLayout.replace('_', ' ')}
            </div>
         </div>
      </div>

      <div className="fixed top-12 right-4 z-[100] flex flex-col items-end pointer-events-none">
         {toasts.map(t => (
            <div key={t.id} className="pointer-events-auto">
               <Toast {...t} onClose={closeToast} />
            </div>
         ))}
      </div>

      {showTemplateBrowser && <TemplateBrowser onSelect={handleCreateProjectFromTemplate} onClose={() => setShowTemplateBrowser(false)} />}
      {showDeviceManager && <DeviceManagerModal onClose={() => setShowDeviceManager(false)} onConnect={handleConnectDevice} isConnected={isDeviceConnected} />}
      {showAboutModal && <AboutModal onClose={() => setShowAboutModal(false)} />}
      {showDocsModal && <DocumentationModal onClose={() => setShowDocsModal(false)} />}
      {showVeoModal && <VeoModal onClose={() => setShowVeoModal(false)} />}
      {showDatasheetModal && (
         <div className="fixed inset-0 z-[100] bg-neuro-primary/50 backdrop-blur-sm flex items-center justify-center p-8">
            <div className="bg-white border border-neuro-primary shadow-hard w-full max-w-lg flex flex-col animate-in zoom-in-95 duration-150">
               <div className="bg-neuro-primary text-white p-3 font-bold flex justify-between items-center">
                  <span>DATASHEET ANALYSIS</span>
                  <button onClick={() => setShowDatasheetModal(false)}><X size={16}/></button>
               </div>
               <div className="p-4">
                  <p className="text-gray-500 mb-2">Paste relevant section from PDF (Timings, Registers, Constraints):</p>
                  <textarea className="w-full h-40 border border-neuro-dim p-2 text-xs font-mono mb-4 outline-none focus:border-neuro-primary" placeholder="Paste text here..." value={datasheetInput} onChange={e => setDatasheetInput(e.target.value)} />
                  <div className="flex justify-end gap-2">
                     <Button variant="ghost" onClick={() => setShowDatasheetModal(false)}>Cancel</Button>
                     <Button onClick={handleAnalyzeDatasheet} disabled={!datasheetInput || isAiLoading}>{isAiLoading ? 'ANALYZING...' : 'EXTRACT RULES'}</Button>
                  </div>
               </div>
            </div>
         </div>
      )}
    </div>
  );
}
