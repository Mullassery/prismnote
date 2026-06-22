import { useEffect, useRef, useState } from 'react'
import axios from 'axios'
import {
  Folder, FileCode, FileText, ChevronUp, RefreshCw, Loader2,
  FilePlus, FolderPlus, Upload, Eye, EyeOff, Pencil, Trash2,
} from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'

interface Entry { name: string; path: string; is_dir: boolean }
interface Listing { path: string; parent: string | null; entries: Entry[] }

// Filesystem browser backed by the local PrismNote server (`/api/fs/*`). Works
// in any browser, with new/rename/delete/upload, a filter, and a hidden-files
// toggle (so the app's own ~/.prismnote/notebooks is reachable).
export default function ServerExplorer({ initialPath }: { initialPath?: string }) {
  const [listing, setListing] = useState<Listing | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showHidden, setShowHidden] = useState(false)
  const [filter, setFilter] = useState('')
  const uploadRef = useRef<HTMLInputElement>(null)

  const cwd = listing?.path

  const load = async (path?: string, hidden = showHidden) => {
    setLoading(true)
    setError(null)
    try {
      const res = await axios.get<Listing>('/api/fs/list', { params: { path, show_hidden: hidden } })
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

  const reload = () => load(cwd)

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
      const nb = { id: `local-${entry.path}`, name: entry.name.replace(/\.ipynb$/, ''), cells, metadata: data.metadata ?? {} }
      useNotebookStore.setState((s: any) => ({
        notebooks: [...s.notebooks.filter((n: any) => n.id !== nb.id), nb],
        currentNotebookId: nb.id,
        currentNotebook: nb,
      }))
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'Could not open notebook')
    }
  }

  const onEntry = (e: Entry) => {
    if (e.is_dir) load(e.path)
    else if (e.name.endsWith('.ipynb')) openNotebook(e)
  }

  const newFile = async () => {
    const name = window.prompt('New file name'); if (!name || !cwd) return
    try { await axios.post('/api/fs/new-file', { path: cwd, name }); reload() }
    catch (e: any) { setError(e?.response?.data?.error || 'create failed') }
  }
  const newFolder = async () => {
    const name = window.prompt('New folder name'); if (!name || !cwd) return
    try { await axios.post('/api/fs/mkdir', { path: cwd, name }); reload() }
    catch (e: any) { setError(e?.response?.data?.error || 'create failed') }
  }
  const rename = async (e: Entry) => {
    const new_name = window.prompt('Rename to', e.name); if (!new_name || new_name === e.name) return
    try { await axios.post('/api/fs/rename', { path: e.path, new_name }); reload() }
    catch (err: any) { setError(err?.response?.data?.error || 'rename failed') }
  }
  const del = async (e: Entry) => {
    if (!window.confirm(`Delete "${e.name}"?`)) return
    try { await axios.post('/api/fs/delete', { path: e.path }); reload() }
    catch (err: any) { setError(err?.response?.data?.error || 'delete failed') }
  }
  const onUpload = async (ev: React.ChangeEvent<HTMLInputElement>) => {
    const file = ev.target.files?.[0]; if (!file || !cwd) return
    const content = await file.text()
    try { await axios.post('/api/fs/write', { path: `${cwd}/${file.name}`, content }); reload() }
    catch (e: any) { setError(e?.response?.data?.error || 'upload failed') }
    finally { if (uploadRef.current) uploadRef.current.value = '' }
  }
  const toggleHidden = () => { const h = !showHidden; setShowHidden(h); load(cwd, h) }

  const shown = (listing?.entries ?? []).filter((e) =>
    !filter || e.name.toLowerCase().includes(filter.toLowerCase()))

  const iconBtn = 'p-1 rounded pn-hover'

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* path bar */}
      <div className="flex items-center gap-1 px-2 py-1 border-b pn-bd text-[11px] pn-muted">
        <button onClick={() => listing?.parent && load(listing.parent)} disabled={!listing?.parent} title="Up one level" className={`${iconBtn} disabled:opacity-30`}><ChevronUp size={13} /></button>
        <span className="truncate flex-1" title={cwd}>{cwd ?? '…'}</span>
        <button onClick={reload} title="Refresh" className={iconBtn}>{loading ? <Loader2 size={12} className="animate-spin" /> : <RefreshCw size={12} />}</button>
      </div>
      {/* toolbar */}
      <div className="flex items-center gap-0.5 px-2 py-1 border-b pn-bd pn-muted">
        <button onClick={newFile} title="New file" className={iconBtn}><FilePlus size={13} /></button>
        <button onClick={newFolder} title="New folder" className={iconBtn}><FolderPlus size={13} /></button>
        <button onClick={() => uploadRef.current?.click()} title="Upload file" className={iconBtn}><Upload size={13} /></button>
        <input ref={uploadRef} type="file" className="hidden" onChange={onUpload} />
        <div className="flex-1" />
        <button onClick={toggleHidden} title={showHidden ? 'Hide hidden files' : 'Show hidden files'} className={`${iconBtn} ${showHidden ? 'text-violet-400' : ''}`}>
          {showHidden ? <Eye size={13} /> : <EyeOff size={13} />}
        </button>
      </div>
      {/* filter */}
      <div className="px-2 py-1 border-b pn-bd">
        <input value={filter} onChange={(e) => setFilter(e.target.value)} placeholder="Filter…"
          className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[12px] outline-none focus:border-violet-500" />
      </div>

      {error && <div className="px-3 py-2 text-[12px] text-rose-400">{error}</div>}

      <div className="flex-1 overflow-y-auto py-1">
        {shown.map((e) => {
          const isNb = e.name.endsWith('.ipynb')
          const Icon = e.is_dir ? Folder : isNb ? FileCode : FileText
          return (
            <div key={e.path} className="group flex items-center gap-2 px-3 py-1 text-[13px] hover:bg-violet-500/15">
              <button onClick={() => onEntry(e)} className="flex items-center gap-2 flex-1 min-w-0 text-left">
                <Icon size={14} className={e.is_dir ? 'text-violet-300' : isNb ? 'text-yellow-400' : 'pn-faint'} />
                <span className={`truncate ${e.is_dir || isNb ? 'pn-text' : 'pn-muted'}`}>{e.name}</span>
              </button>
              <button onClick={() => rename(e)} title="Rename" className="opacity-0 group-hover:opacity-100 pn-faint hover:pn-text"><Pencil size={12} /></button>
              <button onClick={() => del(e)} title="Delete" className="opacity-0 group-hover:opacity-100 text-rose-400"><Trash2 size={12} /></button>
            </div>
          )
        })}
        {listing && shown.length === 0 && !error && (
          <div className="px-3 py-2 text-[12px] pn-faint">{filter ? 'No matches.' : 'Empty folder.'}</div>
        )}
      </div>
    </div>
  )
}
