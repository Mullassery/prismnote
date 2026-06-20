# Library Suggestions Quick Start

Get context-aware library recommendations as you code in PrismNote.

---

## Setup (30 seconds)

Choose your AI provider:

### Option 1: Claude (Recommended)
```bash
export PRISMNOTE_AI_PROVIDER=claude
export ANTHROPIC_API_KEY=sk-ant-YOUR_KEY_HERE
prismnote my_notebook.ipynb
```

### Option 2: Ollama (Local, Free)
```bash
# Install Ollama from https://ollama.ai
# Run in background
ollama serve

# In another terminal
export PRISMNOTE_AI_PROVIDER=ollama
export PRISMNOTE_OLLAMA_URL=http://localhost:11434
export PRISMNOTE_OLLAMA_MODEL=neural-chat
prismnote my_notebook.ipynb
```

### Option 3: OpenAI
```bash
export PRISMNOTE_AI_PROVIDER=openai
export OPENAI_API_KEY=sk-YOUR_KEY_HERE
export PRISMNOTE_OPENAI_MODEL=gpt-4
prismnote my_notebook.ipynb
```

---

## Using Library Suggestions

### 1. See Recommendations

Write code in a cell:
```python
import pandas as pd
df = pd.read_csv('data.csv')
print(df.describe())
```

Press Shift+Enter to execute.

**After execution**, click the **"Libraries"** tab on the right panel. You'll see:

```
pandas-profiling v4.1.0
Why: Auto-generates comprehensive data profiling reports
    with one call

[Install 4.1.0]  [Ignore]  [PyPI]
```

### 2. Install a Suggestion

Click `[Install 4.1.0]` button.

PrismNote adds it to `!pip install pandas-profiling` in a new cell. You can modify or execute it immediately.

### 3. Ignore a Library

Don't want to see a suggestion? Click `[Ignore]`.

That library won't be suggested again **for this notebook**, but other suggestions will keep coming.

### 4. Switch Between AI and Code Help

- AI Tab: Explain code, fix errors, complete code
- Libraries Tab: Discover and install libraries

Click the tab to switch.

---

## Tips & Tricks

### Tip 1: Code in Chunks
```python
# Cell 1: Load data
import pandas as pd
df = pd.read_csv('sales.csv')

# Execute -> Get data library suggestions (pandas-profiling, polars, etc.)
```

Then:
```python
# Cell 2: Analyze
df.groupby('region').sum()

# Execute -> Get analysis suggestions
```

Each execution analyzes fresh context.

### Tip 2: Ignore Patterns
Ignored libraries are **per-notebook**, not global. So:
- Notebook A: Ignore plotly, use matplotlib
- Notebook B: Ignore matplotlib, use plotly

Each notebook has its own preference list.

### Tip 3: Read the Reasoning
The "Why this helps" section explains **exactly** why the library fits your code.

Example:
```
Why: "Your code loads 100K+ row CSVs and filters them.
     Polars is 5-10x faster than pandas for this use case."
```

If you disagree, ignore it. The system learns your style.

### Tip 4: Update Available?
If you have pandas 2.0.0 installed and 2.1.0 is available:

```
numpy: Update available
1.24.0 -> 1.25.0

Why: Security patches + 15% faster matrix operations

[Update]  [Ignore]
```

### Tip 5: Different AI Providers, Different Suggestions
Claude is deeper but slower. Ollama is fast but simpler. Try both:

```bash
# Deep analysis (Claude)
PRISMNOTE_AI_PROVIDER=claude prismnote notebook.ipynb

# Quick suggestions (Ollama)
PRISMNOTE_AI_PROVIDER=ollama prismnote notebook.ipynb
```

---

## Examples

### Data Science Notebook
```python
# Input
import pandas as pd
import numpy as np

df = pd.read_csv('customers.csv')
X = df[['age', 'income']].values
```

**Suggestions:**
- pandas-profiling (EDA automation)
- scikit-learn (ML algorithms)
- plotly (interactive viz)

---

### Web Scraping
```python
# Input
import requests
from bs4 import BeautifulSoup

html = requests.get('https://example.com').text
soup = BeautifulSoup(html, 'html.parser')
```

**Suggestions:**
- scrapy (full-featured scraping framework)
- selenium (JavaScript-heavy sites)
- httpx (faster HTTP client)

---

### Machine Learning
```python
# Input
import pandas as pd
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier

X_train, X_test = train_test_split(X)
model = RandomForestClassifier()
```

**Suggestions:**
- xgboost (faster boosting)
- optuna (hyperparameter optimization)
- mlflow (experiment tracking)

---

## Troubleshooting

### No Suggestions Appearing?

**Problem:** Libraries panel shows empty after execution

**Solutions:**
1. Make sure AI provider is set:
   ```bash
   echo $PRISMNOTE_AI_PROVIDER
   ```
   Should print: `claude`, `ollama`, or `openai`

2. Check your API key:
   ```bash
   echo $ANTHROPIC_API_KEY  # for Claude
   ```

3. For Ollama, verify it's running:
   ```bash
   curl http://localhost:11434/api/tags
   ```

4. Try switching AI providers to see if one works

### Suggestions Take Too Long?

**Problem:** Library suggestions take 5+ seconds

**Solutions:**
1. Using Claude on slow internet? Try Ollama (local)
2. Using Ollama on weak CPU? Try OpenAI (cloud)
3. Notebook is very large? PrismNote debounces—wait a bit

### Wrong Suggestions?

**Problem:** Getting suggestions that don't fit your code

**Solutions:**
1. Click [Ignore] to filter them out
2. Write more specific code—context matters
3. Different AI providers give different results; try switching
4. This is v0.2—suggestions improve over time!

---

## FAQ

**Q: Is my code sent to the cloud?**
A: Only if you use Claude or OpenAI. If you use Ollama, everything stays local.

**Q: Can I see what libraries are already installed?**
A: Yes. The suggestion panel shows:
   - "Not installed" for new libraries
   - "Installed: 2.0.1" for existing ones

**Q: Can I install from the suggestion?**
A: Not yet (future feature). For now, click and add to a cell.

**Q: Why am I seeing the same suggestion again after ignoring it?**
A: Reload the notebook or execute a cell to refresh suggestions.

**Q: Can I sync ignored libraries across notebooks?**
A: Not yet. Each notebook has its own ignore list. Future: team sync.

**Q: Will suggestions slow down my notebook?**
A: No. Suggestions run in the background without blocking editing or execution.

---

## Next Steps

1. Write code in your notebook
2. Execute a cell
3. Click Libraries tab to see suggestions
4. Install or Ignore as you like
5. Keep coding—suggestions evolve with your code

Enjoy discovering better libraries!
