# PrismNote

Enterprise-grade, open-source data science notebook platform with Rust backend performance, Jupyter compatibility, and production-ready features for teams. Built for reproducibility, scalability, and security.

**Latest Release:** v0.3 | **Status:** Production Ready | **License:** MIT

---

## Quick Start (5 Minutes)

### Install

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

**Docker (Recommended for teams)**
```bash
docker-compose up -d
# Access at http://localhost:8000
```

### What Happens Next

1. PrismNote starts server on `http://localhost:8000`
2. Browser opens automatically
3. `~/.prismnote/` directory created for notebooks
4. Ready to create your first notebook

### Create Your First Notebook

```python
import pandas as pd

# Create sample data
data = pd.DataFrame({
    'name': ['Alice', 'Bob', 'Charlie'],
    'age': [25, 30, 35],
    'score': [85, 90, 88]
})

# Press Shift+Enter to execute
print(data)
```

---

## Docker Setup (Recommended for Teams & Production)

### Quick Start with Docker

**Single command:**
```bash
docker run -p 8000:8000 -v ~/.prismnote:/root/.prismnote prismnote:latest
```

**Access:**
```
http://localhost:8000
```

### Docker Compose (Multi-container Setup)

**Create `docker-compose.yml`:**
```yaml
version: '3.8'

services:
  prismnote:
    image: prismnote:latest
    ports:
      - "8000:8000"
    volumes:
      - ./notebooks:/root/.prismnote/notebooks
      - ./data:/root/.prismnote/data
    environment:
      - PRISMNOTE_DIR=/root/.prismnote
      - PRISMNOTE_PORT=8000
      - RUST_LOG=info
    restart: unless-stopped
    networks:
      - prismnote-network

  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: prismnote
      POSTGRES_DB: prismnote
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped
    networks:
      - prismnote-network

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    networks:
      - prismnote-network

volumes:
  postgres_data:

networks:
  prismnote-network:
```

**Start:**
```bash
docker-compose up -d

# View logs
docker-compose logs -f prismnote

# Stop
docker-compose down
```

### Docker with Environment Variables

```bash
docker run \
  -p 8000:8000 \
  -v ~/.prismnote:/root/.prismnote \
  -e PRISMNOTE_AUTH_PROVIDER=aad \
  -e PRISMNOTE_AAD_TENANT_ID=your-tenant-id \
  -e PRISMNOTE_AAD_CLIENT_ID=your-client-id \
  -e ANTHROPIC_API_KEY=sk-your-key \
  prismnote:latest
```

### Docker with Enterprise Features

**Enable all features:**
```bash
docker-compose -f docker-compose.yml up -d

# With Spark support
docker run \
  -p 8000:8000 \
  -v ~/.prismnote:/root/.prismnote \
  -e PRISMNOTE_SPARK_MASTER=local[*] \
  -e PRISMNOTE_SPARK_MEMORY=4g \
  prismnote:latest
```

### Multi-User Docker Deployment

**Production-ready `docker-compose.yml` with nginx:**
```yaml
version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - prismnote
    restart: unless-stopped

  prismnote:
    image: prismnote:latest
    environment:
      - PRISMNOTE_DIR=/data/prismnote
      - PRISMNOTE_AUTH_PROVIDER=aad
      - PRISMNOTE_AAD_TENANT_ID=${AAD_TENANT_ID}
      - PRISMNOTE_AAD_CLIENT_ID=${AAD_CLIENT_ID}
    volumes:
      - ./data:/data
    restart: unless-stopped
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure

  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: prismnote
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
```

### Docker Image Platforms

PrismNote images available for:
- `linux/amd64` - Intel/AMD 64-bit
- `linux/arm64` - ARM 64-bit (Raspberry Pi, Apple Silicon)
- `linux/arm/v7` - ARM 32-bit

**Auto-detection:**
```bash
docker pull prismnote:latest
# Automatically selects correct platform for your machine
```

### Building Custom Docker Image

```bash
# Clone repo
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Build image
docker build -t my-prismnote:latest .

# Run custom image
docker run -p 8000:8000 my-prismnote:latest
```

### Docker Security Best Practices

1. **Use secrets for credentials:**
```bash
docker secret create db_password -
# (enter password, Ctrl+D)

docker service create \
  --secret db_password \
  prismnote:latest
```

2. **Run as non-root:**
```yaml
services:
  prismnote:
    image: prismnote:latest
    user: "1000:1000"
    # Instead of running as root
```

3. **Use volume mounts for persistence:**
```bash
docker run \
  -v prismnote_data:/root/.prismnote \
  prismnote:latest
```

4. **Network isolation:**
```yaml
networks:
  prismnote-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

### Docker Registry

**Docker Hub:**
```bash
docker pull mullassery/prismnote:latest
docker run -p 8000:8000 mullassery/prismnote:latest
```

**GitHub Container Registry:**
```bash
docker pull ghcr.io/mullassery/prismnote:latest
docker run -p 8000:8000 ghcr.io/mullassery/prismnote:latest
```

### Docker Health Checks

Built-in health check:
```bash
docker inspect --format='{{.State.Health.Status}}' <container-id>
```

View health logs:
```bash
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' <container-id>
```

### Troubleshooting Docker

**Port already in use:**
```bash
docker run -p 8001:8000 prismnote:latest
# Access at http://localhost:8001
```

**Permission denied:**
```bash
# Use sudo
sudo docker run -p 8000:8000 prismnote:latest

# Or add user to docker group
sudo usermod -aG docker $USER
newgrp docker
```

**Persistent data not saving:**
```bash
# Check volume
docker volume ls

# Inspect volume
docker volume inspect prismnote_data

# Mount with correct permissions
docker run -v /path/to/host:/root/.prismnote prismnote:latest
```

**Memory issues:**
```bash
docker run --memory=2g --memory-swap=2g prismnote:latest
```

---

## Current Features (v0.3)

### Notebook Execution
- Python 3.8+ code cells with Jupyter kernel
- Markdown cells with syntax highlighting
- SQL cells (--sql or %sql markers)
- PySpark cells for big data
- Cell timeout control (30s default)
- Auto-save (5s interval)
- Execution history and rollback

### Output Rendering
- Text, images (PNG, JPEG), HTML, SVG
- pandas DataFrames as tables
- matplotlib, plotly, altair charts
- folium maps
- JSON pretty-printing
- Jupyter-compatible .ipynb format

### Code Intelligence (AI Features)
- Claude API code explanation
- AI-powered error fixes
- Code completion with context
- Library recommendations (unique feature)
- SQL query optimization (7 pattern types)
- Ollama local model support (free)

### Notebook Management
- Search across notebooks and cells
- Variable inspector with types
- Cell execution statistics
- Output truncation (10MB limit)
- Notebook versioning with branching
- Code validation (unsafe pattern detection)

### Version Control
- Git-like notebook versioning
- Automatic version snapshots
- Version branching for parallel work
- Rollback to previous versions
- Version diffing
- Author tracking with messages

### Access Control & Governance
- 4-tier RBAC (Owner/Editor/Commenter/Viewer)
- Complete audit logging
- User and group management
- Session tracking (IP, user-agent)
- Fine-grained permission control
- Audit trail export (CSV/JSON)

### Data Management
- Automatic DataFrame profiling
- Data quality issue detection
- Missing data pattern analysis
- Type inference
- Column statistics (mean, median, std dev)
- Outlier detection
- Low variance identification

### Big Data Support
- Apache Spark integration (local & distributed)
- PySpark notebook cells
- DataFrame caching and optimization
- Shuffle analysis for performance
- Executor memory configuration
- Session management for Spark clusters

### SQL & Analytics (v0.3)
- Native SQL cell execution
- Query optimization suggestions (7 patterns)
- Support for 5 databases (PostgreSQL, MySQL, SQLite, DuckDB, MongoDB)
- Cloud data warehouse support (8 platforms):
  - Snowflake (per-credit billing)
  - Google BigQuery (per-TB scanned)
  - AWS Redshift (per-hour)
  - Azure Synapse (per-DWU)
  - Databricks (per-DBU)
  - AWS Athena (S3 analytics)
  - Presto / Trino (open source)
- Query cost estimation

### Notebook Scheduling
- Cron-based scheduling (minute, hour, day, month, weekday)
- Job execution history
- Automatic retry logic
- Email notifications
- Timeout and resource limits
- Manual execution trigger

### AI Model Training (v0.3)
- Fine-tune open-source LLMs (LLaMA, Mistral, Falcon, Code Llama)
- LoRA and QLoRA optimization
- RunPod, Lambda Labs, Vast.ai integration
- Cost estimation and tracking
- Model checkpoint management
- Inference endpoint deployment

### Enterprise Authentication (v0.3)
- Microsoft Azure AD integration
- LDAP/Active Directory support
- SAML 2.0 (Okta, OneLogin, Ping)
- OAuth2 (Google, GitHub, custom)
- Multi-tenant support with isolation
- Multi-factor authentication (TOTP, SMS)
- SSO (Single Sign-On)
- Group-based RBAC

### Performance Optimization
- Smart DAG execution with dependency detection
- Skip unchanged cells (incremental execution)
- Parallel execution support
- Execution result caching
- Memory-aware output truncation

### Platform Support
- macOS (Intel & Apple Silicon M1-M5+)
- Linux (x86_64, ARM64)
- Windows (WSL2)
- Kubernetes deployment ready

---

## Upcoming Features (v0.4 - Q3 2026)

### Real-Time Collaboration
- Live cell editing with cursor position sync
- User presence with color coding
- Comment threads on cells
- @mentions in comments
- Real-time presence updates
- Automatic conflict resolution (OT)

### File Upload/Download UI
- Drag-and-drop file upload
- File browser and preview
- Download management
- File size validation (500MB limit)
- Notebook-scoped file storage
- Safe filename handling

### Cloud Storage Integration
- Google Drive mounting
- Amazon S3 bucket access
- Google Cloud Storage (GCS)
- Azure Blob Storage
- Encrypted credential storage
- Multi-provider support

---

## Enterprise Features (v1.0 - Q1 2027)

### GitHub Integration (v0.5)
- Push notebooks to GitHub
- Pull notebooks from GitHub
- Bidirectional sync
- Automatic backups
- Branch management
- Commit history

### Output Display Enhancements (v0.5)
- Zoom in/out on outputs (0.5x - 3.0x)
- Fullscreen mode for visualizations
- Pan and scroll support
- Download as image
- Copy to clipboard
- Auto-fit to width

### Typography Settings (v0.5)
- Font size adjustment (10-20px)
- macOS fonts: SF Mono, Monaco, Menlo (⚠️ macOS only)
- Cross-platform fonts: Roboto Mono, JetBrains Mono, Cascadia Code, Courier New
- Line height adjustment
- Theme selection (dark/light)

### Kubernetes Deployment (v1.0)
- Auto-generated Kubernetes manifests
- Multi-replica deployments
- Resource requests/limits
- Ingress with TLS
- Pod scaling and monitoring
- StatefulSet for persistence

### Docker Support (v1.0)
- Pre-built images (x86_64, ARM64, Apple Silicon)
- Docker Compose for single-server
- Multi-stage optimized builds
- Health check configuration
- Registry distribution

### dbt Integration (v1.0)
- dbt project configuration
- Model discovery and documentation
- Test execution and reporting
- Lineage visualization
- Profile management
- Project scaffolding

### Apache Airflow Integration (v1.0)
- DAG creation and management
- Task dependency visualization
- DAG execution triggering
- Run status monitoring
- Logs streaming
- Scheduling configuration

### Roadmap (v1.1+)
- Kafka topic management and streaming
- Apache Flink (PyFlink) for stream processing
- Advanced analytics integrations
- Real-time kernel sync across users
- Mobile apps (view-only)

---

## Feature Comparison

| Feature | PrismNote | JupyterLab | Zeppelin | Colab |
|---------|-----------|-----------|----------|-------|
| Code Execution | Python | Python | Multi-lang | Python |
| SQL Support | Native | Plugins | Native | Basic |
| Spark Support | Native | None | Native | Limited |
| Versioning | Git-like | No | Basic | Limited |
| RBAC | 4-tier | Minimal | Basic | Basic |
| Audit Logging | Complete | None | Limited | None |
| Scheduling | Cron | No | Yes | No |
| Data Profiling | Auto | No | Plugins | No |
| AI Assistance | Claude/OpenAI | No | No | Yes |
| Cloud Warehouses | 8 platforms | No | Limited | No |
| Kubernetes | Ready | No | No | No |
| Self-Hosted | Full | Yes | Yes | No |
| MIT License | Yes | Yes | Yes | No |

---

## Installation Details

### System Requirements

**Minimum:**
- Python 3.8+
- 4GB RAM
- 500MB disk space
- macOS 11+, Linux, or Windows (WSL2)

**For Spark/ML:**
- 8GB RAM recommended
- Java 11+ (for Spark)
- GPU optional (for model training)

**For Enterprise:**
- Azure AD, LDAP, or SAML provider (optional)

### What Gets Installed

```
~/.prismnote/
├── notebooks/          # Your .ipynb files
├── files/             # Uploaded files
├── versions/          # Notebook versions
├── acl/              # Access control
├── schedules/        # Scheduled jobs
├── cloud-storage/    # Cloud credentials
├── config.json       # User settings
└── cache/            # Execution cache
```

### Environment Variables

**Core:**
- `PRISMNOTE_DIR`: Notebook directory (~/.prismnote)
- `PRISMNOTE_PORT`: Server port (8000)
- `RUST_LOG`: Logging level (info)

**AI Features:**
- `ANTHROPIC_API_KEY`: Claude API key
- `OPENAI_API_KEY`: OpenAI API key
- `PRISMNOTE_AI_PROVIDER`: Provider (claude/openai/ollama)
- `PRISMNOTE_OLLAMA_URL`: Ollama server URL

**Big Data:**
- `PRISMNOTE_SPARK_MASTER`: Spark master URL
- `PRISMNOTE_SPARK_MEMORY`: Executor memory (2g-8g)

**Enterprise:**
- `PRISMNOTE_AUTH_PROVIDER`: Auth method (aad/ldap/saml/oauth2)
- `PRISMNOTE_AAD_TENANT_ID`: Azure AD tenant
- `PRISMNOTE_AAD_CLIENT_ID`: Azure AD app ID
- `RUNPOD_API_KEY`: RunPod API key for GPU training

---

## Cell Types & Syntax

### Python Code Cell
```python
import pandas as pd
df = pd.read_csv('data.csv')
print(df.head())
```
**Run with:** Shift+Enter

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

### Markdown Cell
```markdown
# Title
This is **bold** and *italic*
- Bullet point
- Another point
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
  "ai_provider": "claude",
  "font_family": "Roboto Mono",
  "font_size": 14
}
```

### Enable Features

**Enable Spark:**
```python
from pyspark.sql import SparkSession
spark = SparkSession.builder \
    .appName("my-app") \
    .config("spark.executor.memory", "4g") \
    .getOrCreate()
```

**Enable SQL:**
1. Go to Settings → Databases
2. Add database connection
3. Use --sql marker in cells

**Enable Scheduling:**
1. Go to Notebook → Schedule
2. Enter cron: `0 9 * * 1` (9am Mondays)
3. Enable notifications

**Enable Enterprise Auth:**
1. Set `PRISMNOTE_AUTH_PROVIDER=aad`
2. Configure credentials
3. Restart PrismNote

---

## Common Use Cases

### Data Analysis
```python
import pandas as pd
df = pd.read_csv('sales.csv')
print(df.info())
print(df.describe())
print(df.groupby('region').sum())
df.plot(x='month', y='revenue')
```

### SQL + Python
```sql
--sql
SELECT date, SUM(revenue) as total
FROM sales
WHERE date > '2024-01-01'
GROUP BY date
```

### Big Data with Spark
```python
from pyspark.sql import SparkSession
spark = SparkSession.builder.getOrCreate()

df = spark.read.parquet('s3://bucket/data/')
df.filter(df.amount > 100).show()
```

### Scheduled Report
```python
# Runs every Monday at 9am
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

### "Module not found"
```bash
pip install missing_module
# Restart PrismNote
```

### Notebooks not saving
```bash
chmod 755 ~/.prismnote/
```

### Slow execution
- Use smaller notebooks
- Enable Spark for big data
- Increase cell timeout if needed
- Check disk space in ~/.prismnote/

### "Kernel not available"
```bash
pip install ipykernel
# Restart PrismNote
```

### Database connection fails
- Verify database is running
- Check credentials
- Verify firewall allows connection
- Use Test Connection button

---

## Architecture

```
Browser (React)
    ↓ REST/WebSocket
Rust Server (Axum)
    ├── Notebook Manager
    ├── Cell Executor
    ├── Database Connector
    ├── Spark Manager
    ├── Cloud Warehouse Manager
    ├── AI Training Manager
    └── Auth Manager
    ↓
Jupyter Kernel (ipykernel)
    ↓
Python Runtime
```

---

## Build from Source

```bash
git clone https://github.com/Mullassery/prismnote.git
cd prismnote

# Build Rust backend
cargo build --release

# Build React frontend
cd frontend
npm install
npm run build

# Run
cargo run --release
```

---

## Development Mode

**Terminal 1 (Rust backend):**
```bash
cargo watch -x 'run --release'
```

**Terminal 2 (React frontend):**
```bash
cd frontend && npm run dev
```

**Browser:** `http://localhost:5173`

---

## API Reference

### Notebooks
```
GET    /api/notebooks                 List all notebooks
POST   /api/notebooks                 Create notebook
GET    /api/notebooks/:id             Get notebook
PUT    /api/notebooks/:id             Update notebook
DELETE /api/notebooks/:id             Delete notebook
POST   /api/notebooks/:id/execute     Execute cell
```

### Collaboration (v0.4+)
```
POST   /api/notebooks/:id/collaborate          Join collaboration
GET    /api/notebooks/:id/collaborators        Get active users
POST   /api/notebooks/:id/comments             Add comment
```

### Files (v0.4+)
```
POST   /api/notebooks/:id/files                Upload file
GET    /api/notebooks/:id/files                List files
GET    /api/notebooks/:id/files/:file_id       Download
DELETE /api/notebooks/:id/files/:file_id       Delete
```

### Cloud Storage (v0.4+)
```
POST   /api/cloud-storage              Add cloud storage
GET    /api/cloud-storage              List storages
DELETE /api/cloud-storage/:name        Remove storage
```

### SQL Execution
```
POST   /api/sql/execute                Execute query
POST   /api/sql/optimize               Get optimization suggestions
```

### Spark
```
POST   /api/spark/sessions             Create session
GET    /api/spark/sessions             List sessions
GET    /api/spark/sessions/:id         Get session details
```

### Cloud Warehouses
```
POST   /api/cloud-warehouses                   Create connection
GET    /api/cloud-warehouses                   List connections
POST   /api/cloud-warehouses/:id/test          Test connection
POST   /api/cloud-warehouses/:id/query         Execute query
GET    /api/cloud-warehouses/:id/estimate-cost Estimate cost
```

### AI Features
```
POST   /api/ai/explain                 Explain code
POST   /api/ai/fix                     Fix error
POST   /api/ai/complete                Complete code
GET    /api/notebooks/:id/suggest-libraries    Suggest libraries
```

### Model Training
```
POST   /api/ai/fine-tuning/jobs               Create job
GET    /api/ai/fine-tuning/jobs               List jobs
GET    /api/ai/fine-tuning/jobs/:id           Get job details
POST   /api/ai/fine-tuning/jobs/:id/start     Start training
POST   /api/ai/inference/endpoints            Deploy endpoint
```

### GitHub Integration (v0.5+)
```
POST   /api/github/configure           Setup GitHub auth
POST   /api/notebooks/:id/github/sync  Sync with repo
POST   /api/notebooks/:id/github/push  Push changes
GET    /api/notebooks/:id/github/pull  Pull changes
```

### Display Settings (v0.5+)
```
GET    /api/settings/display           Get display settings
PUT    /api/settings/display           Update settings
GET    /api/settings/fonts/mac         Get Mac-compatible fonts
```

### Infrastructure (v1.0+)
```
GET    /api/infra/k8s/manifest         Kubernetes manifest
POST   /api/infra/k8s/deploy           Deploy to Kubernetes
GET    /api/infra/docker/compose       Docker Compose config
GET    /api/airflow/dags               List DAGs
POST   /api/dbt/config                 dbt configuration
```

---

## Requirements

### For Basic Use
- Python 3.8+
- Modern web browser (Chrome, Firefox, Safari, Edge)

### For SQL/Database
- PostgreSQL 10+, MySQL 5.7+, or SQLite (no install)

### For Spark
- Java 11+ (`java -version`)

### For Model Training
- GPU: NVIDIA (CUDA 11.8+) or Mac GPU
- Account: RunPod, Lambda Labs, or Vast.ai

### For Enterprise Auth
- Azure AD, LDAP, SAML, or OAuth2 provider

---

## Documentation

- `INSTALLATION.md` — Detailed setup instructions
- `SQL_EXECUTION.md` — SQL cells and optimization
- `SPARK_MANAGEMENT.md` — Spark configuration
- `CLOUD_WAREHOUSES.md` — Data warehouse integration
- `AI_TRAINING_FINETUNING.md` — Model fine-tuning
- `ENTERPRISE_AUTHENTICATION.md` — Auth setup
- `CRITICAL_GAPS_V04.md` — v0.4 features
- `V04_AND_V05_FEATURES.md` — v0.4/v0.5 reference
- `ENTERPRISE_V10_FEATURES.md` — v1.0 roadmap
- `COMPETITIVE_GAP_ANALYSIS.md` — Feature comparison

---

## FAQ

**Q: Is PrismNote free?**
A: Yes, it's MIT licensed and completely free for all uses.

**Q: Can I use it offline?**
A: Yes, it runs 100% locally. AI features are optional.

**Q: How do I share notebooks?**
A: Export as .ipynb or enable RBAC for team access.

**Q: Does it work on Windows?**
A: Yes, via WSL2.

**Q: Can I run it on a server?**
A: Yes, set `PRISMNOTE_LISTEN=0.0.0.0:8000`.

**Q: How do I backup notebooks?**
A: Copy `~/.prismnote/notebooks/` to any location.

**Q: Can I migrate from Jupyter?**
A: Yes, PrismNote uses `.ipynb` format. Copy files to `~/.prismnote/notebooks/`.

**Q: What about data privacy?**
A: Data stays on your machine. No cloud upload unless you configure cloud storage.

**Q: Can I use it in production?**
A: Yes, v0.3+ is production-ready with enterprise auth and audit logging.

**Q: When is v1.0 releasing?**
A: Planned for Q1 2027 with Kubernetes and dbt/Airflow support.

---

## Community & Support

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

## Contributing

Contributions welcome! See `CONTRIBUTING.md` for guidelines.

**Code of Conduct:** See `CODE_OF_CONDUCT.md`

---

## License & Acknowledgments

**License:** MIT (free for personal, commercial, educational use)

**Built with:**
- Rust (Axum, Tokio)
- React (TypeScript, Vite)
- Python (Jupyter kernel)

**Thanks to the open-source community!**

---

## Roadmap

```
v0.1 (June 2024)   → Basic notebook functionality
v0.2 (June 2026)   → Versioning, RBAC, scheduling
v0.3 (June 2026)   → SQL, Spark, AI training, enterprise auth
v0.4 (Aug 2026)    → Real-time collaboration, files, cloud storage
v0.5 (Nov 2026)    → GitHub integration, display settings
v1.0 (Jan 2027)    → Kubernetes, dbt, Airflow
v1.1 (Apr 2027)    → Kafka, Flink, streaming
```

---

**Made with Rust + React | Open Source | MIT License**

**Star us on GitHub:** [Mullassery/prismnote](https://github.com/Mullassery/prismnote)

Join our community and help shape the future of data science notebooks!
