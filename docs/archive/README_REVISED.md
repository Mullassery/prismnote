# PrismNote v0.3

**Modern, open-source Jupyter-compatible notebook platform with Python code execution, SQL support, and cloud warehouse integration.**

**License:** MIT | **Status:** Beta (v0.3) | **Latest:** 2026-06-20

---

## What is PrismNote?

PrismNote is a modern alternative to Jupyter with a focus on **simplicity, speed, and reproducibility**. Write Python code, execute SQL queries, and manage data workflows all in one beautiful, responsive notebook.

**Best for:**
- Data scientists and analysts who want a modern Jupyter replacement
- Teams building SQL-based data workflows
- Users who need Python notebooks with better UX
- Projects requiring dark mode and keyboard-driven workflows

---

## What Works Right Now (v0.3)

### ✓ Core Notebook Features
- **Python code cells** with Jupyter kernel (ipykernel required)
- **Markdown cells** with syntax highlighting
- **Cell execution** with Shift+Enter (run) or Ctrl+Enter (run and stay)
- **Dark/Light theme** with smooth switching
- **Modern, responsive UI** (WCAG 2.1 AA accessible)
- **Auto-save** (5-second interval)
- **Mobile-friendly** design for tablets
- **.ipynb format** (100% Jupyter compatible)

### ✓ Code Execution
- Python 3.8+ with full library support
- Real-time code output (text, HTML, images, tables)
- Rich output rendering (pandas DataFrames, matplotlib, plotly)
- Execution history tracking
- Cell execution statistics

### ✓ SQL & Data Features
- **SQL cells** with `--sql` marker support
- PostgreSQL and MySQL connections
- Basic query results as tables
- Simple query optimization suggestions

### ✓ Search & Discovery
- **Global search (Cmd+K)** across all notebooks and cells (NEW - v0.3)
- 8 search categories: notebooks, files, tables, variables, history, comments, chat, connections
- Real-time fuzzy matching with relevance scoring
- Keyboard navigation (arrow keys, Enter to select)

### ✓ Cloud Data Warehouse Foundation
- **API framework** for 8 cloud platforms (Snowflake, BigQuery, Redshift, Azure Synapse, Databricks, Athena, Presto, Trino)
- Connection management interface
- Test connection endpoints
- *Note: Full integration requires additional setup; see [CLOUD_WAREHOUSES.md](CLOUD_WAREHOUSES.md)*

### ✓ AI Code Assistance
- **Code explanation** via Claude API
- **Error fixing** suggestions
- **Code completion** with context awareness
- Optional (requires ANTHROPIC_API_KEY)

### ✓ Theme & Customization
- Dark mode enabled by default
- Light mode available
- Design system with 30+ CSS variables
- Responsive typography (10-20px font sizes)
- Cross-platform font support

---

## What Doesn't Work Yet (Planned for v0.4+)

**These features have framework code but are not yet integrated:**

- ❌ Real-time collaborative editing
- ❌ File upload/download UI (framework exists, not integrated)
- ❌ Cloud storage browser (S3, GCS, Azure)
- ❌ Enterprise authentication (AAD, LDAP, SAML) - framework only
- ❌ RBAC enforcement - framework exists, not enforced
- ❌ Audit logging - framework exists, not logging events
- ❌ Variable inspector - UI component exists, not tracking variables
- ❌ Notebook versioning - module exists, not integrated
- ❌ Docker code execution - framework only
- ❌ Spark session management - module exists, not tested
- ❌ Scheduling/cron jobs - framework only

**These are coming in v0.4-v1.0.**

---

## Quick Start (5 Minutes)

### Requirements
- Python 3.8+
- ipykernel: `pip install ipykernel`
- macOS, Linux, or Windows (WSL2)

### Installation

**Build from Source (Only method in v0.3):**
```bash
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Build Rust backend
cargo build --release

# Build React frontend
cd frontend
npm install
npm run build

# Start
cargo run --release
```

**Access:** http://localhost:8000

*Note: Pre-built packages (pip, brew, Docker) coming in v0.5*

### Create Your First Notebook

1. Open http://localhost:8000
2. Click "Create Notebook"
3. Enter code in the first cell:
```python
import pandas as pd

data = pd.DataFrame({
    'name': ['Alice', 'Bob', 'Charlie'],
    'age': [25, 30, 35]
})

print(data)
```
4. Press **Shift+Enter** to execute
5. See output below the cell

### Global Search (New in v0.3)

Press **Cmd+K** (Mac) or **Ctrl+K** (Windows/Linux) to:
- Search across all notebooks
- Search cell contents
- Filter by category
- Get instant results

---

## Features in Detail

### Global Search (Cmd+K)
- 8 searchable categories
- Real-time fuzzy matching
- Keyboard navigation (↑↓ arrows, Enter to select, Esc to close)
- Execution time < 100ms
- Dark mode support

### SQL Cells
Mark cells with `--sql` to execute SQL queries:
```sql
--sql
SELECT COUNT(*) as users FROM database_table
```

### AI Assistance (Requires ANTHROPIC_API_KEY)
```bash
export ANTHROPIC_API_KEY=sk-your-key-here
```

Then in the UI:
- Select code
- Use AI explain/fix buttons
- Get suggestions from Claude

### Dark Mode
Enabled by default. Toggle in theme selector (top right).

### Responsive Design
Works on:
- Desktop (1920px+)
- Laptop (1366px-1920px)
- Tablet (768px-1024px)
- Mobile (< 768px)

---

## API Reference

**Available in v0.3:**

### Notebooks
```
GET    /api/notebooks              List all notebooks
POST   /api/notebooks              Create notebook
GET    /api/notebooks/:id          Get notebook
PUT    /api/notebooks/:id          Update notebook
DELETE /api/notebooks/:id          Delete notebook
POST   /api/notebooks/:id/execute  Execute cell
```

### Search
```
POST   /api/search                 Global search across notebooks
```

### SQL
```
POST   /api/sql/execute            Execute SQL query
POST   /api/sql/optimize           Get optimization suggestions
```

### AI Features
```
POST   /api/ai/explain             Explain code
POST   /api/ai/fix                 Fix error
POST   /api/ai/complete            Complete code
```

### Display Settings
```
GET    /api/settings/display       Get display settings
PUT    /api/settings/display       Update settings
```

**Coming in v0.4+:** Collaboration, file upload, cloud storage, GitHub sync, and more.

---

## Configuration

### Environment Variables

**Core:**
```bash
PRISMNOTE_DIR=~/.prismnote              # Notebook storage location
PRISMNOTE_PORT=8000                     # Server port
RUST_LOG=info                           # Log level
```

**AI Features (Optional):**
```bash
ANTHROPIC_API_KEY=sk-your-key           # Enable Claude AI assistance
```

**Cloud Warehouses (Coming in v0.4):**
```bash
PRISMNOTE_SNOWFLAKE_ACCOUNT=your-account
PRISMNOTE_BIGQUERY_PROJECT=your-project
# ... more in CLOUD_WAREHOUSES.md
```

### User Settings
~/.prismnote/config.json:
```json
{
  "theme": "dark",
  "auto_save_interval_seconds": 5,
  "cell_timeout_seconds": 30,
  "font_family": "Roboto Mono",
  "font_size": 14
}
```

---

## Keyboard Shortcuts

### Navigation
- **Shift+Enter** - Execute cell
- **Ctrl+Enter** - Execute cell (in-place, don't move)
- **Cmd+K / Ctrl+K** - Open global search
- **Escape** - Close search / dialogs

### Editing
- **Cmd+/ / Ctrl+/** - Toggle comment
- **Tab** - Indent
- **Shift+Tab** - Unindent

### Notebook
- **A** - Insert cell above (coming v0.4)
- **B** - Insert cell below (coming v0.4)
- **D, D** - Delete cell (coming v0.4)

---

## Architecture

```
Browser (React 18 + TypeScript)
    ↓ REST API + WebSocket
Rust Backend (Axum + Tokio)
    ├── Cell Executor
    ├── Notebook Manager
    ├── Search Engine
    ├── SQL Executor
    └── AI Integration
    ↓
Jupyter Kernel (ipykernel)
    ↓
Python Runtime
```

### Development

**Terminal 1 - Rust Backend:**
```bash
cargo watch -x 'run --release'
```

**Terminal 2 - React Frontend:**
```bash
cd frontend
npm run dev
```

**Browser:** http://localhost:5173

---

## Known Limitations (v0.3)

1. **Single-user only** - Real-time collaboration coming in v0.4
2. **Local storage only** - Cloud storage mounting coming in v0.4
3. **Basic SQL support** - PostgreSQL and MySQL only
4. **No enterprise auth** - Framework exists, will be implemented in v0.4
5. **No Spark integration yet** - Framework exists, needs testing
6. **Output size limit** - 10MB per cell (prevents memory issues)
7. **No notebook branching** - Version tracking framework exists, not implemented
8. **No scheduled execution** - Framework exists, not integrated

**Full roadmap:** See [ROADMAP](#roadmap)

---

## Troubleshooting

### "Module 'ipykernel' not found"
```bash
pip install ipykernel
# Restart PrismNote
```

### "Port 8000 already in use"
```bash
PRISMNOTE_PORT=9000 cargo run --release
```

### Notebooks not saving
```bash
mkdir -p ~/.prismnote/notebooks
chmod 755 ~/.prismnote
```

### Slow execution
- Use smaller notebooks (< 100 cells)
- Disable auto-save if not needed: `"auto_save_interval_seconds": 0`
- Check disk space: `df ~/.prismnote`

### Global search returns no results
- Check search is enabled (it is by default)
- Try broader search terms
- Notebook must be saved before appearing in search

---

## Roadmap

### v0.3 ✓ (Current)
- [x] Python code cells with Jupyter kernel
- [x] Markdown cells
- [x] Dark/light theme
- [x] Modern UI (WCAG 2.1 AA)
- [x] Global search (Cmd+K)
- [x] SQL cells (PostgreSQL, MySQL)
- [x] AI code assistance (Claude API)
- [x] Responsive design (mobile-friendly)

### v0.4 (August 2026 - Planned)
- [ ] Real-time collaborative editing
- [ ] File upload/download UI
- [ ] Cloud storage browser (S3, GCS, Azure)
- [ ] Variable inspector
- [ ] Notebook versioning
- [ ] Spark session management
- [ ] Enterprise authentication framework

### v0.5 (November 2026 - Planned)
- [ ] GitHub notebook sync
- [ ] Pre-built packages (pip, brew, Docker)
- [ ] Display settings UI
- [ ] Extended keyboard shortcuts
- [ ] Plugin system foundation

### v1.0 (Q1 2027 - Planned)
- [ ] Kubernetes deployment
- [ ] dbt integration
- [ ] Airflow support
- [ ] Enterprise audit logging (enforced)
- [ ] Multi-tenant support with RBAC enforcement

---

## System Requirements

**Minimum:**
- Python 3.8+
- 2GB RAM
- 500MB disk space
- Modern web browser

**Recommended:**
- Python 3.10+
- 4GB+ RAM
- 2GB disk space for notebooks
- Chrome, Firefox, Safari, or Edge (latest version)

**For SQL features:**
- PostgreSQL 10+ OR MySQL 5.7+ (optional)

**For AI assistance:**
- Active Anthropic API key with credit balance

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- How to report bugs
- How to submit feature requests
- How to set up development environment
- Code style guidelines

---

## Support

**Free Community Support:**
- 🐛 Report bugs on [GitHub Issues](https://github.com/Mullassery/prismnote/issues)
- 💬 Ask questions on [GitHub Discussions](https://github.com/Mullassery/prismnote/discussions)
- 📚 Check [documentation files](./docs/)

**Commercial Support:**
- Coming soon (contact: support@prismnote.dev)

---

## License & Attribution

**License:** MIT (free for personal, commercial, and educational use)

**Built with:**
- Rust (Axum, Tokio, Serde)
- React 18 (TypeScript, Vite)
- Python (Jupyter kernel protocol)

**Thanks to:**
- Jupyter project for the notebook format and kernel protocol
- Open-source Rust and JavaScript communities
- All contributors and users!

---

## Project Status

- **Latest Release:** v0.3 (2026-06-20)
- **Phase:** Beta
- **Actively Maintained:** Yes
- **Production Ready:** For single-user data science workflows (v1.0 for enterprises)

---

**Made with Rust + React | Open Source | MIT Licensed**

⭐ **If you find PrismNote useful, please star us on [GitHub](https://github.com/Mullassery/prismnote)!**

---

## Quick Links

- [GitHub Repository](https://github.com/Mullassery/prismnote)
- [GitHub Issues](https://github.com/Mullassery/prismnote/issues)
- [GitHub Discussions](https://github.com/Mullassery/prismnote/discussions)
- [Contributing Guide](CONTRIBUTING.md)

---

**Questions?** Open an issue on GitHub or start a discussion. We're here to help!
