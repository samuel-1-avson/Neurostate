import { AgentBase, AgentCapabilities, Task, TaskResult, TaskStatus } from '@neurostate/shared-types';
import { McpClient } from '@neurostate/mcp-client';
import { createMessageBus, MessageBus } from '@neurostate/message-bus';

export interface TestAgentConfig {
  mcpServerPath: string;
  testFramework?: 'jest' | 'vitest' | 'mocha' | 'pytest';
  coverageEnabled?: boolean;
  maxRetries?: number;
}

/**
 * Test Agent - Core agent for testing and quality assurance
 * Capabilities: Test Generation, Test Execution, Coverage Analysis, Regression Testing
 */
export class TestAgent extends AgentBase {
  private mcpClient: McpClient;
  private messageBus: MessageBus | null = null;
  private config: TestAgentConfig;
  private testHistory: Array<{ taskId: string; tests: number; passed: number; failed: number; timestamp: number }> = [];

  constructor(id: string, config: TestAgentConfig) {
    super(id, 'tester', {
      name: 'Test Agent',
      description: 'Expert QA engineer agent for test generation, execution, and coverage analysis',
      capabilities: [
        AgentCapabilities.TEST_GENERATION,
        AgentCapabilities.TEST_EXECUTION,
        AgentCapabilities.COVERAGE_ANALYSIS,
        AgentCapabilities.REGRESSION_TESTING,
        AgentCapabilities.PERFORMANCE_TESTING
      ],
      version: '2.0.0',
      status: 'initializing'
    });
    
    this.config = config;
    this.mcpClient = new McpClient({
      serverId: 'mcp-test-server',
      transportType: 'stdio',
      command: 'node',
      args: [config.mcpServerPath]
    });
  }

  async initialize(): Promise<void> {
    this.logger.info(`Initializing Test Agent ${this.id}...`);
    
    try {
      // Initialize message bus connection
      this.messageBus = await createMessageBus(
        { mode: 'memory' },
        false
      );
      
      await this.mcpClient.connect();
      
      // Verify MCP server capabilities
      const tools = await this.mcpClient.listTools();
      this.logger.info(`Connected to MCP Test Server. Available tools: ${tools.length}`);
      
      // Register with message bus
      if (this.messageBus) {
        await this.messageBus.registerAgent({
          id: this.id,
          type: 'tester',
          tier: 'core',
          capabilities: ['test_generation', 'test_execution', 'coverage_analysis'],
          status: 'active'
        });
        
        await this.messageBus.subscribe(`tasks:${this.id}`);
        await this.messageBus.subscribe('tasks:tester:*');
      }
      
      this.updateStatus('idle');
      this.logger.info(`Test Agent ${this.id} initialized successfully`);
    } catch (error) {
      this.logger.error(`Failed to initialize Test Agent: ${error}`);
      this.updateStatus('error');
      throw error;
    }
  }

  async executeTask(task: Task): Promise<TaskResult> {
    if (this.status !== 'idle' && this.status !== 'busy') {
      return {
        taskId: task.id,
        status: TaskStatus.FAILED,
        error: `Agent not ready. Current status: ${this.status}`,
        completedAt: Date.now()
      };
    }

    this.updateStatus('busy');
    this.logger.info(`Executing task ${task.id}: ${task.description}`);

    try {
      let result: any;

      switch (task.type) {
        case 'test_generation':
          result = await this.handleTestGeneration(task);
          break;
        case 'test_execution':
          result = await this.handleTestExecution(task);
          break;
        case 'coverage_analysis':
          result = await this.handleCoverageAnalysis(task);
          break;
        case 'regression_testing':
          result = await this.handleRegressionTesting(task);
          break;
        default:
          throw new Error(`Unsupported task type: ${task.type}`);
      }

      this.testHistory.push({
        taskId: task.id,
        tests: result.totalTests || 0,
        passed: result.passedTests || 0,
        failed: result.failedTests || 0,
        timestamp: Date.now()
      });

      this.updateStatus('idle');
      
      // Publish result via message bus
      if (this.messageBus) {
        await this.messageBus.publish(`results:${task.id}`, result);
      }
      
      return {
        taskId: task.id,
        status: TaskStatus.COMPLETED,
        result: result,
        completedAt: Date.now(),
        metrics: {
          duration: Date.now() - (task.createdAt || Date.now()),
          toolCalls: this.testHistory.filter(h => h.taskId === task.id).length,
          testMetrics: {
            total: result.totalTests || 0,
            passed: result.passedTests || 0,
            failed: result.failedTests || 0,
            coverage: result.coverage || 0
          }
        }
      };
    } catch (error) {
      this.logger.error(`Task ${task.id} failed: ${error}`);
      this.updateStatus('idle');
      
      return {
        taskId: task.id,
        status: TaskStatus.FAILED,
        error: error instanceof Error ? error.message : String(error),
        completedAt: Date.now()
      };
    }
  }

  private async handleTestGeneration(task: Task): Promise<any> {
    const { code, framework, testTypes, requirements } = task.payload;
    
    this.logger.info(`Generating ${framework || this.config.testFramework} tests`);
    
    const response = await this.mcpClient.callTool('generate_tests', {
      code,
      framework: framework || this.config.testFramework || 'jest',
      testTypes: testTypes || ['unit', 'integration'],
      requirements,
      coverageEnabled: this.config.coverageEnabled
    });

    return {
      generatedTests: response.content,
      testFiles: response.files || [],
      testCount: response.testCount,
      framework: framework || this.config.testFramework
    };
  }

  private async handleTestExecution(task: Task): Promise<any> {
    const { testFiles, testSuite, filters } = task.payload;
    
    this.logger.info(`Executing tests: ${testSuite || 'all'}`);
    
    const response = await this.mcpClient.callTool('execute_tests', {
      testFiles,
      testSuite,
      filters,
      framework: this.config.testFramework,
      maxRetries: this.config.maxRetries
    });

    return {
      totalTests: response.total,
      passedTests: response.passed,
      failedTests: response.failed,
      skippedTests: response.skipped,
      duration: response.duration,
      failures: response.failures || [],
      output: response.output
    };
  }

  private async handleCoverageAnalysis(task: Task): Promise<any> {
    const { code, testFiles, coverageThreshold } = task.payload;
    
    this.logger.info(`Analyzing code coverage with threshold: ${coverageThreshold}%`);
    
    const response = await this.mcpClient.callTool('analyze_coverage', {
      code,
      testFiles,
      coverageThreshold: coverageThreshold || 80,
      reportFormat: 'json'
    });

    return {
      coverage: response.coverage,
      lineCoverage: response.lineCoverage,
      branchCoverage: response.branchCoverage,
      functionCoverage: response.functionCoverage,
      uncoveredLines: response.uncoveredLines,
      recommendations: response.recommendations
    };
  }

  private async handleRegressionTesting(task: Task): Promise<any> {
    const { baseline, currentCode, testSuite } = task.payload;
    
    this.logger.info(`Running regression tests against baseline`);
    
    const response = await this.mcpClient.callTool('regression_test', {
      baseline,
      currentCode,
      testSuite,
      framework: this.config.testFramework
    });

    return {
      regressions: response.regressions || [],
      newFailures: response.newFailures || [],
      fixedIssues: response.fixedIssues || [],
      stabilityScore: response.stabilityScore,
      comparison: response.comparison
    };
  }

  async shutdown(): Promise<void> {
    this.logger.info(`Shutting down Test Agent ${this.id}...`);
    this.updateStatus('stopping');
    
    try {
      if (this.messageBus) {
        await this.messageBus.unregisterAgent(this.id);
      }
      await this.mcpClient.disconnect();
      this.updateStatus('stopped');
      this.logger.info(`Test Agent ${this.id} shut down successfully`);
    } catch (error) {
      this.logger.error(`Error shutting down Test Agent: ${error}`);
      this.updateStatus('error');
    }
  }

  getTestHistory(): Array<{ taskId: string; tests: number; passed: number; failed: number; timestamp: number }> {
    return [...this.testHistory];
  }
}
