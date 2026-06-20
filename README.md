# PrismNote

An enterprise-grade, open-source data science notebook platform with Rust backend performance, Jupyter compatibility, and production-ready features for teams. Built for reproducibility, scalability, and security.

## Core Features

**Notebook Execution**
- Code cell execution with Python 3.8+ (via Jupyter kernel)
- Markdown cell support with syntax highlighting
- Jupyter-compatible .ipynb format (full round-trip import/export)
- Reactive execution with smart state management
- Cell timeout control (configurable)

**Rich Output Rendering**
- Text, images (PNG, JPEG), HTML, SVG rendering
- pandas DataFrames as HTML tables
- Matplotlib, Plotly, Altair visualization support
- JSON pretty-printing
- High-resolution display support (Retina, 4K monitors)

**Code Intelligence**
- Claude API integration for code explanation
- AI-powered error fixes and code completion
- Ollama local model support (free)
- OpenAI integration (optional)
- Multi-provider AI assistant

**Notebook Features**
- Auto-save with configurable intervals
- Search across notebooks and cells
- Cell execution history
- Output truncation (10MB limit, configurable)
- Code validation (unsafe pattern detection)

**Version Control & Collaboration**
- Git-like notebook versioning with branching
- Automatic version snapshots with author tracking
- Rollback to previous versions
- Version diffing
- Notebook branching for parallel development

**Access Control & Governance**
- Role-based access control (Owner/Editor/Commenter/Viewer)
- Fine-grained permission management
- Complete audit logging for compliance
- User and group management
- Session tracking with IP and user-agent

**Data Management**
- Automatic data profiling (column stats, data types)
- Data quality issue detection (null counts, outliers)
- Missing data pattern analysis
- DataFrame introspection and statistics
- Variable inspector with type information

**Big Data Support**
- Apache Spark integration (local and distributed)
- PySpark notebook cells
- Spark DataFrame caching and optimization
- Shuffle analysis and recommendations
- Session management for Spark clusters

**SQL & Analytics**
- Native SQL cell execution (--sql and %sql markers)
- Query optimization suggestions (7 pattern types)
- Support for 5 major databases (PostgreSQL, MySQL, SQLite, DuckDB, MongoDB)
- Cloud data warehouse integration (8 platforms):
  - Snowflake
  - Google BigQuery
  - AWS Redshift
  - Azure Synapse
  - Databricks
  - AWS Athena
  - Presto / Trino
- Query cost estimation and tracking

**Notebook Scheduling & Automation**
- Cron-based scheduling (minute, hour, day, month, weekday)
- Job execution history with results
- Automatic retry logic
- Email notifications
- Timeout and resource limits

**AI Model Training**
- Fine-tune open-source LLMs (LLaMA, Mistral, Falcon, Code Llama)
- LoRA and QLoRA optimization
- GPU compute provider integration (RunPod, Lambda Labs, Vast.ai)
- Cost estimation and tracking
- Model checkpoint management
- Inference endpoint deployment

**Enterprise Features**
- Microsoft Azure AD integration with group-based RBAC
- LDAP/Active Directory support
- SAML 2.0 (Okta, OneLogin, Ping)
- OAuth2 and Google Workspace
- Multi-factor authentication (TOTP, SMS, email)
- Multi-tenant architecture with complete isolation
- SSO (Single Sign-On)
- Comprehensive security audit logging
- IP whitelisting and access restrictions
- Password policies and expiration

**Performance Optimization**
- Smart execution DAG (directed acyclic graph)
- Automatic dependency detection
- Skip unchanged cells (incremental execution)
- Parallel execution support
- Execution caching
- Memory-aware output truncation

**Platform Support**
- Apple Silicon (M1-M5+) with auto-detection
- macOS (Intel and Apple Silicon)
- Linux (x86_64, ARM64)
- Windows (with WSL2)
- Kubernetes deployment ready

## Installation

### Option 1: pip (Recommended)
```bash
pip install prismnote
prismnote
```
Automatically detects and downloads native binary for your platform.

### Option 2: uv
```bash
uv tool install prismnote
prismnote
```

### Option 3: curl (Binary Download)
```bash
bash <(curl -fsSL https://raw.githubusercontent.com/Mullassery/prismnote/main/install.sh)
prismnote
```
Detects OS and architecture, downloads and installs to /usr/local/bin.

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

### v0.4 (Planned 2026-Q3)
- Real database driver integration (live query execution)
- RunPod/Lambda Labs live training execution
- OAuth2 real token exchange
- ZMQ kernel protocol (full Jupyter compatibility)
- Real-time collaboration (WebSocket sync)
- Advanced performance dashboards
- Distributed execution framework

### v0.5 (Planned 2026-Q4)
- Multi-factor authentication (TOTP, SMS, email)
- Advanced access controls and device trust
- Conditional access policies
- Risk-based authentication
- Real-time threat detection
- Data retention policies

### v1.0 (Planned 2027-Q1)
- Production-grade maturity and SLA
- Distributed training across multiple machines
- MLOps integration (MLflow, Weights & Biases)
- Kubernetes native deployment with Helm charts
- SOC2 Type II compliance
- HIPAA compliance certification
- Enterprise support programs and SLAs
- Passwordless authentication (WebAuthn/FIDO2)
- Biometric authentication support

## Feature Comparison

| Feature | PrismNote | JupyterLab | Zeppelin | Apache Airflow |
|---------|-----------|-----------|----------|-----------------|
| UI/UX | Modern | Functional | Good | Functional |
| License | MIT | BSD-3-Clause | Apache 2.0 | Apache 2.0 |
| Self-Hosted | Yes | Yes | Yes | Yes |
| Open Source | Yes | Yes | Yes | Yes |
| Cloud Ready | Yes (v0.4) | Plugins | Yes | Yes |
| **Versioning** | Yes | No | Yes | Yes |
| **RBAC** | Yes | Minimal | Yes | Yes |
| **Scheduling** | Yes | No | Yes | Yes |
| **SQL Support** | Yes (native) | Plugins | Yes | Yes |
| **Spark Integration** | Yes (native) | No | Yes | Yes |
| **AI Assistance** | Yes | No | No | No |
| **Data Profiling** | Yes | No | No | No |
| **Model Training** | Yes | No | No | No |
| **Enterprise Auth** | Yes (AAD/LDAP) | Minimal | No | Yes |
| **Multi-Tenant** | Yes | No | No | Yes |
| **Cost Tracking** | Yes | No | Limited | No |
| **Jupyter Compatible** | Yes | Native | No | No |
| **Offline Mode** | Yes | Yes | Yes | Yes |
| **Free Tier** | Unlimited | Unlimited | Unlimited | Unlimited |

## Documentation

Comprehensive documentation is available for all features:

**User Guides**
- INSTALLATION.md - Setup instructions for all platforms
- LIBRARY_SUGGESTIONS_QUICKSTART.md - Quick start for AI library recommendations
- MACBOOK_SUPPORT.md - Complete Apple Silicon (M1-M5+) support guide
- COMPARISON_OSS_NOTEBOOKS.md - Detailed feature comparison with alternatives

**Technical Documentation**
- SQL_EXECUTION.md - SQL cell execution and query optimization
- SPARK_MANAGEMENT.md - Spark session management and optimization
- EXECUTION_PIPELINE.md - DAG-based execution planning
- CLOUD_WAREHOUSES.md - Cloud data warehouse integration (8 platforms)
- AI_TRAINING_FINETUNING.md - Model fine-tuning and training
- ENTERPRISE_AUTHENTICATION.md - AAD, LDAP, SAML, OAuth2 setup
- BUILD_STATUS_V02.md - v0.2 implementation status
- IMPLEMENTATION_SUMMARY_V02.md - Technical implementation details
- V02_FEATURES.md - v0.2 feature overview
- V03_IMPLEMENTATION_COMPLETE.md - v0.3 completion status
- BIGDATA_FEATURES_PRIORITY.md - Big data feature roadmap
- BIGDATA_IMPLEMENTATION.md - Big data feature implementation

**Developer Documentation**
- CONTRIBUTING.md - Contribution guidelines
- .cargo/config.toml - Multi-platform build configuration
- build.sh / build-macos.sh - Build scripts for all platforms

## Dependencies & Licenses

**Backend (Rust)**
- axum (MIT)
- tokio (MIT)
- serde (MIT/Apache-2.0)
- tower (MIT)
- tracing (MIT)

**Frontend (TypeScript/React)**
- React 18 (MIT)
- TypeScript (Apache-2.0)
- Vite (MIT)
- Monaco Editor (MIT)
- Tailwind CSS (MIT)
- Zustand (MIT)

**Database Drivers (OSS-Compliant)**
- PostgreSQL (LGPL with exception)
- MySQL (MIT)
- SQLite (Public Domain)
- DuckDB (MIT)
- MongoDB (SSPL - check license compatibility)

**Optional Cloud Integrations**
- Claude API (optional - Anthropic commercial)
- Ollama (MIT - local, free)
- OpenAI API (optional - OpenAI commercial)
- RunPod (proprietary platform, optional)

All dependencies are open-source compatible. Proprietary integrations (Claude, OpenAI, RunPod) are optional and clearly marked.

## Contributing

Contributions welcome! PrismNote is built by the community for the community.

**How to Contribute**
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Areas for Contribution**
- Bug fixes and performance improvements
- Documentation enhancements
- New database connectors
- UI/UX improvements
- Testing and test coverage
- Localization/translation

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors.
See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

## Support

**Getting Help**
- GitHub Issues for bug reports and feature requests
- GitHub Discussions for questions and ideas
- Documentation at docs/ directory
- Community Slack channel (coming soon)

**Commercial Support**
Enterprise support, custom development, and deployment assistance available.
Contact: support@prismnote.dev

## License

**MIT License**

PrismNote is released under the MIT License, which allows:
- Commercial use
- Modification
- Distribution
- Private use

With conditions:
- License and copyright notice included in copies or derivatives

Copyright © 2026 Georgi Mammen Mullassery

Full license text: [LICENSE](LICENSE)

## Security & Privacy

**Data Security**
- All data stored locally by default (self-hosted)
- End-to-end encryption for cloud features
- No telemetry or tracking (local notebooks only)
- TLS/HTTPS for all remote connections

**Privacy Commitment**
- User notebooks never sent to third parties without explicit consent
- Optional AI features only connect when explicitly requested
- Enterprise deployments can be fully air-gapped

**Reporting Security Issues**
For security vulnerabilities, please email security@prismnote.dev rather than using public issue tracker.

## Acknowledgments

Built with love using:
- Rust ecosystem (Tokio, Axum, Serde)
- React and TypeScript community
- Python scientific computing stack
- Jupyter community and protocols
- Inspiration from Jupyter, Zeppelin, Marimo, and Colab

## Disclaimer

PrismNote is provided as-is for educational, research, and professional use.
Users are responsible for securing their notebooks and data.
Developers are not liable for data loss or misuse of the platform.
