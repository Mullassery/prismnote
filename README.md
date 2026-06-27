<div align="center">

# ◆ PrismNote

**A fast, modern, open-source data-science notebook.**
Rust engine · React UI · local-first · AI-native.

**License:** MIT · **Status:** Beta (v0.4.3) · **PyPI:** [`prismnote`](https://pypi.org/project/prismnote/)

</div>

---

## Overview

PrismNote is a local-first alternative to Jupyter, Zeppelin, and Databricks notebooks.
A **Rust + Axum** backend drives a **persistent Python kernel**, and a modern,
**VS Code-style React** UI gives you Python + SQL, rich output and live charts, local
AI assistance, scheduled jobs, git, and generated cloud-deploy files — all on your machine.

- **Local-first & private** — runs locally; local AI via Ollama, no account required.
- **Jupyter-compatible** — native `.ipynb` import/export.
- **Batteries included** — kernel, SQL, charts, AI, jobs, git, deploy, search, terminal.

> **Status note:** PrismNote is **beta**. The notebook, kernel, AI, jobs, git,
> charts, variable explorer, and SQL (databases + warehouses) are functional. SQL runs
> through the kernel using the OSS driver you install (see [CONNECTORS.md](CONNECTORS.md));
> deployment files are **generated templates** you review before shipping. See the
> [roadmap](#roadmap).

---

## Install & run

### From source (recommended today)
Requires **Rust** (stable), **Node 18+**, and **Python 3.8+** on `PATH`:

```bash
# Python runtime deps for the kernel (ipykernel is required; the rest enable
# rich output, %sql, and charts)
pip install ipykernel pandas matplotlib rich duckdb

# 1) backend — http://localhost:8000
cargo run

# 2) frontend — http://localhost:5173 (proxies /api and /ws to the backend)
cd frontend && npm install && npm run dev
```
Open http://localhost:5173.

### From PyPI
```bash
pip install prismnote
```

OR

```bash
uv add prismnote
```

Then run:

```bash
prismnote
```
The PyPI package is a thin launcher: on first run it downloads the prebuilt server
binary for your platform from the matching **GitHub Release** (`vX.Y.Z`). If a release
binary isn't published for your platform/version yet, use the from-source steps above.
(`pip install prismnote` succeeds regardless; only the prebuilt launch depends on a release.)

### Enable AI (optional)
- **Local, free, private — Ollama:** install from https://ollama.com, then
  `ollama pull qwen2.5-coder`. The UI talks to Ollama from the browser, so allow the
  dev origin once:
  ```bash
  OLLAMA_ORIGINS=http://localhost:5173 ollama serve
  ```
- **Claude / OpenAI:** set it in AI settings, or via env (persisted to
  `~/.prismnote/ai_config.json`):
  ```bash
  export PRISMNOTE_AI_PROVIDER=claude   # or openai / ollama
  export ANTHROPIC_API_KEY=...          # or OPENAI_API_KEY
  ```

---

## Features

### Notebook & execution
- **Persistent shared kernel** — variables/imports/functions from one cell are
  available in later cells.
- **Per-cell interpreters (magics):** `%python` (default), `%sql` (in-process DuckDB;
  can query DataFrames from other cells — needs `pip install duckdb`), `%sh` / `!cmd`
  (shell), `%md` (rendered markdown).
- **Rich output** — text, HTML, matplotlib figures, and pandas DataFrames.
- **Chart switcher** — view any DataFrame as **Table / Bar / Line**.
- **Live streamed output** over WebSocket while a cell runs.
- **Interrupt** a running cell and **restart** the kernel.
- **Dynamic input widgets** — `prism.input / slider / select / checkbox` re-run the
  cell when changed.
- **Variable explorer** — a Variables tab listing the kernel's variables (name, type,
  shape, preview).

### Data & SQL
- **Connections** for SQLite, DuckDB, PostgreSQL, MySQL, and 8 cloud warehouses
  (Snowflake, BigQuery, Redshift, Databricks, Athena, Trino, Presto, Synapse).
- **Real query execution** through the kernel using permissively-licensed (OSS)
  drivers you install — no vendored drivers. See [CONNECTORS.md](CONNECTORS.md).
- Results render with the **Table / Bar / Line** switcher, and **Insert into notebook**
  drops a reproducible `df = …` cell.

### AI assistance
Local models via **Ollama**, or **Claude / OpenAI**.
- **In-cell AI edit** (diff accept/reject), **Fix with AI** on errors, **Explain**.
- **Inline autocomplete** (ghost text) when Ollama is connected.
- **Teacher persona** — explanations of the "why" plus a contextual tip.
- **Agent panel** — Plan/Act assistant that proposes and applies cell edits.

### Errors that help
- **Natural-language explanations** for common Python errors.
- **In-editor markers** on the offending line/column, with a collapsible traceback.

### Workflow
- **Jobs** — save a whole notebook and run it as a unit: **manual / interval / daily**,
  with status + run history.
- **Airflow** — a stable `run-by-name` trigger and a **generated DAG**.
- **Source control** — `init`, `clone`, `commit`, `push`, `pull`, `status` from the UI.
- **Cloud deploy** — generates `Dockerfile`, `docker-compose.yml`, `k8s.yaml`, and
  `fly.toml` (review before deploying).

### Editing & menus
- Cell ops: **cut / copy / paste / delete / move up·down**, add code/markdown.
- **Find & Replace** across the notebook (per-cell find via Monaco `⌘F`).
- Run **all / selected / above / below**, **Restart & Run All**, clear outputs.
- Dedicated **Kernel** menu (interrupt / restart / restart & clear).
- Export as **.ipynb** or **.py**; **inline rename** (click the notebook title).

### Workspace & UX
- VS Code-style layout; **all panels collapsible**, each with **independent font +/-**.
- **Responsive** — side panels auto-collapse on narrow windows.
- Global **search**, **command palette**, integrated **terminal**, a **Python console**
  (shares the kernel), and a **file explorer** (works in any browser) with new/rename/
  delete, **upload**, **drag-and-drop**, multi-select, a filter, hidden-files toggle, and
  **git-status decorations**.
- **Ocean-blue** dark theme + light theme; `.ipynb` import/export.

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

- **Backend** spawns one long-lived `python` process and talks to it over a
  line-framed JSON protocol; outputs are Jupyter-style MIME bundles, streamed live.
- **Kernel** has a shared namespace, is SIGINT-interruptible and restartable, with
  matplotlib (Agg) and `rich` preloaded.

---

## Keyboard shortcuts

All of these are wired:

| Shortcut | Action |
|---|---|
| `⌘N` / `⌘O` / `⌘S` | New / Open / Save notebook |
| `⌘K` | Global search *(in a focused cell, `⌘K` is AI edit instead)* |
| `⇧⌘P` | Command palette |
| `⌘,` | Settings |
| `⌘⇧↵` | Run all cells |
| `⇧↵` | Run the focused cell |

*(`⌘`/`Ctrl` depending on platform.)*

---

## Deploy

Open **Deploy to Cloud** to copy/download the generated artifacts, then:

```bash
docker compose up -d            # Docker
kubectl apply -f k8s.yaml       # Kubernetes
fly launch --copy-config --now  # Fly.io
```

---

## Repository layout

```
crates/server/   Rust backend (api, kernel, jobs, db, ws, deploy, git…)
frontend/        React app (components, hooks, api clients)
python/          PyPI launcher package (prismnote)
*.md             Architecture & comparison docs
```

Docs: [CONNECTORS.md](CONNECTORS.md) (data connectors & OSS licensing) ·
[ZEPPELIN_COMPARISON.md](ZEPPELIN_COMPARISON.md) ·
[DATABRICKS_COMPARISON.md](DATABRICKS_COMPARISON.md) ·
[NOTEBOOK_COMPARISON_MATRIX.md](NOTEBOOK_COMPARISON_MATRIX.md)

---

## Roadmap

- Prebuilt release binaries for all platforms (so `pip install` runs out of the box).
- Distributed compute (Spark) and a catalog/data browser.
- Real-time collaboration (live cursors / co-editing).
- Notebook parameters and multi-notebook job composition.
- Reactive (dependency-aware) cell execution.

---

## License

MIT — see [LICENSE](LICENSE).
