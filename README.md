# PrismNote

An enterprise-grade, open-source data science notebook platform with Rust backend performance, Jupyter compatibility, and production-ready features for teams. Built for reproducibility, scalability, and security.

**Latest Release:** v0.3 | **License:** MIT | **Status:** Production Ready

---

## Quick Start (5 Minutes)

### 1. Install

Choose your installation method:

**pip (Recommended)**
```bash
pip install prismnote
prismnote
```

**uv (Fast Python)**
```bash
uv tool install prismnote
prismnote
```

**curl (Binary)**
```bash
bash <(curl -fsSL https://raw.githubusercontent.com/Mullassery/prismnote/main/install.sh)
prismnote
```

### 2. What Happens Next

PrismNote will:
1. Download the binary for your platform (auto-detected)
2. Start the server on `http://localhost:8000`
3. Open your browser automatically
4. Create `~/.prismnote/` directory for your notebooks

### 3. Create Your First Notebook

1. Click "New Notebook" button
2. Give it a name: "My First Analysis"
3. Click "Create"

### 4. Write Code

In the cell, type:
```python
import pandas as pd

# Create sample data
data = pd.DataFrame({
    'name': ['Alice', 'Bob', 'Charlie'],
    'age': [25, 30, 35],
    'score': [85, 90, 88]
})

print(data)
```

Press **Shift+Enter** to run the cell.

That's it! You now have a working PrismNote notebook.

---

## Key Commands

### Running Notebooks

| Command | What It Does |
|---------|---|
| `prismnote` | Start PrismNote server on localhost:8000 |
| `prismnote --help` | Show all command-line options |
| `prismnote /path/to/notebook.ipynb` | Open specific notebook |
| `prismnote --port 9000` | Run on custom port |
| `prismnote --dir /my/notebooks` | Use custom notebooks directory |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Shift+Enter | Run cell and move to next |
| Ctrl+Enter | Run cell (stay in place) |
| B | Insert cell below |
| A | Insert cell above |
| DD | Delete cell |
| M | Convert to markdown |
| Y | Convert to code |
| / | Search |

---

## Core Features

### Built-In (No Setup Required)

**Code Execution**
- Python 3.8+ code cells
- Markdown cells
- Output: text, images, tables, plots
- Cell timeout: 30 seconds (configurable)
- Auto-save: every 5 seconds (configurable)

**Smart Execution**
- Automatic cell dependency detection
- Skip unchanged cells
- Execute in dependency order
- Parallel execution support
- Execution history and rollback

**Notebooks**
- .ipynb format (Jupyter compatible)
- Import/export support
- Search across notebooks and cells
- Variable inspector (see all variables with types)

**Data Tools**
- Automatic DataFrame profiling
- Data quality issue detection
- Missing data analysis
- Type inference

### Requires Feature Setup

**SQL Queries** (Setup: Configure database connection)
```sql
--sql
SELECT * FROM my_table LIMIT 10
```
Supported: PostgreSQL, MySQL, SQLite, DuckDB, MongoDB

**Spark Big Data** (Setup: 2 lines of code)
```python
from pyspark.sql import SparkSession
spark = SparkSession.builder.appName("my-app").getOrCreate()
```
Features: Distributed execution, DAG optimization, DataFrame caching

**Cloud Data Warehouses** (Setup: Credentials)
Query directly from: Snowflake, BigQuery, Redshift, Azure Synapse, Databricks, Athena, Presto, Trino

**AI Assistance** (Setup: Optional API key)
```bash
export ANTHROPIC_API_KEY=sk-...
```
Features: Code explanation, error fixing, code completion

**Notebook Scheduling** (Setup: Schedule definition)
Run notebooks automatically with cron expressions: `0 9 * * 1` (every Monday at 9am)

**Enterprise Auth** (Setup: AAD/LDAP/SAML/OAuth2)
Features: Multi-tenant, RBAC, audit logging, MFA, SSO

**Model Fine-Tuning** (Setup: GPU provider account)
Providers: RunPod, Lambda Labs, Vast.ai, local GPU

---

## Environment Variables

### Core Configuration

| Variable | Default | Purpose |
|----------|---------|---------|
| `PRISMNOTE_DIR` | `~/.prismnote` | Where notebooks are stored |
| `PRISMNOTE_PORT` | `8000` | Server port |
| `PRISMNOTE_NOTEBOOK_DIR` | `~/.prismnote/notebooks` | Notebook directory |
| `RUST_LOG` | `info` | Logging level (debug, info, warn, error) |

### AI Features

| Variable | Purpose |
|----------|---------|
| `ANTHROPIC_API_KEY` | Claude API key (for Explain/Fix/Complete) |
| `OPENAI_API_KEY` | OpenAI API key (alternative) |
| `PRISMNOTE_AI_PROVIDER` | Which provider to use: `claude`, `openai`, `ollama` |
| `PRISMNOTE_OLLAMA_URL` | Ollama server (default: `http://localhost:11434`) |

### Big Data Features

| Variable | Purpose |
|----------|---------|
| `PRISMNOTE_SPARK_MASTER` | Spark master URL (local[*], spark://host:7077) |
| `PRISMNOTE_SPARK_MEMORY` | Executor memory (2g, 4g, 8g) |

### Enterprise Features

| Variable | Purpose |
|----------|---------|
| `PRISMNOTE_AUTH_PROVIDER` | Authentication: `aad`, `ldap`, `saml`, `oauth2` |
| `PRISMNOTE_AAD_TENANT_ID` | Azure AD tenant |
| `PRISMNOTE_AAD_CLIENT_ID` | Azure AD app ID |
| `RUNPOD_API_KEY` | RunPod for GPU training |

### Example: Enable All Features

```bash
export ANTHROPIC_API_KEY=sk-...
export PRISMNOTE_SPARK_MASTER=local[*]
export PRISMNOTE_AUTH_PROVIDER=aad
export PRISMNOTE_AAD_TENANT_ID=your-tenant-id
export PRISMNOTE_AAD_CLIENT_ID=your-client-id
prismnote
```

---

## Cell Types & Syntax

### Python Code Cell (Default)
```python
# Write any Python 3.8+ code
import pandas as pd
df = pd.read_csv('data.csv')
print(df.head())
```
**Run with:** Shift+Enter

### Markdown Cell
```markdown
# Title
This is **bold** and *italic*
- Bullet point
- Another point
```
**Convert to markdown:** Press M

### SQL Cell
```sql
--sql
SELECT * FROM users WHERE age > 21 LIMIT 10
```
**Requirements:** Database connection configured

### Spark SQL Cell
```sql
%sql
SELECT * FROM my_spark_table LIMIT 10
```
**Requirements:** Spark session created

### AI Features

Explain a cell (hover over cell → click Explain icon)
```
Explains what the code does in plain English
```

Fix an error (cell shows error → click Fix icon)
```
AI suggests how to fix the error
```

Complete code (Ctrl+Space in cell)
```
Auto-completes code suggestions based on context
```

---

## Configuration

### User Settings (`~/.prismnote/config.json`)

```json
{
  "theme": "dark",
  "auto_save_interval_seconds": 5,
  "cell_timeout_seconds": 30,
  "enable_library_suggestions": true,
  "ai_provider": "claude"
}
```

### Enable Features

**Enable Spark:**
```python
# At top of notebook
from pyspark.sql import SparkSession
spark = SparkSession.builder \
    .appName("my-app") \
    .config("spark.executor.memory", "4g") \
    .getOrCreate()
```

**Enable SQL:**
1. Go to Settings → Databases
2. Add database connection
3. Use in cells with `--sql` marker

**Enable Scheduling:**
1. Go to Notebook → Schedule
2. Enter cron expression: `0 9 * * 1` (9am Mondays)
3. Enable notifications

**Enable Enterprise Auth:**
1. Set `PRISMNOTE_AUTH_PROVIDER=aad` (or ldap/saml/oauth2)
2. Set provider credentials via environment variables
3. Restart PrismNote
4. Users will be prompted to authenticate

---

## Installation Details

### System Requirements

**Minimum:**
- Python 3.8 or higher
- 4GB RAM
- 500MB disk space
- macOS 11+, Linux (any), Windows (WSL2)

**For Spark/ML features:**
- 8GB RAM recommended
- Java 11+ (for Spark)
- GPU (optional, for model training)

**For Enterprise features:**
- Azure AD, LDAP, or SAML provider (optional)

### What Gets Installed

```
~/.prismnote/
├── notebooks/          # Your .ipynb files
├── bin/               # Downloaded binary
├── versions/          # Notebook versions (Git-like)
├── acl/              # Access control lists
├── schedules/        # Scheduled jobs
├── config.json       # User settings
└── cache/            # Execution cache
```

### Supported Platforms

| OS | Architecture | Status |
|-----|--|--|
| macOS | Apple Silicon (M1-M5+) | Full support |
| macOS | Intel | Full support |
| Linux | x86_64 | Full support |
| Linux | ARM64 | Full support |
| Windows | x86_64 (WSL2) | Full support |

---

## Common Use Cases

### Case 1: Data Analysis

```python
# 1. Load data
import pandas as pd
df = pd.read_csv('sales.csv')

# 2. Explore
print(df.info())
print(df.describe())

# 3. Analyze
print(df.groupby('region').sum())

# 4. Visualize
df.plot(x='month', y='revenue')
```

### Case 2: SQL + Python

```python
--sql
SELECT date, SUM(revenue) as total
FROM sales
WHERE date > '2024-01-01'
GROUP BY date

# Result is available as _
df = _
df.plot()
```

### Case 3: Big Data with Spark

```python
from pyspark.sql import SparkSession
spark = SparkSession.builder.getOrCreate()

# Load from cloud warehouse
df = spark.read.parquet('s3://bucket/data/')

# Analyze
df.filter(df.amount > 100).show()
```

### Case 4: Scheduled Report

```python
# This runs automatically every Monday at 9am
import pandas as pd
report = pd.read_sql("SELECT * FROM metrics", conn)
report.to_csv('/reports/weekly.csv')
print("Report generated!")
```

---

## Troubleshooting

### "Port 8000 already in use"
```bash
prismnote --port 9000
```

### "Module not found" error
```bash
pip install missing_module
# Restart PrismNote
```

### Notebooks not saving
Check: `~/.prismnote/notebooks/` has write permissions
```bash
chmod 755 ~/.prismnote/
```

### Slow execution
- Check: Do you have many large notebooks?
- Solution: Use smaller notebooks, enable Spark for big data
- Check: Cell timeout too short?
- Solution: Increase `PRISMNOTE_NOTEBOOK_TIMEOUT` or change in cell

### "Kernel not available"
Install: `pip install ipykernel`
Then restart PrismNote

### Database connection fails
- Check: Database is running and accessible
- Check: Credentials are correct
- Check: Firewall allows connection
- Solution: Use Test Connection button in UI

### Can't upload files
Check: `/tmp/` has space available
Check: Browser security settings allow file uploads

---

## Requirements

### For Basic Use
- Python 3.8 or higher
- Modern web browser (Chrome, Firefox, Safari, Edge)

### For SQL/Database
- PostgreSQL 10+, MySQL 5.7+, or SQLite (no install needed)

### For Spark
- Java 11 or higher (check: `java -version`)

### For Model Training
- GPU: NVIDIA (CUDA 11.8+) or Mac GPU
- Account: RunPod, Lambda Labs, or Vast.ai

### For Enterprise Auth
- Azure AD, LDAP, SAML provider, or OAuth2 provider

---

## Core Architecture

```
Browser (React)
    |
    | REST/WebSocket
    |
Rust Server (Axum)
    |
    ├── Notebook Manager
    ├── Cell Executor
    ├── Database Connector
    ├── Spark Manager
    └── Auth Manager
    |
Jupyter Kernel (ipykernel)
    |
Python Runtime
```

---

## Development

### Build from Source

```bash
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Backend
cargo build --release

# Frontend
cd frontend
npm install
npm run build

# Run
cargo run --release
```

### Development Mode

Terminal 1 (Rust backend):
```bash
cargo watch -x 'run --release'
```

Terminal 2 (React frontend):
```bash
cd frontend && npm run dev
```

Browser: `http://localhost:5173`

---

## Feature Comparison

| Feature | PrismNote | JupyterLab | Zeppelin |
|---------|-----------|-----------|----------|
| Versioning | Yes | No | Yes |
| RBAC | Yes | Minimal | Yes |
| Scheduling | Yes | No | Yes |
| SQL Support | Native | Plugins | Native |
| Spark | Native | No | Native |
| AI Assistance | Yes | No | No |
| Data Profiling | Yes | No | No |
| Enterprise Auth | Yes | Minimal | No |

---

## Documentation

**Getting Started**
- `INSTALLATION.md` — Detailed setup for all platforms
- `QUICK_START.md` — 10-minute walkthrough

**Features**
- `SQL_EXECUTION.md` — SQL cells and optimization
- `SPARK_MANAGEMENT.md` — Spark configuration and tuning
- `CLOUD_WAREHOUSES.md` — Connect to Snowflake, BigQuery, etc.
- `AI_TRAINING_FINETUNING.md` — Fine-tune LLMs
- `ENTERPRISE_AUTHENTICATION.md` — AAD, LDAP, SAML setup

**Advanced**
- `EXECUTION_PIPELINE.md` — Cell dependency and DAG execution
- `MACBOOK_SUPPORT.md` — Apple Silicon M1-M5+ support
- `BIGDATA_IMPLEMENTATION.md` — Production big data setup

**Contributing**
- `CONTRIBUTING.md` — How to contribute
- `CODE_OF_CONDUCT.md` — Community standards

---

## License & Support

**License:** MIT (free for personal, commercial, and educational use)

**Free Community Support:**
- GitHub Issues for bugs
- GitHub Discussions for questions
- Documentation at docs/

**Commercial Support:**
- Enterprise deployments
- Custom feature development
- Dedicated support channels
- Contact: support@prismnote.dev

---

## Quick Reference

### Installing Optional Features

**Claude API (for AI features):**
```bash
export ANTHROPIC_API_KEY=sk-your-key
```

**Ollama (free local AI):**
```bash
brew install ollama
ollama pull mistral  # Download model
export PRISMNOTE_AI_PROVIDER=ollama
```

**Spark:**
```bash
pip install pyspark
```

**Cloud Data Warehouses:**
```bash
pip install snowflake-connector  # For Snowflake
# or
pip install google-cloud-bigquery  # For BigQuery
```

### Uninstall

```bash
pip uninstall prismnote
rm -rf ~/.prismnote/  # Remove all data
```

---

## FAQ

**Q: Is PrismNote free?**
A: Yes, it's MIT licensed and completely free for all uses.

**Q: Can I use it offline?**
A: Yes, it runs 100% locally. AI features are optional.

**Q: How do I share notebooks?**
A: Export as .ipynb (fully compatible with Jupyter) or enable RBAC for team access.

**Q: Does it work on Windows?**
A: Yes, via WSL2 (Windows Subsystem for Linux).

**Q: Can I run it on a server?**
A: Yes, set `PRISMNOTE_LISTEN=0.0.0.0:8000` to listen on all interfaces.

**Q: How do I backup notebooks?**
A: Copy `~/.prismnote/notebooks/` to any location.

**Q: Can I migrate from Jupyter?**
A: Yes, PrismNote uses `.ipynb` format. Just copy files to `~/.prismnote/notebooks/`.

---

**Made with Rust + React | Open Source | MIT License | v0.3 Production Ready**
