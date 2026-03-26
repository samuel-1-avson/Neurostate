# LLM Adapter Integration Guide

## Overview

The Neurostate LLM Adapter provides a unified interface for integrating free, local AI models into your agent swarm. It supports:

- **Ollama** (Native) - Direct integration with Ollama for running local LLMs
- **LangChain** - Unified API supporting multiple providers through LangChain
- **Future**: HuggingFace Inference API (free tier)

## Prerequisites

### 1. Install Ollama

```bash
# macOS/Linux
curl -fsSL https://ollama.com/install.sh | sh

# Windows
# Download from https://ollama.com/download
```

### 2. Pull Models

```bash
# Recommended models for different use cases:

# Fast classification (3B parameters)
ollama pull qwen2.5:3b

# General purpose (7B parameters) - RECOMMENDED DEFAULT
ollama pull qwen2.5:7b

# Complex reasoning (14B parameters)
ollama pull qwen2.5:14b

# Alternative: Llama 3.2
ollama pull llama3.2:3b
ollama pull llama3.2:7b
```

### 3. Start Ollama Server

```bash
# Usually starts automatically, but you can manually start it:
ollama serve
```

## Configuration

### Basic Setup (Orchestrator)

```typescript
import { Director } from '@neurostate/orchestrator';
import { defaultConfigs } from '@neurostate/llm-adapter';

const director = new Director({
  llmConfig: defaultConfigs.ollama, // Uses qwen2.5:7b by default
  useRedis: false,
  maxConcurrentTasks: 10
});

await director.initialize();
```

### Custom Configuration

```typescript
import { createLLMAdapter, LLMConfig } from '@neurostate/llm-adapter';

// Custom config for your setup
const customConfig: LLMConfig = {
  provider: 'ollama',
  model: 'qwen2.5:7b', // Your preferred model
  baseUrl: 'http://localhost:11434', // Ollama server URL
  temperature: 0.7, // Creativity (0.0-1.0)
  maxTokens: 2048, // Max response length
  topP: 0.9, // Nucleus sampling
};

const adapter = createLLMAdapter(customConfig);
```

### Model Recommendations

| Use Case | Model | Config |
|----------|-------|--------|
| Intent Classification | `qwen2.5:3b` | temperature: 0.3, maxTokens: 512 |
| Task Decomposition | `qwen2.5:7b` | temperature: 0.5, maxTokens: 2048 |
| Code Generation | `qwen2.5:14b` | temperature: 0.7, maxTokens: 4096 |
| Quick Responses | `qwen2.5:3b` | temperature: 0.5, maxTokens: 1024 |

## Usage Examples

### Direct Chat

```typescript
import { createLLMAdapter } from '@neurostate/llm-adapter';

const adapter = createLLMAdapter({
  provider: 'ollama',
  model: 'qwen2.5:7b'
});

const messages = [
  { role: 'system', content: 'You are a coding assistant.' },
  { role: 'user', content: 'Write a function to reverse a string in TypeScript.' }
];

const response = await adapter.chat(messages);
console.log(response.content);
```

### Intent Classification

```typescript
const result = await adapter.classifyIntent(
  'Generate a GPIO driver for STM32F4 with interrupt support'
);

console.log(result);
// Output:
// {
//   intent: 'code_generation',
//   confidence: 0.95,
//   entities: { mcu: 'STM32F4', feature: 'interrupt support' },
//   suggestedWorkflow: 'driver-generation'
// }
```

### Task Decomposition

```typescript
const availableAgents = ['coder', 'tester', 'debugger', 'optimizer'];

const result = await adapter.decomposeTasks(
  'code_generation',
  'Create an I2C driver with error handling and documentation',
  availableAgents
);

console.log(result.tasks);
// Output:
// [
//   {
//     id: 'task-1',
//     description: 'Implement I2C initialization routine',
//     agentType: 'coder',
//     priority: 10,
//     estimatedComplexity: 'medium'
//   },
//   {
//     id: 'task-2',
//     description: 'Add error handling for bus errors',
//     agentType: 'coder',
//     priority: 9,
//     dependencies: ['task-1'],
//     estimatedComplexity: 'medium'
//   },
//   ...
// ]
```

## Testing

### Run Integration Tests

```bash
cd /workspace/shared/llm-adapter
npx tsx tests/test-integration.ts
```

This will test:
1. ✅ Ollama health check
2. ✅ Simple chat completion
3. ✅ Intent classification
4. ✅ Task decomposition
5. ✅ LangChain adapter (optional)

## Troubleshooting

### Ollama Not Running

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# If not running, start it
ollama serve
```

### Model Not Found

```bash
# List available models
ollama list

# Pull missing model
ollama pull qwen2.5:7b
```

### Connection Refused

```typescript
// Make sure baseUrl matches your Ollama installation
const config: LLMConfig = {
  provider: 'ollama',
  model: 'qwen2.5:7b',
  baseUrl: 'http://localhost:11434', // Default port
  // Or for remote server:
  // baseUrl: 'http://192.168.1.100:11434'
};
```

### Out of Memory

If you get OOM errors with larger models:

```bash
# Use smaller model
ollama pull qwen2.5:3b

# Or quantized version
ollama pull qwen2.5:7b-q4_K_M
```

## Performance Tips

1. **Use appropriate model sizes:**
   - 3B: Fast classification, simple tasks (< 2GB RAM)
   - 7B: General purpose, good balance (~4GB RAM)
   - 14B+: Complex reasoning (~8GB+ RAM)

2. **Adjust temperature:**
   - Low (0.2-0.4): Deterministic, consistent outputs
   - Medium (0.5-0.7): Balanced creativity
   - High (0.8-1.0): More creative, less predictable

3. **Cache frequently used prompts:**
   ```typescript
   const systemPrompt = 'You are a coding assistant...';
   // Reuse this prompt across requests
   ```

4. **Batch similar requests:**
   Group related tasks to minimize context switching

## Advanced Configuration

### Using LangChain for Multi-Provider Support

```typescript
import { createLLMAdapter } from '@neurostate/llm-adapter';

// LangChain wrapper around Ollama
const adapter = createLLMAdapter({
  provider: 'langchain-ollama',
  model: 'qwen2.5:7b',
  baseUrl: 'http://localhost:11434',
  temperature: 0.7
});

// Benefits: Access to LangChain ecosystem (chains, agents, memory)
```

### Custom Stop Sequences

```typescript
const config: LLMConfig = {
  provider: 'ollama',
  model: 'qwen2.5:7b',
  stopSequences: ['\n\n', 'END', '```']
};
```

### Streaming Responses (Future)

```typescript
// TODO: Implement streaming for real-time responses
const stream = await adapter.chatStream(messages);
for await (const chunk of stream) {
  process.stdout.write(chunk);
}
```

## Security Considerations

✅ **Privacy**: All processing happens locally on your machine
✅ **No API Keys**: No need for OpenAI/Anthropic credentials
✅ **Air-Gapped**: Can run completely offline
⚠️ **Model Trust**: Only download models from trusted sources

## Cost Comparison

| Provider | Cost | Privacy | Speed |
|----------|------|---------|-------|
| **Ollama (Local)** | FREE | ✅ Full | ⚡ Fast |
| OpenAI API | $0.50-15/M tokens | ❌ Cloud | 🐢 Network |
| Anthropic API | $0.75-75/M tokens | ❌ Cloud | 🐢 Network |
| HuggingFace Free | FREE | ⚠️ Limited | 🐢 Rate limits |

## Next Steps

1. ✅ Install Ollama and pull a model
2. ✅ Configure the Director with LLM adapter
3. ✅ Test intent classification
4. ✅ Integrate with agent workflow
5. 🔄 Add more models as needed
6. 🔄 Fine-tune prompts for your domain

## Support

- Ollama Docs: https://ollama.com/docs
- LangChain Docs: https://js.langchain.com
- Qwen Models: https://huggingface.co/Qwen

---

**Happy Coding with Local AI! 🤖**
