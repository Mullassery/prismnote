<div align="center">

# ◆ PrismNote

**A fast, modern, open-source data-science notebook.**
Rust engine · React UI · local-first · AI-native.

**License:** MIT · **Status:** Beta (v0.4)

</div>

---

## Overview

PrismNote is a local-first alternative to Jupyter, Zeppelin, and Databricks notebooks.
It pairs a **Rust + Axum** backend and a **persistent Python kernel** with a modern,
**VS Code-style React** interface. Write Python and SQL, get rich output and live
charts, ask a local AI for help, schedule notebooks as jobs, manage git, and ship to
the cloud — all from one app that runs entirely on your machine.

- **Local-first & private** — runs on your machine; local AI via Ollama, no account required.
- **Jupyter-compatible** — native `.ipynb` format.
- **Batteries included** — kernel, SQL, charts, AI, jobs, git, deploy, search, terminal.

---

## Features

### Notebook & execution
- **Persistent shared kernel** — variables, imports, and functions defined in one cell
  are available in every later cell (true notebook semantics).
- **Per-cell interpreters (magics)**:
  - `%python` (default) — runs in the shared kernel.
  - `%sql` — in-process **DuckDB**; can query pandas DataFrames defined in other cells.
  - `%sh` / `%bash` / `!cmd` — shell commands.
  - `%md` / `%markdown` — rendered markdown.
- **Rich output** — text, HTML, matplotlib figures (`image/png`), and pandas DataFrames.
- **Chart switcher** — view any DataFrame as a **Table, Bar, or Line** chart.
- **Live streamed output** over WebSocket while a cell runs.
- **Interrupt** a running cell (Stop) and **restart** the kernel from the toolbar/menu.
- **Dynamic input widgets** — `prism.input / slider / select / checkbox` render controls
  that re-run the cell when changed (Databricks `dbutils.widgets`-style).
- Pretty-printing enabled by default via `rich`.

### AI assistance
Works with **local models via Ollama** (free, private, offline) or **Claude / OpenAI**.
- **⌘K in-cell edit** — describe a change, review a diff, accept or reject.
- **Fix with AI** — one click on an errored cell.
- **Explain** — a teaching-oriented explanation with a contextual tip.
- **Inline autocomplete** — ghost-text completions when Ollama is connected.
- **Teacher persona** — any model explains the "why" and suggests tips/tricks.
- **Agent panel** — Plan/Act assistant that can propose and apply cell edits.

### Errors that help
- **Natural-language explanations** for common Python errors (NameError, KeyError,
  ModuleNotFoundError, TypeError, ZeroDivisionError, SyntaxError, …).
- **In-editor markers** highlighting the offending **line and column**.
- Collapsible full traceback when you want the details.

### Workflow: Jobs, Airflow, Git, Deploy
- **Jobs** — save a whole notebook and run it as a unit, **manual / interval / daily**,
  with status and run history (Airflow-style).
- **Airflow** — a stable `run-by-name` trigger plus a **generated, ready-to-use DAG**.
- **Source control** — `init`, `clone`, `commit`, `push`, `pull`, `status` from the UI.
- **Cloud deploy** — generated `Dockerfile`, `docker-compose.yml`, `k8s.yaml`, and
  `fly.toml`, each with its one-line deploy command.

### Workspace & UX
- VS Code-style layout: activity rail, collapsible panels, status bar.
- **All panels collapsible**, each with **independent font +/-** controls.
- **Responsive** — side panels auto-collapse on narrow windows.
- **Global search (⌘K)**, **command palette (⇧⌘P)**, working top menus.
- Integrated **Terminal**, a **Python Console** (shares the kernel), and a **file browser**
  (works in any browser via a server-side browser).
- Dark/light themes; `.ipynb` import/export.

### Data connectivity
- DuckDB in-process SQL engine.
- Connection scaffolding for 8 cloud warehouses (Snowflake, BigQuery, Redshift,
  Azure Synapse, Databricks, Athena, Presto, Trino) — see [CLOUD_WAREHOUSES.md](CLOUD_WAREHOUSES.md).

---

## Architecture

```
┌──────────────────────────────┐        ┌─────────────────────────────┐
│  React + TypeScript (Vite)   │  HTTP  │      Rust backend (Axum)    │
│  Monaco · Tailwind · zustand │ ─────▶ │  REST API + WebSocket       │
│  cells · panels · AI · jobs  │ ◀───── │  routing, jobs, git, deploy │
└──────────────────────────────┘   WS   └──────────────┬──────────────┘
                                                        │ stdin/stdout (JSON)
                                                ┌───────▼────────┐
                                                │ Persistent      │
                                                │ Python kernel   │
                                                │ (shared globals)│
                                                └─────────────────┘
```

- **Backend**: Rust (Axum + Tokio). Spawns one long-lived Python process and talks to
  it over a line-framed JSON protocol; outputs are Jupyter-style MIME bundles.
- **Frontend**: React 18 + TypeScript + Vite, Monaco editor, Tailwind CSS.
- **Kernel**: a single `python` process with a shared namespace; SIGINT-interruptible,
  restartable, with matplotlib (Agg) and rich preloaded.

---

## Getting started

### Prerequisites
- **Rust** (stable) and **Node.js 18+**
- **Python 3.8+** on your `PATH` with `ipykernel`:
  ```bash
  pip install ipykernel pandas matplotlib rich duckdb
  ```
- *(optional)* **Ollama** for local AI: https://ollama.com — then `ollama pull qwen2.5-coder`

### Run in development
```bash
# 1) backend (port 8000)
cargo run

# 2) frontend (port 5173, proxies /api and /ws to the backend)
cd frontend && npm install && npm run dev
```
Open http://localhost:5173.

### Configure AI (optional)
- **Local (Ollama)**: start Ollama and pick a model in the AI panel.
- **Claude/OpenAI**: set in the AI settings, or via env:
  ```bash
  export PRISMNOTE_AI_PROVIDER=claude   # or openai / ollama
  export ANTHROPIC_API_KEY=...          # or OPENAI_API_KEY
  ```
  Settings saved in the UI persist to `~/.prismnote/ai_config.json`.

---

## Deploy

Open **Deploy to Cloud** (rocket icon) to copy/download ready-to-use artifacts:

```bash
docker compose up -d          # Docker
kubectl apply -f k8s.yaml     # Kubernetes
fly launch --copy-config --now# Fly.io
```

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `⌘N` / `⌘O` / `⌘S` | New / Open / Save notebook |
| `⌘K` | Global search |
| `⇧⌘P` | Command palette |
| `⌘K` *(in a cell)* | AI edit |
| `⌘⇧⏎` | Run all cells |
| `⇧⏎` | Run cell |

---

## Repository layout

```
crates/server/      Rust backend (api, kernel, jobs, db, ws, …)
frontend/           React app (components, hooks, api clients)
*.md                Architecture & comparison docs
```

Comparisons: [ZEPPELIN_COMPARISON.md](ZEPPELIN_COMPARISON.md) ·
[DATABRICKS_COMPARISON.md](DATABRICKS_COMPARISON.md) ·
[NOTEBOOK_COMPARISON_MATRIX.md](NOTEBOOK_COMPARISON_MATRIX.md)

---

## Roadmap

- Distributed compute (Spark) and a catalog/data browser.
- Real-time collaboration (live cursors/co-editing).
- Notebook parameters and multi-notebook job composition.

---

## License

MIT — see [LICENSE](LICENSE).
