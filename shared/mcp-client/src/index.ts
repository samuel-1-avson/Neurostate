/**
 * MCP Client - Wrapper for communicating with MCP servers
 * Part of Neurostate 2.0 Agent Swarm
 */

import { EventEmitter } from 'eventemitter3';
import {
  McpTool,
  McpResource,
  McpPrompt,
  McpToolCall,
  McpToolResult,
  McpInitializeRequest,
  McpInitializeResponse,
} from '@neurostate/types';

// ============================================================================
// MCP CLIENT CONFIGURATION
// ============================================================================

export interface McpClientConfig {
  serverId: string;
  transport: 'stdio' | 'sse' | 'websocket';
  command?: string;
  args?: string[];
  url?: string;
  timeoutMs?: number;
}

export interface McpServerInfo {
  name: string;
  version: string;
  protocolVersion: string;
  capabilities: {
    tools?: boolean;
    resources?: boolean;
    prompts?: boolean;
  };
}

export interface McpConnectionStatus {
  connected: boolean;
  serverInfo?: McpServerInfo;
  lastError?: string;
  uptime: number;
}

// ============================================================================
// STDIO TRANSPORT
// ============================================================================

class StdioTransport extends EventEmitter {
  private process: any = null;
  private buffer = '';
  private requestId = 0;
  private pendingRequests: Map<number, { resolve: Function; reject: Function }> = new Map();

  constructor(private config: McpClientConfig) {
    super();
  }

  async connect(): Promise<void> {
    return new Promise(async (resolve, reject) => {
      try {
        const { spawn } = await import('child_process');
        
        this.process = spawn(this.config.command || 'node', [
          ...(this.config.args || []),
        ]);

        this.process.stdout.on('data', (data: Buffer) => {
          this.handleData(data.toString());
        });

        this.process.stderr.on('data', (data: Buffer) => {
          console.error(`[MCP ${this.config.serverId}]`, data.toString());
        });

        this.process.on('error', (err: Error) => {
          console.error(`[MCP ${this.config.serverId}] Process error:`, err);
          reject(err);
        });

        this.process.on('exit', (code: number) => {
          console.log(`[MCP ${this.config.serverId}] Process exited with code ${code}`);
          this.emit('disconnect');
        });

        // Wait for process to start
        setTimeout(resolve, 100);
      } catch (error) {
        reject(error);
      }
    });
  }

  private handleData(data: string): void {
    this.buffer += data;
    
    const lines = this.buffer.split('\n');
    this.buffer = lines.pop() || '';

    for (const line of lines) {
      if (line.trim()) {
        try {
          const message = JSON.parse(line);
          this.emit('message', message);
          
          // Handle response
          if (message.id !== undefined) {
            const pending = this.pendingRequests.get(message.id);
            if (pending) {
              this.pendingRequests.delete(message.id);
              if (message.error) {
                pending.reject(new Error(message.error.message));
              } else {
                pending.resolve(message.result);
              }
            }
          }
        } catch (error) {
          console.error(`[MCP ${this.config.serverId}] Parse error:`, error);
        }
      }
    }
  }

  async sendRequest(method: string, params?: Record<string, unknown>): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const id = ++this.requestId;
      const request = {
        jsonrpc: '2.0',
        id,
        method,
        params: params || {},
      };

      this.pendingRequests.set(id, { resolve, reject });
      
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new Error(`Request timeout for ${method}`));
      }, this.config.timeoutMs || 30000);

      // Clear timeout on response
      const originalResolve = resolve;
      resolve = (result: unknown) => {
        clearTimeout(timeout);
        originalResolve(result);
      };

      this.process.stdin.write(JSON.stringify(request) + '\n');
    });
  }

  async disconnect(): Promise<void> {
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
  }
}

// ============================================================================
// MCP CLIENT
// ============================================================================

export class McpClient extends EventEmitter {
  private transport: StdioTransport | null = null;
  private status: McpConnectionStatus = {
    connected: false,
    uptime: 0,
  };
  private tools: Map<string, McpTool> = new Map();
  private resources: Map<string, McpResource> = new Map();
  private prompts: Map<string, McpPrompt> = new Map();
  private connectTime: number = 0;

  constructor(private config: McpClientConfig) {
    super();
  }

  async connect(): Promise<void> {
    try {
      this.transport = new StdioTransport(this.config);
      
      this.transport.on('message', (message) => {
        this.handleMessage(message);
      });

      this.transport.on('disconnect', () => {
        this.status.connected = false;
        this.emit('disconnect');
      });

      await this.transport.connect();

      // Initialize connection
      const initRequest: McpInitializeRequest = {
        protocolVersion: '2024-11-05',
        capabilities: {
          roots: { listChanged: true },
        },
        clientInfo: {
          name: 'neurostate-agent-swarm',
          version: '2.0.0',
        },
      };

      const response = await this.transport.sendRequest('initialize', initRequest as any);
      const initResponse = response as McpInitializeResponse;

      this.status.serverInfo = {
        name: initResponse.serverInfo.name,
        version: initResponse.serverInfo.version,
        protocolVersion: initResponse.protocolVersion,
        capabilities: {
          tools: !!initResponse.capabilities.tools,
          resources: !!initResponse.capabilities.resources,
          prompts: !!initResponse.capabilities.prompts,
        },
      };

      this.status.connected = true;
      this.connectTime = Date.now();

      // Discover tools, resources, and prompts
      await this.discoverCapabilities();

      console.log(`[MCP Client] Connected to ${this.config.serverId}`);
    } catch (error) {
      this.status.lastError = (error as Error).message;
      throw error;
    }
  }

  private async discoverCapabilities(): Promise<void> {
    try {
      // Discover tools
      if (this.status.serverInfo?.capabilities.tools) {
        const toolsResult = await this.transport?.sendRequest('tools/list');
        const tools = (toolsResult as { tools?: McpTool[] })?.tools || [];
        tools.forEach(tool => this.tools.set(tool.name, tool));
        console.log(`[MCP Client] Discovered ${tools.length} tools`);
      }

      // Discover resources
      if (this.status.serverInfo?.capabilities.resources) {
        const resourcesResult = await this.transport?.sendRequest('resources/list');
        const resources = (resourcesResult as { resources?: McpResource[] })?.resources || [];
        resources.forEach(resource => this.resources.set(resource.uri, resource));
        console.log(`[MCP Client] Discovered ${resources.length} resources`);
      }

      // Discover prompts
      if (this.status.serverInfo?.capabilities.prompts) {
        const promptsResult = await this.transport?.sendRequest('prompts/list');
        const prompts = (promptsResult as { prompts?: McpPrompt[] })?.prompts || [];
        prompts.forEach(prompt => this.prompts.set(prompt.name, prompt));
        console.log(`[MCP Client] Discovered ${prompts.length} prompts`);
      }
    } catch (error) {
      console.error('[MCP Client] Error discovering capabilities:', error);
    }
  }

  private handleMessage(message: any): void {
    // Handle notifications
    if (message.method) {
      switch (message.method) {
        case 'notifications/tools/list_changed':
          this.discoverCapabilities();
          break;
        case 'notifications/resources/list_changed':
          this.discoverCapabilities();
          break;
        case 'notifications/prompts/list_changed':
          this.discoverCapabilities();
          break;
      }
    }
  }

  async callTool(toolName: string, args: Record<string, unknown>): Promise<McpToolResult> {
    if (!this.status.connected) {
      throw new Error('MCP Client not connected');
    }

    const result = await this.transport!.sendRequest('tools/call', {
      name: toolName,
      arguments: args,
    });

    return result as McpToolResult;
  }

  async getResource(uri: string): Promise<string> {
    if (!this.status.connected) {
      throw new Error('MCP Client not connected');
    }

    const result = await this.transport!.sendRequest('resources/read', {
      uri,
    });

    return (result as { content?: string }).content || '';
  }

  async getPrompt(name: string, args: Record<string, string>): Promise<string> {
    if (!this.status.connected) {
      throw new Error('MCP Client not connected');
    }

    const result = await this.transport!.sendRequest('prompts/get', {
      name,
      arguments: args,
    });

    return (result as { content?: string }).content || '';
  }

  listTools(): McpTool[] {
    return Array.from(this.tools.values());
  }

  listResources(): McpResource[] {
    return Array.from(this.resources.values());
  }

  listPrompts(): McpPrompt[] {
    return Array.from(this.prompts.values());
  }

  getStatus(): McpConnectionStatus {
    return {
      ...this.status,
      uptime: this.connectTime ? Date.now() - this.connectTime : 0,
    };
  }

  async disconnect(): Promise<void> {
    if (this.transport) {
      await this.transport.disconnect();
      this.transport = null;
      this.status.connected = false;
    }
  }
}

// ============================================================================
// MCP CLIENT MANAGER
// ============================================================================

export class McpClientManager {
  private clients: Map<string, McpClient> = new Map();

  async registerClient(config: McpClientConfig): Promise<McpClient> {
    if (this.clients.has(config.serverId)) {
      return this.clients.get(config.serverId)!;
    }

    const client = new McpClient(config);
    await client.connect();
    this.clients.set(config.serverId, client);

    client.on('disconnect', () => {
      this.clients.delete(config.serverId);
    });

    return client;
  }

  getClient(serverId: string): McpClient | undefined {
    return this.clients.get(serverId);
  }

  getAllClients(): Map<string, McpClient> {
    return new Map(this.clients);
  }

  async unregisterClient(serverId: string): Promise<void> {
    const client = this.clients.get(serverId);
    if (client) {
      await client.disconnect();
      this.clients.delete(serverId);
    }
  }

  async disconnectAll(): Promise<void> {
    const promises = Array.from(this.clients.values()).map(client => client.disconnect());
    await Promise.all(promises);
    this.clients.clear();
  }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

export function createMcpClient(config: McpClientConfig): McpClient {
  return new McpClient(config);
}

export default createMcpClient;
