# Changelog

All notable changes to PrismNote will be documented in this file.

## [0.4.3] — 2025-06-28

### Added
- Local-first Jupyter-compatible notebook editor
- Rust + Axum backend with persistent Python kernel
- VS Code-style React UI
- Native `.ipynb` import/export
- Python + SQL execution support
- Rich output rendering with Vega-Lite charts
- Local AI assistance via Ollama
- Scheduled job execution
- Git integration
- Generated cloud deployment files
- Global search across cells
- Integrated terminal
- Variable explorer
- Multi-database SQL connector support

### Features
- **Kernel:** Persistent Python kernel with full REPL
- **SQL:** Execute SQL against local databases and cloud warehouses
- **AI:** Local AI assistance (via Ollama), no account required
- **Charts:** Native Vega-Lite charting
- **Jobs:** Schedule cell execution or full notebook runs
- **Connectors:** DuckDB, PostgreSQL, BigQuery, Snowflake, Databricks, Athena, Redshift, Trino
- **Git:** Version control integration
- **Deploy:** AWS/Docker deployment templates
- **Search:** Full-text search across all cells

### Status
- **Stable:** Notebook, kernel, AI, jobs, git, charts, variable explorer, SQL
- **Beta:** Deployment file generation (templates for review)

## [0.1.0] — Initial Release
- Basic notebook functionality
- Python kernel integration
- Simple SQL support
- UI prototype

---

## Roadmap

### Near-term (v0.5)
- Performance optimizations for large notebooks
- Enhanced SQL debugging
- More cloud database connectors
- Improved collaborative features

### Medium-term (v0.6–v1.0)
- Multi-user collaboration (real-time sync)
- Custom LLM integration
- Workflow automation
- Advanced analytics plugins

### Long-term
- Desktop app (Electron)
- Team deployment guide
- Enterprise features (RBAC, audit logs)
