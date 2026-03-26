/**
 * Neurostate 2.0 - Director / Meta-Orchestrator
 * Central orchestrator for agent swarm coordination
 */

import { EventEmitter } from 'eventemitter3';
import {
  UserRequest,
  Workflow,
  SubTask,
  Intent,
  IntentType,
  TaskAssignment,
  TaskResult,
  AgentDefinition,
  AGENT_DEFINITIONS,
  ToolInfo,
  ToolCategory,
  OrchestratorMetrics,
  HealthStatus,
  AgentMessageType,
} from '@neurostate/types';
import { createMessageBus, MessageBus } from '@neurostate/message-bus';
import { McpClientManager, McpClientConfig } from '@neurostate/mcp-client';
import { 
  createLLMAdapter, 
  LLMAdapter, 
  LLMConfig,
  defaultConfigs as llmDefaultConfigs
} from '@neurostate/llm-adapter';

// ============================================================================
// ORCHESTRATOR CONFIGURATION
// ============================================================================

export interface OrchestratorConfig {
  useRedis?: boolean;
  redisUrl?: string;
  mcpServers?: McpClientConfig[];
  enableSelfReflection?: boolean;
  maxConcurrentTasks?: number;
  llmConfig?: LLMConfig;
}

interface ActiveWorkflow {
  workflow: Workflow;
  activeTasks: Map<string, NodeJS.Timeout>;
  completedSubtasks: Set<string>;
}

// ============================================================================
// DIRECTOR / META-ORCHESTRATOR
// ============================================================================

export class Director extends EventEmitter {
  private messageBus: MessageBus | null = null;
  private mcpClientManager: McpClientManager;
  private llmAdapter: LLMAdapter | null = null;
  private workflows: Map<string, ActiveWorkflow> = new Map();
  private toolRegistry: Map<string, ToolInfo> = new Map();
  private agentRegistry: Map<string, AgentDefinition> = new Map();
  private metrics: OrchestratorMetrics = {
    totalWorkflows: 0,
    completedWorkflows: 0,
    failedWorkflows: 0,
    averageWorkflowDurationMs: 0,
    totalToolCalls: 0,
    totalTokensUsed: 0,
  };
  private config: OrchestratorConfig;
  private isRunning = false;

  constructor(config: OrchestratorConfig = {}) {
    super();
    this.config = {
      useRedis: false,
      enableSelfReflection: true,
      maxConcurrentTasks: 10,
      ...config,
    };
    this.mcpClientManager = new McpClientManager();
    
    // Initialize LLM adapter if configured
    if (this.config.llmConfig) {
      try {
        this.llmAdapter = createLLMAdapter(this.config.llmConfig);
        console.log(`[Director] LLM adapter initialized: ${this.config.llmConfig.provider}/${this.config.llmConfig.model}`);
      } catch (error) {
        console.warn('[Director] Failed to initialize LLM adapter, falling back to keyword-based classification:', error);
      }
    }
    
    // Register all agents
    Object.values(AGENT_DEFINITIONS).forEach(agent => {
      this.agentRegistry.set(agent.id, agent);
    });
  }

  // ============================================================================
  // INITIALIZATION
  // ============================================================================

  async initialize(): Promise<void> {
    console.log('[Director] Initializing...');

    // Initialize message bus
    this.messageBus = await createMessageBus(
      { redisUrl: this.config.redisUrl || 'redis://localhost:6379' },
      this.config.useRedis || false
    );

    // Register orchestrator with message bus
    await this.messageBus.registerAgent('orchestrator');

    // Subscribe to task results
    await this.messageBus.subscribe('task-result', this.handleTaskResult.bind(this));
    await this.messageBus.subscribe('agent-health', this.handleAgentHealth.bind(this));

    // Initialize MCP servers
    if (this.config.mcpServers) {
      for (const serverConfig of this.config.mcpServers) {
        try {
          const client = await this.mcpClientManager.registerClient(serverConfig);
          await this.registerMcpTools(serverConfig.serverId, client.listTools());
          console.log(`[Director] Registered MCP server: ${serverConfig.serverId}`);
        } catch (error) {
          console.error(`[Director] Failed to register MCP server ${serverConfig.serverId}:`, error);
        }
      }
    }

    this.isRunning = true;
    console.log('[Director] Initialized successfully');
  }

  private async registerMcpTools(serverId: string, tools: any[]): Promise<void> {
    for (const tool of tools) {
      const category = this.inferToolCategory(tool.name);
      this.toolRegistry.set(tool.name, {
        name: tool.name,
        description: tool.description,
        inputSchema: tool.inputSchema,
        serverId,
        category,
        capabilities: [],
      });
    }
  }

  private inferToolCategory(toolName: string): ToolCategory {
    if (toolName.includes('hardware') || toolName.includes('pinout') || toolName.includes('vecu')) {
      return 'HARDWARE';
    }
    if (toolName.includes('test') || toolName.includes('verify')) {
      return 'TEST';
    }
    if (toolName.includes('deploy') || toolName.includes('flash')) {
      return 'DEPLOY';
    }
    if (toolName.includes('ml') || toolName.includes('nas') || toolName.includes('quantize')) {
      return 'AI_ML';
    }
    if (toolName.includes('security') || toolName.includes('vulnerability')) {
      return 'SECURITY';
    }
    if (toolName.includes('doc') || toolName.includes('template')) {
      return 'DOCS';
    }
    return 'CODE';
  }

  // ============================================================================
  // INTENT CLASSIFICATION & TASK DECOMPOSITION
  // ============================================================================

  async processUserRequest(request: UserRequest): Promise<string> {
    console.log(`[Director] Processing request: ${request.text}`);

    // Classify intent
    const intent = await this.classifyIntent(request.text);
    console.log(`[Director] Classified intent: ${intent.type} (confidence: ${intent.confidence})`);

    // Create workflow based on intent
    const workflow = await this.createWorkflow(intent, request);

    // Decompose into subtasks and assign to agents
    await this.decomposeAndAssign(workflow);

    return workflow.id;
  }

  private async classifyIntent(text: string): Promise<Intent> {
    // Try LLM-based classification if adapter is available
    if (this.llmAdapter) {
      try {
        console.log('[Director] Using LLM for intent classification...');
        const result = await this.llmAdapter.classifyIntent(text);
        
        // Map LLM intent to our IntentType
        const intentTypeMap: Record<string, IntentType> = {
          'code_generation': 'GENERATE_CODE',
          'code_refactoring': 'REFACTOR_CODE',
          'testing': 'RUN_TESTS',
          'debugging': 'DEBUG_ISSUE',
          'architecture': 'DESIGN_SYSTEM',
          'documentation': 'DOCUMENT_PROJECT',
          'deployment': 'DEPLOY_FIRMWARE',
          'security': 'SECURITY_AUDIT',
          'hardware': 'VALIDATE_HARDWARE',
          'optimization': 'OPTIMIZE_PERFORMANCE',
          'analysis': 'ANALYZE_ARCHITECTURE',
          'configuration': 'GENERATE_CODE',
        };

        const mappedType = intentTypeMap[result.intent] || 'GENERATE_CODE';
        
        return {
          type: mappedType,
          confidence: result.confidence,
          entities: result.entities || {},
          suggestedWorkflow: result.suggestedWorkflow,
        };
      } catch (error) {
        console.warn('[Director] LLM classification failed, falling back to keyword-based:', error);
      }
    }

    // Fallback to keyword-based classification
    const textLower = text.toLowerCase();
    
    const intentMap: Record<string, IntentType> = {
      'generate': 'GENERATE_CODE',
      'create': 'GENERATE_CODE',
      'write': 'GENERATE_CODE',
      'validate': 'VALIDATE_HARDWARE',
      'check': 'VALIDATE_HARDWARE',
      'test': 'RUN_TESTS',
      'run test': 'RUN_TESTS',
      'deploy': 'DEPLOY_FIRMWARE',
      'flash': 'DEPLOY_FIRMWARE',
      'optimize': 'OPTIMIZE_PERFORMANCE',
      'improve': 'OPTIMIZE_PERFORMANCE',
      'debug': 'DEBUG_ISSUE',
      'fix': 'DEBUG_ISSUE',
      'document': 'DOCUMENT_PROJECT',
      'design': 'DESIGN_SYSTEM',
    };

    for (const [keyword, intentType] of Object.entries(intentMap)) {
      if (textLower.includes(keyword)) {
        return {
          type: intentType,
          confidence: 0.8,
          entities: {},
        };
      }
    }

    // Default to code generation
    return {
      type: 'GENERATE_CODE',
      confidence: 0.5,
      entities: {},
    };
  }

  private async createWorkflow(intent: Intent, request: UserRequest): Promise<Workflow> {
    const workflowId = `workflow-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    
    const workflow: Workflow = {
      id: workflowId,
      name: `${intent.type} Workflow`,
      subtasks: [],
      status: 'PENDING',
      createdAt: new Date(),
    };

    this.workflows.set(workflowId, {
      workflow,
      activeTasks: new Map(),
      completedSubtasks: new Set(),
    });

    this.metrics.totalWorkflows++;

    return workflow;
  }

  private async decomposeAndAssign(workflow: Workflow): Promise<void> {
    const activeWorkflow = this.workflows.get(workflow.id);
    if (!activeWorkflow) return;

    // Generate subtasks based on workflow type
    const subtasks = this.generateSubtasks(workflow);
    workflow.subtasks = subtasks;
    workflow.status = 'RUNNING';

    console.log(`[Director] Decomposed workflow ${workflow.id} into ${subtasks.length} subtasks`);

    // Assign each subtask to appropriate agent
    for (const subtask of subtasks) {
      await this.assignTaskToAgent(workflow.id, subtask);
    }
  }

  private generateSubtasks(workflow: Workflow): SubTask[] {
    // Generate subtasks based on workflow type
    const subtasks: SubTask[] = [];

    switch (workflow.name) {
      case 'GENERATE_CODE Workflow':
        subtasks.push({
          id: `task-${Date.now()}-1`,
          name: 'Generate driver code',
          agentId: 'coder',
          requirements: {
            capabilities: ['canGenerateCode'],
            priority: 'HIGH',
            estimatedDurationMs: 5000,
            requiredTools: ['generate_driver_code'],
          },
          payload: { type: 'driver_generation' },
        });
        subtasks.push({
          id: `task-${Date.now()}-2`,
          name: 'Review generated code',
          agentId: 'ghost',
          requirements: {
            capabilities: ['canVerify'],
            priority: 'MEDIUM',
            estimatedDurationMs: 3000,
            requiredTools: ['static_analysis'],
          },
          payload: { type: 'code_review' },
          dependencies: [`task-${Date.now()}-1`],
        });
        break;

      case 'VALIDATE_HARDWARE Workflow':
        subtasks.push({
          id: `task-${Date.now()}-1`,
          name: 'Validate pinout',
          agentId: 'hardware',
          requirements: {
            capabilities: ['canAnalyzeHardware'],
            priority: 'HIGH',
            estimatedDurationMs: 3000,
            requiredTools: ['validate_pinout'],
          },
          payload: { type: 'pinout_validation' },
        });
        break;

      default:
        subtasks.push({
          id: `task-${Date.now()}-1`,
          name: 'Process request',
          agentId: 'super',
          requirements: {
            capabilities: ['canGenerateCode', 'canVerify'],
            priority: 'MEDIUM',
            estimatedDurationMs: 5000,
            requiredTools: [],
          },
          payload: { type: 'general' },
        });
    }

    return subtasks;
  }

  private async assignTaskToAgent(workflowId: string, subtask: SubTask): Promise<void> {
    const agent = this.agentRegistry.get(subtask.agentId);
    if (!agent) {
      console.error(`[Director] Agent not found: ${subtask.agentId}`);
      return;
    }

    const taskAssignment: TaskAssignment = {
      taskId: subtask.id,
      workflowId,
      taskType: 'CODE_GENERATION',
      priority: subtask.requirements.priority,
      payload: subtask.payload,
      requiredCapabilities: subtask.requirements.capabilities,
    };

    console.log(`[Director] Assigning task ${subtask.id} to agent ${subtask.agentId}`);

    // Send task assignment via message bus
    if (this.messageBus) {
      await this.messageBus.publish('task-assign', {
        type: AgentMessageType.TASK_ASSIGN,
        payload: taskAssignment,
        from: 'orchestrator',
        to: subtask.agentId,
        timestamp: new Date(),
      });
    }

    // Set timeout for task
    const activeWorkflow = this.workflows.get(workflowId);
    if (activeWorkflow) {
      const timeout = setTimeout(() => {
        console.warn(`[Director] Task ${subtask.id} timed out`);
      }, subtask.requirements.estimatedDurationMs * 2);
      
      activeWorkflow.activeTasks.set(subtask.id, timeout);
    }
  }

  // ============================================================================
  // TASK RESULT HANDLING
  // ============================================================================

  private async handleTaskResult(message: any): Promise<void> {
    const result = message.payload as TaskResult;
    console.log(`[Director] Received task result: ${result.taskId} - ${result.status}`);

    const workflowEntry = Array.from(this.workflows.values()).find(entry => 
      entry.workflow.subtasks.some(st => st.id === result.taskId)
    );

    if (!workflowEntry) {
      console.warn(`[Director] Unknown task result for: ${result.taskId}`);
      return;
    }

    // Clear task timeout
    const timeout = workflowEntry.activeTasks.get(result.taskId);
    if (timeout) {
      clearTimeout(timeout);
      workflowEntry.activeTasks.delete(result.taskId);
    }

    workflowEntry.completedSubtasks.add(result.taskId);

    // Check if workflow is complete
    const allCompleted = workflowEntry.workflow.subtasks.every(st =>
      workflowEntry.completedSubtasks.has(st.id)
    );

    if (allCompleted) {
      await this.completeWorkflow(workflowEntry.workflow.id);
    }
  }

  private async completeWorkflow(workflowId: string): Promise<void> {
    const workflowEntry = this.workflows.get(workflowId);
    if (!workflowEntry) return;

    workflowEntry.workflow.status = 'COMPLETED';
    workflowEntry.workflow.completedAt = new Date();

    const duration = workflowEntry.workflow.completedAt.getTime() - workflowEntry.workflow.createdAt.getTime();
    
    this.metrics.completedWorkflows++;
    this.metrics.averageWorkflowDurationMs = 
      (this.metrics.averageWorkflowDurationMs * (this.metrics.completedWorkflows - 1) + duration) /
      this.metrics.completedWorkflows;

    console.log(`[Director] Workflow ${workflowId} completed in ${duration}ms`);

    this.emit('workflow-complete', workflowEntry.workflow);

    // Clean up
    this.workflows.delete(workflowId);
  }

  // ============================================================================
  // AGENT HEALTH MONITORING
  // ============================================================================

  private async handleAgentHealth(message: any): Promise<void> {
    const health = message.payload as HealthStatus;
    const agentId = message.from;

    if (health.status === 'UNHEALTHY') {
      console.warn(`[Director] Agent ${agentId} is unhealthy`);
      // Could trigger agent recovery or task reassignment
    }
  }

  // ============================================================================
  // METRICS & MONITORING
  // ============================================================================

  getMetrics(): OrchestratorMetrics {
    return { ...this.metrics };
  }

  getActiveWorkflows(): number {
    return this.workflows.size;
  }

  getRegisteredAgents(): number {
    return this.agentRegistry.size;
  }

  getRegisteredTools(): number {
    return this.toolRegistry.size;
  }

  // ============================================================================
  // SHUTDOWN
  // ============================================================================

  async shutdown(): Promise<void> {
    console.log('[Director] Shutting down...');
    this.isRunning = false;

    // Wait for active workflows to complete (with timeout)
    const shutdownTimeout = setTimeout(() => {
      console.warn('[Director] Forced shutdown due to timeout');
    }, 10000);

    while (this.workflows.size > 0 && !shutdownTimeout) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    clearTimeout(shutdownTimeout);

    // Disconnect message bus
    if (this.messageBus) {
      await this.messageBus.disconnect();
    }

    // Disconnect MCP clients
    await this.mcpClientManager.disconnectAll();

    console.log('[Director] Shutdown complete');
  }
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

async function main() {
  const director = new Director({
    useRedis: false,
    mcpServers: [
      {
        serverId: 'mcp-code-server',
        transport: 'stdio',
        command: 'tsx',
        args: ['mcp-servers/mcp-code-server/src/index.ts'],
      },
      {
        serverId: 'mcp-hardware-server',
        transport: 'stdio',
        command: 'tsx',
        args: ['mcp-servers/mcp-hardware-server/src/index.ts'],
      },
    ],
  });

  await director.initialize();

  // Example: Process a user request
  const workflowId = await director.processUserRequest({
    id: 'req-1',
    text: 'Generate UART driver for STM32F401',
    timestamp: new Date(),
  });

  console.log(`Created workflow: ${workflowId}`);

  // Keep running
  process.on('SIGINT', async () => {
    await director.shutdown();
    process.exit(0);
  });
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export default Director;
