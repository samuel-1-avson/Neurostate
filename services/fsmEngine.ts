
import { Node, Edge } from 'reactflow';
import { HAL } from './hal';
import { SimTelemetry } from '../types';

export type VisualEventType = 'node_entry' | 'node_exit' | 'node_idle' | 'edge_traverse' | 'guard_check' | 'guard_result';

export class FSMExecutor {
  private nodes: Node[];
  private edges: Edge[];
  private currentNodeId: string | null = null;
  private isRunning: boolean = false;
  private isShadowMode: boolean = false;
  
  // Runtime State
  private context: Record<string, any> = {};
  private history: string[] = []; // List of visited node IDs

  // Telemetry Trackers
  private startTime: number = 0;
  private lastTransitionTime: number = 0;
  private transitionCount: number = 0;
  private interruptCount: number = 0;
  private heartbeatTimer: ReturnType<typeof setTimeout> | null = null;

  // Configuration
  private stepDelayMs: number = 500;

  // Callbacks
  private onLog: (msg: string, type?: 'info' | 'error' | 'success' | 'warning') => void;
  private onStateChange: (nodeId: string, history: string[]) => void;
  private onVariableChange: (context: Record<string, any>) => void;
  private onTelemetry: (data: SimTelemetry) => void;
  private onVisualUpdate: (event: VisualEventType, id: string, data?: any) => Promise<void>;

  constructor(
    nodes: Node[], 
    edges: Edge[], 
    onLog: (msg: string, type?: 'info' | 'error' | 'success' | 'warning') => void,
    onStateChange: (nodeId: string, history: string[]) => void,
    onVariableChange: (context: Record<string, any>) => void,
    onTelemetry?: (data: SimTelemetry) => void,
    onVisualUpdate?: (event: VisualEventType, id: string, data?: any) => Promise<void>
  ) {
    this.nodes = nodes;
    this.edges = edges;
    this.onLog = onLog;
    this.onStateChange = onStateChange;
    this.onVariableChange = onVariableChange;
    this.onTelemetry = onTelemetry || (() => {});
    this.onVisualUpdate = onVisualUpdate || (async () => {});
    this.context = {}; // Reset context
  }

  // --- HOT RELOAD SUPPORT ---
  public updateGraph(newNodes: Node[], newEdges: Edge[]) {
    this.nodes = newNodes;
    this.edges = newEdges;
    
    // Check if current node still exists
    if (this.currentNodeId && !this.nodes.find(n => n.id === this.currentNodeId)) {
        this.onLog("HOT RELOAD WARNING: Current state was deleted. Resetting to Input node.", 'warning');
        const entryNode = this.nodes.find(n => n.data.type === 'input' || n.type === 'input');
        if (entryNode) {
            this.currentNodeId = entryNode.id;
            this.onStateChange(this.currentNodeId, this.history);
        } else {
            this.stop();
            this.onLog("HOT RELOAD ERROR: No entry point found. Simulation stopped.", 'error');
        }
    } else {
        // Silent update to not spam logs, but engine now has new logic/edges
    }
  }

  public setSpeed(delayMs: number) {
    this.stepDelayMs = delayMs;
  }

  public setShadowMode(enabled: boolean) {
    this.isShadowMode = enabled;
    if (enabled) {
        this.onLog("FSM Executor switched to SHADOW MODE. Waiting for Hardware Sync...", "warning");
    }
  }

  public async start() {
    const entryNode = this.nodes.find(n => n.data.type === 'input' || n.type === 'input');
    if (!entryNode) {
      this.onLog('NO_ENTRY_POINT_DEFINED: Cannot start simulation.', 'error');
      throw new Error("NO_ENTRY_POINT_DEFINED");
    }
    
    this.isRunning = true;
    this.currentNodeId = entryNode.id;
    this.history = [entryNode.id];
    this.startTime = Date.now();
    this.lastTransitionTime = Date.now();
    this.transitionCount = 0;
    this.interruptCount = 0;
    
    this.onLog(`System initialized. State: ${entryNode.data.label}`, 'success');
    this.onStateChange(entryNode.id, this.history);
    
    // Initial context update
    this.onVariableChange({ ...this.context });
    
    // Reset Hardware
    HAL.reset();

    // Only animate actions if NOT in shadow mode (unless synced)
    if (!this.isShadowMode) {
        // Animate Entry Action
        await this.onVisualUpdate('node_entry', entryNode.id, { code: entryNode.data.entryAction });
        await this.executeAction(entryNode.data.entryAction);
        await this.wait(this.stepDelayMs * 0.5);
        await this.onVisualUpdate('node_idle', entryNode.id);
    } else {
        await this.onVisualUpdate('node_idle', entryNode.id);
    }
    
    // Start Telemetry Heartbeat
    this.startHeartbeat();
  }
  
  public stop() {
    this.isRunning = false;
    if (this.heartbeatTimer) clearInterval(this.heartbeatTimer);
    HAL.reset();
  }

  // --- DIGITAL TWIN SYNC ---
  public async syncState(targetLabel: string) {
      if (!this.isRunning) return;
      
      const targetNode = this.nodes.find(n => n.data.label.toUpperCase() === targetLabel.toUpperCase());
      if (!targetNode) {
          this.onLog(`Shadow Sync Failed: Hardware reports unknown state '${targetLabel}'`, 'error');
          return;
      }

      if (this.currentNodeId === targetNode.id) {
          // Already synchronized
          return;
      }

      this.onLog(`Shadow Sync: Hardware transitioned to [${targetLabel}]`, 'warning');
      
      // Force Visual Transition
      if (this.currentNodeId) {
          await this.onVisualUpdate('node_idle', this.currentNodeId);
      }
      
      this.currentNodeId = targetNode.id;
      this.history.push(targetNode.id);
      this.onStateChange(targetNode.id, this.history);
      
      // Visually ping the node to show active state
      await this.onVisualUpdate('node_entry', targetNode.id, { code: "HW_SYNC" });
      await this.wait(500);
      await this.onVisualUpdate('node_idle', targetNode.id);
  }

  private startHeartbeat() {
    if (this.heartbeatTimer) clearInterval(this.heartbeatTimer);
    
    this.heartbeatTimer = setInterval(() => {
        if (!this.isRunning) return;
        
        const now = Date.now();
        const uptimeMs = now - this.startTime;
        const activeStateTimeMs = now - this.lastTransitionTime;
        
        // Simulate CPU Load: Higher if actions recently executed or complex context
        const baseLoad = 15; // Idle
        const contextComplexity = Object.keys(this.context).length * 2;
        const loadNoise = Math.random() * 5;
        const cpuLoad = Math.min(100, baseLoad + contextComplexity + loadNoise);
        
        // Simulate RAM: Base + Context size
        const ramUsageBytes = 2048 + JSON.stringify(this.context).length * 4 + this.history.length * 8;
        
        // Simulate Power: Base + Peripherals (check HAL indirectly via context names)
        let powerDrawMW = 12; // Base sleep/idle
        if (cpuLoad > 20) powerDrawMW += 25; // Active CPU
        // Check for "peripheral" keywords in context as a heuristic for active HW
        const peripheralCount = Object.keys(this.context).filter(k => k.match(/led|pwm|adc|uart|spi/i)).length;
        powerDrawMW += peripheralCount * 15;
        
        this.onTelemetry({
            uptimeMs,
            cpuLoad,
            ramUsageBytes,
            stackDepth: this.history.length,
            interruptCount: this.interruptCount,
            powerDrawMW,
            activeStateTimeMs,
            transitionsPerSec: this.transitionCount / (uptimeMs / 1000 || 1)
        });
        
    }, 500); // Update every 500ms
  }

  public async triggerEvent(eventName: string) {
    if (!this.currentNodeId || !this.isRunning) return;
    
    // In Shadow Mode, we ignore internal triggers unless they match hardware sync
    // Real hardware drives the state, simulation only mirrors it.
    if (this.isShadowMode) {
        return; 
    }
    
    this.interruptCount++;

    const currentNode = this.nodes.find(n => n.id === this.currentNodeId);
    if (!currentNode) return;

    // Find all potential transitions (matching event name)
    const potentialEdges = this.edges.filter(e => 
      e.source === this.currentNodeId && 
      e.label && 
      (e.label as string).toLowerCase() === eventName.toLowerCase()
    );

    let validEdge: Edge | undefined;

    // Evaluate Guard Conditions with Visuals
    for (const edge of potentialEdges) {
       await this.onVisualUpdate('guard_check', edge.id);
       await this.wait(this.stepDelayMs * 0.2);

       if (edge.data && edge.data.condition) {
          // Has a guard condition, evaluate it
          const passed = this.evaluateCondition(edge.data.condition, currentNode);
          
          await this.onVisualUpdate('guard_result', edge.id, { passed });
          await this.wait(this.stepDelayMs * 0.2);

          if (passed) {
             validEdge = edge;
             break; // Take the first valid one
          } else {
             this.onLog(`Guard blocked transition: ${edge.data.condition}`, 'warning');
          }
       } else {
          // No condition, automatically valid
          validEdge = edge;
          await this.onVisualUpdate('guard_result', edge.id, { passed: true });
          break;
       }
    }

    if (validEdge) {
      const targetNode = this.nodes.find(n => n.id === validEdge!.target);
      if (!targetNode) return;

      this.onLog(`Transition: ${currentNode.data.label} -> ${targetNode.data.label}`, 'info');

      // 1. Exit Action (Current Node)
      await this.onVisualUpdate('node_exit', currentNode.id, { code: currentNode.data.exitAction });
      await this.wait(this.stepDelayMs * 0.2);
      await this.executeAction(currentNode.data.exitAction);
      
      // 2. Edge Traversal
      await this.onVisualUpdate('node_idle', currentNode.id); // Clear exit state
      await this.onVisualUpdate('edge_traverse', validEdge.id);
      await this.wait(this.stepDelayMs * 0.8); // Allow packet animation to fly

      // Transition Logic
      this.currentNodeId = targetNode.id;
      this.history.push(targetNode.id);
      this.transitionCount++;
      this.lastTransitionTime = Date.now();
      
      this.onStateChange(targetNode.id, this.history);

      // 3. Entry Action (Target Node)
      await this.onVisualUpdate('node_entry', targetNode.id, { code: targetNode.data.entryAction });
      await this.wait(this.stepDelayMs * 0.3);
      await this.executeAction(targetNode.data.entryAction);
      await this.wait(this.stepDelayMs * 0.2);
      
      // 4. Settle
      await this.onVisualUpdate('node_idle', targetNode.id);

    } else {
      this.onLog(`Event '${eventName}' ignored in state '${this.currentNodeId}'.`, 'warning');
    }
  }

  private evaluateCondition(condition: string, node?: Node): boolean {
    try {
       const safeFn = new Function('ctx', 'node', 'HAL', `
         "use strict";
         try {
           return (${condition});
         } catch(e) {
           return false;
         }
       `);
       // Pass node.data as 'node' so user can access node.label, node.type, etc.
       return safeFn(this.context, node ? node.data : {}, HAL);
    } catch (e) {
       this.onLog(`Condition Error: ${(e as Error).message}`, 'error');
       return false;
    }
  }

  private async executeAction(code?: string) {
    if (!code || !code.trim()) return;
    
    // Inject a 'dispatch' function into the user code scope
    const dispatch = (eventName: string, delayMs: number = 0) => {
        if (!this.isRunning) return;
        if (delayMs > 0) {
            setTimeout(() => {
                if (this.isRunning) this.triggerEvent(eventName);
            }, delayMs);
        } else {
            this.triggerEvent(eventName);
        }
    };

    try {
      // execute the code with access to system tools
      const safeFn = new Function('console', 'ctx', 'HAL', 'dispatch', `
        "use strict";
        try {
          ${code}
        } catch(e) {
          throw e;
        }
      `);
      safeFn(console, this.context, HAL, dispatch);
      this.onVariableChange({ ...this.context });
    } catch (e) {
      this.onLog(`Runtime Action Error: ${(e as Error).message}`, 'error');
    }
  }

  public getCurrentState(): string | null {
    return this.currentNodeId;
  }

  private wait(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
