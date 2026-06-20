# PrismNote Quick Start — Run It Now

## Prerequisites (1 minute)

```bash
# Install Python dependencies for code execution
pip install ipykernel

# Or use uv
uv pip install ipykernel
```

## Build & Run (2 minutes)

### Option A: Full Build (Recommended)
```bash
cd prismnote

# Build frontend + backend
bash build.sh

# Run
./target/release/prismnote
```

Opens http://localhost:8000 automatically ✨

### Option B: Quick Development
```bash
# Terminal 1: Backend
cargo run --release

# Terminal 2: Frontend (hot reload)
cd frontend
npm run dev
```

Then open http://localhost:5173

## First Steps

1. **Create a notebook** — Click "New Notebook" button
2. **Write code** — Type Python in a code cell:
   ```python
   print("Hello, PrismNote!")
   import pandas as pd
   df = pd.DataFrame({'x': [1, 2, 3]})
   df
   ```
3. **Run cell** — Press `Shift+Enter`
4. **See output** — Should see stdout below
5. **Use AI** — Click ✨ on toolbar, select code, click "Explain Code"

## What Works Right Now ✅

- ✅ Python code execution (print, imports, pandas, numpy, etc.)
- ✅ Output display (text, numbers, DataFrames)
- ✅ Notebook save/load (.ipynb format)
- ✅ Markdown cells
- ✅ AI assistant (Explain, Fix, Complete)
- ✅ Keyboard shortcuts (Shift+Enter, B/A/DD)
- ✅ Dark theme

## What's Coming Soon 🚀

- [ ] Real-time WebSocket kernel communication (ZMQ fully wired)
- [ ] Variable inspector (see active variables)
- [ ] Cell interrupts (cancel long-running code)
- [ ] Package management (pip install in cells)
- [ ] Better error messages and debugging

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Shift+Enter` | Run cell and move to next |
| `Ctrl+Enter` | Run cell (stay in place) |
| `B` | Insert cell below |
| `A` | Insert cell above |
| `DD` | Delete cell |
| `M` | Convert to markdown |
| `Y` | Convert to code |

## Troubleshooting

### "ipykernel not found" error
```bash
pip install ipykernel
# Then restart PrismNote
```

### Code execution fails silently
- Check Python version: `python --version` (should be 3.8+)
- Test directly: `python -c "print('test')"`
- Check imports: `python -c "import pandas"`

### Notebooks don't save
- Check folder: `ls ~/.prismnote/notebooks/`
- Ensure write permissions: `ls -la ~/.prismnote/`
- Try manual save: Click Save button in toolbar

### AI assistant not working
- Optional feature (not required for core functionality)
- See [AI_QUICKSTART.md](./AI_QUICKSTART.md) for setup

## Example Notebook

Try this to test everything:

```python
# Data analysis
import pandas as pd
import numpy as np

data = {
    'name': ['Alice', 'Bob', 'Charlie'],
    'age': [25, 30, 35],
    'salary': [50000, 60000, 75000]
}

df = pd.DataFrame(data)
print(df)

# Statistics
print("\nMean salary:", df['salary'].mean())
print("Max age:", df['age'].max())

# Simple plot (if matplotlib installed)
import matplotlib.pyplot as plt
plt.plot(df['age'], df['salary'], 'o-')
plt.xlabel('Age')
plt.ylabel('Salary')
plt.show()
```

Click "Explain Code" in the AI panel to see what it does.

## Next Steps

1. **Read [GETTING_STARTED.md](./GETTING_STARTED.md)** for full details
2. **Check [AI_INTEGRATION.md](./AI_INTEGRATION.md)** for AI provider setup
3. **View [DEEPNOTE_COMPARISON.md](./DEEPNOTE_COMPARISON.md)** for roadmap (internal doc)

---

**Ready to build?** Start with the full build above, then explore! 🎉
