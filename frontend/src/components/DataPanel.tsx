import { useEffect, useState } from 'react'
import { Database, X, Plus, Trash2, Play, Loader2, Cloud, AlertTriangle } from 'lucide-react'
import {
  listDatabases, createDatabase, deleteDatabase, queryDatabase,
  listWarehouses, queryWarehouse, type DbConnection, type QueryResult,
} from '../api/data'
import DataFrameView from './DataFrameView'

type Conn = { id: string; name: string; kind: 'db' | 'warehouse'; sub: string }

// Connections + SQL runner. Results render through DataFrameView (Table/Bar/Line).
export default function DataPanel({ onClose }: { onClose: () => void }) {
  const [conns, setConns] = useState<Conn[]>([])
  const [sel, setSel] = useState<Conn | null>(null)
  const [sql, setSql] = useState('SELECT 1')
  const [result, setResult] = useState<QueryResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [running, setRunning] = useState(false)
  const [showAdd, setShowAdd] = useState(false)
  const [form, setForm] = useState<Partial<DbConnection>>({ db_type: 'sqlite', name: '', database: '', host: 'localhost', port: 5432 })

  const refresh = async () => {
    const [dbs, whs] = await Promise.all([listDatabases().catch(() => []), listWarehouses().catch(() => [])])
    const list: Conn[] = [
      ...dbs.map((d) => ({ id: d.id, name: d.name, kind: 'db' as const, sub: d.db_type })),
      ...whs.map((w: any) => ({ id: w.id, name: w.name, kind: 'warehouse' as const, sub: String(w.warehouse_type ?? 'warehouse').toLowerCase() })),
    ]
    setConns(list)
    if (!sel && list.length) setSel(list[0])
  }
  useEffect(() => {
    refresh()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const run = async () => {
    if (!sel) return
    setRunning(true)
    setError(null)
    setResult(null)
    try {
      const r = sel.kind === 'db' ? await queryDatabase(sel.id, sql) : await queryWarehouse(sel.id, sql)
      setResult(r)
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'query failed')
    } finally {
      setRunning(false)
    }
  }

  const addDb = async () => {
    if (!form.name || !form.database) return
    await createDatabase(form)
    setShowAdd(false)
    setForm({ db_type: 'sqlite', name: '', database: '', host: 'localhost', port: 5432 })
    refresh()
  }

  return (
    <div className="absolute inset-0 z-30 pn-app flex flex-col">
      <div className="h-10 flex items-center justify-between px-4 border-b pn-bd">
        <span className="flex items-center gap-2 text-sm font-semibold pn-text"><Database size={16} className="text-violet-400" /> Data &amp; SQL</span>
        <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* connections list */}
        <div className="w-60 shrink-0 border-r pn-bd flex flex-col">
          <div className="flex items-center justify-between px-3 py-2 text-[11px] uppercase tracking-wide pn-faint">
            Connections
            <button onClick={() => setShowAdd((s) => !s)} className="p-1 rounded pn-hover" title="Add database"><Plus size={13} /></button>
          </div>
          <div className="flex-1 overflow-y-auto">
            {conns.length === 0 && <div className="px-3 py-2 text-[12px] pn-faint">No connections yet.</div>}
            {conns.map((c) => (
              <div key={c.id} className={`group flex items-center gap-2 px-3 py-1.5 cursor-pointer text-[13px] ${sel?.id === c.id ? 'bg-violet-500/15 pn-text' : 'pn-muted hover:pn-text'}`} onClick={() => setSel(c)}>
                {c.kind === 'warehouse' ? <Cloud size={13} className="text-sky-400" /> : <Database size={13} className="text-emerald-400" />}
                <span className="flex-1 truncate">{c.name}</span>
                <span className="text-[10px] pn-faint">{c.sub}</span>
                {c.kind === 'db' && (
                  <button onClick={async (e) => { e.stopPropagation(); await deleteDatabase(c.id); if (sel?.id === c.id) setSel(null); refresh() }}
                    className="opacity-0 group-hover:opacity-100 text-rose-400"><Trash2 size={12} /></button>
                )}
              </div>
            ))}
          </div>
          {showAdd && (
            <div className="border-t pn-bd p-2 space-y-1.5 text-[12px]">
              <select value={form.db_type} onChange={(e) => setForm({ ...form, db_type: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text">
                {['sqlite', 'duckdb', 'postgresql', 'mysql'].map((t) => <option key={t} value={t}>{t}</option>)}
              </select>
              <input placeholder="name" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
              <input placeholder={form.db_type === 'sqlite' || form.db_type === 'duckdb' ? 'file path / database' : 'database'} value={form.database} onChange={(e) => setForm({ ...form, database: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
              {form.db_type !== 'sqlite' && form.db_type !== 'duckdb' && (
                <>
                  <input placeholder="host" value={form.host ?? ''} onChange={(e) => setForm({ ...form, host: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
                  <input placeholder="port" value={form.port ?? ''} onChange={(e) => setForm({ ...form, port: parseInt(e.target.value) || undefined })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
                  <input placeholder="user" value={form.username ?? ''} onChange={(e) => setForm({ ...form, username: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
                  <input placeholder="password" type="password" value={form.password ?? ''} onChange={(e) => setForm({ ...form, password: e.target.value })} className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text" />
                </>
              )}
              <button onClick={addDb} className="w-full px-2 py-1 rounded prism-bg text-white">Add</button>
            </div>
          )}
        </div>

        {/* SQL editor + results */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="p-3 border-b pn-bd">
            <textarea value={sql} onChange={(e) => setSql(e.target.value)} rows={4} spellCheck={false}
              placeholder={sel ? `SQL for ${sel.name}…` : 'Add or select a connection first'}
              className="w-full px-3 py-2 rounded bg-white/5 border pn-bd pn-text font-mono text-[13px] outline-none focus:border-violet-500" />
            <div className="mt-2 flex items-center gap-2">
              <button onClick={run} disabled={!sel || running} className="flex items-center gap-1 px-3 py-1.5 rounded prism-bg text-white text-[13px] disabled:opacity-40">
                {running ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />} Run
              </button>
              {sel && <span className="text-[12px] pn-faint">{sel.kind === 'warehouse' ? 'cloud warehouse' : sel.sub} · {sel.name}</span>}
            </div>
          </div>
          <div className="flex-1 overflow-auto p-3">
            {error && (
              <div className="rounded border border-rose-700/60 bg-rose-900/20 p-3 text-[13px] text-rose-200 flex gap-2">
                <AlertTriangle size={15} className="shrink-0 mt-0.5" /><pre className="whitespace-pre-wrap font-mono text-[12px]">{error}</pre>
              </div>
            )}
            {result && !error && (
              result.rows.length || result.columns.length
                ? <DataFrameView df={{ columns: result.columns, data: result.rows }} />
                : <div className="pn-faint text-[13px]">Query OK — {result.row_count} rows.</div>
            )}
            {!result && !error && <div className="pn-faint text-[13px]">Run a query to see results.</div>}
          </div>
        </div>
      </div>
    </div>
  )
}
