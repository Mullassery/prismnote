# PrismNote AI — Quick Start

Get AI-powered code assistance running in 5 minutes.

## Step 1: Choose Your AI Provider

### Option A: Local AI (Ollama) — Recommended
**Best for:** Privacy, offline work, no API costs

```bash
# Install Ollama
brew install ollama  # macOS
# or visit https://ollama.ai for Linux/Windows

# Pull a fast model
ollama pull neural-chat

# Start Ollama (keep running)
ollama serve
# Ollama is now at http://localhost:11434
```

### Option B: Claude API — Best Quality
**Best for:** Highest accuracy, complex code

```bash
# 1. Get API key at https://console.anthropic.com
# 2. Copy your sk-ant-xxxxx key
```

### Option C: OpenAI API — GPT-4
**Best for:** Most capable model (if budget allows)

```bash
# 1. Get API key at https://platform.openai.com/api-keys
# 2. Copy your sk-xxxxx key
```

## Step 2: Configure PrismNote

### Using Environment Variables (Terminal)

**For Ollama:**
```bash
export PRISMNOTE_AI_PROVIDER=ollama
export PRISMNOTE_OLLAMA_URL=http://localhost:11434
export PRISMNOTE_OLLAMA_MODEL=neural-chat

cargo run --release
```

**For Claude:**
```bash
export PRISMNOTE_AI_PROVIDER=claude
export ANTHROPIC_API_KEY=sk-ant-xxxxx

cargo run --release
```

**For OpenAI:**
```bash
export PRISMNOTE_AI_PROVIDER=openai
export OPENAI_API_KEY=sk-xxxxx
export PRISMNOTE_OPENAI_MODEL=gpt-4

cargo run --release
```

### Using UI (Easiest)

1. Open http://localhost:8000
2. Create a notebook
3. Click **✨ Sparkles** icon (top right)
4. Select your AI provider
5. Enter API key/URL
6. Click **Save**

## Step 3: Use AI in Your Notebook

1. **Write some Python code** in a cell
2. **Click the cell** to select it (blue highlight)
3. AI panel appears on the right with **3 buttons:**
   - **💡 Explain Code** — Get a brief explanation
   - **🔧 Fix Error** — Suggest fix for error
   - **✨ Complete** — Auto-complete code

4. Click a button, wait for response
5. Click **Insert Code** to add AI suggestion to your cell

## Examples

### Explain Code
```python
df = pd.read_csv('data.csv').groupby('category').apply(lambda x: x.sum())
```
→ Click **💡 Explain Code**
→ Result: "Groups data by category, sums numeric columns, returns aggregated DataFrame"

### Fix Error
```python
df['missing_column'].sum()  # KeyError: 'missing_column'
```
→ Click **🔧 Fix Error** (with error showing in output)
→ Result: "Column doesn't exist. Check available columns with `df.columns` or fix typo."

### Complete Code
```python
def analyze_data(df):
    # Remove nulls
    df = df.dropna()
    # Group by category
    grouped = df.groupby('category')
```
→ Click **✨ Complete**
→ Result: "return grouped.agg({'value': 'sum', 'count': 'size'})"

## Pricing

| Provider | Cost | Setup Time |
|----------|------|-----------|
| **Ollama** | Free | 5 min (+ model download) |
| **Claude** | ~$0.01/cell | <1 min |
| **OpenAI GPT-4** | ~$0.10/cell | <1 min |
| **OpenAI GPT-3.5** | ~$0.001/cell | <1 min |

## Troubleshooting

**"AI not configured" error?**
- Set environment variables before running, OR
- Use Sparkles → AI Settings to configure

**Ollama connection refused?**
- Run `ollama serve` in a separate terminal
- Check URL is `http://localhost:11434`

**Claude/OpenAI API errors?**
- Verify API key is correct
- Check you have billing set up and credits

**Slow responses?**
- Ollama: If using CPU only, responses are slow. GPU recommended.
- Claude/OpenAI: Should be <5 seconds. If slower, check network.

## Next Steps

1. **Read [AI_INTEGRATION.md](./AI_INTEGRATION.md)** for detailed setup & troubleshooting
2. **Check [GETTING_STARTED.md](./GETTING_STARTED.md)** for full project info
3. **Try all three providers** to see which fits your workflow

---

**Happy coding with AI!** 🚀
