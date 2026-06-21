# PrismNote vs. Apache Zeppelin — Gap Analysis

A grounded comparison of PrismNote (this repo) against Apache Zeppelin, focused
on concrete, actionable gaps rather than marketing bullets. Items are ordered by
impact on day-to-day notebook use.

## Where PrismNote already matches or leads
- **Modern UX**: VSCode-style shell (activity rail, command palette ⇧⌘P, unified
  search ⌘K), Monaco editor, dark/light themes. Zeppelin's Angular UI feels dated.
- **In-cell AI**: ⌘K inline edit with diff accept/reject, "Fix with AI" on errors,
  "Explain". Zeppelin has no first-class AI assistance.
- **Shared persistent kernel**: cells share one Python namespace (just fixed —
  see `crates/server/src/kernel.rs`). Pretty-printing (`rich`) preloaded.
- **.ipynb native format**: portable with Jupyter. Zeppelin uses its own `note.json`.
- **Cloud warehouse breadth**: connectors for 8 warehouses already scaffolded.

## High-impact gaps (should fix next)

### 1. Rich output capture is missing — **Plots tab will stay empty**
The kernel captures only `stdout`/`stderr`/`repr(last expr)`. Matplotlib figures,
pandas HTML tables, and `image/png` MIME bundles are **never produced**, so the
"Plots" panel has nothing to show. Zeppelin auto-renders tables and charts.
- **Fix**: in the kernel driver, after exec, detect matplotlib figures
  (`matplotlib.pyplot.get_fignums()`), render to PNG via `savefig` to a buffer,
  base64-encode, and emit an `output_type: "display_data"` with `image/png`.
  Also surface `_repr_html_()` for DataFrames as `text/html`.

### 2. No automatic visualization of tabular/SQL results
Zeppelin's signature feature: run SQL → pick bar/line/pie/scatter inline, with
pivot. PrismNote renders SQL results as text only.
- **Fix**: when a cell yields a table (SQL result or DataFrame), render a grid +
  a chart-type switcher in `Output.tsx` (e.g. via a lightweight chart lib).

### 3. No dynamic forms / parameterization
Zeppelin's `z.input()`, `z.select()`, `z.checkbox()` turn a note into a mini-app.
PrismNote has no equivalent, so notebooks can't be parameterized for non-authors.
- **Fix**: a `prism.input(...)` helper that emits a form spec output; render
  widgets that re-run dependent cells on change.

### 4. Output is one-shot, not streamed
`execute_cell` returns the full result only when the process finishes. Long jobs
show nothing until done, and there's no progress. Zeppelin streams paragraph
output over websocket.
- **Fix**: stream driver stdout line-by-line over the existing `/ws` channel and
  append to the cell incrementally.

### 5. No scheduling / cron for notebooks
Zeppelin can run a note on a cron schedule. `scheduler.rs` exists but isn't wired
to notebook runs end-to-end.

## Medium-impact gaps

### 6. Single interpreter; no per-cell language binding
Zeppelin: `%spark`, `%sql`, `%python`, `%md`, `%sh` per paragraph, with shared
context across interpreters. PrismNote special-cases `--sql`/`%sql` only.
- **Fix**: a `%`-magics router so a cell header chooses the interpreter.

### 7. Timeout recovery wipes all state
A cell that exceeds the 60s timeout triggers a full kernel restart (correct for
safety) but silently drops every variable. Users won't expect this.
- **Fix**: surface a clear "kernel restarted — state cleared" banner, and offer
  a per-cell timeout override / interrupt (SIGINT) before hard restart.

### 8. No interrupt / "stop" for a running cell
There's no way to interrupt a runaway cell short of the timeout. Zeppelin has a
cancel button per paragraph.

### 9. Collaboration is a stub
`realtime_collab.rs` exists but there's no live cursor / co-editing like
Zeppelin's websocket-based multi-user notes.

## Smaller correctness issues found in this pass
- `execution_count` was hardcoded to `1` for every run — **fixed** to use the
  kernel's real counter (`api.rs` → `k.execution_count()`).
- `set_ai_config` (`api.rs`) checks an env var instead of persisting the posted
  config, so saving AI settings from the UI is effectively a no-op.
- `list_databases` returns `[]` with a `TODO` — DB connections aren't persisted.
- Frontend `lib/` was caught by the Python `lib/` `.gitignore` rule; frontend
  helpers now live in `src/api/`. Consider scoping that ignore to Python paths.

## Suggested priority order
1. Rich output capture (matplotlib + DataFrame HTML) — unblocks the Plots tab.
2. Tabular result viz with chart switcher.
3. Streamed output + per-cell interrupt.
4. Dynamic forms.
5. Per-cell interpreter magics.
