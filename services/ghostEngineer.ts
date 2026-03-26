/**
 * Ghost Engineer v2 - Enhanced Static FSM Analyzer
 * Performs deep analysis: dead ends, unreachable states, race conditions,
 * loop detection, timing analysis, complexity metrics, and hardware constraints
 */

import { Edge, Node } from 'reactflow';
import { GhostIssue } from '../types';

export interface ComplexityMetrics {
  cyclomaticComplexity: number;
  nodeCount: number;
  edgeCount: number;
  maxBranchFactor: number;
  averageBranchFactor: number;
  depth: number;
  loops: string[][];
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
}

export interface TimingAnalysis {
  estimatedMinCycleMs: number;
  estimatedMaxCycleMs: number;
  longestPath: string[];
  criticalNodes: string[];
}

export interface HardwareConstraints {
  estimatedFlashBytes: number;
  estimatedRamBytes: number;
  estimatedMipsRequired: number;
  suitableFor: string[];
}

export class GhostEngineer {
  
  /**
   * Run all analysis checks and return issues
   */
  static analyze(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];

    // 1. Dead End Detection
    issues.push(...this.detectDeadEnds(nodes, edges));
    
    // 2. Unreachable State Detection
    issues.push(...this.detectUnreachableStates(nodes, edges));
    
    // 3. Determinism Check (Race Conditions)
    issues.push(...this.detectRaceConditions(nodes, edges));
    
    // 4. Loop Detection
    issues.push(...this.detectInfiniteLoops(nodes, edges));
    
    // 5. Missing Error Handlers
    issues.push(...this.detectMissingErrorHandlers(nodes, edges));
    
    // 6. Complexity Warnings
    issues.push(...this.analyzeComplexityIssues(nodes, edges));
    
    // 7. Timing Issues
    issues.push(...this.detectTimingIssues(nodes, edges));

    return issues;
  }

  /**
   * Detect nodes with no outgoing transitions (dead ends)
   */
  private static detectDeadEnds(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    
    nodes.forEach(node => {
      const isFinal = node.data.type === 'output' || 
                      node.data.type === 'error' || 
                      node.data.label.toLowerCase().includes('end') ||
                      node.data.label.toLowerCase().includes('complete') ||
                      node.data.label.toLowerCase().includes('done');
      
      if (!isFinal) {
        const outgoing = edges.filter(e => e.source === node.id);
        if (outgoing.length === 0) {
          issues.push({
            id: `dead-end-${node.id}`,
            severity: 'WARNING',
            title: 'Dead End Detected',
            description: `Node "${node.data.label}" has no outgoing transitions but is not marked as a final state.`,
            nodeId: node.id
          });
        }
      }
    });
    
    return issues;
  }

  /**
   * Detect nodes unreachable from the start node (BFS)
   */
  private static detectUnreachableStates(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    const inputNode = nodes.find(n => n.data.type === 'input' || n.type === 'input');
    
    if (!inputNode) {
      issues.push({
        id: 'no-entry',
        severity: 'CRITICAL',
        title: 'No Entry Point',
        description: 'No node is marked as "input" type. The FSM cannot start.',
      });
      return issues;
    }
    
    const visited = new Set<string>();
    const queue = [inputNode.id];
    visited.add(inputNode.id);

    while (queue.length > 0) {
      const currentId = queue.shift()!;
      const outgoingEdges = edges.filter(e => e.source === currentId);
      for (const edge of outgoingEdges) {
        if (!visited.has(edge.target)) {
          visited.add(edge.target);
          queue.push(edge.target);
        }
      }
    }

    nodes.forEach(node => {
      if (!visited.has(node.id)) {
        issues.push({
          id: `unreachable-${node.id}`,
          severity: 'WARNING',
          title: 'Unreachable State',
          description: `Node "${node.data.label}" cannot be reached from the Start node.`,
          nodeId: node.id
        });
      }
    });
    
    return issues;
  }

  /**
   * Detect non-deterministic transitions (same event triggers multiple paths)
   */
  private static detectRaceConditions(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    
    nodes.forEach(node => {
      const outgoing = edges.filter(e => e.source === node.id);
      const labels = outgoing.map(e => (e.label as string || '').toLowerCase());
      const labelCounts = new Map<string, number>();
      
      labels.forEach(label => {
        if (label) {
          labelCounts.set(label, (labelCounts.get(label) || 0) + 1);
        }
      });
      
      labelCounts.forEach((count, label) => {
        if (count > 1) {
          issues.push({
            id: `race-${node.id}-${label}`,
            severity: 'CRITICAL',
            title: 'Race Condition (Nondeterministic)',
            description: `Node "${node.data.label}" has ${count} transitions triggered by "${label}".`,
            nodeId: node.id
          });
        }
      });
    });
    
    return issues;
  }

  /**
   * Detect infinite loops using cycle detection (Tarjan's algorithm simplified)
   */
  private static detectInfiniteLoops(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    const adjacency = new Map<string, string[]>();
    
    // Build adjacency list
    nodes.forEach(n => adjacency.set(n.id, []));
    edges.forEach(e => {
      const list = adjacency.get(e.source);
      if (list) list.push(e.target);
    });
    
    // Find all cycles using DFS
    const cycles: string[][] = [];
    const visited = new Set<string>();
    const recursionStack = new Set<string>();
    
    const dfs = (nodeId: string, path: string[]): void => {
      visited.add(nodeId);
      recursionStack.add(nodeId);
      path.push(nodeId);
      
      const neighbors = adjacency.get(nodeId) || [];
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          dfs(neighbor, [...path]);
        } else if (recursionStack.has(neighbor)) {
          // Found a cycle
          const cycleStart = path.indexOf(neighbor);
          if (cycleStart !== -1) {
            const cycle = path.slice(cycleStart);
            cycle.push(neighbor); // Complete the cycle
            cycles.push(cycle);
          }
        }
      }
      
      recursionStack.delete(nodeId);
    };
    
    nodes.forEach(node => {
      if (!visited.has(node.id)) {
        dfs(node.id, []);
      }
    });
    
    // Analyze cycles for infinite loops
    cycles.forEach((cycle, index) => {
      // Check if any edge in the cycle has a guard condition
      let hasGuard = false;
      for (let i = 0; i < cycle.length - 1; i++) {
        const edge = edges.find(e => e.source === cycle[i] && e.target === cycle[i + 1]);
        if (edge?.data?.condition) {
          hasGuard = true;
          break;
        }
      }
      
      if (!hasGuard) {
        const cycleLabels = cycle.map(id => {
          const node = nodes.find(n => n.id === id);
          return node?.data.label || id;
        });
        
        issues.push({
          id: `infinite-loop-${index}`,
          severity: 'WARNING',
          title: 'Potential Infinite Loop',
          description: `Cycle detected without exit conditions: ${cycleLabels.join(' → ')}`,
          nodeId: cycle[0]
        });
      }
    });
    
    return issues;
  }

  /**
   * Detect missing error handlers (no error states or no transitions to error states)
   */
  private static detectMissingErrorHandlers(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    
    const errorNodes = nodes.filter(n => 
      n.data.type === 'error' || 
      n.data.label.toLowerCase().includes('error') ||
      n.data.label.toLowerCase().includes('fault') ||
      n.data.label.toLowerCase().includes('exception')
    );
    
    if (errorNodes.length === 0 && nodes.length > 3) {
      issues.push({
        id: 'no-error-handler',
        severity: 'WARNING',
        title: 'No Error Handler',
        description: 'FSM has no error handling states. Consider adding fault recovery.',
      });
    }
    
    // Check for watchdog/timeout handling
    const hasWatchdog = nodes.some(n => 
      n.data.label.toLowerCase().includes('watchdog') ||
      n.data.label.toLowerCase().includes('timeout') ||
      n.data.entryAction?.includes('watchdog') ||
      n.data.entryAction?.includes('timeout')
    );
    
    const hasTimers = nodes.some(n => n.data.type === 'timer');
    
    if (!hasWatchdog && nodes.length > 5 && !hasTimers) {
      issues.push({
        id: 'no-watchdog',
        severity: 'INFO',
        title: 'No Watchdog Timer',
        description: 'Consider adding a watchdog timer for embedded reliability.',
      });
    }
    
    return issues;
  }

  /**
   * Analyze complexity and warn about overly complex FSMs
   */
  private static analyzeComplexityIssues(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    const metrics = this.calculateComplexity(nodes, edges);
    
    if (metrics.cyclomaticComplexity > 15) {
      issues.push({
        id: 'high-complexity',
        severity: 'WARNING',
        title: 'High Cyclomatic Complexity',
        description: `Complexity score of ${metrics.cyclomaticComplexity} exceeds recommended threshold of 15. Consider refactoring into hierarchical states.`,
      });
    }
    
    if (metrics.maxBranchFactor > 6) {
      issues.push({
        id: 'high-branching',
        severity: 'INFO',
        title: 'High Branching Factor',
        description: `A node has ${metrics.maxBranchFactor} outgoing transitions. This may indicate design issues.`,
      });
    }
    
    return issues;
  }

  /**
   * Detect timing-related issues
   */
  private static detectTimingIssues(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];
    
    // Check for long-running actions that could block
    nodes.forEach(node => {
      const action = node.data.entryAction || '';
      
      // Detect synchronous delays
      if (action.includes('delay(') || action.includes('sleep(') || action.includes('wait(')) {
        issues.push({
          id: `blocking-delay-${node.id}`,
          severity: 'WARNING',
          title: 'Blocking Delay',
          description: `Node "${node.data.label}" contains a blocking delay. Consider using async dispatch() instead.`,
          nodeId: node.id
        });
      }
      
      // Detect potential infinite while loops in actions
      if (action.includes('while(') || action.includes('while (')) {
        issues.push({
          id: `while-loop-${node.id}`,
          severity: 'WARNING',
          title: 'While Loop in Action',
          description: `Node "${node.data.label}" contains a while loop. This may block the FSM executor.`,
          nodeId: node.id
        });
      }
    });
    
    return issues;
  }

  /**
   * Calculate FSM complexity metrics
   */
  static calculateComplexity(nodes: Node[], edges: Edge[]): ComplexityMetrics {
    const nodeCount = nodes.length;
    const edgeCount = edges.length;
    
    // Cyclomatic complexity: E - N + 2P (P = connected components, assume 1)
    const cyclomaticComplexity = edgeCount - nodeCount + 2;
    
    // Branch factors
    const branchFactors = nodes.map(n => edges.filter(e => e.source === n.id).length);
    const maxBranchFactor = Math.max(...branchFactors, 0);
    const averageBranchFactor = branchFactors.length > 0 
      ? branchFactors.reduce((a, b) => a + b, 0) / branchFactors.length 
      : 0;
    
    // Calculate depth (longest path from input)
    const depth = this.calculateDepth(nodes, edges);
    
    // Find loops
    const loops = this.findAllCycles(nodes, edges);
    
    // Grade
    let grade: 'A' | 'B' | 'C' | 'D' | 'F';
    if (cyclomaticComplexity <= 5) grade = 'A';
    else if (cyclomaticComplexity <= 10) grade = 'B';
    else if (cyclomaticComplexity <= 15) grade = 'C';
    else if (cyclomaticComplexity <= 25) grade = 'D';
    else grade = 'F';
    
    return {
      cyclomaticComplexity,
      nodeCount,
      edgeCount,
      maxBranchFactor,
      averageBranchFactor,
      depth,
      loops,
      grade
    };
  }

  /**
   * Calculate the depth (longest path) of the FSM
   */
  private static calculateDepth(nodes: Node[], edges: Edge[]): number {
    const inputNode = nodes.find(n => n.data.type === 'input' || n.type === 'input');
    if (!inputNode) return 0;
    
    const distances = new Map<string, number>();
    nodes.forEach(n => distances.set(n.id, -1));
    distances.set(inputNode.id, 0);
    
    const queue = [inputNode.id];
    while (queue.length > 0) {
      const current = queue.shift()!;
      const currentDist = distances.get(current) || 0;
      
      edges.filter(e => e.source === current).forEach(edge => {
        const targetDist = distances.get(edge.target) || -1;
        if (targetDist < currentDist + 1) {
          distances.set(edge.target, currentDist + 1);
          queue.push(edge.target);
        }
      });
    }
    
    return Math.max(...Array.from(distances.values()), 0);
  }

  /**
   * Find all cycles in the graph
   */
  private static findAllCycles(nodes: Node[], edges: Edge[]): string[][] {
    const cycles: string[][] = [];
    const adjacency = new Map<string, string[]>();
    
    nodes.forEach(n => adjacency.set(n.id, []));
    edges.forEach(e => {
      const list = adjacency.get(e.source);
      if (list) list.push(e.target);
    });
    
    const visited = new Set<string>();
    const recursionStack: string[] = [];
    
    const dfs = (nodeId: string): void => {
      visited.add(nodeId);
      recursionStack.push(nodeId);
      
      const neighbors = adjacency.get(nodeId) || [];
      for (const neighbor of neighbors) {
        const stackIndex = recursionStack.indexOf(neighbor);
        if (stackIndex !== -1) {
          // Found cycle
          const cycle = recursionStack.slice(stackIndex);
          const cycleLabels = cycle.map(id => {
            const node = nodes.find(n => n.id === id);
            return node?.data.label || id;
          });
          cycles.push(cycleLabels);
        } else if (!visited.has(neighbor)) {
          dfs(neighbor);
        }
      }
      
      recursionStack.pop();
    };
    
    nodes.forEach(node => {
      visited.clear();
      dfs(node.id);
    });
    
    // Remove duplicates
    const uniqueCycles: string[][] = [];
    const seen = new Set<string>();
    
    for (const cycle of cycles) {
      const key = [...cycle].sort().join('|');
      if (!seen.has(key)) {
        seen.add(key);
        uniqueCycles.push(cycle);
      }
    }
    
    return uniqueCycles;
  }

  /**
   * Estimate timing characteristics
   */
  static estimateTiming(nodes: Node[], edges: Edge[]): TimingAnalysis {
    const depth = this.calculateDepth(nodes, edges);
    
    // Rough estimates based on typical embedded execution
    const avgActionTimeMs = 5; // Average time per node action
    const avgTransitionTimeMs = 1; // Average transition overhead
    
    const estimatedMinCycleMs = avgActionTimeMs + avgTransitionTimeMs;
    const estimatedMaxCycleMs = (avgActionTimeMs + avgTransitionTimeMs) * depth;
    
    // Find critical nodes (high connectivity or timer-based)
    const criticalNodes = nodes
      .filter(n => {
        const inDegree = edges.filter(e => e.target === n.id).length;
        const outDegree = edges.filter(e => e.source === n.id).length;
        return (inDegree + outDegree) > 4 || n.data.type === 'timer' || n.data.type === 'interrupt';
      })
      .map(n => n.data.label);
    
    // Find longest path
    const longestPath = this.findLongestPath(nodes, edges);
    
    return {
      estimatedMinCycleMs,
      estimatedMaxCycleMs,
      longestPath,
      criticalNodes
    };
  }

  /**
   * Find the longest path in the FSM
   */
  private static findLongestPath(nodes: Node[], edges: Edge[]): string[] {
    const inputNode = nodes.find(n => n.data.type === 'input' || n.type === 'input');
    if (!inputNode) return [];
    
    const paths: string[][] = [];
    
    const dfs = (current: string, path: string[], visited: Set<string>): void => {
      path.push(current);
      visited.add(current);
      
      const outgoing = edges.filter(e => e.source === current);
      
      if (outgoing.length === 0 || outgoing.every(e => visited.has(e.target))) {
        paths.push([...path]);
      } else {
        for (const edge of outgoing) {
          if (!visited.has(edge.target)) {
            dfs(edge.target, path, new Set(visited));
          }
        }
      }
      
      path.pop();
    };
    
    dfs(inputNode.id, [], new Set());
    
    const longest = paths.reduce((a, b) => a.length > b.length ? a : b, []);
    
    return longest.map(id => {
      const node = nodes.find(n => n.id === id);
      return node?.data.label || id;
    });
  }

  /**
   * Estimate hardware resource requirements
   */
  static estimateHardwareConstraints(nodes: Node[], edges: Edge[]): HardwareConstraints {
    const complexity = this.calculateComplexity(nodes, edges);
    
    // Rough estimates for embedded systems
    const bytesPerNode = 64; // Node metadata
    const bytesPerEdge = 32; // Edge/transition
    const bytesPerAction = 128; // Average action code
    
    const actionsCount = nodes.filter(n => n.data.entryAction || n.data.exitAction).length;
    
    const estimatedFlashBytes = 
      (nodes.length * bytesPerNode) + 
      (edges.length * bytesPerEdge) + 
      (actionsCount * bytesPerAction) +
      2048; // Base runtime
    
    const estimatedRamBytes = 
      (nodes.length * 16) + // State tracking
      512 + // Context variables
      256; // Stack/heap
    
    // MIPS estimate based on complexity
    const estimatedMipsRequired = complexity.cyclomaticComplexity * 0.5;
    
    // Suitability
    const suitableFor: string[] = [];
    
    if (estimatedFlashBytes < 8192 && estimatedRamBytes < 512) {
      suitableFor.push('ATtiny85', 'ATmega328');
    }
    if (estimatedFlashBytes < 65536 && estimatedRamBytes < 8192) {
      suitableFor.push('ESP8266', 'STM32F0');
    }
    if (estimatedFlashBytes < 262144) {
      suitableFor.push('ESP32', 'STM32F4', 'nRF52', 'RP2040');
    }
    suitableFor.push('All Cortex-M4+');
    
    return {
      estimatedFlashBytes,
      estimatedRamBytes,
      estimatedMipsRequired,
      suitableFor
    };
  }
}