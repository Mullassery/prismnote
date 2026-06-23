import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import {
  X, Table2, ListTree, Sigma, Info, Search, ArrowUp, ArrowDown,
  NotebookPen, Download, BarChart3, Loader2, Hash, Type, Calendar, ToggleLeft, Filter as FilterIcon, AlertTriangle,
  Brackets, Braces, Workflow, FileText, Database, Variable as VariableIcon, ArrowRight,
  Minus, Plus, ChevronDown,
} from 'lucide-react'
import { useFontSize } from '../hooks/useFontSize'
import {
  exploreOverview, exploreSchema, explorePage, exploreProfile, exploreExportCode, exploreDescribe, exploreLineage,
  type Source, type SchemaResult, type Overview, type ColumnProfile, type ColumnSchema,
  type Sort, type Filter, type LogicalType, type ColumnStat, type Lineage,
} from '../api/explore'
import { useNotebookStore } from '../hooks/useNotebook'
import { listVariables } from '../api/kernel'
import { useAIContext } from '../hooks/useAIContext'

export type ExplorerTarget = { var: string } | { source: Source }

const PAGE = 200

const typeIcon = (lg: LogicalType) =>
  lg === 'number' ? Hash
    : lg === 'datetime' ? Calendar
    : lg === 'bool' ? ToggleLeft
    : lg === 'array' ? Brackets
    : lg === 'struct' ? Braces
    : Type
const typeColor = (lg: LogicalType) =>
  lg === 'number' ? 'text-sky-400'
    : lg === 'datetime' ? 'text-amber-400'
    : lg === 'bool' ? 'text-fuchsia-400'
    : lg === 'array' ? 'text-indigo-400'
    : lg === 'struct' ? 'text-violet-400'
    : 'text-emerald-400'

function fmtBytes(n: number) {
  if (n < 1024) return `${n} B`
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(1)} KB`
  if (n < 1024 ** 3) return `${(n / 1024 ** 2).toFixed(1)} MB`
  return `${(n / 1024 ** 3).toFixed(2)} GB`
}
function fmtNum(n: number | undefined) {
  if (n == null || !Number.isFinite(n)) return '—'
  return Math.abs(n) >= 1000 || Number.isInteger(n) ? n.toLocaleString() : n.toPrecision(4)
}

/** Parse a per-column filter box into Filter[]: numeric supports `>5`, `<=10`,
 *  `5..10`, `=5`; everything else is a case-insensitive contains. */
function parseFilter(col: string, lg: LogicalType, raw: string): Filter[] {
  const t = raw.trim()
  if (!t) return []
  if (lg === 'number') {
    const range = t.match(/^(-?\d+\.?\d*)\s*\.\.\s*(-?\d+\.?\d*)$/)
    if (range) return [{ col, op: '>=', value: +range[1] }, { col, op: '<=', value: +range[2] }]
    const cmp = t.match(/^(>=|<=|>|<|=|!=)\s*(-?\d+\.?\d*)$/)
    if (cmp) return [{ col, op: (cmp[1] === '=' ? '==' : cmp[1]) as any, value: +cmp[2] }]
    if (/^-?\d+\.?\d*$/.test(t)) return [{ col, op: '==', value: +t }]
  }
  if (t === 'null') return [{ col, op: 'isnull' }]
  if (t === '!null') return [{ col, op: 'notnull' }]
  return [{ col, op: 'contains', value: t }]
}

// Tiny inline sparkline for a column header (histogram for numbers, top bars else).
function MiniDist({ profile }: { profile?: ColumnProfile }) {
  if (!profile) return <div className="h-5" />
  let bars: number[] = []
  if (profile.kind === 'number') bars = profile.hist.counts
  else if (profile.kind === 'category') bars = profile.top.map((t) => t.count)
  if (!bars.length) return <div className="h-5" />
  const max = Math.max(1, ...bars)
  return (
    <div className="flex items-end gap-px h-5" title="distribution">
      {bars.slice(0, 20).map((v, i) => (
        <div key={i} className="w-1 bg-blue-400/70 rounded-sm" style={{ height: `${Math.max(2, (v / max) * 18)}px` }} />
      ))}
    </div>
  )
}

// Chooser shown when the Data Explorer is opened without a specific target:
// pick a live DataFrame, or point at an open table/file format via DuckDB.
export function ExplorerPicker({
  onPick,
  onClose,
}: {
  onPick: (target: ExplorerTarget, title: string) => void
  onClose: () => void
}) {
  const [vars, setVars] = useState<{ name: string; type: string; preview?: string }[]>([])
  const [path, setPath] = useState('')
  const [sql, setSql] = useState('')

  useEffect(() => {
    listVariables().then((vs) => setVars(vs.filter((v: any) => /DataFrame|ndarray|Series/.test(v.type)))).catch(() => {})
  }, [])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose() }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div className="absolute inset-0 z-40 pn-app flex flex-col">
      <div className="h-10 flex items-center gap-2 px-4 border-b pn-bd">
        <Table2 size={16} className="prism-text" />
        <span className="text-sm font-semibold pn-text">Data Explorer</span>
        <span className="text-[12px] pn-faint">— choose a dataset</span>
        <div className="flex-1" />
        <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
      </div>
      <div className="flex-1 overflow-auto p-6 max-w-3xl mx-auto w-full space-y-7">
        <section>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">DataFrames in the kernel</div>
          {vars.length ? (
            <div className="grid gap-2" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))' }}>
              {vars.map((v) => (
                <button key={v.name} onClick={() => onPick({ var: v.name }, v.name)}
                  className="text-left p-3 rounded-lg border pn-bd bg-white/[0.02] hover:bg-white/5">
                  <div className="font-mono text-[13px] pn-text truncate">{v.name}</div>
                  <div className="text-[11px] pn-faint truncate mt-0.5">{v.preview || v.type}</div>
                </button>
              ))}
            </div>
          ) : (
            <div className="text-[12px] pn-faint">No DataFrames yet. Run a cell that creates one, or open a file below.</div>
          )}
        </section>
        <section>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">Open a file (DuckDB)</div>
          <div className="flex gap-2">
            <input autoFocus value={path} onChange={(e) => setPath(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && path.trim() && onPick({ source: { kind: 'file', path: path.trim() } }, path.trim().split('/').pop() || path)}
              placeholder="path/to/data.parquet · .csv · .json · Iceberg dir"
              className="flex-1 px-3 py-2 rounded bg-white/5 border pn-bd pn-text text-[13px] font-mono outline-none focus:border-blue-500" />
            <button onClick={() => path.trim() && onPick({ source: { kind: 'file', path: path.trim() } }, path.trim().split('/').pop() || path)}
              className="px-4 py-2 rounded prism-bg text-white text-[13px]">Open</button>
          </div>
          <div className="text-[11px] pn-faint mt-1">Parquet, CSV, JSON, Arrow, and Apache Iceberg are read natively via DuckDB.</div>
        </section>
        <section>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">DuckDB query</div>
          <textarea value={sql} onChange={(e) => setSql(e.target.value)} rows={3} spellCheck={false}
            placeholder="SELECT * FROM read_parquet('s3://bucket/*.parquet')"
            className="w-full px-3 py-2 rounded bg-white/5 border pn-bd pn-text text-[13px] font-mono outline-none focus:border-blue-500" />
          <button onClick={() => sql.trim() && onPick({ source: { kind: 'sql', query: sql.trim() } }, 'query')}
            className="mt-2 px-4 py-2 rounded prism-bg text-white text-[13px]">Run &amp; explore</button>
        </section>
      </div>
    </div>
  )
}

type Tab = 'preview' | 'schema' | 'stats' | 'metadata' | 'lineage'

export default function DataExplorer({
  target,
  title,
  onClose,
  onVisualize,
}: {
  target: ExplorerTarget
  title: string
  onClose: () => void
  onVisualize?: (t: ExplorerTarget, title: string) => void
}) {
  const [tab, setTab] = useState<Tab>('preview')
  const { size: fontSize, inc: fontInc, dec: fontDec } = useFontSize('pn-explorer-size', 13)
  const [collapsed, setCollapsed] = useState(false)
  const [schema, setSchema] = useState<SchemaResult | null>(null)
  const [overview, setOverview] = useState<Overview | null>(null)
  const [profiles, setProfiles] = useState<Record<string, ColumnProfile>>({})
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const [sort, setSort] = useState<Sort[]>([])
  const [colFilters, setColFilters] = useState<Record<string, string>>({})
  const [search, setSearch] = useState('')
  const [showFilters, setShowFilters] = useState(false)
  const [selectedCol, setSelectedCol] = useState<string | null>(null)

  // paged row cache
  const [rows, setRows] = useState<(any[] | undefined)[]>([])
  const [total, setTotal] = useState(0)
  const loadingPages = useRef<Set<number>>(new Set())
  const reqId = useRef(0)

  const filters: Filter[] = useMemo(
    () =>
      schema
        ? Object.entries(colFilters).flatMap(([col, raw]) => {
            const c = schema.columns.find((c) => c.name === col)
            return c ? parseFilter(col, c.logical, raw) : []
          })
        : [],
    [colFilters, schema],
  )

  // initial schema + overview + column profiles
  useEffect(() => {
    let alive = true
    setLoading(true)
    setError(null)
    Promise.all([exploreSchema(target), exploreOverview(target)])
      .then(([sc, ov]) => {
        if (!alive) return
        setSchema(sc)
        setOverview(ov)
        // publish to the AI panel so the agent understands the open dataset
        useAIContext.getState().setDataset({ title, columns: sc.columns.map((c) => c.name), shape: ov ? [ov.rows, ov.cols] : sc.shape })
        // lazy-load profiles (cap to keep it snappy on very wide frames)
        sc.columns.slice(0, 60).forEach((c) =>
          exploreProfile(target, c.name)
            .then((p) => alive && setProfiles((m) => ({ ...m, [c.name]: p })))
            .catch(() => {}),
        )
      })
      .catch((e: any) => alive && setError(e?.response?.data?.error || e?.message || 'failed to load'))
      .finally(() => alive && setLoading(false))
    return () => {
      alive = false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [JSON.stringify(target)])

  // (re)load first page whenever the query (sort/filter/search) changes
  const reload = useCallback(() => {
    const id = ++reqId.current
    loadingPages.current = new Set([0])
    explorePage(target, { offset: 0, limit: PAGE, sort, filters, search })
      .then((r) => {
        if (id !== reqId.current) return
        const arr: (any[] | undefined)[] = new Array(r.total)
        r.data.forEach((row, i) => (arr[i] = row))
        setRows(arr)
        setTotal(r.total)
        loadingPages.current = new Set()
      })
      .catch((e: any) => id === reqId.current && setError(e?.response?.data?.error || e?.message || 'query failed'))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [JSON.stringify(target), JSON.stringify(sort), JSON.stringify(filters), search])

  useEffect(() => {
    if (schema) reload()
  }, [schema, reload])

  const ensurePage = useCallback(
    (page: number) => {
      if (loadingPages.current.has(page)) return
      const offset = page * PAGE
      if (rows[offset] !== undefined) return
      loadingPages.current.add(page)
      const id = reqId.current
      explorePage(target, { offset, limit: PAGE, sort, filters, search })
        .then((r) => {
          if (id !== reqId.current) return
          setRows((prev) => {
            const next = prev.slice()
            r.data.forEach((row, i) => (next[offset + i] = row))
            return next
          })
        })
        .finally(() => loadingPages.current.delete(page))
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [rows, sort, filters, search, JSON.stringify(target)],
  )

  const cols = schema?.columns ?? []

  const toggleSort = (col: string, additive: boolean) => {
    setSort((prev) => {
      const existing = prev.find((s) => s.col === col)
      let nextForCol: Sort | null
      if (!existing) nextForCol = { col, dir: 'asc' }
      else if (existing.dir === 'asc') nextForCol = { col, dir: 'desc' }
      else nextForCol = null // cycle off
      const without = prev.filter((s) => s.col !== col)
      if (!additive) return nextForCol ? [nextForCol] : []
      return nextForCol ? [...without, nextForCol] : without
    })
  }

  // ── virtualized grid ──
  const rowH = fontSize + 15 // row height scales with the zoom level
  const scrollRef = useRef<HTMLDivElement>(null)
  const rowVirt = useVirtualizer({
    count: total,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowH,
    overscan: 12,
  })
  const items = rowVirt.getVirtualItems()
  useEffect(() => {
    const pages = new Set(items.map((it) => Math.floor(it.index / PAGE)))
    pages.forEach((p) => ensurePage(p))
  }, [items, ensurePage])
  // re-measure rows when the zoom level changes
  useEffect(() => { rowVirt.measure() }, [rowH]) // eslint-disable-line react-hooks/exhaustive-deps

  // clear the AI dataset context when the Explorer closes
  useEffect(() => () => useAIContext.getState().setDataset(null), [])

  // Esc closes the Explorer (unless the user is mid-typing in a field).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      const t = e.target as HTMLElement
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) { (t as HTMLInputElement).blur(); return }
      onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  // ── actions ──
  const insertAsCell = async () => {
    try {
      const code = await exploreExportCode(target, { sort, filters })
      const store = useNotebookStore.getState() as any
      if (!store.currentNotebook) await store.createNotebook('Explore')
      store.addCell('code')
      const s2 = useNotebookStore.getState() as any
      const idx = s2.currentNotebook.cells.length - 1
      s2.updateCell(idx, { source: code.split(/(?<=\n)/) })
      onClose()
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'could not generate code')
    }
  }

  const downloadCsv = async () => {
    const cap = Math.min(total, 50000)
    const parts: any[][] = []
    for (let off = 0; off < cap; off += 500) {
      const r = await explorePage(target, { offset: off, limit: 500, sort, filters, search })
      parts.push(...r.data)
      if (!r.data.length) break
    }
    const esc = (v: any) => {
      const s = v == null ? '' : String(v)
      return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s
    }
    const csv = [cols.map((c) => esc(c.name)).join(','), ...parts.map((row) => row.map(esc).join(','))].join('\n')
    const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }))
    const a = document.createElement('a')
    a.href = url
    a.download = `${title.replace(/\W+/g, '_')}.csv`
    a.click()
    URL.revokeObjectURL(url)
  }

  const activeFilterCount = filters.length

  return (
    <div className={`absolute z-40 pn-app flex flex-col ${collapsed ? 'inset-x-0 top-0' : 'inset-0'}`}>
      {/* header */}
      <div className="h-10 flex items-center gap-3 px-4 border-b pn-bd">
        <button onClick={() => setCollapsed((c) => !c)} title={collapsed ? 'Expand' : 'Collapse'}
          className="p-1 rounded pn-hover pn-muted">
          <ChevronDown size={15} className={collapsed ? '-rotate-90 transition-transform' : 'transition-transform'} />
        </button>
        <span className="flex items-center gap-2 text-sm font-semibold pn-text">
          <Table2 size={16} className="prism-text" /> Data Explorer
        </span>
        <span className="text-[12px] pn-faint truncate max-w-[24rem]">· {title}</span>
        {overview && (
          <span className="text-[11px] pn-faint hidden md:inline">
            {overview.rows.toLocaleString()} rows × {overview.cols} cols · {fmtBytes(overview.mem_bytes)}
          </span>
        )}
        <div className="flex-1" />
        <button onClick={insertAsCell} title="Insert reproducible code into the notebook"
          className="flex items-center gap-1 px-2.5 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px]">
          <NotebookPen size={13} /> Insert as cell
        </button>
        <button onClick={downloadCsv} title="Download current view as CSV"
          className="flex items-center gap-1 px-2.5 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px]">
          <Download size={13} /> CSV
        </button>
        {onVisualize && (
          <button onClick={() => onVisualize(target, title)} title="Open in Visualization Pane"
            className="flex items-center gap-1 px-2.5 py-1 rounded prism-bg text-white text-[12px]">
            <BarChart3 size={13} /> Visualize
          </button>
        )}
        {/* zoom (grid font size) */}
        <div className="flex items-center gap-0.5 ml-1">
          <button onClick={fontDec} title="Zoom out" className="p-1 rounded pn-hover pn-muted"><Minus size={13} /></button>
          <span className="text-[10px] tabular-nums w-5 text-center pn-faint" title="Zoom">{fontSize}</span>
          <button onClick={fontInc} title="Zoom in" className="p-1 rounded pn-hover pn-muted"><Plus size={13} /></button>
        </div>
        <button onClick={onClose} title="Close" className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
      </div>

      {collapsed && (
        <div className="px-4 py-1.5 text-[11px] pn-faint border-b pn-bd">
          Collapsed · {title}{overview ? ` — ${overview.rows.toLocaleString()} × ${overview.cols}` : ''} (click ⌄ to expand)
        </div>
      )}

      {!collapsed && <>
      {/* tabs */}
      <div className="h-9 flex items-center gap-1 px-3 border-b pn-bd">
        {([
          ['preview', Table2, 'Preview'],
          ['schema', ListTree, 'Schema'],
          ['stats', Sigma, 'Statistics'],
          ['metadata', Info, 'Metadata'],
          ['lineage', Workflow, 'Lineage'],
        ] as const).map(([id, Icon, label]) => (
          <button key={id} onClick={() => setTab(id)}
            className={`flex items-center gap-1.5 px-3 py-1 rounded text-[12px] ${
              tab === id ? 'bg-blue-500/20 text-blue-200' : 'pn-muted hover:pn-text'
            }`}>
            <Icon size={13} /> {label}
          </button>
        ))}
        {tab === 'preview' && (
          <>
            <div className="mx-2 flex items-center gap-1.5 px-2 py-1 rounded bg-white/5 border pn-bd">
              <Search size={12} className="pn-faint" />
              <input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search all columns…"
                className="bg-transparent outline-none text-[12px] pn-text w-44" />
            </div>
            <button onClick={() => setShowFilters((s) => !s)}
              className={`flex items-center gap-1 px-2 py-1 rounded text-[12px] ${showFilters || activeFilterCount ? 'bg-blue-500/20 text-blue-200' : 'pn-muted hover:pn-text'}`}>
              <FilterIcon size={12} /> Filters{activeFilterCount ? ` (${activeFilterCount})` : ''}
            </button>
          </>
        )}
        <div className="flex-1" />
        <span className="text-[11px] pn-faint">
          {tab === 'preview' ? `${total.toLocaleString()}${activeFilterCount || search ? ' filtered' : ''} rows` : ''}
        </span>
      </div>

      {error && (
        <div className="m-3 rounded border border-rose-700/60 bg-rose-900/20 p-3 text-[13px] text-rose-200 flex gap-2">
          <AlertTriangle size={15} className="shrink-0 mt-0.5" />
          <span className="font-mono text-[12px]">{error}</span>
        </div>
      )}

      {loading ? (
        <div className="flex-1 flex items-center justify-center pn-faint text-sm">
          <Loader2 size={18} className="animate-spin mr-2" /> Loading…
        </div>
      ) : (
        <div className="flex-1 flex overflow-hidden" style={{ fontSize }}>
          <div className="flex-1 overflow-hidden flex flex-col">
            {tab === 'preview' && schema && (
              <PreviewGrid
                cols={cols}
                rows={rows}
                total={total}
                sort={sort}
                profiles={profiles}
                scrollRef={scrollRef}
                rowVirt={rowVirt}
                items={items}
                rowH={rowH}
                font={fontSize}
                showFilters={showFilters}
                colFilters={colFilters}
                setColFilters={setColFilters}
                onSort={toggleSort}
                onSelectCol={setSelectedCol}
                selectedCol={selectedCol}
              />
            )}
            {tab === 'schema' && schema && <SchemaTab schema={schema} profiles={profiles} />}
            {tab === 'stats' && schema && <StatsTab target={target} cols={cols} profiles={profiles} />}
            {tab === 'metadata' && overview && <MetadataTab overview={overview} title={title} />}
            {tab === 'lineage' && <LineageTab target={target} title={title} />}
          </div>

          {/* column detail drawer */}
          {selectedCol && tab === 'preview' && (
            <ColumnDrawer
              col={cols.find((c) => c.name === selectedCol)!}
              profile={profiles[selectedCol]}
              onClose={() => setSelectedCol(null)}
            />
          )}
        </div>
      )}
      </>}
    </div>
  )
}

// ── Preview grid ──────────────────────────────────────────────────────────────
function PreviewGrid({
  cols, rows, sort, profiles, scrollRef, rowVirt, items, font,
  showFilters, colFilters, setColFilters, onSort, onSelectCol, selectedCol,
}: any) {
  const colW = 168
  const idxW = 56
  return (
    <div ref={scrollRef} className="flex-1 overflow-auto">
      <div style={{ width: idxW + cols.length * colW }}>
        {/* header */}
        <div className="sticky top-0 z-10 flex pn-surface border-b pn-bd">
          <div style={{ width: idxW }} className="shrink-0 px-2 py-1 text-[10px] pn-faint border-r pn-bd">#</div>
          {cols.map((c: ColumnSchema) => {
            const Icon = typeIcon(c.logical)
            const s = sort.find((s: Sort) => s.col === c.name)
            return (
              <div key={c.name} style={{ width: colW }}
                className={`shrink-0 px-2 py-1 border-r pn-bd cursor-pointer hover:bg-white/5 ${selectedCol === c.name ? 'bg-blue-500/10' : ''}`}
                onClick={(e) => onSort(c.name, e.shiftKey)}
                onContextMenu={(e) => { e.preventDefault(); onSelectCol(c.name) }}
                title="Click: sort · Shift-click: multi-sort · Right-click: details">
                <div className="flex items-center gap-1">
                  <Icon size={11} className={typeColor(c.logical)} />
                  <span className="pn-text truncate flex-1" style={{ fontSize: font }}>{c.name}</span>
                  {s && (s.dir === 'asc' ? <ArrowUp size={11} className="text-blue-300" /> : <ArrowDown size={11} className="text-blue-300" />)}
                  <button onClick={(e) => { e.stopPropagation(); onSelectCol(c.name) }} className="pn-faint hover:pn-text"><Info size={11} /></button>
                </div>
                <div className="flex items-center justify-between mt-0.5">
                  <MiniDist profile={profiles[c.name]} />
                  {c.null_pct > 0 && <span className="text-[9px] text-rose-400/80" title="null %">{c.null_pct}%∅</span>}
                </div>
              </div>
            )
          })}
        </div>
        {/* per-column filter row */}
        {showFilters && (
          <div className="flex pn-surface border-b pn-bd sticky top-[42px] z-10">
            <div style={{ width: idxW }} className="shrink-0 border-r pn-bd" />
            {cols.map((c: ColumnSchema) => (
              <div key={c.name} style={{ width: colW }} className="shrink-0 px-1 py-1 border-r pn-bd">
                <input
                  value={colFilters[c.name] ?? ''}
                  onChange={(e) => setColFilters((m: any) => ({ ...m, [c.name]: e.target.value }))}
                  placeholder={c.logical === 'number' ? '>0, 1..9' : 'contains…'}
                  className="w-full px-1.5 py-0.5 rounded bg-white/5 border pn-bd text-[11px] pn-text outline-none focus:border-blue-500" />
              </div>
            ))}
          </div>
        )}
        {/* body */}
        <div style={{ height: rowVirt.getTotalSize(), position: 'relative' }}>
          {items.map((it: any) => {
            const row = rows[it.index]
            return (
              <div key={it.key} className="flex absolute left-0 hover:bg-white/5"
                style={{ top: it.start, height: it.size, width: '100%' }}>
                <div style={{ width: idxW, fontSize: font - 1 }} className="shrink-0 px-2 pn-faint border-r pn-bd flex items-center tabular-nums">{it.index}</div>
                {cols.map((c: ColumnSchema, ci: number) => (
                  <div key={c.name} style={{ width: colW, fontSize: font }}
                    className="shrink-0 px-2 pn-text border-r pn-bd flex items-center truncate font-mono">
                    {row ? <CellValue v={row[ci]} lg={c.logical} /> : <span className="pn-faint">·</span>}
                  </div>
                ))}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

function CellValue({ v, lg }: { v: any; lg: LogicalType }) {
  if (v == null) return <span className="text-rose-400/60 italic">null</span>
  if (lg === 'number') return <span className="tabular-nums">{typeof v === 'number' ? fmtNum(v) : String(v)}</span>
  if (lg === 'array' || lg === 'struct' || (typeof v === 'object')) {
    let s: string
    try { s = JSON.stringify(v) } catch { s = String(v) }
    const color = lg === 'array' ? 'text-indigo-300' : 'text-violet-300'
    return <span className={`truncate ${color}`} title={s}>{s.length > 60 ? s.slice(0, 60) + '…' : s}</span>
  }
  return <span className="truncate">{String(v)}</span>
}

// ── Schema tab ────────────────────────────────────────────────────────────────
function SchemaTab({ schema, profiles }: { schema: SchemaResult; profiles: Record<string, ColumnProfile> }) {
  return (
    <div className="flex-1 overflow-auto p-4">
      <table className="w-full text-[13px]">
        <thead>
          <tr className="text-left pn-faint text-[11px] uppercase tracking-wide border-b pn-bd">
            <th className="py-2 pr-3">#</th><th className="pr-3">Column</th><th className="pr-3">Type</th>
            <th className="pr-3">Logical</th><th className="pr-3">Null %</th><th className="pr-3">Distribution</th>
          </tr>
        </thead>
        <tbody>
          {schema.columns.map((c, i) => {
            const Icon = typeIcon(c.logical)
            return (
              <tr key={c.name} className="border-b pn-bd/50 hover:bg-white/5">
                <td className="py-1.5 pr-3 pn-faint tabular-nums">{i + 1}</td>
                <td className="pr-3 font-mono pn-text">{c.name}</td>
                <td className="pr-3 pn-muted font-mono text-[12px]">{c.dtype}</td>
                <td className="pr-3"><span className={`inline-flex items-center gap-1 ${typeColor(c.logical)}`}><Icon size={12} />{c.logical}</span></td>
                <td className="pr-3"><span className={c.null_pct > 0 ? 'text-rose-400' : 'pn-faint'}>{c.null_pct}%</span></td>
                <td className="pr-3"><MiniDist profile={profiles[c.name]} /></td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}

// ── Statistics tab ────────────────────────────────────────────────────────────
function StatsTab({ target, cols, profiles }: { target: ExplorerTarget; cols: ColumnSchema[]; profiles: Record<string, ColumnProfile> }) {
  const [stats, setStats] = useState<ColumnStat[] | null>(null)
  const [view, setView] = useState<'table' | 'cards'>('table')
  const [statErr, setStatErr] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    setStats(null)
    setStatErr(null)
    exploreDescribe(target)
      .then((d) => alive && setStats(d.columns))
      .catch((e: any) => alive && setStatErr(e?.response?.data?.error || e?.message || 'failed'))
    return () => { alive = false }
  }, [JSON.stringify(target)])

  return (
    <div className="flex-1 overflow-auto">
      <div className="sticky top-0 z-10 flex items-center gap-1 px-4 py-2 pn-surface border-b pn-bd">
        <span className="text-[12px] font-semibold pn-text mr-2">Column statistics</span>
        {(['table', 'cards'] as const).map((v) => (
          <button key={v} onClick={() => setView(v)}
            className={`px-2 py-0.5 rounded text-[11px] ${view === v ? 'bg-blue-500/20 text-blue-200' : 'pn-muted hover:pn-text'}`}>
            {v === 'table' ? 'Table' : 'Distributions'}
          </button>
        ))}
      </div>

      {view === 'table' ? (
        statErr ? (
          <div className="p-4 text-rose-300 text-[13px] font-mono">{statErr}</div>
        ) : !stats ? (
          <div className="p-4 pn-faint text-sm flex items-center gap-2"><Loader2 size={15} className="animate-spin" /> Computing statistics…</div>
        ) : (
          <StatsTable stats={stats} />
        )
      ) : (
        <div className="p-4 grid gap-3" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))' }}>
          {cols.map((c) => (
            <div key={c.name} className="rounded-lg border pn-bd p-3 bg-white/[0.02]">
              <div className="flex items-center gap-1.5 mb-2">
                {(() => { const I = typeIcon(c.logical); return <I size={13} className={typeColor(c.logical)} /> })()}
                <span className="font-mono text-[13px] pn-text truncate">{c.name}</span>
                <span className="ml-auto text-[10px] pn-faint">{c.dtype}</span>
              </div>
              <ProfileBody profile={profiles[c.name]} />
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

// describe()-style table: one row per column, numeric + categorical stats together.
function StatsTable({ stats }: { stats: ColumnStat[] }) {
  const num = (v: number | null | undefined) => (v == null ? '' : fmtNum(v))
  const headers = ['Column', 'Type', 'Count', 'Missing', 'Distinct', 'Mean', 'Std', 'Min', '25%', '50%', '75%', 'Max', 'Sum', 'Skew', 'Kurt', 'Top', 'Freq']
  return (
    <div className="overflow-auto">
      <table className="text-[12px] w-full whitespace-nowrap">
        <thead>
          <tr className="sticky top-0 text-left pn-faint text-[10px] uppercase tracking-wide pn-surface border-b pn-bd">
            {headers.map((h) => <th key={h} className="px-2.5 py-1.5 font-medium">{h}</th>)}
          </tr>
        </thead>
        <tbody>
          {stats.map((s) => {
            const Icon = typeIcon(s.logical)
            return (
              <tr key={s.name} className="border-b pn-bd/40 hover:bg-white/5">
                <td className="px-2.5 py-1 font-mono pn-text">{s.name}</td>
                <td className="px-2.5 py-1"><span className={`inline-flex items-center gap-1 ${typeColor(s.logical)}`}><Icon size={11} />{s.logical}</span></td>
                <td className="px-2.5 py-1 tabular-nums pn-text">{s.count.toLocaleString()}</td>
                <td className="px-2.5 py-1 tabular-nums"><span className={s.nulls ? 'text-rose-400' : 'pn-faint'}>{s.nulls.toLocaleString()}{s.nulls ? ` (${s.null_pct}%)` : ''}</span></td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{s.distinct.toLocaleString()}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.mean)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.std)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.min)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.q1)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.median)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.q3)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.max)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.sum)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.skew)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{num(s.kurtosis)}</td>
                <td className="px-2.5 py-1 pn-muted max-w-[10rem] truncate" title={s.top == null ? '' : String(s.top)}>{s.top == null ? '' : String(s.top)}</td>
                <td className="px-2.5 py-1 tabular-nums pn-muted">{s.freq == null ? '' : s.freq.toLocaleString()}</td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}

function ProfileBody({ profile }: { profile?: ColumnProfile }) {
  if (!profile) return <div className="text-[12px] pn-faint">Loading…</div>
  if (profile.kind === 'number') {
    const max = Math.max(1, ...profile.hist.counts)
    return (
      <div className="space-y-2">
        <div className="flex items-end gap-px h-12">
          {profile.hist.counts.map((v, i) => (
            <div key={i} className="flex-1 bg-blue-400/70 rounded-sm" style={{ height: `${Math.max(2, (v / max) * 46)}px` }} />
          ))}
        </div>
        <div className="grid grid-cols-2 gap-x-3 gap-y-0.5 text-[11px]">
          <Stat k="min" v={fmtNum(profile.min)} /><Stat k="max" v={fmtNum(profile.max)} />
          <Stat k="mean" v={fmtNum(profile.mean)} /><Stat k="median" v={fmtNum(profile.median)} />
          <Stat k="std" v={fmtNum(profile.std)} /><Stat k="nulls" v={`${profile.null_pct}%`} />
        </div>
      </div>
    )
  }
  if (profile.kind === 'datetime') {
    return (
      <div className="grid gap-0.5 text-[11px]">
        <Stat k="min" v={String(profile.min ?? '—')} /><Stat k="max" v={String(profile.max ?? '—')} />
        <Stat k="unique" v={profile.cardinality.toLocaleString()} /><Stat k="nulls" v={`${profile.null_pct}%`} />
      </div>
    )
  }
  if (profile.kind === 'nested') {
    return (
      <div className="space-y-1.5">
        <div className="text-[11px] pn-faint">{profile.subtype} · {profile.count.toLocaleString()} non-null · {profile.null_pct}% null</div>
        <div className="grid grid-cols-3 gap-x-3 gap-y-0.5 text-[11px]">
          <Stat k="min len" v={profile.min_len == null ? '—' : String(profile.min_len)} />
          <Stat k="avg len" v={profile.avg_len == null ? '—' : String(profile.avg_len)} />
          <Stat k="max len" v={profile.max_len == null ? '—' : String(profile.max_len)} />
        </div>
        {profile.fields && profile.fields.length > 0 && (
          <div>
            <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">fields</div>
            <div className="flex flex-wrap gap-1">
              {profile.fields.map((f) => (
                <span key={f} className="px-1.5 py-0.5 rounded bg-violet-500/15 text-violet-200 text-[10px] font-mono">{f}</span>
              ))}
            </div>
          </div>
        )}
      </div>
    )
  }
  const max = Math.max(1, ...profile.top.map((t) => t.count))
  return (
    <div className="space-y-1.5">
      <div className="text-[11px] pn-faint">{profile.cardinality.toLocaleString()} unique · {profile.null_pct}% null</div>
      {profile.top.slice(0, 8).map((t) => (
        <div key={String(t.value)} className="flex items-center gap-2 text-[11px]">
          <span className="w-24 truncate pn-text font-mono">{String(t.value)}</span>
          <div className="flex-1 h-2 rounded bg-white/5 overflow-hidden">
            <div className="h-full bg-emerald-400/70" style={{ width: `${(t.count / max) * 100}%` }} />
          </div>
          <span className="tabular-nums pn-faint w-10 text-right">{t.count}</span>
        </div>
      ))}
    </div>
  )
}

function Stat({ k, v }: { k: string; v: string }) {
  return <div className="flex justify-between"><span className="pn-faint">{k}</span><span className="pn-text font-mono">{v}</span></div>
}

// ── Metadata tab ──────────────────────────────────────────────────────────────
function MetadataTab({ overview, title }: { overview: Overview; title: string }) {
  const cards: [string, string][] = [
    ['Rows', overview.rows.toLocaleString()],
    ['Columns', String(overview.cols)],
    ['In-memory size', fmtBytes(overview.mem_bytes)],
    ['Total cells', overview.total_cells.toLocaleString()],
    ['Missing cells', `${overview.total_nulls.toLocaleString()} (${overview.null_pct}%)`],
    ['Complete columns', `${overview.complete_columns} / ${overview.cols}`],
    ['Constant columns', String(overview.constant_columns)],
    ['Duplicate rows', overview.duplicate_rows == null ? '—' : overview.duplicate_rows.toLocaleString()],
    ['Index', overview.index_name ?? '(range)'],
  ]
  return (
    <div className="flex-1 overflow-auto p-5 space-y-5">
      <div>
        <div className="text-[11px] uppercase tracking-wide pn-faint mb-1">Source</div>
        <div className="font-mono text-[13px] pn-text">{title}</div>
      </div>
      <div className="grid gap-3" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))' }}>
        {cards.map(([k, v]) => (
          <div key={k} className="rounded-lg border pn-bd p-3 bg-white/[0.02]">
            <div className="text-[11px] pn-faint">{k}</div>
            <div className="text-lg font-semibold pn-text mt-0.5">{v}</div>
          </div>
        ))}
      </div>
      <div>
        <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">Column types</div>
        <div className="flex flex-wrap gap-2">
          {Object.entries(overview.type_breakdown).map(([t, n]) => (
            <span key={t} className="px-2.5 py-1 rounded-full bg-white/5 border pn-bd text-[12px] pn-text">{t}: {n}</span>
          ))}
        </div>
      </div>
      {overview.worst_columns.some((c) => c.null_pct > 0) && (
        <div>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">Most-incomplete columns</div>
          <div className="space-y-1.5 max-w-md">
            {overview.worst_columns.filter((c) => c.null_pct > 0).map((c) => (
              <div key={c.name} className="flex items-center gap-2 text-[12px]">
                <span className="w-40 truncate font-mono pn-text">{c.name}</span>
                <div className="flex-1 h-2 rounded bg-white/5 overflow-hidden">
                  <div className="h-full bg-rose-400/70" style={{ width: `${c.null_pct}%` }} />
                </div>
                <span className="w-12 text-right tabular-nums pn-faint">{c.null_pct}%</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

// ── Lineage tab ───────────────────────────────────────────────────────────────
// Provenance from the backend (variable / file / DuckDB query + upstream refs),
// enriched for variables with notebook-derived lineage: which cells define the
// frame, which upstream variables it was built from, and where it's used next.
function LineageTab({ target, title }: { target: ExplorerTarget; title: string }) {
  const [lin, setLin] = useState<Lineage | null>(null)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    setLin(null); setErr(null)
    exploreLineage(target)
      .then((l) => alive && setLin(l))
      .catch((e: any) => alive && setErr(e?.response?.data?.error || e?.message || 'failed'))
    return () => { alive = false }
  }, [JSON.stringify(target)])

  const varName = 'var' in target ? target.var : (lin?.kind === 'variable' ? lin.name : undefined)
  const nb = (useNotebookStore.getState() as any).currentNotebook
  const cells: { source: string[] }[] = nb?.cells ?? []
  const cellText = (i: number) => (Array.isArray(cells[i]?.source) ? cells[i].source.join('') : String(cells[i]?.source ?? ''))

  // Notebook lineage for a live variable: defining cells + upstream/downstream.
  const { definedIn, usedIn, upstream } = useMemo(() => {
    const definedIn: number[] = []
    const usedIn: number[] = []
    const upstream = new Set<string>()
    if (!varName) return { definedIn, usedIn, upstream: [] as string[] }
    const assign = new RegExp(`(^|\\n)\\s*${varName}\\s*(=|\\+=|:[^=]+=)(?!=)`)
    const ref = new RegExp(`\\b${varName}\\b`)
    const otherVars = new Set<string>()
    cells.forEach((_, i) => {
      const t = cellText(i)
      const m = t.match(/(^|\n)\s*([A-Za-z_]\w*)\s*=(?!=)/g)
      m?.forEach((s) => { const v = s.trim().split(/\s*=/)[0]; if (v && v !== varName) otherVars.add(v) })
    })
    cells.forEach((_, i) => {
      const t = cellText(i)
      if (assign.test(t)) {
        definedIn.push(i)
        // upstream = other known variables referenced in the defining cell's RHS
        otherVars.forEach((v) => { if (new RegExp(`\\b${v}\\b`).test(t)) upstream.add(v) })
      } else if (ref.test(t)) {
        usedIn.push(i)
      }
    })
    return { definedIn, usedIn, upstream: [...upstream] }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [varName, nb?.cells?.length])

  if (err) return <div className="flex-1 p-4 text-rose-300 text-[13px] font-mono">{err}</div>
  if (!lin) return <div className="flex-1 p-4 pn-faint text-sm flex items-center gap-2"><Loader2 size={15} className="animate-spin" /> Tracing lineage…</div>

  const KindIcon = lin.kind === 'file' ? FileText : lin.kind === 'sql' ? Database : VariableIcon
  const node = (icon: any, label: string, sub?: string, accent = 'pn-bd') => {
    const I = icon
    return (
      <div className={`inline-flex items-center gap-2 px-3 py-2 rounded-lg border ${accent} bg-white/[0.03]`}>
        <I size={14} className="prism-text shrink-0" />
        <div className="min-w-0">
          <div className="text-[12px] pn-text font-mono truncate max-w-[16rem]">{label}</div>
          {sub && <div className="text-[10px] pn-faint truncate max-w-[16rem]">{sub}</div>}
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto p-5 space-y-6">
      {/* provenance */}
      <section>
        <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">Provenance</div>
        <div className="flex items-center gap-2 flex-wrap">
          <span className="px-2 py-1 rounded-full bg-blue-500/15 text-blue-200 text-[11px] inline-flex items-center gap-1">
            <KindIcon size={12} /> {lin.kind}
          </span>
          {lin.format && <span className="text-[12px] pn-muted">{lin.format}</span>}
          {lin.engine && <span className="text-[12px] pn-muted">via {lin.engine}</span>}
          {lin.shape && <span className="text-[12px] pn-faint">→ {lin.shape[0].toLocaleString()} × {lin.shape[1]}</span>}
        </div>
      </section>

      {/* upstream → this → downstream chain */}
      <section>
        <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">Lineage graph</div>
        <div className="flex items-center gap-3 flex-wrap">
          {lin.kind === 'sql' && (lin.references ?? []).length > 0 && (
            <>
              <div className="flex flex-col gap-1.5">
                {lin.references!.map((r) => node(r.type.includes('table') ? Database : FileText, r.target, r.type))}
              </div>
              <ArrowRight size={16} className="pn-faint" />
            </>
          )}
          {lin.kind === 'file' && (
            <>
              {node(FileText, (lin.path ?? '').split('/').pop() || lin.path || 'file', lin.path)}
              <ArrowRight size={16} className="pn-faint" />
            </>
          )}
          {lin.kind === 'variable' && upstream.length > 0 && (
            <>
              <div className="flex flex-col gap-1.5">
                {upstream.map((u) => node(VariableIcon, u, 'upstream variable'))}
              </div>
              <ArrowRight size={16} className="pn-faint" />
            </>
          )}
          {node(KindIcon, title, lin.kind === 'variable' ? 'this dataset' : undefined, 'border-blue-400/60')}
          {lin.kind === 'variable' && usedIn.length > 0 && (
            <>
              <ArrowRight size={16} className="pn-faint" />
              {node(NotebookPen, `${usedIn.length} cell${usedIn.length > 1 ? 's' : ''}`, 'downstream usage')}
            </>
          )}
        </div>
      </section>

      {/* details */}
      {lin.kind === 'file' && (
        <section className="text-[12px] space-y-1">
          <Stat k="path" v={lin.path ?? '—'} />
          <Stat k="exists" v={lin.exists ? 'yes' : 'no'} />
          {lin.size_bytes != null && <Stat k="size on disk" v={fmtBytes(lin.size_bytes)} />}
          {lin.modified && <Stat k="last modified" v={lin.modified} />}
        </section>
      )}
      {lin.kind === 'sql' && (
        <section>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-1">Query</div>
          <pre className="text-[12px] font-mono pn-text bg-white/[0.03] border pn-bd rounded p-3 overflow-auto whitespace-pre-wrap">{lin.query}</pre>
        </section>
      )}
      {lin.kind === 'variable' && (
        <section>
          <div className="text-[11px] uppercase tracking-wide pn-faint mb-2">
            {definedIn.length ? `Defined in notebook cell${definedIn.length > 1 ? 's' : ''} ${definedIn.map((i) => `[${i + 1}]`).join(', ')}` : 'No defining cell found in the current notebook'}
          </div>
          {definedIn.map((i) => (
            <pre key={i} className="text-[12px] font-mono pn-text bg-white/[0.03] border pn-bd rounded p-3 mb-2 overflow-auto whitespace-pre-wrap">{cellText(i).trim()}</pre>
          ))}
        </section>
      )}
    </div>
  )
}

// ── Column detail drawer ──────────────────────────────────────────────────────
function ColumnDrawer({ col, profile, onClose }: { col: ColumnSchema; profile?: ColumnProfile; onClose: () => void }) {
  const Icon = typeIcon(col.logical)
  return (
    <div className="w-72 shrink-0 border-l pn-bd flex flex-col pn-surface">
      <div className="h-9 flex items-center gap-2 px-3 border-b pn-bd">
        <Icon size={13} className={typeColor(col.logical)} />
        <span className="font-mono text-[13px] pn-text truncate flex-1">{col.name}</span>
        <button onClick={onClose} className="pn-faint hover:pn-text"><X size={14} /></button>
      </div>
      <div className="p-3 overflow-auto">
        <div className="text-[11px] pn-faint mb-3">{col.dtype} · {col.logical} · {col.null_pct}% null</div>
        <ProfileBody profile={profile} />
      </div>
    </div>
  )
}
