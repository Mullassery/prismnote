import { useMemo, useState } from 'react'
import { Table2, BarChart3, LineChart } from 'lucide-react'

interface DF {
  columns: (string | number)[]
  data: any[][]
}

type View = 'table' | 'bar' | 'line'

const PALETTE = ['#3b82f6', '#34d399', '#60a5fa', '#f472b6', '#fbbf24', '#f87171']

function isNum(v: any) {
  return typeof v === 'number' && Number.isFinite(v)
}

export default function DataFrameView({ df, html }: { df: DF; html?: string }) {
  const [view, setView] = useState<View>('table')

  const cols = df.columns.map(String)
  // numeric columns (≥60% numbers) are chartable series; first non-numeric is the x label
  const numericCols = useMemo(
    () => cols.filter((_, ci) => df.data.filter((r) => isNum(r[ci])).length >= df.data.length * 0.6),
    [df, cols],
  )
  const labelCol = cols.findIndex((c) => !numericCols.includes(c))
  const labels = df.data.map((r, i) => (labelCol >= 0 ? String(r[labelCol]) : String(i)))
  const series = (numericCols.length ? numericCols : cols).slice(0, 4).map((c) => ({
    name: c,
    values: df.data.map((r) => Number(r[cols.indexOf(c)]) || 0),
  }))
  const chartable = series.length > 0 && df.data.length > 0

  return (
    <div className="bg-slate-800 rounded">
      <div className="flex items-center gap-1 px-2 py-1 border-b border-white/10">
        {([
          ['table', Table2, 'Table'],
          ['bar', BarChart3, 'Bar'],
          ['line', LineChart, 'Line'],
        ] as const).map(([v, Icon, label]) => (
          <button
            key={v}
            onClick={() => setView(v)}
            disabled={v !== 'table' && !chartable}
            className={`flex items-center gap-1 px-2 py-0.5 rounded text-[11px] ${
              view === v ? 'bg-blue-500/30 text-blue-200' : 'text-gray-400 hover:text-gray-200'
            } disabled:opacity-30`}
          >
            <Icon size={12} /> {label}
          </button>
        ))}
        <span className="ml-auto text-[10px] text-gray-500">{df.data.length} rows</span>
      </div>

      {view === 'table' ? (
        html ? (
          <div className="viz-container p-2 overflow-auto max-h-80 text-gray-300" dangerouslySetInnerHTML={{ __html: html }} />
        ) : (
          <FallbackTable cols={cols} data={df.data} />
        )
      ) : (
        <Chart kind={view} labels={labels} series={series} />
      )}
    </div>
  )
}

function FallbackTable({ cols, data }: { cols: string[]; data: any[][] }) {
  return (
    <div className="overflow-auto max-h-80 p-2">
      <table className="text-[12px] text-gray-300">
        <thead>
          <tr>{cols.map((c) => <th key={c} className="px-2 py-1 text-left border-b border-white/10">{c}</th>)}</tr>
        </thead>
        <tbody>
          {data.slice(0, 200).map((r, i) => (
            <tr key={i}>{r.map((v, j) => <td key={j} className="px-2 py-0.5 border-b border-white/5">{String(v)}</td>)}</tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function Chart({
  kind,
  labels,
  series,
}: {
  kind: 'bar' | 'line'
  labels: string[]
  series: { name: string; values: number[] }[]
}) {
  const W = 600
  const H = 280
  const pad = { l: 44, r: 12, t: 12, b: 40 }
  const iw = W - pad.l - pad.r
  const ih = H - pad.t - pad.b
  const n = labels.length
  const allVals = series.flatMap((s) => s.values)
  const max = Math.max(1, ...allVals)
  const min = Math.min(0, ...allVals)
  const y = (v: number) => pad.t + ih - ((v - min) / (max - min || 1)) * ih
  const xBand = iw / Math.max(1, n)

  return (
    <div className="p-2 overflow-x-auto">
      <svg width={W} height={H} className="max-w-full">
        {/* y gridlines */}
        {[0, 0.25, 0.5, 0.75, 1].map((t) => {
          const v = min + t * (max - min)
          const yy = y(v)
          return (
            <g key={t}>
              <line x1={pad.l} x2={W - pad.r} y1={yy} y2={yy} stroke="rgba(255,255,255,0.08)" />
              <text x={pad.l - 6} y={yy + 3} textAnchor="end" fontSize="9" fill="#94a3b8">{v.toFixed(1)}</text>
            </g>
          )
        })}
        {/* x labels (thinned) */}
        {labels.map((l, i) =>
          i % Math.ceil(n / 12 || 1) === 0 ? (
            <text key={i} x={pad.l + i * xBand + xBand / 2} y={H - pad.b + 14} textAnchor="middle" fontSize="9" fill="#94a3b8">
              {l.length > 8 ? l.slice(0, 8) + '…' : l}
            </text>
          ) : null,
        )}
        {kind === 'bar'
          ? series.map((s, si) =>
              s.values.map((v, i) => {
                const bw = (xBand * 0.8) / series.length
                const x = pad.l + i * xBand + xBand * 0.1 + si * bw
                return <rect key={`${si}-${i}`} x={x} y={y(v)} width={Math.max(1, bw - 1)} height={Math.max(0, y(min) - y(v))} fill={PALETTE[si % PALETTE.length]} />
              }),
            )
          : series.map((s, si) => (
              <polyline
                key={si}
                fill="none"
                stroke={PALETTE[si % PALETTE.length]}
                strokeWidth="2"
                points={s.values.map((v, i) => `${pad.l + i * xBand + xBand / 2},${y(v)}`).join(' ')}
              />
            ))}
      </svg>
      <div className="flex flex-wrap gap-3 px-2 pt-1">
        {series.map((s, i) => (
          <span key={s.name} className="flex items-center gap-1 text-[11px] text-gray-400">
            <span className="w-2.5 h-2.5 rounded-sm" style={{ background: PALETTE[i % PALETTE.length] }} /> {s.name}
          </span>
        ))}
      </div>
    </div>
  )
}
