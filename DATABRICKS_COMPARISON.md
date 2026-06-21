# PrismNote vs. Databricks Notebooks — Functionality Gap Analysis

How PrismNote (this repo, current state) compares to Databricks notebooks, for
data-science/engineering work. Grouped by what's at parity, what's partial, and
the real gaps — ordered by impact.

## At or near parity
- **Polyglot cells via magics**: `%python`, `%sql`, `%sh`, `%md` (Databricks:
  `%python/%sql/%scala/%r/%sh/%md`). Per-cell language switching works.
- **Shared session state across cells**: one persistent kernel namespace.
- **In-cell SQL → DataFrame**: `%sql` runs through DuckDB and can query DataFrames
  defined in other cells (Databricks queries Spark tables/temp views).
- **Rich output**: tables (DataFrame HTML), matplotlib images, markdown.
- **AI assistant**: in-cell ⌘K edit / Fix / Explain + an agent panel. Comparable
  in spirit to Databricks Assistant (PrismNote uses local Ollama or Claude/OpenAI).
- **Jobs / scheduling**: run a whole notebook as a job, manual or scheduled
  (interval/daily). Databricks Workflows is far richer but the core loop exists.
- **Notebook autosave + .ipynb**; per-cell run; interrupt/restart kernel.

## Partial
- **SQL**: DuckDB (single-node, in-process) vs Databricks' distributed Spark SQL
  over the lakehouse. Great for local/medium data; not a cluster.
- **Visualization**: we render DataFrames/plots, but lack Databricks' built-in
  chart builder (pick chart type from a result grid) — see PrismNote gap #1 below.
- **Jobs**: no multi-task DAGs, retries, alerts, cluster selection, or parameters
  (Databricks Workflows has all of these).
- **Collaboration**: comments/RBAC scaffolding exists; no real-time co-editing or
  co-presence like Databricks.

## Real gaps (Databricks has, PrismNote doesn't)

1. **Built-in visualization builder** — Databricks turns any result into bar/line/
   pie/map/etc. with a few clicks, plus the `display()` function. PrismNote shows
   tables/plots but has no point-and-click chart builder over a result set.
   *(Tracked: "Tabular result viz with chart switcher".)*

2. **`display()` / `dbutils`-style helpers** — Databricks has `display(df)`,
   `dbutils.fs`, `dbutils.widgets`, `dbutils.secrets`. PrismNote has no equivalent
   utility namespace. Widgets/parameters are the most useful subset
   *(Tracked: "Dynamic forms").*

3. **Spark / distributed compute & the lakehouse** — Unity Catalog, Delta Lake,
   cluster autoscaling, photon. PrismNote is single-node (DuckDB + local Python).
   This is the fundamental architectural difference.

4. **Streaming output & progress** — Databricks streams Spark job progress and
   long-running output live. PrismNote returns cell output once it finishes.
   *(Tracked: "Streamed cell output over WebSocket".)*

5. **Notebook parameters + `%run`/workflows composition** — Databricks notebooks
   take parameters and call each other (`%run`, `dbutils.notebook.run`). PrismNote
   jobs run a single notebook snapshot with no parameterization or composition.

6. **Data/catalog browser** — Databricks' Data tab browses catalogs/schemas/tables
   with previews. PrismNote browses the filesystem and DuckDB tables, not a catalog.

7. **Version history & reproducible revisions** — Databricks keeps automatic
   notebook revision history with restore. PrismNote relies on git/.ipynb.

8. **Governance** — Unity Catalog lineage, table ACLs, audit. Out of scope for a
   local tool, but worth noting for enterprise parity.

## Where PrismNote is actually nicer than Databricks
- Local-first, zero-cost, offline, no cluster spin-up.
- Modern VS Code-style UX (command palette, ⌘K search, Monaco, themes).
- Local AI via Ollama (private, free) with a teacher persona + inline completion.
- Native `.ipynb` portability.

## Suggested priorities to close the gap
1. Visualization builder over result grids (chart switcher).
2. Widgets/parameters (`prism.input/select/slider`) → also unlocks job parameters.
3. Streamed output + progress.
4. Notebook parameters for Jobs + simple multi-notebook composition.
5. A catalog/data browser over DuckDB + configured connections.
