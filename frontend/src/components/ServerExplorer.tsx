import { useEffect, useState } from 'react'
import axios from 'axios'
import { Folder, FileCode, FileText, ChevronUp, RefreshCw, Loader2 } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'

interface Entry {
  name: string
  path: string
  is_dir: boolean
}

interface Listing {
  path: string
  parent: string | null
  entries: Entry[]
}

// A filesystem browser backed by the local PrismNote server (`/api/fs/*`).
// Works in any browser — unlike the File System Access API it doesn't need a
// native picker, so it also functions in embedded/automated windows.
export default function ServerExplorer({ initialPath }: { initialPath?: string }) {
  const [listing, setListing] = useState<Listing | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const load = async (path?: string) => {
    setLoading(true)
    setError(null)
    try {
      const res = await axios.get<Listing>('/api/fs/list', { params: { path } })
      setListing(res.data)
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'Could not list folder')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    load(initialPath)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const openNotebook = async (entry: Entry) => {
    try {
      const res = await axios.get<{ content: string }>('/api/fs/read', { params: { path: entry.path } })
      const data = JSON.parse(res.data.content)
      const cells = (data.cells ?? []).map((c: any, i: number) => ({
        id: `${entry.name}-${i}`,
        cell_type: c.cell_type ?? 'code',
        source: c.source ?? [],
        outputs: c.outputs ?? [],
        execution_count: c.execution_count ?? null,
        metadata: c.metadata ?? {},
      }))
      const nb = {
        id: `local-${entry.path}`,
        name: entry.name.replace(/\.ipynb$/, ''),
        cells,
        metadata: data.metadata ?? {},
      }
      useNotebookStore.setState((s: any) => ({
        notebooks: [...s.notebooks.filter((n: any) => n.id !== nb.id), nb],
        currentNotebookId: nb.id,
        currentNotebook: nb,
      }))
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'Could not open notebook')
    }
  }

  const onEntry = (entry: Entry) => {
    if (entry.is_dir) load(entry.path)
    else if (entry.name.endsWith('.ipynb')) openNotebook(entry)
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* path bar */}
      <div className="flex items-center gap-1 px-2 py-1 border-b pn-bd text-[11px] pn-muted">
        <button
          onClick={() => listing?.parent && load(listing.parent)}
          disabled={!listing?.parent}
          title="Up one level"
          className="p-1 rounded pn-hover disabled:opacity-30"
        >
          <ChevronUp size={13} />
        </button>
        <span className="truncate flex-1" title={listing?.path}>{listing?.path ?? '…'}</span>
        <button onClick={() => load(listing?.path)} title="Refresh" className="p-1 rounded pn-hover">
          {loading ? <Loader2 size={12} className="animate-spin" /> : <RefreshCw size={12} />}
        </button>
      </div>

      {error && <div className="px-3 py-2 text-[12px] text-rose-400">{error}</div>}

      <div className="flex-1 overflow-y-auto py-1">
        {(listing?.entries ?? []).map((e) => {
          const isNb = e.name.endsWith('.ipynb')
          const Icon = e.is_dir ? Folder : isNb ? FileCode : FileText
          return (
            <button
              key={e.path}
              onClick={() => onEntry(e)}
              className={`w-full flex items-center gap-2 px-3 py-1 text-[13px] text-left hover:bg-violet-500/15 ${
                e.is_dir ? 'pn-text' : isNb ? 'pn-text' : 'pn-muted'
              }`}
            >
              <Icon size={14} className={e.is_dir ? 'text-violet-300' : isNb ? 'text-yellow-400' : 'pn-faint'} />
              <span className="truncate">{e.name}</span>
            </button>
          )
        })}
        {listing && listing.entries.length === 0 && !error && (
          <div className="px-3 py-2 text-[12px] pn-faint">Empty folder.</div>
        )}
      </div>
    </div>
  )
}
