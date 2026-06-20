# PrismNote AI Integration Guide

PrismNote supports multiple AI providers for code explanation, error fixing, and completion. Choose the option that works best for you.

## Quick Start

### Option 1: Ollama (Recommended for Local/Offline)

**Ollama** runs open-source LLMs locally on your machine. No API keys needed, fully private.

#### Setup:

1. **Install Ollama**
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Windows: Download from https://ollama.ai
   ```

2. **Pull a Model**
   ```bash
   ollama pull neural-chat  # Fast, good for coding
   # or
   ollama pull llama2        # More capable, larger
   # or
   ollama pull mistral       # Fast and smart
   ```

3. **Start Ollama Server**
   ```bash
   ollama serve
   # Runs on http://localhost:11434
   ```

4. **Configure PrismNote**
   - Open PrismNote
   - Click **Sparkles** icon (top toolbar) → **AI Settings**
   - Select **Ollama**
   - Set URL: `http://localhost:11434`
   - Set Model: `neural-chat` (or whichever you pulled)
   - Click **Save**

#### Environment Setup (Alternative)
```bash
export PRISMNOTE_AI_PROVIDER=ollama
export PRISMNOTE_OLLAMA_URL=http://localhost:11434
export PRISMNOTE_OLLAMA_MODEL=neural-chat

# Then run PrismNote
cargo run --release
```

**Pros:**
- Fully offline
- Private (no data sent to servers)
- Free
- Fast on local hardware
- Works without internet

**Cons:**
- Requires local compute (GPU recommended)
- Less capable models than Claude/GPT-4
- Slower on CPU-only machines

---

### Option 2: Claude API (Best Accuracy)

**Claude** by Anthropic is the most capable open-source model. Requires paid API.

#### Setup:

1. **Get API Key**
   - Sign up at https://console.anthropic.com
   - Go to API Keys section
   - Create new key (starts with `sk-ant-`)

2. **Configure PrismNote**
   - Open PrismNote
   - Click **Sparkles** icon → **AI Settings**
   - Select **Claude**
   - Paste your API key
   - Click **Save**

#### Environment Setup (Alternative)
```bash
export PRISMNOTE_AI_PROVIDER=claude
export ANTHROPIC_API_KEY=sk-ant-xxxxx

cargo run --release
```

**Pros:**
- Most intelligent responses
- Handles complex code
- Fast API
- Good error fixing
- Supports latest Python features

**Cons:**
- Requires API key + billing
- Data sent to Anthropic servers
- Requires internet

**Pricing:** ~$0.003 per 1K input tokens, $0.015 per 1K output tokens. Typical cell explanation uses <1000 tokens (~$0.01).

---

### Option 3: OpenAI (GPT-4)

**GPT-4** is very capable but expensive. Good for heavy usage.

#### Setup:

1. **Get API Key**
   - Sign up at https://platform.openai.com
   - Go to API keys
   - Create new key

2. **Configure PrismNote**
   - Click **Sparkles** icon → **AI Settings**
   - Select **OpenAI**
   - Paste your API key
   - Set Model: `gpt-4` (or `gpt-3.5-turbo` for cheaper, less capable)
   - Click **Save**

#### Environment Setup (Alternative)
```bash
export PRISMNOTE_AI_PROVIDER=openai
export OPENAI_API_KEY=sk-xxxxx
export PRISMNOTE_OPENAI_MODEL=gpt-4

cargo run --release
```

**Pros:**
- Very capable (GPT-4)
- Multiple model options
- Fast

**Cons:**
- Expensive ($0.03 per 1K input tokens for GPT-4)
- Data sent to OpenAI
- Requires internet

---

## Advanced Configuration

### Using Environment Variables

Set these before running PrismNote:

**For Ollama:**
```bash
export PRISMNOTE_AI_PROVIDER=ollama
export PRISMNOTE_OLLAMA_URL=http://localhost:11434
export PRISMNOTE_OLLAMA_MODEL=neural-chat
```

**For Claude:**
```bash
export PRISMNOTE_AI_PROVIDER=claude
export ANTHROPIC_API_KEY=sk-ant-xxxxx
```

**For OpenAI:**
```bash
export PRISMNOTE_AI_PROVIDER=openai
export OPENAI_API_KEY=sk-xxxxx
export PRISMNOTE_OPENAI_MODEL=gpt-4
```

Then run:
```bash
cargo run --release
```

### Using UI Configuration

Click the **Sparkles**  icon in the toolbar to configure providers directly in the UI.

---

## Features

Once configured, you get three AI actions for each code cell:

###  Explain Code
Provides a brief 2-3 sentence explanation of what the code does.

**Example:**
```python
df = pd.read_csv('data.csv').groupby('category').sum()
```
→ "Groups data by category and sums all numeric columns."

###  Fix Error
Analyzes error output and suggests a fix.

**Example:**
- Error: `KeyError: 'column_name'`
- Suggestion: "The column doesn't exist. Check column names with `df.columns` or rename the reference."

###  Complete Code
Auto-completes or continues your code based on context.

**Example:**
```python
import numpy as np
import pandas as pd

data = {'x': [1, 2, 3], 'y': [4, 5, 6]}
df = pd.DataFrame(data)
df.groupby('x').apply(lambda x:
```
→ Completes the lambda function based on context.

---

## Comparing Providers

| Feature | Ollama | Claude | OpenAI |
|---------|--------|--------|--------|
| **Cost** | Free | ~$0.01/request | ~$0.10/request (GPT-4) |
| **Offline** |  |  |  |
| **Speed** | Slow (CPU), Fast (GPU) | Fast | Fast |
| **Accuracy** | Good | Excellent | Excellent |
| **Setup Complexity** | Medium | Easy | Easy |
| **Privacy** | Excellent | Good | Fair |
| **Internet Required** |  |  |  |

---

## Troubleshooting

### "AI not configured" Error
- Make sure environment variables are set OR
- Use the UI to configure (Sparkles → AI Settings)
- Restart PrismNote after configuration

### Ollama Connection Refused
- Is Ollama running? `ollama serve`
- Check URL: should be `http://localhost:11434`
- Try: `curl http://localhost:11434/api/tags` to test

### Claude API Errors
- Check API key is valid (starts with `sk-ant-`)
- Verify billing is set up in console.anthropic.com
- Check you have API credits

### OpenAI Errors
- Verify API key (starts with `sk-`)
- Check billing in platform.openai.com
- Model must exist (gpt-4, gpt-3.5-turbo, etc.)

### Slow Responses
- **Ollama:** Consider a faster model (neural-chat) or enable GPU
- **Claude/OpenAI:** Should be <5 seconds, if slower check network

---

## Tips for Best Results

1. **Ollama + GPU:** Install CUDA/Metal acceleration for 10x speedup
   ```bash
   # macOS with Metal (automatic)
   ollama serve
   
   # Linux with NVIDIA GPU
   # Install CUDA, then:
   ollama serve
   ```

2. **Model Selection:**
   - **Neural-chat:** Best for coding, fast
   - **Llama2:** More general purpose, slower
   - **Mistral:** Balanced, good for long context
   - **Claude/GPT-4:** Use when accuracy matters most

3. **Cost Optimization:**
   - Ollama is free (uses your hardware)
   - Claude's Haiku model is cheapest: ~$0.0008 per request
   - OpenAI's GPT-3.5-turbo is cheaper than GPT-4: ~$0.002 per request

---

## What Data Gets Sent?

### Ollama
- **Nothing.** Code runs locally on your machine.

### Claude / OpenAI
- Your code snippet (~100-500 tokens)
- The action (explain, fix, complete)
- No notebook metadata or other cells

Example sent to Claude:
```
{
  "code": "import pandas as pd\ndf = pd.read_csv('data.csv')",
  "action": "explain"
}
```

---

## Future AI Features (Roadmap)

- [ ] LLaVA for analyzing charts/images
- [ ] Multi-turn conversations (remember context across cells)
- [ ] Inline code suggestions as you type
- [ ] Performance profiling suggestions
- [ ] Test case generation
- [ ] Documentation generation

---

## Questions?

See [README.md](./README.md) and [GETTING_STARTED.md](./GETTING_STARTED.md) for more info.
