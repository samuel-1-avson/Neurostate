/**
 * Neurostate Coder Agent
 * Core agent for code generation, refactoring, and analysis
 */

import { MessageBus } from '@neurostate/message-bus';
import { MCPClient } from '@neurostate/mcp-client';
import { FileSystem, createAgentFileSystem } from '@neurostate/file-system';
import { 
  AgentConfig, 
  AgentState, 
  Task, 
  TaskResult,
  CodeGenerationTask,
  CodeRefactorTask,
  CodeAnalysisTask
} from '@neurostate/types';
import * as path from 'path';

export interface CoderAgentConfig extends AgentConfig {
  sandboxRoot: string;
  mcpServerPath: string;
}

export class CoderAgent {
  private config: CoderAgentConfig;
  private messageBus: MessageBus;
  private mcpClient: MCPClient;
  private fileSystem: FileSystem;
  private state: AgentState = 'idle';
  private currentTask: Task | null = null;

  constructor(config: CoderAgentConfig) {
    this.config = config;
    this.messageBus = new MessageBus({ mode: 'memory' });
    this.mcpClient = new MCPClient();
    this.fileSystem = createAgentFileSystem(config.sandboxRoot);
  }

  /**
   * Initialize the agent - connect to MCP server and register with message bus
   */
  async initialize(): Promise<void> {
    try {
      this.state = 'initializing';
      
      // Connect to MCP Code Server
      await this.mcpClient.connect(this.config.mcpServerPath, 'stdio');
      
      // Register with message bus
      await this.messageBus.registerAgent({
        id: this.config.agentId,
        type: 'coder',
        tier: 'core',
        capabilities: [
          'code_generation',
          'code_refactoring',
          'code_optimization',
          'architecture_analysis',
          'static_analysis'
        ],
        status: 'active'
      });

      // Subscribe to task queue
      await this.messageBus.subscribe(`tasks:${this.config.agentId}`);
      await this.messageBus.subscribe('tasks:coder:*');

      this.state = 'idle';
      console.log(`[CoderAgent ${this.config.agentId}] Initialized successfully`);
    } catch (error) {
      this.state = 'error';
      console.error(`[CoderAgent ${this.config.agentId}] Initialization failed:`, error);
      throw error;
    }
  }

  /**
   * Process a task assigned to this agent
   */
  async processTask(task: Task): Promise<TaskResult> {
    this.currentTask = task;
    this.state = 'busy';

    try {
      console.log(`[CoderAgent ${this.config.agentId}] Processing task: ${task.id} (${task.type})`);

      let result: TaskResult;

      switch (task.type) {
        case 'code_generation':
          result = await this.handleCodeGeneration(task as CodeGenerationTask);
          break;
        case 'code_refactor':
          result = await this.handleCodeRefactor(task as CodeRefactorTask);
          break;
        case 'code_analysis':
          result = await this.handleCodeAnalysis(task as CodeAnalysisTask);
          break;
        default:
          result = {
            success: false,
            taskId: task.id,
            agentId: this.config.agentId,
            error: `Unknown task type: ${task.type}`
          };
      }

      // Publish result
      await this.messageBus.publish(`results:${task.id}`, result);

      this.state = 'idle';
      this.currentTask = null;
      return result;
    } catch (error) {
      this.state = 'error';
      this.currentTask = null;
      
      const errorResult: TaskResult = {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
      
      await this.messageBus.publish(`results:${task.id}`, errorResult);
      return errorResult;
    }
  }

  /**
   * Handle code generation task
   */
  private async handleCodeGeneration(task: CodeGenerationTask): Promise<TaskResult> {
    const startTime = Date.now();

    // Call MCP tool to generate code
    const mcpResult = await this.mcpClient.callTool('generate_driver_code', {
      language: task.language || 'typescript',
      framework: task.framework,
      description: task.description,
      requirements: task.requirements,
      constraints: task.constraints
    });

    if (!mcpResult.success || !mcpResult.result) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: mcpResult.error || 'Code generation failed'
      };
    }

    // Extract generated code from MCP result
    const generatedCode = mcpResult.result.code || '';
    const filePath = task.outputPath || `generated/${Date.now()}.ts`;

    // Write to file system
    const writeResult = await this.fileSystem.writeFile(filePath, generatedCode);

    if (!writeResult.success) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: writeResult.error
      };
    }

    const duration = Date.now() - startTime;

    return {
      success: true,
      taskId: task.id,
      agentId: this.config.agentId,
      output: {
        generatedCode,
        filePath: writeResult.path,
        metadata: {
          language: task.language,
          framework: task.framework,
          linesOfCode: generatedCode.split('\n').length,
          duration
        }
      },
      logs: [`Generated ${task.language || 'typescript'} code`, `Written to ${writeResult.path}`]
    };
  }

  /**
   * Handle code refactoring task
   */
  private async handleCodeRefactor(task: CodeRefactorTask): Promise<TaskResult> {
    const startTime = Date.now();

    // Read source file if specified
    let sourceCode = task.sourceCode;
    if (task.sourcePath && !sourceCode) {
      const readResult = await this.fileSystem.readFile(task.sourcePath);
      if (!readResult.success) {
        return {
          success: false,
          taskId: task.id,
          agentId: this.config.agentId,
          error: `Failed to read source file: ${readResult.error}`
        };
      }
      sourceCode = readResult.content || '';
    }

    if (!sourceCode) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: 'No source code provided'
      };
    }

    // Call MCP tool to refactor code
    const mcpResult = await this.mcpClient.callTool('refactor_code', {
      code: sourceCode,
      language: task.language || 'typescript',
      goals: task.goals,
      constraints: task.constraints
    });

    if (!mcpResult.success || !mcpResult.result) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: mcpResult.error || 'Code refactoring failed'
      };
    }

    const refactoredCode = mcpResult.result.refactoredCode || sourceCode;
    const outputPath = task.outputPath || task.sourcePath;

    // Write refactored code if output path specified
    if (outputPath) {
      const writeResult = await this.fileSystem.writeFile(outputPath, refactoredCode);
      if (!writeResult.success) {
        return {
          success: false,
          taskId: task.id,
          agentId: this.config.agentId,
          error: writeResult.error
        };
      }
    }

    const duration = Date.now() - startTime;

    return {
      success: true,
      taskId: task.id,
      agentId: this.config.agentId,
      output: {
        refactoredCode,
        outputPath,
        changes: mcpResult.result.changes || [],
        metrics: mcpResult.result.metrics || {},
        duration
      },
      logs: [`Refactored code with goals: ${task.goals?.join(', ')}`]
    };
  }

  /**
   * Handle code analysis task
   */
  private async handleCodeAnalysis(task: CodeAnalysisTask): Promise<TaskResult> {
    const startTime = Date.now();

    // Read source file if specified
    let sourceCode = task.sourceCode;
    if (task.sourcePath && !sourceCode) {
      const readResult = await this.fileSystem.readFile(task.sourcePath);
      if (!readResult.success) {
        return {
          success: false,
          taskId: task.id,
          agentId: this.config.agentId,
          error: `Failed to read source file: ${readResult.error}`
        };
      }
      sourceCode = readResult.content || '';
    }

    if (!sourceCode) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: 'No source code provided for analysis'
      };
    }

    // Call appropriate MCP tool based on analysis type
    let mcpResult;
    if (task.analysisType === 'architecture') {
      mcpResult = await this.mcpClient.callTool('analyze_architecture', {
        code: sourceCode,
        language: task.language || 'typescript'
      });
    } else {
      // Default to static analysis
      mcpResult = await this.mcpClient.callTool('static_analysis', {
        code: sourceCode,
        language: task.language || 'typescript',
        rules: task.rules
      });
    }

    if (!mcpResult.success || !mcpResult.result) {
      return {
        success: false,
        taskId: task.id,
        agentId: this.config.agentId,
        error: mcpResult.error || 'Code analysis failed'
      };
    }

    const duration = Date.now() - startTime;

    return {
      success: true,
      taskId: task.id,
      agentId: this.config.agentId,
      output: {
        analysis: mcpResult.result,
        duration
      },
      logs: [`Completed ${task.analysisType || 'static'} analysis`]
    };
  }

  /**
   * Get current agent state
   */
  getState(): AgentState {
    return this.state;
  }

  /**
   * Get current task being processed
   */
  getCurrentTask(): Task | null {
    return this.currentTask;
  }

  /**
   * Shutdown the agent gracefully
   */
  async shutdown(): Promise<void> {
    this.state = 'stopping';
    
    try {
      // Unregister from message bus
      await this.messageBus.unregisterAgent(this.config.agentId);
      
      // Disconnect from MCP server
      await this.mcpClient.disconnect();
      
      this.state = 'stopped';
      console.log(`[CoderAgent ${this.config.agentId}] Shut down successfully`);
    } catch (error) {
      this.state = 'error';
      console.error(`[CoderAgent ${this.config.agentId}] Shutdown error:`, error);
      throw error;
    }
  }
}

/**
 * Create a coder agent instance
 */
export function createCoderAgent(config: Partial<CoderAgentConfig>): CoderAgent {
  const defaultConfig: CoderAgentConfig = {
    agentId: `coder-${Date.now()}`,
    tier: 'core',
    sandboxRoot: path.join(process.cwd(), 'sandbox'),
    mcpServerPath: path.join(process.cwd(), 'mcp-servers', 'mcp-code-server', 'src', 'index.ts'),
    ...config
  };

  return new CoderAgent(defaultConfig);
}
