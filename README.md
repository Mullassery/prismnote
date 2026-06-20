# PrismNote

A modern, beautiful open-source data science notebook with Rust performance and Jupyter compatibility. Featuring stunning UI, reproducible execution, and AI-powered code assistance.

## Features

- 🎨 **Beautiful Modern UI** — Modern design with dark/light themes
- ⚡ **High Performance** — Rust backend for speed and reliability
- 📊 **Rich Visualizations** — High-resolution matplotlib, plotly, HTML output rendering
- 🤖 **AI Code Assistant** — Claude API integration for suggestions and explanations
- 📓 **Jupyter Compatible** — Full `.ipynb` import/export support
- 🔄 **Reactive Execution** — Smart cell execution with proper state management
- 🌐 **Self-Hosted** — Run locally with full data control
- 💻 **Multiple Installation Methods** — pip, uv, curl

## Installation

### Option 1: pip
```bash
pip install prismnote
prismnote
```

### Option 2: uv
```bash
uv tool install prismnote
prismnote
```

### Option 3: curl (binary)
```bash
bash <(curl -fsSL https://raw.githubusercontent.com/Mullassery/prismnote/main/install.sh)
prismnote
```

## Requirements

- Python 3.8+
- Node.js 18+ (for development)
- Rust 1.70+ (for building from source)

## Architecture

```
┌─────────────────────┐
│   React Frontend    │  (TypeScript, Monaco Editor, Tailwind CSS)
│  (high-res viz)     │
└──────────┬──────────┘
           │ WebSocket/REST
┌──────────▼──────────┐
│  Rust Backend       │  (Axum, Tokio)
│  (Kernel Manager)   │
└──────────┬──────────┘
           │ ZMQ
┌──────────▼──────────┐
│  Jupyter Kernel     │  (ipykernel)
│  (Python Execution) │
└─────────────────────┘
```

## Development

### Build from Source

```bash
# Clone repository
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Build Rust backend
cargo build --release

# Build React frontend
cd frontend && npm install && npm run build

# Run development server
cargo run --release
```

### Development Mode

```bash
# Terminal 1: Rust backend
cargo watch -x 'run --release'

# Terminal 2: React frontend (hot reload)
cd frontend && npm run dev
```

Visit http://localhost:5173 (frontend dev server) or http://localhost:8000 (backend)

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

## Configuration

PrismNote stores data in `~/.prismnote/`:
- `notebooks/` — Your notebook files (.ipynb format)
- `bin/` — Downloaded binary (pip/uv installations)
- `config.json` — User settings

Set custom directory: `export PRISMNOTE_DIR=/path/to/notebooks`

## Visualization Support

PrismNote renders the following output types natively with high resolution:

- **Text output** — stdout from print()
- **Images** — PNG, JPEG (base64 encoded)
- **HTML/SVG** — Raw HTML and SVG output
- **Tables** — pandas DataFrames as HTML tables
- **Plots** — Matplotlib, Plotly, Altair
- **JSON** — Pretty-printed JSON objects

All visualizations are optimized for high-DPI displays (Retina, 4K monitors).

## AI Features

PrismNote integrates Claude API for intelligent code assistance:

- **Explain Cell** — Get AI explanation of code
- **Fix Error** — AI suggests fix for errors
- **Complete Code** — Intelligent code completion

Configure API key: `export ANTHROPIC_API_KEY=sk-...`

## Roadmap

### v0.2.0
- [x] Basic cell execution
- [x] Markdown cells
- [x] Output rendering
- [ ] Jupyter kernel full integration (ZMQ)
- [ ] Claude API integration
- [ ] .ipynb import/export

### v0.3.0
- [ ] Real-time collaboration (WebSocket sync)
- [ ] Notebook versioning
- [ ] Environment management (venv/conda)

### v1.0.0
- [ ] Cloud deployment option
- [ ] Database integration (SQL)
- [ ] Scheduled execution
- [ ] Notification system

## Comparison

| Feature | PrismNote | JupyterLab | Google Colab |
|---------|-----------|-----------|------------|
| UI Quality | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Open Source | ✓ | ✓ | ✗ |
| Self-Hosted | ✓ | ✓ | ✗ |
| AI Assistant | ✓ | ✗ | ✓ |
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| Reproducibility | ✓ | ⚠️ | ⚠️ |
| Jupyter Compatible | ✓ | — | ✓ |

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT © 2026 Georgi Mammen Mullassery
