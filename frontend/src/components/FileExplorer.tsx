import { useEffect, useRef, useState } from 'react'
import {
  ChevronRight,
  FilePlus,
  FolderPlus,
  RefreshCw,
  ChevronsDownUp,
  FolderOpen,
  Folder,
  File,
  FileCode,
  FileText,
  FileSpreadsheet,
  FileImage,
  Plus,
  Minus,
  ChevronDown,
} from 'lucide-react'
import { useWorkspace } from '../hooks/useWorkspace'
import { useNotebookStore } from '../hooks/useNotebook'
import { useFontSize } from '../hooks/useFontSize'
import ServerExplorer from './ServerExplorer'

interface Entry {
  name: string
  kind: 'file' | 'directory'
  handle: any
}

// ── file-type icon mapping (VS Code-ish) ──
function iconFor(name: string) {
  const ext = name.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'py':
      return <FileCode size={15} className="text-yellow-400 shrink-0" />
    case 'ipynb':
      return <FileCode size={15} className="text-orange-400 shrink-0" />
    case 'csv':
    case 'tsv':
    case 'parquet':
    case 'xlsx':
      return <FileSpreadsheet size={15} className="text-emerald-400 shrink-0" />
    case 'json':
      return <FileCode size={15} className="text-amber-300 shrink-0" />
    case 'md':
    case 'txt':
      return <FileText size={15} className="text-sky-300 shrink-0" />
    case 'png':
    case 'jpg':
    case 'jpeg':
    case 'svg':
    case 'gif':
      return <FileImage size={15} className="text-sky-300 shrink-0" />
    default:
      return <File size={15} className="pn-faint shrink-0" />
  }
}

async function readDir(handle: any): Promise<Entry[]> {
  const items: Entry[] = []
  for await (const [name, h] of handle.entries()) {
    if (name.startsWith('.')) continue // hide dotfiles, VS Code default-ish
    items.push({ name, kind: h.kind, handle: h })
  }
  items.sort((a, b) => (a.kind === b.kind ? a.name.localeCompare(b.name) : a.kind === 'directory' ? -1 : 1))
  return items
}

// load an opened .ipynb directly into the notebook store (no backend needed)
function loadIpynb(name: string, data: any) {
  const cells = (data.cells ?? []).map((c: any, i: number) => ({
    id: `${name}-${i}`,
    cell_type: c.cell_type ?? 'code',
    source: c.source ?? [],
    outputs: c.outputs ?? [],
    metadata: c.metadata ?? {},
  }))
  const nb = { id: `local-${name}`, name: name.replace(/\.ipynb$/, ''), cells, metadata: data.metadata ?? {} }
  useNotebookStore.setState((s: any) => ({
    notebooks: [...s.notebooks.filter((n: any) => n.id !== nb.id), nb],
    currentNotebookId: nb.id,
    currentNotebook: nb,
  }))
}

type MenuState = { x: number; y: number; entry: Entry; parent: any } | null

export default function FileExplorer() {
  const { rootHandle, rootName, rev, openFolder, refresh, supported } = useWorkspace()
  const [menu, setMenu] = useState<MenuState>(null)
  // Server-backed browse: the reliable path when the File System Access API
  // isn't available (e.g. embedded/automated browsers), or on user request.
  const [serverMode, setServerMode] = useState(!supported)
  const [selected, setSelected] = useState<string>('')
  const [collapsed, setCollapsed] = useState(false)
  const { size: filesFont, inc, dec } = useFontSize('pn-files-font', 13)

  useEffect(() => {
    const close = () => setMenu(null)
    window.addEventListener('click', close)
    return () => window.removeEventListener('click', close)
  }, [])

  const openFile = async (e: Entry) => {
    setSelected(e.name)
    if (e.name.endsWith('.ipynb')) {
      const file = await e.handle.getFile()
      try {
        loadIpynb(e.name, JSON.parse(await file.text()))
      } catch {
        /* not a valid notebook */
      }
    }
  }

  const newEntry = async (dir: any, kind: 'file' | 'directory') => {
    const name = window.prompt(kind === 'file' ? 'New file name' : 'New folder name')
    if (!name) return
    try {
      if (kind === 'file') await dir.getFileHandle(name, { create: true })
      else await dir.getDirectoryHandle(name, { create: true })
      refresh()
    } catch (err) {
      alert('Could not create: ' + err)
    }
  }

  const del = async (parent: any, e: Entry) => {
    if (!window.confirm(`Delete "${e.name}"?`)) return
    try {
      await parent.removeEntry(e.name, { recursive: e.kind === 'directory' })
      refresh()
    } catch (err) {
      alert('Could not delete: ' + err)
    }
  }

  const rename = async (_parent: any, e: Entry) => {
    const name = window.prompt('Rename to', e.name)
    if (!name || name === e.name) return
    try {
      if (typeof e.handle.move === 'function') await e.handle.move(name)
      else alert('Rename needs a newer Chrome (FileSystemHandle.move).')
      refresh()
    } catch (err) {
      alert('Could not rename: ' + err)
    }
  }

  return (
    <aside className="w-64 shrink-0 pn-surface border-r pn-bd flex flex-col overflow-hidden select-none">
      <div className="h-8 flex items-center px-2 text-[11px] font-semibold uppercase tracking-wider pn-muted">
        <button onClick={() => setCollapsed((c) => !c)} className="p-0.5 rounded pn-hover" title={collapsed ? 'Expand' : 'Collapse'}>
          <ChevronDown size={13} className={collapsed ? '-rotate-90 transition-transform' : 'transition-transform'} />
        </button>
        <span className="ml-1">Files</span>
        <div className="flex-1" />
        <button onClick={dec} title="Decrease font" className="p-0.5 rounded pn-hover"><Minus size={11} /></button>
        <span className="tabular-nums text-[10px] w-4 text-center normal-case">{filesFont}</span>
        <button onClick={inc} title="Increase font" className="p-0.5 rounded pn-hover"><Plus size={11} /></button>
      </div>

      {collapsed ? null : (
      <div className="flex-1 flex flex-col overflow-hidden" style={{ fontSize: filesFont }}>
      {serverMode && !rootHandle ? (
        <ServerExplorer />
      ) : !rootHandle ? (
        // VS Code-style empty state
        <div className="px-4 py-3">
          <p className="text-[13px] pn-muted leading-relaxed mb-3">You have not yet opened a folder.</p>
          <button
            onClick={openFolder}
            className="w-full px-3 py-1.5 rounded-md prism-bg text-white text-[13px] font-medium hover:brightness-110"
          >
            Open Folder
          </button>
          <button
            onClick={() => setServerMode(true)}
            className="w-full mt-2 px-3 py-1.5 rounded-md bg-white/5 hover:bg-white/10 pn-text text-[13px]"
            title="Browse the machine running PrismNote (works in any browser)"
          >
            Browse server files
          </button>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto" onContextMenu={(e) => e.preventDefault()}>
          {/* workspace section header with hover toolbar */}
          <Section
            title={rootName}
            dirHandle={rootHandle}
            onNewFile={() => newEntry(rootHandle, 'file')}
            onNewFolder={() => newEntry(rootHandle, 'directory')}
            onRefresh={refresh}
          >
            <Dir
              key={rev}
              dirHandle={rootHandle}
              parent={null}
              depth={0}
              selected={selected}
              onOpenFile={openFile}
              onMenu={setMenu}
            />
          </Section>
        </div>
      )}
      </div>
      )}

      {/* context menu */}
      {menu && (
        <div
          style={{ left: menu.x, top: menu.y }}
          className="fixed z-50 min-w-[170px] pn-solid-bg border pn-bd pn-text rounded-lg shadow-2xl shadow-black/40 py-1 text-[13px]"
          onClick={(e) => e.stopPropagation()}
        >
          {menu.entry.kind === 'directory' && (
            <>
              <MenuRow label="New File" onClick={() => { newEntry(menu.entry.handle, 'file'); setMenu(null) }} />
              <MenuRow label="New Folder" onClick={() => { newEntry(menu.entry.handle, 'directory'); setMenu(null) }} />
              <div className="my-1 border-t pn-bd" />
            </>
          )}
          {menu.entry.kind === 'file' && (
            <MenuRow label="Open" onClick={() => { openFile(menu.entry); setMenu(null) }} />
          )}
          <MenuRow label="Rename…" onClick={() => { rename(menu.parent ?? rootHandle, menu.entry); setMenu(null) }} />
          <MenuRow label="Delete" danger onClick={() => { del(menu.parent ?? rootHandle, menu.entry); setMenu(null) }} />
        </div>
      )}
    </aside>
  )
}

function MenuRow({ label, onClick, danger }: { label: string; onClick: () => void; danger?: boolean }) {
  return (
    <button
      onClick={onClick}
      className={`w-full text-left px-3 py-1.5 rounded-md hover:bg-blue-600 hover:text-white ${danger ? 'text-red-400' : ''}`}
    >
      {label}
    </button>
  )
}

function Section({
  title,
  onNewFile,
  onNewFolder,
  onRefresh,
  children,
}: {
  title: string
  dirHandle: any
  onNewFile: () => void
  onNewFolder: () => void
  onRefresh: () => void
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(true)
  return (
    <div className="group/section">
      <div className="flex items-center justify-between pr-1 h-6 hover:bg-[var(--pn-hover)]">
        <button onClick={() => setOpen(!open)} className="flex items-center gap-1 px-1 text-[11px] font-bold uppercase tracking-wide pn-text">
          <ChevronRight size={14} className={`transition-transform ${open ? 'rotate-90' : ''}`} />
          {title}
        </button>
        <div className="flex items-center gap-0.5 opacity-0 group-hover/section:opacity-100 pn-muted">
          <button onClick={onNewFile} title="New File" className="p-1 rounded pn-hover hover:pn-text"><FilePlus size={14} /></button>
          <button onClick={onNewFolder} title="New Folder" className="p-1 rounded pn-hover hover:pn-text"><FolderPlus size={14} /></button>
          <button onClick={onRefresh} title="Refresh" className="p-1 rounded pn-hover hover:pn-text"><RefreshCw size={13} /></button>
          <button title="Collapse All" className="p-1 rounded pn-hover hover:pn-text"><ChevronsDownUp size={13} /></button>
        </div>
      </div>
      {open && children}
    </div>
  )
}

function Dir({
  dirHandle,
  depth,
  selected,
  onOpenFile,
  onMenu,
}: {
  dirHandle: any
  parent?: any
  depth: number
  selected: string
  onOpenFile: (e: Entry) => void
  onMenu: (m: MenuState) => void
}) {
  const [entries, setEntries] = useState<Entry[] | null>(null)
  const [loading, setLoading] = useState(false)
  const loadedRef = useRef(false)

  useEffect(() => {
    // root dir (depth 0) auto-loads
    if (depth === 0 && !loadedRef.current) {
      loadedRef.current = true
      setLoading(true)
      readDir(dirHandle).then((e) => {
        setEntries(e)
        setLoading(false)
      })
    }
  }, [dirHandle, depth])

  if (depth === 0) {
    return <>{loading ? <Loading depth={1} /> : entries?.map((e) => (
      <Node key={e.name} entry={e} parent={dirHandle} depth={1} selected={selected} onOpenFile={onOpenFile} onMenu={onMenu} />
    ))}</>
  }
  return null
}

function Node({
  entry,
  parent,
  depth,
  selected,
  onOpenFile,
  onMenu,
}: {
  entry: Entry
  parent: any
  depth: number
  selected: string
  onOpenFile: (e: Entry) => void
  onMenu: (m: MenuState) => void
}) {
  const [open, setOpen] = useState(false)
  const [entries, setEntries] = useState<Entry[] | null>(null)
  const [loading, setLoading] = useState(false)

  const toggle = async () => {
    if (entry.kind !== 'directory') return onOpenFile(entry)
    if (!open && entries === null) {
      setLoading(true)
      setEntries(await readDir(entry.handle))
      setLoading(false)
    }
    setOpen(!open)
  }

  const isSel = selected === entry.name
  const indent = depth * 12 + 6

  return (
    <div>
      <div
        onClick={toggle}
        onContextMenu={(e) => {
          e.preventDefault()
          e.stopPropagation()
          onMenu({ x: e.clientX, y: e.clientY, entry, parent })
        }}
        style={{ paddingLeft: indent }}
        className={`group flex items-center gap-1 h-[22px] pr-2 cursor-pointer text-[13px] ${
          isSel ? 'bg-[rgba(56,139,253,0.25)] pn-text' : 'pn-muted hover:bg-[var(--pn-hover)]'
        }`}
      >
        {entry.kind === 'directory' ? (
          <ChevronRight size={14} className={`shrink-0 transition-transform ${open ? 'rotate-90' : ''}`} />
        ) : (
          <span className="w-[14px] shrink-0" />
        )}
        {entry.kind === 'directory' ? (
          open ? <FolderOpen size={15} className="text-sky-400 shrink-0" /> : <Folder size={15} className="text-sky-400 shrink-0" />
        ) : (
          iconFor(entry.name)
        )}
        <span className="truncate">{entry.name}</span>
      </div>

      {/* children with indent guide */}
      {open && (
        <div className="border-l ml-[14px] pn-bd" style={{ marginLeft: indent + 1 }}>
          {loading ? (
            <Loading depth={depth + 1} />
          ) : (
            entries?.map((e) => (
              <Node key={e.name} entry={e} parent={entry.handle} depth={depth + 1} selected={selected} onOpenFile={onOpenFile} onMenu={onMenu} />
            ))
          )}
        </div>
      )}
    </div>
  )
}

function Loading({ depth }: { depth: number }) {
  return (
    <div style={{ paddingLeft: depth * 12 + 22 }} className="h-[22px] flex items-center text-[12px] pn-faint">
      loading…
    </div>
  )
}
