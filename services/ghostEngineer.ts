import { Edge, Node } from 'reactflow';
import { GhostIssue } from '../types';

export class GhostEngineer {
  static analyze(nodes: Node[], edges: Edge[]): GhostIssue[] {
    const issues: GhostIssue[] = [];

    // 1. Dead End Detection
    // Any node that is NOT a final state (input, process) and has 0 outgoing edges.
    nodes.forEach(node => {
      // Assuming 'output' or 'error' types are final by default.
      const isFinal = node.data.type === 'output' || node.data.type === 'error' || node.data.label.toLowerCase().includes('end');
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

    // 2. Unreachable State Detection (BFS)
    const inputNode = nodes.find(n => n.data.type === 'input' || n.type === 'input');
    if (inputNode) {
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
    } else {
      issues.push({
        id: 'no-entry',
        severity: 'CRITICAL',
        title: 'No Entry Point',
        description: 'No node is marked as "input" type. The FSM cannot start.',
      });
    }

    // 3. Determinism Check (Race Conditions)
    nodes.forEach(node => {
      const outgoing = edges.filter(e => e.source === node.id);
      const labels = outgoing.map(e => (e.label as string || '').toLowerCase());
      const uniqueLabels = new Set(labels);
      
      if (uniqueLabels.size !== labels.length) {
         issues.push({
            id: `race-${node.id}`,
            severity: 'CRITICAL',
            title: 'Race Condition (Nondeterministic)',
            description: `Node "${node.data.label}" has multiple transitions with the exact same event trigger.`,
            nodeId: node.id
          });
      }
    });

    return issues;
  }
}