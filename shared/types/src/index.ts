/**
 * Neurostate 2.0 - Shared Types for Agent Swarm & MCP
 * Based on NEUROSTATE_ENHANCED_SYSTEM_DESIGN.md and AI_AGENT_SWARM_MCP_IMPLEMENTATION_GUIDE.md
 */

import { z } from 'zod';

// ============================================================================
// MCP PROTOCOL TYPES
// ============================================================================

export interface McpTool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface McpResource {
  uri: string;
  name: string;
  mimeType: string;
  description?: string;
}

export interface McpPrompt {
  name: string;
  description: string;
  arguments: Array<{
    name: string;
    description: string;
    required: boolean;
  }>;
}

export interface McpToolCall {
  toolName: string;
  arguments: Record<string, unknown>;
  _meta?: {
    progressToken?: string | number;
  };
}

export interface McpToolResult {
  content: Array<{
    type: 'text' | 'image' | 'resource';
    text?: string;
    data?: unknown;
    mimeType?: string;
  }>;
  isError?: boolean;
}

export interface McpInitializeRequest {
  protocolVersion: string;
  capabilities: {
    roots?: { listChanged?: boolean };
    sampling?: object;
  };
  clientInfo: {
    name: string;
    version: string;
  };
}

export interface McpInitializeResponse {
  protocolVersion: string;
  capabilities: {
    tools?: { listChanged?: boolean };
    resources?: { subscribe?: boolean; listChanged?: boolean };
    prompts?: { listChanged?: boolean };
  };
  serverInfo: {
    name: string;
    version: string;
  };
}

// ============================================================================
// AGENT TYPES
// ============================================================================

export type AgentTier = 'CORE' | 'QUALITY' | 'ML' | 'DEVOPS' | 'UX' | 'META';

export type AgentState = 
  | 'CREATED'
  | 'INITIALIZING'
  | 'IDLE'
  | 'BUSY'
  | 'PAUSED'
  | 'SHUTTING_DOWN'
  | 'TERMINATED'
  | 'ERROR';

export interface AgentCapabilities {
  canGenerateCode: boolean;
  canAnalyzeHardware: boolean;
  canRunTests: boolean;
  canDeploy: boolean;
  canOptimize: boolean;
  canVerify: boolean;
  requiredTools: string[];
}

export interface AgentDefinition {
  id: string;
  name: string;
  tier: AgentTier;
  description: string;
  capabilities: AgentCapabilities;
  mcpServers: string[];
}

// ============================================================================
// TASK & WORKFLOW TYPES
// ============================================================================

export type TaskType = 
  | 'CODE_GENERATION'
  | 'HARDWARE_VALIDATION'
  | 'TEST_EXECUTION'
  | 'DEPLOYMENT'
  | 'OPTIMIZATION'
  | 'VERIFICATION'
  | 'DOCUMENTATION'
  | 'ANALYSIS';

export type TaskStatus = 
  | 'PENDING'
  | 'IN_PROGRESS'
  | 'COMPLETED'
  | 'FAILED'
  | 'CANCELLED';

export type Priority = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface TaskAssignment {
  taskId: string;
  workflowId: string;
  taskType: TaskType;
  priority: Priority;
  payload: Record<string, unknown>;
  deadline?: Date;
  parentTask?: string;
  requiredCapabilities: string[];
}

export interface TaskResult {
  taskId: string;
  status: TaskStatus;
  output: Record<string, unknown>;
  metrics: TaskMetrics;
  artifacts: Artifact[];
  error?: string;
}

export interface TaskMetrics {
  durationMs: number;
  tokensUsed?: number;
  toolCalls?: number;
  retries?: number;
}

export interface Artifact {
  id: string;
  type: 'code' | 'config' | 'test' | 'document' | 'binary';
  name: string;
  content: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// WORKFLOW TYPES
// ============================================================================

export interface Workflow {
  id: string;
  name: string;
  subtasks: SubTask[];
  status: WorkflowStatus;
  createdAt: Date;
  completedAt?: Date;
}

export type WorkflowStatus = 'PENDING' | 'RUNNING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';

export interface SubTask {
  id: string;
  name: string;
  agentId: string;
  requirements: TaskRequirements;
  payload: Record<string, unknown>;
  deadline?: Date;
  dependencies?: string[];
}

export interface TaskRequirements {
  capabilities: string[];
  priority: Priority;
  estimatedDurationMs: number;
  requiredTools: string[];
}

// ============================================================================
// ORCHESTRATOR TYPES
// ============================================================================

export interface UserRequest {
  id: string;
  text: string;
  context?: Record<string, unknown>;
  timestamp: Date;
}

export interface OrchestratorResult {
  workflowId: string;
  output: Record<string, unknown>;
  metrics: TaskMetrics;
}

export interface Intent {
  type: IntentType;
  confidence: number;
  entities: Record<string, string>;
}

export type IntentType = 
  | 'GENERATE_CODE'
  | 'VALIDATE_HARDWARE'
  | 'RUN_TESTS'
  | 'DEPLOY_FIRMWARE'
  | 'OPTIMIZE_PERFORMANCE'
  | 'DEBUG_ISSUE'
  | 'DOCUMENT_PROJECT'
  | 'DESIGN_SYSTEM';

// ============================================================================
// TOOL REGISTRY TYPES
// ============================================================================

export interface ToolInfo {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
  serverId: string;
  category: ToolCategory;
  capabilities: string[];
}

export type ToolCategory = 
  | 'HARDWARE'
  | 'CODE'
  | 'TEST'
  | 'DEPLOY'
  | 'AI_ML'
  | 'SECURITY'
  | 'DOCS'
  | 'ANALYTICS';

export interface ToolRegistryEvent {
  type: 'TOOLS_UPDATED' | 'SERVER_ADDED' | 'SERVER_REMOVED';
  count?: number;
  serverId?: string;
}

// ============================================================================
// MESSAGE BUS TYPES
// ============================================================================

export enum AgentMessageType {
  TASK_ASSIGN = 'TASK_ASSIGN',
  TASK_RESULT = 'TASK_RESULT',
  TASK_CANCEL = 'TASK_CANCEL',
  REQUEST_COLLABORATION = 'REQUEST_COLLABORATION',
  COLLABORATION_RESPONSE = 'COLLABORATION_RESPONSE',
  SHARE_CONTEXT = 'SHARE_CONTEXT',
  QUERY_CONTEXT = 'QUERY_CONTEXT',
  HEALTH_CHECK = 'HEALTH_CHECK',
  HEALTH_RESPONSE = 'HEALTH_RESPONSE',
  SHUTDOWN = 'SHUTDOWN',
  MCP_TOOL_CALL = 'MCP_TOOL_CALL',
  MCP_TOOL_RESULT = 'MCP_TOOL_RESULT',
  MCP_RESOURCE_REQUEST = 'MCP_RESOURCE_REQUEST',
}

export interface AgentMessage {
  type: AgentMessageType;
  payload: unknown;
  from: string;
  to: string;
  timestamp: Date;
  correlationId?: string;
}

// ============================================================================
// HEALTH & METRICS TYPES
// ============================================================================

export interface HealthStatus {
  status: 'HEALTHY' | 'DEGRADED' | 'UNHEALTHY';
  uptime: number;
  activeTasks: number;
  queuedTasks: number;
  errors: string[];
  lastHeartbeat: Date;
}

export interface OrchestratorMetrics {
  totalWorkflows: number;
  completedWorkflows: number;
  failedWorkflows: number;
  averageWorkflowDurationMs: number;
  totalToolCalls: number;
  totalTokensUsed: number;
}

// ============================================================================
// SECURITY TYPES
// ============================================================================

export interface SecurityContext {
  agentId: string;
  allowedTools: Set<string>;
  rateLimits: Map<string, number>;
  auditLog: AuditEntry[];
}

export interface AuditEntry {
  timestamp: Date;
  agentId: string;
  action: string;
  details: string;
  result: 'SUCCESS' | 'FAILURE' | 'DENIED';
}

// ============================================================================
// AGENT DEFINITIONS (25+ AGENTS)
// ============================================================================

export const AGENT_DEFINITIONS: Record<string, AgentDefinition> = {
  // TIER 1: CORE DEVELOPMENT AGENTS
  architect: {
    id: 'architect',
    name: 'Architect Agent',
    tier: 'CORE',
    description: 'System design & pattern selection',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: true,
      requiredTools: ['generate_code', 'analyze_architecture', 'select_pattern'],
    },
    mcpServers: ['mcp-code-server', 'mcp-hardware-server', 'mcp-ml-server'],
  },
  coder: {
    id: 'coder',
    name: 'Coder Agent',
    tier: 'CORE',
    description: 'Code generation (C/C++/Rust/Zig)',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: false,
      canRunTests: true,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['generate_driver_code', 'refactor_code', 'optimize_code'],
    },
    mcpServers: ['mcp-code-server', 'mcp-test-server'],
  },
  hardware: {
    id: 'hardware',
    name: 'Hardware Agent',
    tier: 'CORE',
    description: 'Pinout validation & electrical analysis',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['validate_pinout', 'analyze_timing', 'check_electrical'],
    },
    mcpServers: ['mcp-hardware-server', 'mcp-analytics-server'],
  },
  fsm: {
    id: 'fsm',
    name: 'FSM Agent',
    tier: 'CORE',
    description: 'State machine optimization',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: false,
      canRunTests: true,
      canDeploy: false,
      canOptimize: true,
      canVerify: true,
      requiredTools: ['optimize_fsm', 'verify_state_machine'],
    },
    mcpServers: ['mcp-code-server', 'mcp-test-server'],
  },
  canvas: {
    id: 'canvas',
    name: 'Canvas Agent',
    tier: 'CORE',
    description: 'Visual graph manipulation',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['auto_layout', 'manage_nodes', 'optimize_graph'],
    },
    mcpServers: ['mcp-ui-server', 'mcp-analytics-server'],
  },

  // TIER 2: QUALITY & VALIDATION AGENTS
  ghost: {
    id: 'ghost',
    name: 'Ghost Agent',
    tier: 'QUALITY',
    description: 'Static analysis & dead code detection',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['static_analysis', 'detect_dead_code', 'complexity_analysis'],
    },
    mcpServers: ['mcp-code-server', 'mcp-security-server'],
  },
  formal: {
    id: 'formal',
    name: 'Formal Agent',
    tier: 'QUALITY',
    description: 'Formal verification & proof generation',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['kani_verify', 'cbmc_verify', 'prove_safety'],
    },
    mcpServers: ['mcp-code-server', 'mcp-test-server'],
  },
  test: {
    id: 'test',
    name: 'Test Agent',
    tier: 'QUALITY',
    description: 'Automated test generation (unit/HIL)',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: false,
      canRunTests: true,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['generate_unit_tests', 'generate_hil_tests', 'run_tests'],
    },
    mcpServers: ['mcp-test-server', 'mcp-code-server'],
  },
  security: {
    id: 'security',
    name: 'Security Agent',
    tier: 'QUALITY',
    description: 'Vulnerability scanning & threat modeling',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['threat_model', 'vulnerability_scan', 'secure_code_review'],
    },
    mcpServers: ['mcp-security-server', 'mcp-code-server'],
  },
  compliance: {
    id: 'compliance',
    name: 'Compliance Agent',
    tier: 'QUALITY',
    description: 'MISRA/CERT/CWE compliance checking',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['misra_check', 'cert_check', 'cwe_check'],
    },
    mcpServers: ['mcp-code-server', 'mcp-docs-server'],
  },

  // TIER 3: AI/ML & ADVANCED FEATURES
  tinyml: {
    id: 'tinyml',
    name: 'TinyML Agent',
    tier: 'ML',
    description: 'Neural architecture search & deployment',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: true,
      canRunTests: true,
      canDeploy: true,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['nas_search', 'quantize_model', 'deploy_tinyml'],
    },
    mcpServers: ['mcp-ml-server', 'mcp-hardware-server'],
  },
  nas: {
    id: 'nas',
    name: 'NAS Agent',
    tier: 'ML',
    description: 'Hardware-aware model optimization',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['hardware_aware_nas', 'constraint_optimize'],
    },
    mcpServers: ['mcp-ml-server', 'mcp-hardware-server'],
  },
  digitalTwin: {
    id: 'digitalTwin',
    name: 'Digital Twin Agent',
    tier: 'ML',
    description: 'vECU creation & twin synchronization',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: true,
      canRunTests: true,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['generate_vecu', 'sync_twin', 'simulate_peripheral'],
    },
    mcpServers: ['mcp-hardware-server', 'mcp-analytics-server'],
  },
  predictive: {
    id: 'predictive',
    name: 'Predictive Agent',
    tier: 'ML',
    description: 'Failure prediction & anomaly detection',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['predict_failure', 'detect_anomaly', 'forecast'],
    },
    mcpServers: ['mcp-analytics-server', 'mcp-ml-server'],
  },
  optimization: {
    id: 'optimization',
    name: 'Optimization Agent',
    tier: 'ML',
    description: 'Power/performance trade-off analysis',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['power_analysis', 'performance_profile', 'auto_tune'],
    },
    mcpServers: ['mcp-code-server', 'mcp-hardware-server'],
  },

  // TIER 4: DEVOPS & OPERATIONS
  build: {
    id: 'build',
    name: 'Build Agent',
    tier: 'DEVOPS',
    description: 'Toolchain management & CI/CD orchestration',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: true,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['setup_toolchain', 'run_cicd', 'manage_dependencies'],
    },
    mcpServers: ['mcp-code-server', 'mcp-deploy-server'],
  },
  deploy: {
    id: 'deploy',
    name: 'Deploy Agent',
    tier: 'DEVOPS',
    description: 'Flashing, OTA & bootloader management',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: true,
      canOptimize: false,
      canVerify: false,
      requiredTools: ['flash_device', 'ota_update', 'manage_bootloader'],
    },
    mcpServers: ['mcp-deploy-server', 'mcp-hardware-server'],
  },
  debug: {
    id: 'debug',
    name: 'Debug Agent',
    tier: 'DEVOPS',
    description: 'RTT/Serial analysis & crash investigation',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: true,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['analyze_rtt', 'parse_crash', 'trace_execution'],
    },
    mcpServers: ['mcp-hardware-server', 'mcp-analytics-server'],
  },
  monitor: {
    id: 'monitor',
    name: 'Monitor Agent',
    tier: 'DEVOPS',
    description: 'Telemetry collection & alerting',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['collect_telemetry', 'setup_alerts', 'generate_dashboard'],
    },
    mcpServers: ['mcp-analytics-server', 'mcp-hardware-server'],
  },
  sre: {
    id: 'sre',
    name: 'SRE Agent',
    tier: 'DEVOPS',
    description: 'Reliability engineering & SLO management',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: true,
      requiredTools: ['define_slo', 'track_reliability', 'incident_response'],
    },
    mcpServers: ['mcp-analytics-server', 'mcp-deploy-server'],
  },

  // TIER 5: USER EXPERIENCE & COLLABORATION
  voice: {
    id: 'voice',
    name: 'Voice Agent',
    tier: 'UX',
    description: 'Hands-free voice commands',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: false,
      requiredTools: ['speech_to_text', 'execute_voice_command'],
    },
    mcpServers: ['mcp-ui-server', 'mcp-analytics-server'],
  },
  docs: {
    id: 'docs',
    name: 'Docs Agent',
    tier: 'UX',
    description: 'Documentation generation & maintenance',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: false,
      requiredTools: ['generate_docs', 'update_api_docs', 'create_tutorial'],
    },
    mcpServers: ['mcp-docs-server', 'mcp-code-server'],
  },
  tutorial: {
    id: 'tutorial',
    name: 'Tutorial Agent',
    tier: 'UX',
    description: 'Interactive onboarding & learning paths',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: false,
      requiredTools: ['create_learning_path', 'interactive_guide'],
    },
    mcpServers: ['mcp-ui-server', 'mcp-docs-server'],
  },
  collaboration: {
    id: 'collaboration',
    name: 'Collaboration Agent',
    tier: 'UX',
    description: 'Real-time multi-user editing',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: false,
      requiredTools: ['sync_editing', 'resolve_conflicts', 'share_context'],
    },
    mcpServers: ['mcp-ui-server', 'mcp-analytics-server'],
  },
  feedback: {
    id: 'feedback',
    name: 'Feedback Agent',
    tier: 'UX',
    description: 'User behavior analysis & UX improvements',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['analyze_behavior', 'suggest_improvements'],
    },
    mcpServers: ['mcp-analytics-server', 'mcp-ui-server'],
  },

  // TIER 6: META & SYSTEM AGENTS
  selfReflection: {
    id: 'selfReflection',
    name: 'Self-Reflection Agent',
    tier: 'META',
    description: 'Output quality review & improvement',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['review_output', 'suggest_improvements'],
    },
    mcpServers: ['all'],
  },
  consensus: {
    id: 'consensus',
    name: 'Consensus Agent',
    tier: 'META',
    description: 'Conflict resolution between agents',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['resolve_conflict', 'build_consensus'],
    },
    mcpServers: ['all'],
  },
  learning: {
    id: 'learning',
    name: 'Learning Agent',
    tier: 'META',
    description: 'Continuous improvement from feedback',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: true,
      canVerify: false,
      requiredTools: ['learn_from_feedback', 'update_models'],
    },
    mcpServers: ['all'],
  },
  super: {
    id: 'super',
    name: 'Super Agent',
    tier: 'META',
    description: 'General-purpose fallback agent',
    capabilities: {
      canGenerateCode: true,
      canAnalyzeHardware: true,
      canRunTests: true,
      canDeploy: true,
      canOptimize: true,
      canVerify: true,
      requiredTools: ['*'],
    },
    mcpServers: ['all'],
  },
  director: {
    id: 'director',
    name: 'Director Agent',
    tier: 'META',
    description: 'Central orchestration, task routing, coordination',
    capabilities: {
      canGenerateCode: false,
      canAnalyzeHardware: false,
      canRunTests: false,
      canDeploy: false,
      canOptimize: false,
      canVerify: true,
      requiredTools: ['classify_intent', 'select_agent', 'decompose_task'],
    },
    mcpServers: ['all'],
  },
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export function getAgentById(id: string): AgentDefinition | undefined {
  return AGENT_DEFINITIONS[id];
}

export function getAgentsByTier(tier: AgentTier): AgentDefinition[] {
  return Object.values(AGENT_DEFINITIONS).filter(agent => agent.tier === tier);
}

export function getAgentsWithCapability(capability: keyof AgentCapabilities): AgentDefinition[] {
  return Object.values(AGENT_DEFINITIONS).filter(agent => agent.capabilities[capability]);
}

export function generateTaskId(): string {
  return `task_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

export function generateWorkflowId(): string {
  return `workflow_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}
