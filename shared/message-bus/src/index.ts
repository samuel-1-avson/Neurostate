/**
 * Neurostate Message Bus - Redis-based pub/sub for agent communication
 * Part of Neurostate 2.0 Agent Swarm
 */

import { EventEmitter } from 'eventemitter3';
import { AgentMessage, AgentMessageType, HealthStatus } from '@neurostate/types';

// ============================================================================
// MESSAGE BUS CONFIGURATION
// ============================================================================

export interface MessageBusConfig {
  redisUrl: string;
  namespace?: string;
  reconnectAttempts?: number;
  reconnectDelayMs?: number;
}

export interface MessageBusStats {
  messagesPublished: number;
  messagesReceived: number;
  activeSubscriptions: number;
  connectedAgents: Set<string>;
}

// ============================================================================
// IN-MEMORY MESSAGE BUS (Development/Testing)
// ============================================================================

export class InMemoryMessageBus extends EventEmitter {
  private stats: MessageBusStats = {
    messagesPublished: 0,
    messagesReceived: 0,
    activeSubscriptions: 0,
    connectedAgents: new Set(),
  };

  private messageQueue: Map<string, AgentMessage[]> = new Map();

  constructor(private config?: MessageBusConfig) {
    super();
  }

  async connect(): Promise<void> {
    console.log('[MessageBus] Connected (in-memory mode)');
  }

  async disconnect(): Promise<void> {
    console.log('[MessageBus] Disconnected');
    this.removeAllListeners();
  }

  async publish(channel: string, message: AgentMessage): Promise<void> {
    this.stats.messagesPublished++;
    
    // Emit to all listeners on this channel
    this.emit(channel, message);
    this.emit('*', { ...message, type: AgentMessageType.SHARE_CONTEXT });
    
    // Store in queue for subscribers
    if (!this.messageQueue.has(channel)) {
      this.messageQueue.set(channel, []);
    }
    this.messageQueue.get(channel)?.push(message);
    
    // Keep queue size manageable
    const queue = this.messageQueue.get(channel);
    if (queue && queue.length > 1000) {
      queue.shift();
    }
  }

  async subscribe(channel: string, callback: (message: AgentMessage) => void): Promise<() => void> {
    this.stats.activeSubscriptions++;
    
    this.on(channel, callback);
    
    // Return unsubscribe function
    return () => {
      this.off(channel, callback);
      this.stats.activeSubscriptions--;
    };
  }

  async registerAgent(agentId: string): Promise<void> {
    this.stats.connectedAgents.add(agentId);
    console.log(`[MessageBus] Agent registered: ${agentId}`);
  }

  async unregisterAgent(agentId: string): Promise<void> {
    this.stats.connectedAgents.delete(agentId);
    console.log(`[MessageBus] Agent unregistered: ${agentId}`);
  }

  getStats(): MessageBusStats {
    return { ...this.stats };
  }

  async sendToAgent(agentId: string, message: AgentMessage): Promise<void> {
    await this.publish(`agent:${agentId}`, message);
  }

  async broadcast(message: AgentMessage): Promise<void> {
    await this.publish('broadcast', message);
  }
}

// ============================================================================
// REDIS MESSAGE BUS (Production)
// ============================================================================

export class RedisMessageBus extends EventEmitter {
  private client: any = null;
  private subscriber: any = null;
  private isConnected = false;
  private subscriptions: Map<string, Set<(message: AgentMessage) => void>> = new Map();
  private stats: MessageBusStats = {
    messagesPublished: 0,
    messagesReceived: 0,
    activeSubscriptions: 0,
    connectedAgents: new Set(),
  };

  constructor(private config: MessageBusConfig) {
    super();
  }

  async connect(): Promise<void> {
    try {
      // Dynamic import to avoid hard dependency when using in-memory
      const Redis = (await import('ioredis')).default;
      
      this.client = new Redis(this.config.redisUrl, {
        retryStrategy: (times: number) => {
          if (times > (this.config.reconnectAttempts || 3)) {
            return null;
          }
          return Math.min(times * 100, 3000);
        },
      });

      this.subscriber = new Redis(this.config.redisUrl);

      this.client.on('connect', () => {
        this.isConnected = true;
        console.log('[MessageBus] Connected to Redis');
      });

      this.client.on('error', (err: Error) => {
        console.error('[MessageBus] Redis error:', err);
        this.isConnected = false;
      });

      this.subscriber.on('message', (channel: string, message: string) => {
        this.handleMessage(channel, message);
      });

    } catch (error) {
      console.error('[MessageBus] Failed to connect to Redis:', error);
      throw error;
    }
  }

  private handleMessage(channel: string, messageStr: string): void {
    try {
      const message: AgentMessage = JSON.parse(messageStr);
      this.stats.messagesReceived++;

      const callbacks = this.subscriptions.get(channel);
      if (callbacks) {
        callbacks.forEach(cb => cb(message));
      }

      // Also emit to wildcard subscribers
      const wildcardCallbacks = this.subscriptions.get('*');
      if (wildcardCallbacks) {
        wildcardCallbacks.forEach(cb => cb(message));
      }
    } catch (error) {
      console.error('[MessageBus] Error parsing message:', error);
    }
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.quit();
    }
    if (this.subscriber) {
      await this.subscriber.quit();
    }
    this.isConnected = false;
    console.log('[MessageBus] Disconnected from Redis');
  }

  async publish(channel: string, message: AgentMessage): Promise<void> {
    if (!this.isConnected) {
      throw new Error('MessageBus not connected');
    }

    this.stats.messagesPublished++;
    const messageStr = JSON.stringify(message);
    const fullChannel = `${this.config.namespace || 'neurostate'}:${channel}`;
    
    await this.client.publish(fullChannel, messageStr);
  }

  async subscribe(channel: string, callback: (message: AgentMessage) => void): Promise<() => void> {
    if (!this.isConnected) {
      throw new Error('MessageBus not connected');
    }

    const fullChannel = `${this.config.namespace || 'neurostate'}:${channel}`;
    
    if (!this.subscriptions.has(channel)) {
      this.subscriptions.set(channel, new Set());
      await this.subscriber.subscribe(fullChannel);
      this.stats.activeSubscriptions++;
    }

    this.subscriptions.get(channel)?.add(callback);

    // Return unsubscribe function
    return () => {
      const subs = this.subscriptions.get(channel);
      if (subs) {
        subs.delete(callback);
        if (subs.size === 0) {
          this.subscriptions.delete(channel);
          this.subscriber.unsubscribe(fullChannel);
          this.stats.activeSubscriptions--;
        }
      }
    };
  }

  async registerAgent(agentId: string): Promise<void> {
    this.stats.connectedAgents.add(agentId);
    await this.subscribe(`agent:${agentId}`, (msg) => {
      // Handle agent-specific messages
      this.emit(`agent:${agentId}`, msg);
    });
    console.log(`[MessageBus] Agent registered: ${agentId}`);
  }

  async unregisterAgent(agentId: string): Promise<void> {
    this.stats.connectedAgents.delete(agentId);
    const unsubscribe = await this.subscribe(`agent:${agentId}`, () => {});
    unsubscribe();
    console.log(`[MessageBus] Agent unregistered: ${agentId}`);
  }

  getStats(): MessageBusStats {
    return { ...this.stats };
  }

  async sendToAgent(agentId: string, message: AgentMessage): Promise<void> {
    await this.publish(`agent:${agentId}`, message);
  }

  async broadcast(message: AgentMessage): Promise<void> {
    await this.publish('broadcast', message);
  }

  async getHealthStatus(): Promise<HealthStatus> {
    return {
      status: this.isConnected ? 'HEALTHY' : 'UNHEALTHY',
      uptime: process.uptime(),
      activeTasks: 0,
      queuedTasks: 0,
      errors: [],
      lastHeartbeat: new Date(),
    };
  }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

export type MessageBus = InMemoryMessageBus | RedisMessageBus;

export async function createMessageBus(
  config?: MessageBusConfig,
  useRedis: boolean = false
): Promise<MessageBus> {
  if (useRedis && config?.redisUrl) {
    const bus = new RedisMessageBus(config);
    await bus.connect();
    return bus;
  } else {
    const bus = new InMemoryMessageBus(config);
    await bus.connect();
    return bus;
  }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

export function createTaskAssignMessage(
  taskId: string,
  workflowId: string,
  taskType: string,
  payload: Record<string, unknown>,
  from: string,
  to: string
): AgentMessage {
  return {
    type: AgentMessageType.TASK_ASSIGN,
    payload: { taskId, workflowId, taskType, payload },
    from,
    to,
    timestamp: new Date(),
  };
}

export function createTaskResultMessage(
  taskId: string,
  status: string,
  output: Record<string, unknown>,
  from: string,
  to: string
): AgentMessage {
  return {
    type: AgentMessageType.TASK_RESULT,
    payload: { taskId, status, output },
    from,
    to,
    timestamp: new Date(),
  };
}

export function createHealthCheckMessage(from: string): AgentMessage {
  return {
    type: AgentMessageType.HEALTH_CHECK,
    payload: {},
    from,
    to: 'orchestrator',
    timestamp: new Date(),
  };
}

export default createMessageBus;
