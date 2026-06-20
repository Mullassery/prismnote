# PrismNote — Getting Started Guide

## What We Built

A complete scaffold for a modern, high-performance data science notebook with:

- **Rust backend** (Axum + Tokio) for REST API & WebSocket server
- **React frontend** (TypeScript + Monaco Editor + Tailwind CSS) with high-resolution visualization
- **Python package** installable via pip, uv, or curl
- **Jupyter compatibility** (.ipynb import/export)
- **Kernel manager** for Python code execution via Jupyter protocol

## Project Structure

```
prismnote/
├── crates/server/              # Rust backend (Axum HTTP server)
│   ├── src/
│   │   ├── main.rs             # Entry point, routes setup
│   │   ├── api.rs              # Notebook CRUD endpoints
│   │   ├── models.rs           # Data structures
│   │   ├── ws.rs               # WebSocket handler
│   │   ├── kernel.rs           # Jupyter kernel manager (scaffolding)
│   │   └── files.rs            # .ipynb format conversion
│   └── Cargo.toml
│
├── frontend/                   # React + TypeScript UI
│   ├── src/
│   │   ├── components/
│   │   │   ├── Notebook.tsx    # Main notebook layout
│   │   │   ├── Cell.tsx        # Code + Markdown cells (Monaco editor)
│   │   │   ├── Output.tsx      # High-res visualization (PNG, HTML, SVG, tables)
│   │   │   ├── Sidebar.tsx     # File tree + notebook list
│   │   │   └── Toolbar.tsx     # Save, export, theme toggle
│   │   ├── hooks/
│   │   │   └── useNotebook.ts  # Zustand state management + API calls
│   │   ├── index.css           # Tailwind + high-res viz support
│   │   └── main.tsx
│   ├── tailwind.config.js
│   ├── vite.config.ts
│   └── package.json
│
├── python/                     # pip/uv installable package
│   ├── prismnote/
│   │   ├── _cli.py             # Smart binary downloader (platform-aware)
│   │   ├── __main__.py         # python -m prismnote entry
│   │   └── __init__.py
│   └── pyproject.toml
│
├── install.sh                  # curl installer (detects OS/arch)
├── build.sh                    # Build script (frontend + backend)
├── Cargo.toml                  # Workspace root
├── README.md                   # Project overview
└── GETTING_STARTED.md          # This file
```

## What's Implemented

### Backend (Rust)
- [x] HTTP server (Axum) on `localhost:8000`
- [x] REST API for notebook CRUD
  - `GET /api/notebooks` — List all notebooks
  - `POST /api/notebooks` — Create new notebook
  - `GET /api/notebooks/:id` — Load notebook
  - `DELETE /api/notebooks/:id` — Delete notebook
  - `POST /api/notebooks/:id/execute` — Execute cell (placeholder)
- [x] WebSocket support (`/ws/notebook/:id`)
- [x] .ipynb file format (parse + serialize)
- [x] Static file serving (frontend dist/)

### Frontend (React)
- [x] Notebook editor with cell management
- [x] Code cells with Monaco editor (syntax highlighting, snippets)
- [x] Markdown cells with preview/edit toggle
- [x] Output rendering: text, images (PNG, JPEG), HTML, SVG, tables
- [x] **High-resolution visualization support** (crisp-edges, image-rendering optimized for Retina/4K)
- [x] Sidebar with notebook list + file tree
- [x] Toolbar (save, export, import, theme toggle)
- [x] Keyboard shortcuts (Shift+Enter, Ctrl+Enter, B/A/DD for cell management)
- [x] Dark/light theme support

### Installation
- [x] Python package structure (pip/uv compatible)
- [x] Smart CLI downloader (`_cli.py`) — detects platform, downloads pre-built binary
- [x] curl installer script with OS/arch detection

## Next Steps to Get It Working

### 1. Install Jupyter Kernel
The backend needs a Jupyter kernel to execute Python. Install `ipykernel`:

```bash
pip install ipykernel
# or
uv pip install ipykernel
```

### 2. Build the Project

Option A: Full build (frontend + backend)
```bash
bash build.sh
```

Option B: Manual build
```bash
# Frontend
cd frontend && npm install && npm run build && cd ..

# Backend
cargo build --release
```

### 3. Run the Server

```bash
./target/release/prismnote
# or
cargo run --release
```

Server will:
- Start on `http://localhost:8000`
- Serve frontend from `./frontend/dist/`
- Listen for notebook API requests on `/api/`
- Open browser automatically (if run from CLI wrapper)

### 4. Test It

1. Open http://localhost:8000 in browser
2. Create a notebook via "New Notebook" button
3. Try:
   - Writing Python code: `print("Hello, PrismNote!")`
   - Run it: `Shift+Enter`
   - Add markdown cell: `B` then switch to markdown
   - Export: Download button in toolbar

## Key Design Decisions

### High-Resolution Visualizations
The frontend includes special CSS (`viz-container`) for crisp rendering:
- `image-rendering: crisp-edges` — pixel-perfect rendering
- Supports Retina displays natively
- Works with matplotlib/plotly/custom SVG

Outputs rendered:
- `text/plain` — preformatted text
- `image/png`, `image/jpeg` — base64 embedded images
- `text/html` — raw HTML (tables, SVG, interactive plots)
- `application/json` — pretty-printed JSON

### State Management
Uses Zustand (simple, no boilerplate). Notebook state lives in the browser, synced via:
- REST API for CRUD (notebooks)
- WebSocket ready for future real-time collab

### Jupyter Compatibility
- Full `.ipynb` v4 format support (in `files.rs`)
- Can import/export any `.ipynb` file
- Future: ZMQ connection to real `ipykernel` process

## Development Workflow

### Backend Development
```bash
cargo watch -x 'run --release'
# Edit crates/server/src/* and changes auto-reload
```

### Frontend Development
```bash
cd frontend
npm run dev  # Vite hot-reload on port 5173
# Edit src/* and changes auto-reload
# Backend API still at localhost:8000
```

## What's Still Todo (v0.2+)

1. **Jupyter Kernel Integration**
   - [ ] ZMQ connection from Rust to ipykernel
   - [ ] Proper message protocol (execute_request/reply)
   - [ ] Handle rich output (DataFrame display)

2. **AI Integration**
   - [ ] Claude API proxy in Rust (explain cell, fix error, complete code)
   - [ ] API key configuration

3. **Persistence**
   - [ ] Save notebooks to disk (currently in memory only)
   - [ ] Load notebooks on startup

4. **UI Polish**
   - [ ] Better error messages
   - [ ] Loading spinners for execution
   - [ ] Keyboard shortcut hints

5. **Collaboration** (v0.3+)
   - [ ] Real-time WebSocket sync between browser and kernel
   - [ ] Multi-user editing (CRDT-based)

## Troubleshooting

**Frontend not loading?**
- Make sure `cargo run` is serving `./frontend/dist/`
- Check browser console (F12) for API errors
- Confirm backend is on http://localhost:8000

**API errors?**
- Backend logs go to console (env `RUST_LOG=debug`)
- Check notebooks are being created in `~/.prismnote/notebooks/`

**Can't execute cells?**
- Jupyter kernel manager is scaffolded but not yet integrated
- Next step: implement ZMQ connection in `kernel.rs`

## Where to Go Next

1. **Integrate Jupyter kernel** (`kernel.rs` + ZMQ protocol)
   - Fork the `jupyter-client` Rust crate or use `zeromq`
   - Implement the Jupyter message protocol
   - Wire WebSocket → Kernel → Output

2. **Add Claude API** (`api.rs` new endpoint)
   - `/api/ai/explain`, `/api/ai/fix`, `/api/ai/complete`
   - Proxy requests to Claude API

3. **Notebooks persistence**
   - Implement `UPDATE /api/notebooks/:id` endpoint
   - Save notebook JSON to `~/.prismnote/notebooks/{id}.ipynb`

4. **Polish the UI**
   - Add loading states
   - Better error UI
   - Keyboard help modal

5. **Binary releases**
   - Build on GitHub Actions (macOS, Linux, Windows)
   - Upload to Releases
   - Make `pip install prismnote` work end-to-end

Good luck, and thanks for building PrismNote! 🎉
