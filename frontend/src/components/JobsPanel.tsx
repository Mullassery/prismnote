import { useEffect, useState } from 'react'
import { Play, Trash2, X, Plus, Clock, CheckCircle2, XCircle, Loader2, Briefcase, Workflow, Copy } from 'lucide-react'
import { listJobs, createJob, runJob, deleteJob, airflowDag, type JobSummary, type Schedule } from '../api/jobs'
import { useNotebookStore } from '../hooks/useNotebook'

// Airflow-like Jobs view: save the current notebook as a runnable job, run it on
// demand or on a schedule, and see status/history.
export default function JobsPanel({ onClose, initialCreate }: { onClose: () => void; initialCreate?: boolean }) {
  const { currentNotebook } = useNotebookStore()
  const [jobs, setJobs] = useState<JobSummary[]>([])
  const [loading, setLoading] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const [showCreate, setShowCreate] = useState(!!initialCreate && !!currentNotebook)
  const [dag, setDag] = useState<{ dag: string; filename: string } | null>(null)

  // create form
  const [name, setName] = useState('')
  const [kind, setKind] = useState<Schedule['kind']>('manual')
  const [minutes, setMinutes] = useState(60)
  const [time, setTime] = useState('09:00')

  const refresh = async () => {
    setLoading(true)
    try {
      setJobs(await listJobs())
    } finally {
      setLoading(false)
    }
  }
  useEffect(() => {
    refresh()
  }, [])

  const codeCells = () =>
    (currentNotebook?.cells ?? [])
      .filter((c: any) => c.cell_type === 'code')
      .map((c: any) => (Array.isArray(c.source) ? c.source.join('') : c.source))

  const submitCreate = async () => {
    const cells = codeCells()
    if (!name.trim() || cells.length === 0) return
    const schedule: Schedule =
      kind === 'interval' ? { kind, minutes } : kind === 'daily' ? { kind, time } : { kind: 'manual' }
    await createJob(name.trim(), cells, schedule)
    setShowCreate(false)
    setName('')
    refresh()
  }

  const run = async (id: string) => {
    setBusy(id)
    try {
      await runJob(id)
      await refresh()
    } finally {
      setBusy(null)
    }
  }

  const scheduleLabel = (s: Schedule) =>
    s.kind === 'interval' ? `every ${s.minutes}m` : s.kind === 'daily' ? `daily ${s.time}` : 'manual'

  return (
    <div className="absolute inset-0 z-30 pn-app flex flex-col">
      <div className="h-10 flex items-center justify-between px-4 border-b pn-bd">
        <span className="flex items-center gap-2 text-sm font-semibold pn-text">
          <Briefcase size={16} className="text-blue-400" /> Jobs
        </span>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowCreate((s) => !s)}
            disabled={!currentNotebook}
            title={currentNotebook ? 'Save the current notebook as a job' : 'Open a notebook first'}
            className="flex items-center gap-1 px-3 py-1 rounded-md prism-bg text-white text-[13px] disabled:opacity-40"
          >
            <Plus size={14} /> Run current notebook as Job
          </button>
          <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
        </div>
      </div>

      {showCreate && (
        <div className="border-b pn-bd p-3 flex flex-wrap items-end gap-3 bg-blue-500/5">
          <label className="text-[12px] pn-muted">
            <div className="mb-1">Job name</div>
            <input value={name} onChange={(e) => setName(e.target.value)} placeholder="nightly etl"
              className="px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-blue-500" />
          </label>
          <label className="text-[12px] pn-muted">
            <div className="mb-1">Schedule</div>
            <select value={kind} onChange={(e) => setKind(e.target.value as Schedule['kind'])}
              className="px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none">
              <option value="manual">Manual</option>
              <option value="interval">Every N minutes</option>
              <option value="daily">Daily at time</option>
            </select>
          </label>
          {kind === 'interval' && (
            <label className="text-[12px] pn-muted">
              <div className="mb-1">Minutes</div>
              <input type="number" min={1} value={minutes} onChange={(e) => setMinutes(parseInt(e.target.value) || 1)}
                className="w-24 px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none" />
            </label>
          )}
          {kind === 'daily' && (
            <label className="text-[12px] pn-muted">
              <div className="mb-1">Time</div>
              <input type="time" value={time} onChange={(e) => setTime(e.target.value)}
                className="px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none" />
            </label>
          )}
          <button onClick={submitCreate} disabled={!name.trim()}
            className="px-3 py-1.5 rounded-md prism-bg text-white text-[13px] disabled:opacity-40">Create job</button>
          <span className="text-[12px] pn-faint">{codeCells().length} code cells will be saved</span>
        </div>
      )}

      {dag && (
        <div className="absolute inset-0 z-40 bg-black/50 flex items-center justify-center p-6" onClick={() => setDag(null)}>
          <div className="pn-solid-bg border pn-bd rounded-xl max-w-2xl w-full max-h-[80vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between px-4 py-2 border-b pn-bd">
              <span className="text-sm pn-text flex items-center gap-2"><Workflow size={15} className="text-blue-400" /> Airflow DAG — <code className="pn-faint">{dag.filename}</code></span>
              <div className="flex items-center gap-2">
                <button onClick={() => navigator.clipboard.writeText(dag.dag)} className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px]"><Copy size={12} /> Copy</button>
                <button onClick={() => setDag(null)} className="p-1 rounded pn-hover pn-muted"><X size={15} /></button>
              </div>
            </div>
            <pre className="flex-1 overflow-auto p-3 text-[12px] font-mono pn-muted whitespace-pre">{dag.dag}</pre>
            <div className="px-4 py-2 border-t pn-bd text-[12px] pn-faint">Drop this file in your Airflow <code>dags/</code> folder. It triggers the job via <code>/api/jobs/run-by-name</code>.</div>
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-4">
        {loading && <div className="pn-faint text-sm flex items-center gap-2"><Loader2 size={14} className="animate-spin" /> Loading…</div>}
        {!loading && jobs.length === 0 && (
          <div className="pn-faint text-sm">No jobs yet. Open a notebook and click “Run current notebook as Job”.</div>
        )}
        <div className="space-y-2 max-w-3xl">
          {jobs.map((j) => (
            <div key={j.id} className="rounded-lg border pn-bd pn-solid-bg p-3 flex items-center gap-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium pn-text truncate">{j.name}</span>
                  {j.last_status === 'success' && <CheckCircle2 size={14} className="text-emerald-400" />}
                  {j.last_status === 'failed' && <XCircle size={14} className="text-rose-400" />}
                </div>
                <div className="text-[12px] pn-faint flex items-center gap-3 mt-0.5">
                  <span className="flex items-center gap-1"><Clock size={11} /> {scheduleLabel(j.schedule)}</span>
                  <span>{j.cells} cells</span>
                  <span>{j.runs} runs</span>
                  {j.last_run && <span>last: {new Date(j.last_run).toLocaleString()}</span>}
                </div>
              </div>
              <button onClick={() => run(j.id)} disabled={busy === j.id}
                className="flex items-center gap-1 px-2.5 py-1 rounded bg-emerald-500/20 text-emerald-300 hover:bg-emerald-500/30 text-[12px] disabled:opacity-50">
                {busy === j.id ? <Loader2 size={13} className="animate-spin" /> : <Play size={13} />} Run now
              </button>
              <button onClick={async () => setDag(await airflowDag(j.id))}
                className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-muted hover:pn-text text-[12px]" title="Get Airflow DAG to trigger this job remotely">
                <Workflow size={13} /> Airflow
              </button>
              <button onClick={async () => { await deleteJob(j.id); refresh() }}
                className="p-1.5 rounded pn-hover text-rose-400" title="Delete job"><Trash2 size={14} /></button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
