# Neurostate AI Agent Swarm & MCP Implementation Guide
## Complete Technical Documentation for Building the 25+ Agent System

---

## Table of Contents

1. [Executive Overview](#1-executive-overview)
2. [MCP (Model Context Protocol) Deep Dive](#2-mcp-model-context-protocol-deep-dive)
3. [Agent Swarm Architecture](#3-agent-swarm-architecture)
4. [Implementation Guide](#4-implementation-guide)
5. [Agent Lifecycle & Communication](#5-agent-lifecycle--communication)
6. [Tool Registry & Discovery](#6-tool-registry--discovery)
7. [Code Examples](#7-code-examples)
8. [Deployment & Scaling](#8-deployment--scaling)
9. [Best Practices & Patterns](#9-best-practices--patterns)

---

## 1. Executive Overview

### 1.1 What We're Building

The Neurostate AI Agent Swarm is a **25+ agent orchestration system** built on the **Model Context Protocol (MCP)** standard. It enables:

- **Standardized Tool Discovery:** Agents automatically discover capabilities across the ecosystem
- **Pluggable Architecture:** New tools/agents added without code changes
- **Cross-Agent Communication:** Seamless context sharing via MCP
- **External Integration:** Easy connection to third-party services
- **Type Safety:** Strongly typed message passing between agents

### 1.2 Core Philosophy

> "Tools matter more than prompts." — Anthropic Engineering Team

The MCP approach shifts the burden of tool definition and execution from the orchestrator to specialized MCP servers. This means:

1. **Agents focus on reasoning**, not tool implementation
2. **Tools are self-describing** via JSON Schema
3. **New capabilities are discovered dynamically**
4. **Cross-language interoperability** (Rust, Python, TypeScript)

---

## 2. MCP (Model Context Protocol) Deep Dive

### 2.1 What is MCP?

MCP is an open protocol that standardizes how AI applications connect to external context and tools. Think of it as **"USB-C for AI"** — a universal interface for capabilities.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MCP ARCHITECTURE OVERVIEW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐         MCP Protocol          ┌─────────────────┐    │
│   │   MCP Client    │  ◄──────────────────────────► │   MCP Server    │    │
│   │  (AI Agent)     │    JSON-RPC over stdio/SSE    │  (Tool Provider)│    │
│   └─────────────────┘                               └─────────────────┘    │
│                                                                              │
│   CAPABILITIES:                                                              │
│   • Tools: Model-controlled functions (read, write, calculate)              │
│   • Resources: App-controlled data (files, databases, APIs)                 │
│   • Prompts: User-controlled templates (system prompts, workflows)          │
│                                                                              │
│   TRANSPORTS:                                                                │
│   • stdio: Local process communication (fast, secure)                       │
│   • SSE: Server-Sent Events over HTTP (remote, scalable)                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 MCP Core Primitives

#### 2.2.1 Tools (Model-Controlled)

Tools are functions that the AI model can invoke to perform actions.

```json
{
  "name": "generate_driver_code",
  "description": "Generate embedded driver code for specified peripheral",
  "inputSchema": {
    "type": "object",
    "properties": {
      "mcu": { "type": "string", "enum": ["stm32f401", "esp32s3", "rp2040"] },
      "peripheral": { "type": "string", "enum": ["uart", "spi", "i2c", "pwm"] },
      "config": { "type": "object" }
    },
    "required": ["mcu", "peripheral"]
  }
}
```

**When to use:** Actions that modify state, perform calculations, or interact with external systems.

#### 2.2.2 Resources (App-Controlled)

Resources are read-only data sources that provide context to the model.

```json
{
  "uri": "mcu://stm32f401/pinout",
  "name": "STM32F401 Pinout Diagram",
  "mimeType": "application/json",
  "description": "Complete pin mapping for STM32F401 MCU"
}
```

**When to use:** Reference data, documentation, configuration files, sensor readings.

#### 2.2.3 Prompts (User-Controlled)

Prompts are pre-crafted instructions for common workflows.

```json
{
  "name": "bldc_controller_design",
  "description": "Design a complete BLDC motor controller",
  "arguments": [
    {
      "name": "power_rating",
      "description": "Motor power rating in watts",
      "required": true
    }
  ]
}
```

**When to use:** Standardized workflows, onboarding, complex multi-step tasks.

### 2.3 MCP Message Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MCP REQUEST-RESPONSE FLOW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. INITIALIZE                                                               │
│     Client ──► Server: initialize { protocolVersion, capabilities }         │
│     Server ──► Client: initialize { protocolVersion, capabilities, serverInfo }
│                                                                              │
│  2. CAPABILITY DISCOVERY                                                     │
│     Client ──► Server: tools/list {}                                        │
│     Server ──► Client: { tools: [...] }                                     │
│                                                                              │
│  3. TOOL INVOCATION                                                          │
│     Client ──► Server: tools/call { name, arguments, _meta: { progressToken } }
│     Server ──► Client: { content: [...], isError: false }                   │
│     Server ──► Client: notifications/progress { token, value, message }     │
│                                                                              │
│  4. RESOURCE ACCESS                                                          │
│     Client ──► Server: resources/read { uri }                               │
│     Server ──► Client: { contents: [{ uri, mimeType, text }] }              │
│                                                                              │
│  5. CLEANUP                                                                  │
│     Client ──► Server: notifications/cancelled { requestId, reason }        │
│     Client ──► Server: notifications/initialized (lifecycle)                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Agent Swarm Architecture

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    NEUROSTATE AGENT SWARM ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    META-ORCHESTRATOR (Director v2)                   │   │
│  │  • Intent Classification: Parse user requests into structured tasks   │   │
│  │  • Agent Selection: Dynamic capability matching from registry         │   │
│  │  • Task Decomposition: Break complex tasks into sub-tasks             │   │
│  │  • Conflict Resolution: Consensus building when agents disagree       │   │
│  │  • Quality Gates: Self-reflection before returning results            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    MCP SERVER REGISTRY                               │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│  │  │  Hardware   │ │   Code      │ │   Test      │ │  Deploy     │   │   │
│  │  │   Server    │ │   Server    │ │   Server    │ │   Server    │   │   │
│  │  │  (Rust)     │ │  (Rust)     │ │  (Python)   │ │  (Rust)     │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│  │  │   AI/ML     │ │  Security   │ │   Docs      │ │  Analytics  │   │   │
│  │  │   Server    │ │   Server    │ │   Server    │ │   Server    │   │   │
│  │  │  (Python)   │ │  (Rust)     │ │  (Python)   │ │  (Rust)     │   │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    AGENT POOL (25+ Specialized Agents)               │   │
│  │                                                                       │   │
│  │  Each agent is an MCP Client that:                                    │   │
│  │  • Connects to relevant MCP Servers                                   │   │
│  │  • Maintains its own state and context                                │   │
│  │  • Communicates via async message passing                             │   │
│  │  • Can spawn child agents for sub-tasks                               │   │
│  │                                                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    SHARED MEMORY & STATE STORE                       │   │
│  │  • Redis: Ephemeral context, pub/sub, distributed locks               │   │
│  │  • PostgreSQL/pgvector: Persistent knowledge, embeddings              │   │
│  │  • Event Store: Audit trail, replay, time travel                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Agent Categories & Responsibilities

#### Tier 1: Core Development Agents (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **Architect Agent** | Code, Hardware, AI/ML | System design, pattern selection, trade-off analysis |
| **Coder Agent** | Code, Test, Security | Code generation (C/C++/Rust/Zig), refactoring, optimization |
| **Hardware Agent** | Hardware, Analytics | Pinout validation, electrical analysis, timing verification |
| **FSM Agent** | Code, Test | State machine design, optimization, formal verification |
| **Canvas Agent** | UI, Analytics | Visual graph manipulation, auto-layout, node management |

#### Tier 2: Quality & Validation Agents (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **Ghost Agent** | Code, Security | Static analysis, dead code detection, complexity analysis |
| **Formal Agent** | Code, Test | Formal verification (Kani, CBMC), proof generation |
| **Test Agent** | Test, Code | Automated test generation (unit, integration, HIL) |
| **Security Agent** | Security, Code | Vulnerability scanning, threat modeling, SAST |
| **Compliance Agent** | Code, Docs | MISRA/CERT/CWE compliance checking |

#### Tier 3: AI/ML & Advanced Features (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **TinyML Agent** | AI/ML, Hardware | Neural architecture search, model deployment |
| **NAS Agent** | AI/ML, Hardware | Hardware-aware NAS, constraint optimization |
| **Digital Twin Agent** | Hardware, Analytics | vECU creation, twin synchronization, simulation |
| **Predictive Agent** | Analytics, AI/ML | Failure prediction, anomaly detection, forecasting |
| **Optimization Agent** | Code, Hardware | Power/performance trade-off analysis, auto-tuning |

#### Tier 4: DevOps & Operations (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **Build Agent** | Code, Deploy | Toolchain management, CI/CD orchestration |
| **Deploy Agent** | Deploy, Hardware | Flashing, OTA updates, bootloader management |
| **Debug Agent** | Hardware, Analytics | RTT/Serial analysis, crash investigation |
| **Monitor Agent** | Analytics, Hardware | Telemetry collection, alerting, dashboards |
| **SRE Agent** | Analytics, Deploy | Reliability engineering, SLO management |

#### Tier 5: User Experience & Collaboration (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **Voice Agent** | UI, Analytics | Hands-free voice commands, speech-to-text |
| **Docs Agent** | Docs, Code | Documentation generation, API docs, tutorials |
| **Tutorial Agent** | UI, Docs | Interactive onboarding, learning paths |
| **Collaboration Agent** | UI, Analytics | Real-time multi-user editing, conflict resolution |
| **Feedback Agent** | Analytics, UI | User behavior analysis, UX improvements |

#### Tier 6: Meta & System Agents (5 agents)

| Agent | Primary MCP Servers | Key Capabilities |
|-------|---------------------|------------------|
| **Self-Reflection Agent** | All | Output quality review, improvement suggestions |
| **Consensus Agent** | All | Conflict resolution between agents, voting |
| **Learning Agent** | Analytics, All | Continuous improvement from feedback, model updates |
| **Super Agent** | All | General-purpose fallback, cross-domain tasks |
| **Director Agent** | All | Central orchestration, task routing, coordination |

### 3.3 Communication Patterns

#### Pattern 1: Hub-and-Spoke (Supervisor-Based)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HUB-AND-SPOKE PATTERN                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                         ┌─────────────┐                                      │
│                         │  Director   │                                      │
│                         │   Agent     │                                      │
│                         └──────┬──────┘                                      │
│                                │                                             │
│            ┌───────────────────┼───────────────────┐                        │
│            │                   │                   │                        │
│            ▼                   ▼                   ▼                        │
│      ┌──────────┐       ┌──────────┐       ┌──────────┐                    │
│      │  Coder   │       │ Hardware │       │   Test   │                    │
│      │  Agent   │       │  Agent   │       │  Agent   │                    │
│      └──────────┘       └──────────┘       └──────────┘                    │
│                                                                              │
│  USE CASE: Sequential tasks with clear dependencies                          │
│  EXAMPLE: "Generate UART driver → Validate pinout → Run tests"              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Best for:** Tightly scoped, sequential reasoning problems (financial analysis, compliance checks, step-by-step pipelines)

#### Pattern 2: Blackboard (Shared Memory)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BLACKBOARD PATTERN                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│      ┌──────────┐         ┌───────────────┐         ┌──────────┐           │
│      │  Coder   │◄───────►│               │◄───────►│  Test    │           │
│      │  Agent   │         │   SHARED      │         │  Agent   │           │
│      └──────────┘         │   MEMORY      │         └──────────┘           │
│            ▲              │   (Redis)     │              ▲                 │
│            │              │               │              │                 │
│      ┌──────────┐         │ • Code drafts │         ┌──────────┐           │
│      │ Security │◄───────►│ • Test results│◄───────►│  Docs    │           │
│      │  Agent   │         │ • Annotations │         │  Agent   │           │
│      └──────────┘         │ • Decisions   │         └──────────┘           │
│                           └───────────────┘                                  │
│                                                                              │
│  USE CASE: Creative tasks with iterative refinement                          │
│  EXAMPLE: "Design a motor controller" - all agents contribute to solution   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Best for:** Creative settings where multiple specialists contribute partial solutions (design, architecture, creative writing)

#### Pattern 3: Swarm (Peer-to-Peer)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SWARM PATTERN                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│      ┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐    │
│      │ Agent A  │◄────►│ Agent B  │◄────►│ Agent C  │◄────►│ Agent D  │    │
│      │(Research)│      │(Research)│      │(Research)│      │(Research)│    │
│      └──────────┘      └──────────┘      └──────────┘      └──────────┘    │
│           │                 │                 │                 │           │
│           └─────────────────┴─────────────────┴─────────────────┘           │
│                                    │                                         │
│                                    ▼                                         │
│                           ┌───────────────┐                                  │
│                           │  Aggregator   │                                  │
│                           │    Agent      │                                  │
│                           └───────────────┘                                  │
│                                                                              │
│  USE CASE: Coverage tasks where redundancy is a feature                      │
│  EXAMPLE: "Research all available MCUs for motor control" - parallel search │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Best for:** Tasks requiring coverage (web research, data collection), where overlap validates signals

---

## 4. Implementation Guide

### 4.1 Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Agent Runtime** | Rust + `ractor` | Type-safe actor model, high performance |
| **MCP SDK** | `mcp-rs` (Rust) + `mcp` (Python/TS) | Official SDKs, protocol compliance |
| **Message Transport** | Redis Streams / NATS | Distributed, scalable, durable |
| **State Storage** | Redis + PostgreSQL | Ephemeral + persistent |
| **Vector DB** | pgvector / Pinecone | Embeddings, semantic search |
| **Observability** | OpenTelemetry + Jaeger | Distributed tracing |

### 4.2 Project Structure

```
neurostate-agent-swarm/
├── Cargo.toml                          # Rust workspace
├── mcp-servers/                        # MCP Server implementations
│   ├── mcp-code-server/               # Code generation tools
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── tools/
│   │   │   │   ├── generate_code.rs
│   │   │   │   ├── refactor.rs
│   │   │   │   └── optimize.rs
│   │   │   └── resources/
│   │   │       └── code_templates.rs
│   │   └── tests/
│   ├── mcp-hardware-server/           # Hardware validation tools
│   ├── mcp-test-server/               # Testing tools (Python)
│   ├── mcp-security-server/           # Security scanning tools
│   ├── mcp-ml-server/                 # ML/TinyML tools (Python)
│   └── mcp-deploy-server/             # Deployment tools
│
├── agents/                             # Agent implementations
│   ├── core-agents/                   # Tier 1: Core development
│   │   ├── architect-agent/
│   │   ├── coder-agent/
│   │   ├── hardware-agent/
│   │   ├── fsm-agent/
│   │   └── canvas-agent/
│   ├── quality-agents/                # Tier 2: Quality & validation
│   │   ├── ghost-agent/
│   │   ├── formal-agent/
│   │   ├── test-agent/
│   │   ├── security-agent/
│   │   └── compliance-agent/
│   ├── ml-agents/                     # Tier 3: AI/ML
│   │   ├── tinyml-agent/
│   │   ├── nas-agent/
│   │   ├── digital-twin-agent/
│   │   ├── predictive-agent/
│   │   └── optimization-agent/
│   ├── devops-agents/                 # Tier 4: DevOps
│   │   ├── build-agent/
│   │   ├── deploy-agent/
│   │   ├── debug-agent/
│   │   ├── monitor-agent/
│   │   └── sre-agent/
│   ├── ux-agents/                     # Tier 5: UX
│   │   ├── voice-agent/
│   │   ├── docs-agent/
│   │   ├── tutorial-agent/
│   │   ├── collaboration-agent/
│   │   └── feedback-agent/
│   └── meta-agents/                   # Tier 6: Meta
│       ├── self-reflection-agent/
│       ├── consensus-agent/
│       ├── learning-agent/
│       ├── super-agent/
│       └── director-agent/
│
├── orchestrator/                       # Meta-orchestrator
│   ├── src/
│   │   ├── lib.rs
│   │   ├── intent_classifier.rs
│   │   ├── agent_selector.rs
│   │   ├── task_decomposer.rs
│   │   ├── conflict_resolver.rs
│   │   └── quality_gate.rs
│   └── tests/
│
├── shared/                             # Shared components
│   ├── mcp-client/                    # MCP client wrapper
│   ├── message-bus/                   # Message passing utilities
│   ├── state-store/                   # State management
│   └── telemetry/                     # Observability
│
├── proto/                              # Protocol definitions
│   └── agent.proto
│
└── docs/                               # Documentation
    └── api/
```

---

## 5. Agent Lifecycle & Communication

### 5.1 Agent Lifecycle

```rust
// Agent lifecycle states
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Created,      // Agent instance created
    Initializing, // Connecting to MCP servers
    Idle,         // Ready to accept tasks
    Busy,         // Processing a task
    Paused,       // Temporarily suspended
    ShuttingDown, // Graceful shutdown in progress
    Terminated,   // Agent has stopped
    Error,        // Error state, needs recovery
}

// Agent lifecycle trait
#[async_trait]
pub trait AgentLifecycle: Actor {
    /// Called when agent is first created
    async fn on_created(&self, ctx: &Context) -> Result<(), AgentError>;
    
    /// Called during initialization - connect to MCP servers
    async fn on_initializing(&self, ctx: &Context) -> Result<(), AgentError>;
    
    /// Called when agent is ready to work
    async fn on_idle(&self, ctx: &Context) -> Result<(), AgentError>;
    
    /// Called when agent receives a task
    async fn on_task_received(&self, ctx: &Context, task: Task) -> Result<TaskId, AgentError>;
    
    /// Called periodically for health checks
    async fn on_health_check(&self, ctx: &Context) -> HealthStatus;
    
    /// Called during graceful shutdown
    async fn on_shutdown(&self, ctx: &Context) -> Result<(), AgentError>;
}
```

### 5.2 Message Types

```rust
/// All messages that can be sent between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    // Task-related messages
    TaskAssign(TaskAssignment),
    TaskResult(TaskResult),
    TaskCancel(TaskId),
    
    // Collaboration messages
    RequestCollaboration(CollaborationRequest),
    CollaborationResponse(CollaborationResponse),
    
    // Information sharing
    ShareContext(ContextShare),
    QueryContext(ContextQuery),
    
    // Control messages
    HealthCheck,
    HealthResponse(HealthStatus),
    Shutdown,
    
    // MCP-related
    McpToolCall(McpToolCall),
    McpToolResult(McpToolResult),
    McpResourceRequest(McpResourceRequest),
}

/// Task assignment from orchestrator to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: TaskId,
    pub task_type: TaskType,
    pub priority: Priority,
    pub payload: serde_json::Value,
    pub deadline: Option<DateTime<Utc>>,
    pub parent_task: Option<TaskId>,
    pub required_capabilities: Vec<Capability>,
}

/// Task result from agent to orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: serde_json::Value,
    pub metrics: TaskMetrics,
    pub artifacts: Vec<Artifact>,
}
```

### 5.3 Actor-Based Agent Implementation (using ractor)

```rust
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use serde::{Deserialize, Serialize};

/// Coder Agent - Generates and optimizes code
pub struct CoderAgent {
    mcp_client: McpClient,
    config: CoderConfig,
}

/// Messages that Coder Agent can receive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoderMessage {
    GenerateCode(GenerateCodeRequest, RpcReplyPort<CodeResult>),
    RefactorCode(RefactorRequest, RpcReplyPort<CodeResult>),
    OptimizeCode(OptimizeRequest, RpcReplyPort<CodeResult>),
    GetCapabilities(RpcReplyPort<Vec<Capability>>),
}

/// State maintained by Coder Agent
pub struct CoderState {
    active_tasks: HashMap<TaskId, TaskHandle>,
    code_cache: LruCache<String, CodeArtifact>,
    metrics: AgentMetrics,
}

#[async_trait]
impl Actor for CoderAgent {
    type Msg = CoderMessage;
    type State = CoderState;
    type Arguments = CoderConfig;

    /// Initialize the agent
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        config: CoderConfig,
    ) -> Result<Self::State, ActorProcessingErr> {
        // Connect to MCP servers
        let mcp_client = McpClient::new(&config.mcp_servers).await?;
        
        // Discover available tools
        let tools = mcp_client.list_tools().await?;
        log::info!("Coder Agent connected with {} tools", tools.len());
        
        Ok(CoderState {
            active_tasks: HashMap::new(),
            code_cache: LruCache::new(100),
            metrics: AgentMetrics::default(),
        })
    }

    /// Main message handler
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            CoderMessage::GenerateCode(request, reply) => {
                let result = self.generate_code(request, state).await;
                let _ = reply.send(result);
            }
            CoderMessage::RefactorCode(request, reply) => {
                let result = self.refactor_code(request, state).await;
                let _ = reply.send(result);
            }
            CoderMessage::OptimizeCode(request, reply) => {
                let result = self.optimize_code(request, state).await;
                let _ = reply.send(result);
            }
            CoderMessage::GetCapabilities(reply) => {
                let caps = self.get_capabilities().await;
                let _ = reply.send(caps);
            }
        }
        Ok(())
    }

    /// Cleanup on shutdown
    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        log::info!("Coder Agent shutting down");
        self.mcp_client.close().await?;
        Ok(())
    }
}

impl CoderAgent {
    async fn generate_code(
        &self,
        request: GenerateCodeRequest,
        state: &mut CoderState,
    ) -> Result<CodeResult, CoderError> {
        // Check cache first
        let cache_key = format!("{}:{}", request.mcu, request.peripheral);
        if let Some(cached) = state.code_cache.get(&cache_key) {
            return Ok(CodeResult::from_cached(cached));
        }

        // Call MCP tool for code generation
        let tool_result = self.mcp_client
            .call_tool("generate_driver_code", json!({
                "mcu": request.mcu,
                "peripheral": request.peripheral,
                "config": request.config
            }))
            .await?;

        // Process and validate result
        let code = self.process_tool_result(tool_result)?;
        
        // Cache the result
        let artifact = CodeArtifact::new(&code);
        state.code_cache.put(cache_key, artifact.clone());
        
        Ok(CodeResult::new(code, artifact))
    }
}
```

---

## 6. Tool Registry & Discovery

### 6.1 Dynamic Tool Discovery

```rust
/// Tool registry that aggregates tools from all MCP servers
pub struct ToolRegistry {
    servers: HashMap<ServerId, McpServerConnection>,
    tools: RwLock<HashMap<ToolName, ToolInfo>>,
    update_channel: broadcast::Sender<ToolRegistryEvent>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub server_id: ServerId,
    pub category: ToolCategory,
    pub capabilities: Vec<Capability>,
}

impl ToolRegistry {
    /// Discover all tools from connected MCP servers
    pub async fn discover_tools(&self) -> Result<Vec<ToolInfo>, RegistryError> {
        let mut all_tools = Vec::new();
        
        for (server_id, connection) in &self.servers {
            match connection.client.list_tools().await {
                Ok(tools_response) => {
                    for tool in tools_response.tools {
                        let tool_info = ToolInfo {
                            name: tool.name.clone(),
                            description: tool.description.unwrap_or_default(),
                            input_schema: tool.input_schema,
                            server_id: server_id.clone(),
                            category: self.categorize_tool(&tool.name),
                            capabilities: self.infer_capabilities(&tool),
                        };
                        all_tools.push(tool_info);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to list tools from server {}: {}", server_id, e);
                }
            }
        }
        
        // Update registry
        let mut tools = self.tools.write().await;
        for tool in &all_tools {
            tools.insert(tool.name.clone(), tool.clone());
        }
        
        // Notify subscribers
        let _ = self.update_channel.send(ToolRegistryEvent::ToolsUpdated {
            count: all_tools.len(),
        });
        
        Ok(all_tools)
    }
    
    /// Find tools matching a capability requirement
    pub async fn find_tools_by_capability(
        &self,
        capability: Capability,
    ) -> Vec<ToolInfo> {
        let tools = self.tools.read().await;
        tools
            .values()
            .filter(|t| t.capabilities.contains(&capability))
            .cloned()
            .collect()
    }
    
    /// Execute a tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolResult, RegistryError> {
        let tools = self.tools.read().await;
        let tool = tools
            .get(tool_name)
            .ok_or(RegistryError::ToolNotFound(tool_name.to_string()))?;
        
        let server = self.servers
            .get(&tool.server_id)
            .ok_or(RegistryError::ServerNotFound(tool.server_id.clone()))?;
        
        let result = server.client.call_tool(tool_name, arguments).await?;
        Ok(ToolResult::from_mcp_result(result))
    }
}
```

### 6.2 Capability-Based Agent Selection

```rust
/// Matches tasks to agents based on required capabilities
pub struct AgentSelector {
    registry: Arc<ToolRegistry>,
    agent_pool: Arc<AgentPool>,
    selection_strategy: SelectionStrategy,
}

#[derive(Debug, Clone)]
pub struct TaskRequirements {
    pub capabilities: Vec<Capability>,
    pub priority: Priority,
    pub estimated_duration: Duration,
    pub required_tools: Vec<String>,
}

impl AgentSelector {
    /// Select the best agent for a given task
    pub async fn select_agent(
        &self,
        task: &TaskRequirements,
    ) -> Result<AgentRef, SelectionError> {
        // Get all available agents
        let agents = self.agent_pool.get_available_agents().await;
        
        // Score each agent based on capability match
        let mut scored_agents: Vec<(AgentRef, f32)> = Vec::new();
        
        for agent in agents {
            let score = self.score_agent(&agent, task).await;
            if score > 0.0 {
                scored_agents.push((agent, score));
            }
        }
        
        // Sort by score descending
        scored_agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Apply selection strategy
        match self.selection_strategy {
            SelectionStrategy::BestFit => {
                scored_agents.into_iter().next().map(|(a, _)| a)
            }
            SelectionStrategy::RoundRobin => {
                // Select based on workload distribution
                self.select_round_robin(&scored_agents).await
            }
            SelectionStrategy::LoadBalanced => {
                // Consider current load
                self.select_load_balanced(&scored_agents).await
            }
        }
        .ok_or(SelectionError::NoSuitableAgent)
    }
    
    /// Score an agent's suitability for a task (0.0 - 1.0)
    async fn score_agent(&self, agent: &AgentRef, task: &TaskRequirements) -> f32 {
        let agent_caps = agent.get_capabilities().await.unwrap_or_default();
        
        // Calculate capability overlap
        let matching_caps: Vec<_> = task.capabilities
            .iter()
            .filter(|c| agent_caps.contains(c))
            .collect();
        
        let capability_score = matching_caps.len() as f32 / task.capabilities.len().max(1) as f32;
        
        // Check tool availability
        let registry = self.registry.tools.read().await;
        let available_tools = task.required_tools
            .iter()
            .filter(|t| registry.contains_key(*t))
            .count();
        let tool_score = available_tools as f32 / task.required_tools.len().max(1) as f32;
        
        // Combine scores (weighted)
        capability_score * 0.6 + tool_score * 0.4
    }
}
```

---

## 7. Code Examples

### 7.1 MCP Server Implementation (Rust)

```rust
// mcp-code-server/src/main.rs
use mcp_rs::server::{Server, ServerBuilder};
use mcp_rs::types::{Tool, Resource, Prompt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the MCP server
    let server = ServerBuilder::new("neurostate-code-server", "1.0.0")
        // Register tools
        .register_tool(generate_code_tool())
        .register_tool(refactor_code_tool())
        .register_tool(optimize_code_tool())
        // Register resources
        .register_resource(code_templates_resource())
        .register_resource(driver_patterns_resource())
        // Register prompts
        .register_prompt(bldc_controller_prompt())
        .build();
    
    // Start server with stdio transport
    server.run_stdio().await?;
    
    Ok(())
}

fn generate_code_tool() -> Tool {
    Tool::new(
        "generate_driver_code",
        "Generate embedded driver code for specified MCU and peripheral",
        json!({
            "type": "object",
            "properties": {
                "mcu": {
                    "type": "string",
                    "enum": ["stm32f401", "stm32h743", "esp32s3", "nrf52840", "rp2040"],
                    "description": "Target microcontroller"
                },
                "peripheral": {
                    "type": "string",
                    "enum": ["uart", "spi", "i2c", "pwm", "adc", "dma", "timer"],
                    "description": "Peripheral to generate driver for"
                },
                "config": {
                    "type": "object",
                    "properties": {
                        "baud_rate": { "type": "integer" },
                        "clock_speed": { "type": "integer" },
                        "dma_channels": { "type": "array", "items": { "type": "integer" } }
                    }
                },
                "language": {
                    "type": "string",
                    "enum": ["c", "cpp", "rust"],
                    "default": "c"
                }
            },
            "required": ["mcu", "peripheral"]
        }),
        |args| async move {
            let mcu = args["mcu"].as_str().unwrap();
            let peripheral = args["peripheral"].as_str().unwrap();
            let language = args["language"].as_str().unwrap_or("c");
            
            // Generate code using template engine
            let code = code_generator::generate_driver(mcu, peripheral, language).await?;
            
            Ok(json!({
                "content": [{"type": "text", "text": code}],
                "isError": false
            }))
        }
    )
}

fn code_templates_resource() -> Resource {
    Resource::new(
        "code://templates/{language}/{template_name}",
        "Code templates for various embedded patterns",
        "application/json",
        |uri| async move {
            // Parse URI template
            let parts: Vec<_> = uri.split('/').collect();
            let language = parts.get(2)?;
            let template_name = parts.get(3)?;
            
            // Load template
            let template = template_store::load(language, template_name).await?;
            
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string(&template)?
                }]
            }))
        }
    )
}

fn bldc_controller_prompt() -> Prompt {
    Prompt::new(
        "bldc_controller_design",
        "Design a complete BLDC motor controller",
        vec![
            PromptArgument::new(
                "power_rating",
                "Motor power rating in watts",
                true
            ),
            PromptArgument::new(
                "voltage",
                "System voltage (optional)",
                false
            )
        ],
        |args| async move {
            let power = args["power_rating"].as_i64().unwrap_or(100);
            let voltage = args["voltage"].as_i64().unwrap_or(24);
            
            let prompt = format!(
                r#"You are an expert embedded systems engineer specializing in motor control.

Design a complete BLDC motor controller for the following specifications:
- Power Rating: {}W
- System Voltage: {}V

Your design should include:
1. MCU selection with justification
2. Power stage design (MOSFETs, gate drivers)
3. Current sensing approach
4. Control algorithm (FOC or trapezoidal)
5. Safety features (overcurrent, overtemperature)
6. Pinout configuration
7. Code structure overview

Provide detailed technical reasoning for each decision."#,
                power, voltage
            );
            
            Ok(json!({
                "description": "BLDC Controller Design Prompt",
                "messages": [{"role": "user", "content": {"type": "text", "text": prompt}}]
            }))
        }
    )
}
```

### 7.2 MCP Client Implementation (Agent)

```rust
// agents/coder-agent/src/mcp_client.rs
use mcp_rs::client::{Client, StdioTransport};
use mcp_rs::types::Tool;

pub struct AgentMcpClient {
    client: Client,
    connected_servers: Vec<ServerConnection>,
}

#[derive(Debug, Clone)]
pub struct ServerConnection {
    pub id: String,
    pub name: String,
    pub tools: Vec<Tool>,
}

impl AgentMcpClient {
    /// Create and connect to MCP servers
    pub async fn new(server_configs: Vec<ServerConfig>) -> Result<Self, McpError> {
        let mut client = Client::new("coder-agent", "1.0.0");
        let mut connections = Vec::new();
        
        for config in server_configs {
            // Create transport based on config
            let transport = match config.transport {
                TransportConfig::Stdio { command, args } => {
                    StdioTransport::new(&command, &args)
                }
                TransportConfig::Sse { url, headers } => {
                    SseTransport::new(&url, headers)
                }
            };
            
            // Connect to server
            client.connect(transport).await?;
            
            // Discover capabilities
            let tools = client.list_tools().await?;
            let resources = client.list_resources().await?;
            let prompts = client.list_prompts().await?;
            
            log::info!(
                "Connected to {}: {} tools, {} resources, {} prompts",
                config.name,
                tools.tools.len(),
                resources.resources.len(),
                prompts.prompts.len()
            );
            
            connections.push(ServerConnection {
                id: config.id,
                name: config.name,
                tools: tools.tools,
            });
        }
        
        Ok(Self {
            client,
            connected_servers: connections,
        })
    }
    
    /// Call a tool by name (searches across all connected servers)
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolResult, McpError> {
        // Find which server has this tool
        let server = self.connected_servers
            .iter()
            .find(|s| s.tools.iter().any(|t| t.name == tool_name))
            .ok_or(McpError::ToolNotFound(tool_name.to_string()))?;
        
        log::debug!("Calling tool '{}' on server '{}'", tool_name, server.name);
        
        // Call the tool
        let result = self.client.call_tool(tool_name, arguments).await?;
        
        Ok(ToolResult::from(result))
    }
    
    /// Get all available tools across all servers
    pub fn get_all_tools(&self) -> Vec<&Tool> {
        self.connected_servers
            .iter()
            .flat_map(|s| &s.tools)
            .collect()
    }
}
```

### 7.3 Meta-Orchestrator Implementation

```rust
// orchestrator/src/lib.rs
use ractor::{concurrency::Duration, Actor, ActorProcessingErr, ActorRef};

/// The central orchestrator that coordinates all agents
pub struct DirectorAgent {
    agent_pool: Arc<AgentPool>,
    tool_registry: Arc<ToolRegistry>,
    intent_classifier: IntentClassifier,
    task_decomposer: TaskDecomposer,
    conflict_resolver: ConflictResolver,
}

/// Messages the Director can receive
#[derive(Debug, Clone)]
pub enum DirectorMessage {
    /// User request to process
    UserRequest(UserRequest, RpcReplyPort<OrchestratorResult>),
    
    /// Task completed by an agent
    TaskCompleted(TaskResult),
    
    /// Agent is requesting collaboration
    CollaborationRequest(CollaborationRequest, RpcReplyPort<CollaborationResponse>),
    
    /// Conflict between agents needs resolution
    ResolveConflict(Conflict, RpcReplyPort<Resolution>),
    
    /// Health check
    HealthCheck(RpcReplyPort<HealthStatus>),
}

#[async_trait]
impl Actor for DirectorAgent {
    type Msg = DirectorMessage;
    type State = DirectorState;
    type Arguments = DirectorConfig;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        config: DirectorConfig,
    ) -> Result<Self::State, ActorProcessingErr> {
        // Initialize all subsystems
        let intent_classifier = IntentClassifier::load(&config.model_path).await?;
        let task_decomposer = TaskDecomposer::new(config.max_subtasks);
        let conflict_resolver = ConflictResolver::new(&config.resolution_policy);
        
        log::info!("Director Agent initialized");
        
        Ok(DirectorState {
            active_workflows: HashMap::new(),
            pending_tasks: VecDeque::new(),
            metrics: OrchestratorMetrics::default(),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            DirectorMessage::UserRequest(request, reply) => {
                let result = self.handle_user_request(request, state).await;
                let _ = reply.send(result);
            }
            DirectorMessage::TaskCompleted(result) => {
                self.handle_task_completion(result, state).await?;
            }
            DirectorMessage::CollaborationRequest(req, reply) => {
                let response = self.handle_collaboration_request(req).await;
                let _ = reply.send(response);
            }
            DirectorMessage::ResolveConflict(conflict, reply) => {
                let resolution = self.conflict_resolver.resolve(&conflict).await;
                let _ = reply.send(resolution);
            }
            DirectorMessage::HealthCheck(reply) => {
                let status = self.check_health().await;
                let _ = reply.send(status);
            }
        }
        Ok(())
    }
}

impl DirectorAgent {
    /// Main entry point for user requests
    async fn handle_user_request(
        &self,
        request: UserRequest,
        state: &mut DirectorState,
    ) -> Result<OrchestratorResult, OrchestratorError> {
        let start_time = Instant::now();
        
        // Step 1: Classify intent
        let intent = self.intent_classifier.classify(&request.text).await?;
        log::info!("Classified intent: {:?}", intent);
        
        // Step 2: Decompose into sub-tasks
        let subtasks = self.task_decomposer.decompose(&intent, &request).await?;
        log::info!("Decomposed into {} subtasks", subtasks.len());
        
        // Step 3: Create workflow
        let workflow_id = WorkflowId::new();
        let workflow = Workflow::new(workflow_id.clone(), subtasks.clone());
        state.active_workflows.insert(workflow_id.clone(), workflow);
        
        // Step 4: Execute subtasks (parallel where possible)
        let results = self.execute_subtasks(subtasks, &workflow_id).await?;
        
        // Step 5: Synthesize results
        let final_result = self.synthesize_results(results, &intent).await?;
        
        // Step 6: Quality gate
        let quality_check = self.quality_gate.check(&final_result).await?;
        if !quality_check.passed {
            log::warn!("Quality check failed: {}", quality_check.reason);
            // Retry or escalate
            return self.handle_quality_failure(final_result, quality_check).await;
        }
        
        // Update metrics
        state.metrics.record_completion(start_time.elapsed());
        
        Ok(OrchestratorResult {
            workflow_id,
            output: final_result,
            metrics: TaskMetrics {
                duration: start_time.elapsed(),
                tokens_used: 0, // TODO: track
            },
        })
    }
    
    /// Execute subtasks, parallelizing where dependencies allow
    async fn execute_subtasks(
        &self,
        subtasks: Vec<SubTask>,
        workflow_id: &WorkflowId,
    ) -> Result<Vec<TaskResult>, OrchestratorError> {
        let mut handles = Vec::new();
        
        for subtask in subtasks {
            let agent = self.agent_selector.select_agent(&subtask.requirements).await?;
            
            let handle = tokio::spawn(async move {
                let result = agent.ask(
                    AgentMessage::TaskAssign(TaskAssignment {
                        task_id: TaskId::new(),
                        workflow_id: workflow_id.clone(),
                        payload: subtask.payload,
                        deadline: subtask.deadline,
                    }),
                    Duration::from_secs(300),
                ).await?;
                
                Ok::<TaskResult, AgentError>(result)
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let results = futures::future::try_join_all(handles).await?;
        
        Ok(results)
    }
}
```

### 7.4 Message Bus Implementation (Redis)

```rust
// shared/message-bus/src/lib.rs
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};

/// Distributed message bus for inter-agent communication
pub struct MessageBus {
    conn: MultiplexedConnection,
    namespace: String,
}

impl MessageBus {
    /// Create new message bus connection
    pub async fn new(redis_url: &str, namespace: &str) -> Result<Self, MessageBusError> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        
        Ok(Self {
            conn,
            namespace: namespace.to_string(),
        })
    }
    
    /// Publish a message to a channel
    pub async fn publish<T: Serialize>(
        &mut self,
        channel: &str,
        message: &T,
    ) -> Result<(), MessageBusError> {
        let payload = serde_json::to_string(message)?;
        let full_channel = format!("{}:{}", self.namespace, channel);
        
        self.conn.publish(&full_channel, payload).await?;
        
        Ok(())
    }
    
    /// Subscribe to a channel pattern
    pub async fn subscribe<T: DeserializeOwned + Send + 'static>(
        &mut self,
        pattern: &str,
        handler: impl Fn(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    ) -> Result<Subscription, MessageBusError> {
        let client = redis::Client::open("redis://localhost")?;
        let mut pubsub = client.get_async_pubsub().await?;
        
        let full_pattern = format!("{}:{}", self.namespace, pattern);
        pubsub.psubscribe(&full_pattern).await?;
        
        let mut msg_stream = pubsub.on_message();
        
        tokio::spawn(async move {
            while let Some(msg) = msg_stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                
                match serde_json::from_str::<T>(&payload) {
                    Ok(message) => {
                        handler(message).await;
                    }
                    Err(e) => {
                        log::error!("Failed to deserialize message: {}", e);
                    }
                }
            }
        });
        
        Ok(Subscription { pattern: full_pattern })
    }
    
    /// Push to a stream (for persistent messaging)
    pub async fn stream_add<T: Serialize>(
        &mut self,
        stream: &str,
        message: &T,
    ) -> Result<String, MessageBusError> {
        let payload = serde_json::to_string(message)?;
        let full_stream = format!("{}:stream:{}", self.namespace, stream);
        
        let id: String = self.conn
            .xadd(&full_stream, "*", &[("data", payload)])
            .await?;
        
        Ok(id)
    }
    
    /// Read from a stream (consumer group)
    pub async fn stream_read_group<T: DeserializeOwned>(
        &mut self,
        stream: &str,
        group: &str,
        consumer: &str,
        count: usize,
    ) -> Result<Vec<StreamMessage<T>>, MessageBusError> {
        let full_stream = format!("{}:stream:{}", self.namespace, stream);
        
        // Create group if not exists
        let _: Result<(), _> = self.conn
            .xgroup_create_mkstream(&full_stream, group, "$")
            .await;
        
        let options = StreamReadOptions::default()
            .group(group, consumer)
            .count(count)
            .block(5000);
        
        let results: StreamReadReply = self.conn
            .xread_options(&[&full_stream], &[">"], options)
            .await?;
        
        let mut messages = Vec::new();
        
        for stream_key in results.keys {
            for item in stream_key.ids {
                if let Some(data) = item.map.get("data") {
                    if let redis::Value::Data(bytes) = data {
                        let payload = String::from_utf8_lossy(bytes);
                        match serde_json::from_str::<T>(&payload) {
                            Ok(message) => {
                                messages.push(StreamMessage {
                                    id: item.id.to_string(),
                                    message,
                                });
                            }
                            Err(e) => {
                                log::error!("Failed to deserialize stream message: {}", e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(messages)
    }
    
    /// Acknowledge message processing
    pub async fn stream_ack(
        &mut self,
        stream: &str,
        group: &str,
        id: &str,
    ) -> Result<(), MessageBusError> {
        let full_stream = format!("{}:stream:{}", self.namespace, stream);
        self.conn.xack(&full_stream, group, &[id]).await?;
        Ok(())
    }
}
```

---

## 8. Deployment & Scaling

### 8.1 Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  # Redis for message bus and state
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes

  # PostgreSQL for persistent storage
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: neurostate
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: neurostate
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql

  # MCP Servers
  mcp-code-server:
    build: ./mcp-servers/mcp-code-server
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
    volumes:
      - code_templates:/templates

  mcp-hardware-server:
    build: ./mcp-servers/mcp-hardware-server
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  mcp-test-server:
    build: ./mcp-servers/mcp-test-server
    environment:
      - PYTHONUNBUFFERED=1
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  # Meta-Orchstrator
  director-agent:
    build: ./orchestrator
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://neurostate:${DB_PASSWORD}@postgres:5432/neurostate
    depends_on:
      - redis
      - postgres
      - mcp-code-server
      - mcp-hardware-server
      - mcp-test-server
    ports:
      - "8080:8080"

  # Core Agents (scaled horizontally)
  coder-agent:
    build: ./agents/core-agents/coder-agent
    deploy:
      replicas: 3
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - MCP_CODE_SERVER=mcp-code-server:8080
    depends_on:
      - redis
      - mcp-code-server

  hardware-agent:
    build: ./agents/core-agents/hardware-agent
    deploy:
      replicas: 2
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - MCP_HARDWARE_SERVER=mcp-hardware-server:8080
    depends_on:
      - redis
      - mcp-hardware-server

  # Observability
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14268:14268"

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  redis_data:
  postgres_data:
  code_templates:
  grafana_data:
```

### 8.2 Kubernetes Deployment

```yaml
# k8s/director-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: director-agent
  namespace: neurostate
spec:
  replicas: 2
  selector:
    matchLabels:
      app: director-agent
  template:
    metadata:
      labels:
        app: director-agent
    spec:
      containers:
      - name: director
        image: neurostate/director-agent:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: neurostate-secrets
              key: redis-url
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: neurostate-secrets
              key: database-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: director-agent
  namespace: neurostate
spec:
  selector:
    app: director-agent
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
---
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: director-agent-hpa
  namespace: neurostate
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: director-agent
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 9. Best Practices & Patterns

### 9.1 Agent Design Principles

1. **Single Responsibility:** Each agent should do one thing well
2. **Fail Fast:** Agents should fail quickly and report errors clearly
3. **Idempotency:** Tool calls should be idempotent where possible
4. **Observability:** All agents must emit structured logs and traces
5. **Graceful Degradation:** Agents should handle partial failures gracefully

### 9.2 MCP Best Practices

1. **Tool Naming:** Use descriptive, action-oriented names (`generate_code`, not `code_gen`)
2. **Schema Completeness:** Provide complete JSON schemas with descriptions
3. **Error Handling:** Return structured errors with actionable messages
4. **Progress Reporting:** Use progress tokens for long-running operations
5. **Resource Efficiency:** Clean up resources when connections close

### 9.3 Security Considerations

```rust
// Security middleware for MCP tool calls
pub struct SecurityMiddleware {
    allowed_tools: HashSet<String>,
    rate_limiter: RateLimiter,
    audit_log: AuditLog,
}

impl SecurityMiddleware {
    pub async fn validate_tool_call(
        &self,
        agent_id: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<(), SecurityError> {
        // Check if tool is allowed
        if !self.allowed_tools.contains(tool_name) {
            self.audit_log.record(
                agent_id,
                "TOOL_CALL_DENIED",
                &format!("Tool '{}' not in allowlist", tool_name),
            ).await;
            return Err(SecurityError::ToolNotAllowed(tool_name.to_string()));
        }
        
        // Check rate limits
        if !self.rate_limiter.allow(agent_id, tool_name).await {
            return Err(SecurityError::RateLimitExceeded);
        }
        
        // Validate arguments against injection attacks
        if contains_dangerous_patterns(arguments) {
            return Err(SecurityError::DangerousArguments);
        }
        
        // Log the call
        self.audit_log.record(
            agent_id,
            "TOOL_CALL_APPROVED",
            &format!("{} called with {:?}", tool_name, arguments),
        ).await;
        
        Ok(())
    }
}
```

### 9.4 Performance Optimization

1. **Connection Pooling:** Reuse MCP connections across requests
2. **Caching:** Cache tool results and resources when appropriate
3. **Batching:** Batch multiple tool calls when possible
4. **Lazy Loading:** Only connect to MCP servers when needed
5. **Circuit Breakers:** Fail fast when services are unhealthy

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **MCP** | Model Context Protocol - Standard for AI tool integration |
| **Agent** | Autonomous entity that can receive tasks and use tools |
| **Tool** | Function that an agent can invoke to perform actions |
| **Resource** | Read-only data source providing context to agents |
| **Prompt** | Pre-crafted instructions for common workflows |
| **vECU** | Virtual Electronic Control Unit - Digital twin of embedded device |
| **NAS** | Neural Architecture Search - Automated ML model design |
| **HIL** | Hardware-in-the-Loop - Testing with real hardware |
| **RTOS** | Real-Time Operating System |

---

## Appendix B: References

1. [MCP Specification](https://modelcontextprotocol.io)
2. [ractor - Rust Actor Framework](https://github.com/slawlor/ractor)
3. [Redis Streams Documentation](https://redis.io/docs/data-types/streams/)
4. [Anthropic MCP Course](https://anthropic.skilljar.com/introduction-to-model-context-protocol)

---

*Document Version: 1.0*
*Last Updated: March 2026*
*Author: Neurostate Engineering Team*
