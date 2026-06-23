import { useEffect, useRef, useState } from 'react'
import axios from 'axios'
import {
  Folder, FileCode, FileText, ChevronUp, RefreshCw, Loader2,
  FilePlus, FolderPlus, Upload, Eye, EyeOff, Pencil, Trash2, Scissors, Copy, ClipboardPaste,
} from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'
import { useExplorerRequest, isDataFile } from '../hooks/useExplorerRequest'
import { useAIContext } from '../hooks/useAIContext'

interface Entry { name: string; path: string; is_dir: boolean }
interface Listing { path: string; parent: string | null; entries: Entry[] }
type Clip = { paths: string[]; op: 'cut' | 'copy' } | null

// git porcelain code -> colour
const gitColor: Record<string, string> = { M: 'text-amber-400', A: 'text-emerald-400', '?': 'text-emerald-400', D: 'text-rose-400', R: 'text-sky-400' }

export default function ServerExplorer({ initialPath }: { initialPath?: string }) {
  const [listing, setListing] = useState<Listing | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showHidden, setShowHidden] = useState(false)
  const [filter, setFilter] = useState('')
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [clip, setClip] = useState<Clip>(null)
  const [git, setGit] = useState<Record<string, string>>({}) // basename -> code
  const [dropActive, setDropActive] = useState(false)
  const anchor = useRef<number>(-1)
  const uploadRef = useRef<HTMLInputElement>(null)
  const cwd = listing?.path

  const load = async (path?: string, hidden = showHidden) => {
    setLoading(true); setError(null)
    try {
      const res = await axios.get<Listing>('/api/fs/list', { params: { path, show_hidden: hidden } })
      setListing(res.data); setSelected(new Set())
      // publish workspace context for the AI panel
      useAIContext.getState().setWorkspace(
        res.data.path.split('/').filter(Boolean).pop() || res.data.path,
        (res.data.entries ?? []).map((e) => (e.is_dir ? e.name + '/' : e.name)),
      )
      // git status decorations (best-effort)
      try {
        const g = await axios.get('/api/git/status', { params: { path: res.data.path } })
        const map: Record<string, string> = {}
        if (g.data?.is_repo) {
          for (const line of String(g.data.status || '').split('\n')) {
            if (!line.trim()) continue
            const code = line.trim()[0]
            const name = line.slice(3).trim().split('/').pop() || ''
            if (name) map[name] = code
          }
        }
        setGit(map)
      } catch { setGit({}) }
    } catch (e: any) {
      setError(e?.response?.data?.error || e?.message || 'Could not list folder')
    } finally { setLoading(false) }
  }

  useEffect(() => { load(initialPath); /* eslint-disable-next-line */ }, [])
  // auto-refresh when the window regains focus (catches external changes)
  useEffect(() => {
    const onFocus = () => { if (cwd) load(cwd) }
    window.addEventListener('focus', onFocus)
    return () => window.removeEventListener('focus', onFocus)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cwd, showHidden])

  const reload = () => load(cwd)

  const openNotebook = async (entry: Entry) => {
    try {
      const res = await axios.get<{ content: string }>('/api/fs/read', { params: { path: entry.path } })
      const data = JSON.parse(res.data.content)
      const cells = (data.cells ?? []).map((c: any, i: number) => ({
        id: `${entry.name}-${i}`, cell_type: c.cell_type ?? 'code', source: c.source ?? [],
        outputs: c.outputs ?? [], execution_count: c.execution_count ?? null, metadata: c.metadata ?? {},
      }))
      const nb = { id: `local-${entry.path}`, name: entry.name.replace(/\.ipynb$/, ''), cells, metadata: data.metadata ?? {} }
      useNotebookStore.setState((s: any) => ({
        notebooks: [...s.notebooks.filter((n: any) => n.id !== nb.id), nb],
        currentNotebookId: nb.id, currentNotebook: nb,
      }))
    } catch (e: any) { setError(e?.response?.data?.error || 'Could not open notebook') }
  }

  const shown = (listing?.entries ?? []).filter((e) => !filter || e.name.toLowerCase().includes(filter.toLowerCase()))

  const onRowClick = (ev: React.MouseEvent, e: Entry, idx: number) => {
    if (ev.metaKey || ev.ctrlKey) {
      setSelected((s) => { const n = new Set(s); n.has(e.path) ? n.delete(e.path) : n.add(e.path); return n })
      anchor.current = idx
    } else if (ev.shiftKey && anchor.current >= 0) {
      const [a, b] = [anchor.current, idx].sort((x, y) => x - y)
      setSelected(new Set(shown.slice(a, b + 1).map((x) => x.path)))
    } else {
      setSelected(new Set([e.path])); anchor.current = idx
    }
  }
  const open = (e: Entry) => {
    if (e.is_dir) load(e.path)
    else if (e.name.endsWith('.ipynb')) openNotebook(e)
    // Parquet/CSV/JSON/Arrow open straight in the Data Explorer (read via DuckDB).
    else if (isDataFile(e.name)) useExplorerRequest.getState().open({ source: { kind: 'file', path: e.path } }, e.name)
  }

  // ── file ops ──
  const newFile = async () => { const name = window.prompt('New file name'); if (name && cwd) { await axios.post('/api/fs/new-file', { path: cwd, name }).catch(er => setError(er?.response?.data?.error)); reload() } }
  const newFolder = async () => { const name = window.prompt('New folder name'); if (name && cwd) { await axios.post('/api/fs/mkdir', { path: cwd, name }).catch(er => setError(er?.response?.data?.error)); reload() } }
  const rename = async (e: Entry) => { const new_name = window.prompt('Rename to', e.name); if (new_name && new_name !== e.name) { await axios.post('/api/fs/rename', { path: e.path, new_name }).catch(er => setError(er?.response?.data?.error)); reload() } }
  const delPaths = async (paths: string[]) => { if (!paths.length || !window.confirm(`Delete ${paths.length} item(s)?`)) return; for (const p of paths) await axios.post('/api/fs/delete', { path: p }).catch(() => {}); reload() }
  const cut = () => selected.size && setClip({ paths: [...selected], op: 'cut' })
  const copy = () => selected.size && setClip({ paths: [...selected], op: 'copy' })
  const paste = async (toDir = cwd) => {
    if (!clip || !toDir) return
    for (const from of clip.paths) await axios.post(`/api/fs/${clip.op === 'cut' ? 'move' : 'copy'}`, { from, to_dir: toDir }).catch((er) => setError(er?.response?.data?.error))
    if (clip.op === 'cut') setClip(null)
    reload()
  }

  // ── drag & drop ──
  const onEntryDragStart = (ev: React.DragEvent, e: Entry) => {
    const paths = selected.has(e.path) ? [...selected] : [e.path]
    ev.dataTransfer.setData('application/x-pn-paths', JSON.stringify(paths))
  }
  const onFolderDrop = async (ev: React.DragEvent, folder: Entry) => {
    ev.preventDefault(); ev.stopPropagation()
    const raw = ev.dataTransfer.getData('application/x-pn-paths')
    if (raw) { for (const from of JSON.parse(raw)) await axios.post('/api/fs/move', { from, to_dir: folder.path }).catch((er) => setError(er?.response?.data?.error)); reload() }
  }
  const uploadFiles = async (files: FileList) => {
    if (!cwd) return
    for (const file of Array.from(files)) {
      const buf = new Uint8Array(await file.arrayBuffer())
      let bin = ''; for (let i = 0; i < buf.length; i++) bin += String.fromCharCode(buf[i])
      await axios.post('/api/fs/upload', { path: `${cwd}/${file.name}`, content_base64: btoa(bin) }).catch((er) => setError(er?.response?.data?.error || 'upload failed'))
    }
    reload()
  }
  const onContainerDrop = (ev: React.DragEvent) => {
    ev.preventDefault(); setDropActive(false)
    if (ev.dataTransfer.files?.length) uploadFiles(ev.dataTransfer.files)
  }
  const toggleHidden = () => { const h = !showHidden; setShowHidden(h); load(cwd, h) }
  const ib = 'p-1 rounded pn-hover'

  return (
    <div className="flex-1 flex flex-col overflow-hidden"
      onDragOver={(e) => { if (e.dataTransfer.types.includes('Files')) { e.preventDefault(); setDropActive(true) } }}
      onDragLeave={() => setDropActive(false)} onDrop={onContainerDrop}>
      <div className="flex items-center gap-1 px-2 py-1 border-b pn-bd text-[11px] pn-muted">
        <button onClick={() => listing?.parent && load(listing.parent)} disabled={!listing?.parent} title="Up" className={`${ib} disabled:opacity-30`}><ChevronUp size={13} /></button>
        <span className="truncate flex-1" title={cwd}>{cwd ?? '…'}</span>
        <button onClick={reload} title="Refresh" className={ib}>{loading ? <Loader2 size={12} className="animate-spin" /> : <RefreshCw size={12} />}</button>
      </div>
      <div className="flex items-center gap-0.5 px-2 py-1 border-b pn-bd pn-muted">
        <button onClick={newFile} title="New file" className={ib}><FilePlus size={13} /></button>
        <button onClick={newFolder} title="New folder" className={ib}><FolderPlus size={13} /></button>
        <button onClick={() => uploadRef.current?.click()} title="Upload file(s)" className={ib}><Upload size={13} /></button>
        <input ref={uploadRef} type="file" multiple className="hidden" onChange={(e) => e.target.files && uploadFiles(e.target.files).then(() => { if (uploadRef.current) uploadRef.current.value = '' })} />
        <span className="w-px h-4 bg-white/10 mx-0.5" />
        <button onClick={cut} disabled={!selected.size} title="Cut" className={`${ib} disabled:opacity-30`}><Scissors size={13} /></button>
        <button onClick={copy} disabled={!selected.size} title="Copy" className={`${ib} disabled:opacity-30`}><Copy size={13} /></button>
        <button onClick={() => paste()} disabled={!clip} title="Paste" className={`${ib} disabled:opacity-30 ${clip ? 'text-blue-400' : ''}`}><ClipboardPaste size={13} /></button>
        <button onClick={() => delPaths([...selected])} disabled={!selected.size} title="Delete" className={`${ib} disabled:opacity-30 text-rose-400`}><Trash2 size={13} /></button>
        <div className="flex-1" />
        <button onClick={toggleHidden} title={showHidden ? 'Hide hidden' : 'Show hidden'} className={`${ib} ${showHidden ? 'text-blue-400' : ''}`}>{showHidden ? <Eye size={13} /> : <EyeOff size={13} />}</button>
      </div>
      <div className="px-2 py-1 border-b pn-bd">
        <input value={filter} onChange={(e) => setFilter(e.target.value)} placeholder="Filter…" className="w-full px-2 py-1 rounded bg-white/5 border pn-bd pn-text text-[12px] outline-none focus:border-blue-500" />
      </div>

      {error && <div className="px-3 py-2 text-[12px] text-rose-400">{error}</div>}

      <div className={`flex-1 overflow-y-auto py-1 ${dropActive ? 'ring-2 ring-blue-500/60 ring-inset' : ''}`}>
        {shown.map((e, idx) => {
          const isNb = e.name.endsWith('.ipynb')
          const Icon = e.is_dir ? Folder : isNb ? FileCode : FileText
          const code = git[e.name]
          return (
            <div key={e.path} draggable onDragStart={(ev) => onEntryDragStart(ev, e)}
              onDragOver={(ev) => e.is_dir && ev.preventDefault()} onDrop={(ev) => e.is_dir && onFolderDrop(ev, e)}
              onClick={(ev) => onRowClick(ev, e, idx)} onDoubleClick={() => open(e)}
              className={`group flex items-center gap-2 px-3 py-1 text-[13px] cursor-pointer ${selected.has(e.path) ? 'bg-blue-500/25' : 'hover:bg-blue-500/10'}`}>
              <Icon size={14} className={e.is_dir ? 'text-blue-300' : isNb ? 'text-yellow-400' : 'pn-faint'} />
              <span className={`truncate flex-1 ${e.is_dir || isNb ? 'pn-text' : 'pn-muted'}`}>{e.name}</span>
              {code && <span className={`text-[11px] font-bold ${gitColor[code] || 'pn-faint'}`} title="git status">{code === '?' ? 'U' : code}</span>}
              <button onClick={(ev) => { ev.stopPropagation(); rename(e) }} title="Rename" className="opacity-0 group-hover:opacity-100 pn-faint hover:pn-text"><Pencil size={12} /></button>
              <button onClick={(ev) => { ev.stopPropagation(); delPaths([e.path]) }} title="Delete" className="opacity-0 group-hover:opacity-100 text-rose-400"><Trash2 size={12} /></button>
            </div>
          )
        })}
        {listing && shown.length === 0 && !error && <div className="px-3 py-2 text-[12px] pn-faint">{filter ? 'No matches.' : 'Empty folder.'}</div>}
        {dropActive && <div className="px-3 py-4 text-center text-[12px] text-blue-300">Drop files to upload</div>}
      </div>
    </div>
  )
}
