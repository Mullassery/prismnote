import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { VegaEmbed } from 'react-vega'
import type { View } from 'vega'
import {
  Images, Wand2, ZoomIn, ZoomOut, Maximize2, Copy, Download, ExternalLink, Trash2,
  ChevronLeft, ChevronRight, Sun, Moon, NotebookPen, BarChart3, LineChart, AreaChart,
  ScatterChart, PieChart, Grid3x3, Loader2,
} from 'lucide-react'
import { usePlots } from '../hooks/usePlots'
import { useViz } from '../hooks/useViz'
import { exploreSchema, exploreAggregate, type ColumnSchema, type Measure } from '../api/explore'
import { listVariables } from '../api/kernel'
import { useNotebookStore } from '../hooks/useNotebook'
import type { ExplorerTarget } from './DataExplorer'

type Mode = 'gallery' | 'explore'

export default function VizPane() {
  const [mode, setMode] = useState<Mode>('gallery')
  const { nonce } = useViz()

  // a request from the Data Explorer's "Visualize" button switches to Explore
  useEffect(() => {
    if (nonce > 0) setMode('explore')
  }, [nonce])

  return (
    <div className="h-full flex flex-col">
      <div className="h-8 flex items-center gap-1 px-2 border-b pn-bd">
        {([
          ['gallery', Images, 'Gallery'],
          ['explore', Wand2, 'Chart Builder'],
        ] as const).map(([id, Icon, label]) => (
          <button key={id} onClick={() => setMode(id)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded text-[12px] ${
              mode === id ? 'bg-blue-500/20 text-blue-200' : 'pn-muted hover:pn-text'
            }`}>
            <Icon size={13} /> {label}
          </button>
        ))}
      </div>
      <div className="flex-1 overflow-hidden">
        {mode === 'gallery' ? <Gallery /> : <Explore />}
      </div>
    </div>
  )
}

// ── Gallery (plot history) ────────────────────────────────────────────────────
function Gallery() {
  const { plots, currentIndex, select, next, prev, clear } = usePlots()
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [light, setLight] = useState(false)
  const dragging = useRef<{ x: number; y: number } | null>(null)
  const cur = plots[currentIndex]

  useEffect(() => { setZoom(1); setPan({ x: 0, y: 0 }) }, [currentIndex])

  const onWheel = (e: React.WheelEvent) => {
    e.preventDefault()
    setZoom((z) => Math.min(8, Math.max(0.2, z * (e.deltaY < 0 ? 1.1 : 0.9))))
  }

  const dataUrl = useCallback((p = cur) => {
    if (!p) return null
    if (p.svg) return 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(p.svg)
    if (p.png) return `data:image/png;base64,${p.png}`
    return null
  }, [cur])

  const save = (kind: 'png' | 'svg') => {
    if (!cur) return
    let url: string, ext: string
    if (kind === 'svg' && cur.svg) { url = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(cur.svg); ext = 'svg' }
    else if (cur.png) { url = `data:image/png;base64,${cur.png}`; ext = 'png' }
    else if (cur.svg) { url = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(cur.svg); ext = 'svg' }
    else return
    const a = document.createElement('a')
    a.href = url; a.download = `plot-${currentIndex + 1}.${ext}`; a.click()
  }

  const copy = async () => {
    if (!cur?.png) return
    try {
      const blob = await (await fetch(`data:image/png;base64,${cur.png}`)).blob()
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
    } catch { /* clipboard image may be unsupported */ }
  }

  if (!plots.length) {
    return (
      <div className="h-full flex flex-col items-center justify-center pn-faint text-sm gap-2">
        <Images size={28} className="opacity-40" />
        <div>No plots yet. Run a cell that draws a figure.</div>
        <div className="text-[11px]">matplotlib, plotly, and other figures appear here automatically.</div>
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      {/* toolbar */}
      <div className="h-9 flex items-center gap-1 px-2 border-b pn-bd">
        <button onClick={prev} disabled={currentIndex === 0} className="p-1 rounded pn-hover pn-muted disabled:opacity-30"><ChevronLeft size={15} /></button>
        <span className="text-[11px] pn-faint tabular-nums w-14 text-center">{currentIndex + 1} / {plots.length}</span>
        <button onClick={next} disabled={currentIndex >= plots.length - 1} className="p-1 rounded pn-hover pn-muted disabled:opacity-30"><ChevronRight size={15} /></button>
        <div className="w-px h-4 bg-white/10 mx-1" />
        <button onClick={() => setZoom((z) => Math.min(8, z * 1.25))} className="p-1 rounded pn-hover pn-muted" title="Zoom in"><ZoomIn size={15} /></button>
        <button onClick={() => setZoom((z) => Math.max(0.2, z / 1.25))} className="p-1 rounded pn-hover pn-muted" title="Zoom out"><ZoomOut size={15} /></button>
        <button onClick={() => { setZoom(1); setPan({ x: 0, y: 0 }) }} className="p-1 rounded pn-hover pn-muted" title="Fit"><Maximize2 size={15} /></button>
        <span className="text-[10px] pn-faint w-10">{Math.round(zoom * 100)}%</span>
        <div className="flex-1" />
        <button onClick={() => setLight((l) => !l)} className="p-1 rounded pn-hover pn-muted" title="Background">{light ? <Moon size={15} /> : <Sun size={15} />}</button>
        <button onClick={copy} className="p-1 rounded pn-hover pn-muted" title="Copy PNG"><Copy size={15} /></button>
        <button onClick={() => save('png')} className="p-1 rounded pn-hover pn-muted" title="Save PNG"><Download size={15} /></button>
        {cur?.svg && <button onClick={() => save('svg')} className="px-1.5 py-1 rounded pn-hover pn-muted text-[10px] font-semibold" title="Save SVG (vector)">SVG</button>}
        <button onClick={() => { const u = dataUrl(); if (u) window.open(u, '_blank') }} className="p-1 rounded pn-hover pn-muted" title="Open in new tab"><ExternalLink size={15} /></button>
        <button onClick={clear} className="p-1 rounded hover:bg-rose-900/40 text-rose-400" title="Clear all"><Trash2 size={15} /></button>
      </div>

      {/* active plot */}
      <div className="flex-1 overflow-hidden relative" style={{ background: light ? '#ffffff' : 'transparent' }}
        onWheel={onWheel}
        onMouseDown={(e) => { dragging.current = { x: e.clientX - pan.x, y: e.clientY - pan.y } }}
        onMouseMove={(e) => { if (dragging.current) setPan({ x: e.clientX - dragging.current.x, y: e.clientY - dragging.current.y }) }}
        onMouseUp={() => { dragging.current = null }}
        onMouseLeave={() => { dragging.current = null }}>
        <div className="absolute inset-0 flex items-center justify-center cursor-grab active:cursor-grabbing">
          {cur?.html ? (
            <iframe title="plot" srcDoc={cur.html} className="w-full h-full border-0 bg-white" />
          ) : (
            <img src={dataUrl() ?? ''} alt={`plot ${currentIndex + 1}`}
              draggable={false}
              style={{ transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`, maxWidth: '92%', maxHeight: '92%' }}
              className="object-contain select-none" />
          )}
        </div>
      </div>

      {/* filmstrip */}
      {plots.length > 1 && (
        <div className="h-16 shrink-0 border-t pn-bd flex items-center gap-2 px-2 overflow-x-auto">
          {plots.map((p, i) => {
            const src = p.svg ? 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(p.svg) : p.png ? `data:image/png;base64,${p.png}` : ''
            return (
              <button key={p.id} onClick={() => select(i)}
                className={`h-12 w-16 shrink-0 rounded border overflow-hidden bg-white ${i === currentIndex ? 'border-blue-400 ring-1 ring-blue-400' : 'pn-bd'}`}>
                {src ? <img src={src} alt="" className="w-full h-full object-contain" /> : <BarChart3 className="m-auto pn-faint" size={16} />}
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}

// ── Explore (Looker-style no-code builder) ────────────────────────────────────
type ChartType = 'bar' | 'line' | 'area' | 'scatter' | 'pie' | 'heatmap'
type AggType = Measure['agg']

const CHARTS: { id: ChartType; icon: any; label: string }[] = [
  { id: 'bar', icon: BarChart3, label: 'Bar' },
  { id: 'line', icon: LineChart, label: 'Line' },
  { id: 'area', icon: AreaChart, label: 'Area' },
  { id: 'scatter', icon: ScatterChart, label: 'Scatter' },
  { id: 'pie', icon: PieChart, label: 'Pie' },
  { id: 'heatmap', icon: Grid3x3, label: 'Heatmap' },
]

function Explore() {
  const { target, title } = useViz()
  const [vars, setVars] = useState<{ name: string; type: string }[]>([])
  const [localTarget, setLocalTarget] = useState<ExplorerTarget | null>(target)
  const [localTitle, setLocalTitle] = useState(title)
  const [schema, setSchema] = useState<ColumnSchema[]>([])
  const [chart, setChart] = useState<ChartType>('bar')
  const [dim, setDim] = useState<string>('')
  const [dim2, setDim2] = useState<string>('') // color / heatmap-y
  const [measure, setMeasure] = useState<string>('')
  const [agg, setAgg] = useState<AggType>('sum')
  const [data, setData] = useState<Record<string, any>[]>([])
  const [loading, setLoading] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const viewRef = useRef<View | null>(null)

  useEffect(() => { if (target) { setLocalTarget(target); setLocalTitle(title) } }, [target, title])

  // populate the variable picker (DataFrame-typed only) when no target supplied
  useEffect(() => {
    if (target) return
    listVariables().then((vs) => setVars(vs.filter((v: any) => /DataFrame/.test(v.type)))).catch(() => {})
  }, [target])

  // load schema for the active target
  useEffect(() => {
    if (!localTarget) return
    exploreSchema(localTarget)
      .then((sc) => {
        setSchema(sc.columns)
        const firstCat = sc.columns.find((c) => c.logical !== 'number')
        const firstNum = sc.columns.find((c) => c.logical === 'number')
        setDim(firstCat?.name ?? sc.columns[0]?.name ?? '')
        setMeasure(firstNum?.name ?? '')
      })
      .catch((e: any) => setErr(e?.response?.data?.error || e?.message || 'failed to load schema'))
  }, [JSON.stringify(localTarget)])

  const dims = chart === 'heatmap' ? [dim, dim2].filter(Boolean) : [dim].filter(Boolean)
  const measures: Measure[] = measure ? [{ col: measure, agg }] : []
  const measureField = measure ? `${measure}_${agg}` : 'count'

  // run aggregation whenever the spec changes
  useEffect(() => {
    if (!localTarget || !dim) return
    setLoading(true)
    setErr(null)
    exploreAggregate(localTarget, { dims, measures, limit: 5000 })
      .then((r) => {
        const recs = r.data.map((row) => Object.fromEntries(r.columns.map((c, i) => [c, row[i]])))
        setData(recs)
      })
      .catch((e: any) => setErr(e?.response?.data?.error || e?.message || 'aggregation failed'))
      .finally(() => setLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [JSON.stringify(localTarget), chart, dim, dim2, measure, agg])

  const spec = useMemo(() => buildSpec(chart, dim, dim2, measureField, data), [chart, dim, dim2, measureField, data])

  const insertAltair = async () => {
    const code = altairCode(localTitle || 'df', chart, dim, dim2, measure, agg)
    const store = useNotebookStore.getState() as any
    if (!store.currentNotebook) await store.createNotebook('Viz')
    store.addCell('code')
    const s2 = useNotebookStore.getState() as any
    const idx = s2.currentNotebook.cells.length - 1
    s2.updateCell(idx, { source: code.split(/(?<=\n)/) })
  }

  const exportImg = async (kind: 'png' | 'svg') => {
    if (!viewRef.current) return
    const url = await viewRef.current.toImageURL(kind, kind === 'png' ? 2 : 1)
    const a = document.createElement('a')
    a.href = url; a.download = `chart.${kind}`; a.click()
  }

  if (!localTarget) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-3 pn-faint text-sm">
        <Wand2 size={26} className="opacity-40" />
        <div>Pick a DataFrame to visualize</div>
        {vars.length ? (
          <div className="flex flex-wrap gap-2 max-w-md justify-center">
            {vars.map((v) => (
              <button key={v.name} onClick={() => { setLocalTarget({ var: v.name }); setLocalTitle(v.name) }}
                className="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px] font-mono">{v.name}</button>
            ))}
          </div>
        ) : (
          <div className="text-[11px]">No DataFrames in the kernel yet. Run a cell that creates one.</div>
        )}
      </div>
    )
  }

  const cats = schema.filter((c) => c.logical !== 'number').map((c) => c.name)
  const nums = schema.filter((c) => c.logical === 'number').map((c) => c.name)

  return (
    <div className="h-full flex overflow-hidden">
      {/* shelves */}
      <div className="w-56 shrink-0 border-r pn-bd overflow-y-auto p-3 space-y-4 text-[12px]">
        <div>
          <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">Source</div>
          <div className="font-mono pn-text truncate">{localTitle}</div>
          {!target && <button onClick={() => setLocalTarget(null)} className="text-[11px] text-blue-300 hover:underline mt-0.5">change…</button>}
        </div>
        <div>
          <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">Chart</div>
          <div className="grid grid-cols-3 gap-1">
            {CHARTS.map((c) => (
              <button key={c.id} onClick={() => setChart(c.id)} title={c.label}
                className={`flex flex-col items-center gap-0.5 py-1.5 rounded ${chart === c.id ? 'bg-blue-500/20 text-blue-200' : 'pn-muted hover:pn-text bg-white/5'}`}>
                <c.icon size={15} /><span className="text-[9px]">{c.label}</span>
              </button>
            ))}
          </div>
        </div>
        <Shelf label={chart === 'scatter' ? 'X (measure)' : 'Dimension (X)'} value={dim} options={chart === 'scatter' ? [...nums, ...cats] : [...cats, ...nums]} onChange={setDim} />
        {chart === 'heatmap' && <Shelf label="Dimension (Y)" value={dim2} options={[...cats, ...nums]} onChange={setDim2} />}
        <div>
          <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">{chart === 'scatter' ? 'Y (measure)' : 'Measure (Y)'}</div>
          <select value={measure} onChange={(e) => setMeasure(e.target.value)} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text">
            <option value="">count (rows)</option>
            {nums.map((n) => <option key={n} value={n}>{n}</option>)}
          </select>
          {measure && (
            <select value={agg} onChange={(e) => setAgg(e.target.value as AggType)} className="w-full mt-1 px-2 py-1 rounded bg-white/5 border pn-bd pn-text">
              {(['sum', 'mean', 'count', 'min', 'max'] as AggType[]).map((a) => <option key={a} value={a}>{a}</option>)}
            </select>
          )}
        </div>
      </div>

      {/* canvas */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="h-9 flex items-center gap-1 px-2 border-b pn-bd">
          <span className="text-[12px] pn-muted truncate">{measureField} by {dims.join(' × ') || '—'}</span>
          <div className="flex-1" />
          <button onClick={insertAltair} className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[11px]"><NotebookPen size={12} /> Copy as code</button>
          <button onClick={() => exportImg('png')} className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[11px]"><Download size={12} /> PNG</button>
          <button onClick={() => exportImg('svg')} className="px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[11px] font-semibold">SVG</button>
        </div>
        <div className="flex-1 overflow-auto p-4 flex items-center justify-center">
          {err ? (
            <div className="text-rose-300 text-[13px] font-mono">{err}</div>
          ) : loading ? (
            <div className="pn-faint flex items-center gap-2 text-sm"><Loader2 size={16} className="animate-spin" /> Aggregating…</div>
          ) : data.length ? (
            <VegaEmbed className="w-full" spec={spec as any}
              options={{ actions: false, renderer: 'svg' }}
              onEmbed={(r) => { viewRef.current = r.view }} />
          ) : (
            <div className="pn-faint text-sm">No data for this selection.</div>
          )}
        </div>
      </div>
    </div>
  )
}

function Shelf({ label, value, options, onChange }: { label: string; value: string; options: string[]; onChange: (v: string) => void }) {
  return (
    <div>
      <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">{label}</div>
      <select value={value} onChange={(e) => onChange(e.target.value)} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text">
        {options.map((o) => <option key={o} value={o}>{o}</option>)}
      </select>
    </div>
  )
}

// Build a Vega-Lite spec from the shelf selections, with rows inlined.
function buildSpec(chart: ChartType, dim: string, dim2: string, measureField: string, data: Record<string, any>[]) {
  const base: any = {
    $schema: 'https://vega.github.io/schema/vega-lite/v5.json',
    data: { values: data },
    width: 'container',
    height: 320,
    background: 'transparent',
    config: {
      axis: { labelColor: '#94a3b8', titleColor: '#cbd5e1', gridColor: 'rgba(255,255,255,0.08)', domainColor: 'rgba(255,255,255,0.2)' },
      legend: { labelColor: '#94a3b8', titleColor: '#cbd5e1' },
      view: { stroke: 'transparent' },
    },
  }
  const q = { field: measureField, type: 'quantitative' as const }
  switch (chart) {
    case 'bar':
      return { ...base, mark: { type: 'bar', tooltip: true, cornerRadiusEnd: 2 }, encoding: { x: { field: dim, type: 'nominal', sort: '-y' }, y: q, color: { value: '#3b82f6' } } }
    case 'line':
      return { ...base, mark: { type: 'line', tooltip: true, point: true }, encoding: { x: { field: dim, type: 'ordinal' }, y: q, color: { value: '#34d399' } } }
    case 'area':
      return { ...base, mark: { type: 'area', tooltip: true, opacity: 0.7 }, encoding: { x: { field: dim, type: 'ordinal' }, y: q, color: { value: '#60a5fa' } } }
    case 'scatter':
      return { ...base, mark: { type: 'point', tooltip: true, filled: true }, encoding: { x: { field: dim, type: 'quantitative' }, y: q, color: { value: '#f472b6' } } }
    case 'pie':
      return { ...base, height: 320, mark: { type: 'arc', tooltip: true, innerRadius: 50 }, encoding: { theta: q, color: { field: dim, type: 'nominal' } } }
    case 'heatmap':
      return { ...base, mark: { type: 'rect', tooltip: true }, encoding: { x: { field: dim, type: 'nominal' }, y: { field: dim2, type: 'nominal' }, color: { ...q, scale: { scheme: 'blues' } } } }
  }
}

// Emit equivalent Altair (Python) so the no-code chart is reproducible in a cell.
function altairCode(df: string, chart: ChartType, dim: string, dim2: string, measure: string, agg: AggType) {
  const yExpr = measure ? `alt.Y('${measure}', aggregate='${agg === 'mean' ? 'mean' : agg}')` : `alt.Y('count()')`
  const markMap: Record<ChartType, string> = { bar: 'mark_bar', line: 'mark_line', area: 'mark_area', scatter: 'mark_point', pie: 'mark_arc', heatmap: 'mark_rect' }
  let enc: string
  if (chart === 'pie') enc = `theta=${yExpr}, color='${dim}'`
  else if (chart === 'heatmap') enc = `x='${dim}', y='${dim2}', color=${yExpr}`
  else if (chart === 'scatter') enc = `x='${dim}', y='${measure || 'count()'}'`
  else enc = `x='${dim}', y=${yExpr}`
  return [
    'import altair as alt',
    `chart = alt.Chart(${df}).${markMap[chart]}().encode(`,
    `    ${enc}`,
    ').properties(width=600, height=360).interactive()',
    'chart',
  ].join('\n')
}
