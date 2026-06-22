import { useEffect, useState } from 'react'
import './App.css'
import './styles/animations.css'
import './styles/components.css'
import {
  Files,
  Search as SearchIcon,
  Sparkles,
  TerminalSquare,
  Settings as SettingsIcon,
  CircleUserRound,
  Plus,
  BookOpen,
  FolderOpen,
  FileUp,
  Save,
  Palette,
  Play,
  PanelLeft,
  PanelRight,
  PanelBottom,
  Command as CommandIcon,
} from 'lucide-react'
import { Briefcase, GitBranch, Rocket, Database } from 'lucide-react'
import Notebook from './components/Notebook'
import JobsPanel from './components/JobsPanel'
import GitPanel from './components/GitPanel'
import DeployPanel from './components/DeployPanel'
import DataPanel from './components/DataPanel'
import FileExplorer from './components/FileExplorer'
import BottomPanel from './components/BottomPanel'
import AgentPanel from './components/AgentPanel'
import MenuBar from './components/MenuBar'
import UnifiedSearch from './components/UnifiedSearch'
import CommandPalette, { type Command } from './components/CommandPalette'
import SettingsModal from './components/SettingsModal'
import { useNotebookStore } from './hooks/useNotebook'
import { useWorkspace, openNotebookFile, saveJsonAs } from './hooks/useWorkspace'

function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark')
  const [panels, setPanels] = useState({ files: true, terminal: true, ai: true })
  const [searchOpen, setSearchOpen] = useState(false)
  const [jobsOpen, setJobsOpen] = useState(false)
  const [gitOpen, setGitOpen] = useState(false)
  const [gitFocus, setGitFocus] = useState<'commit' | 'clone' | undefined>(undefined)
  const [jobsCreate, setJobsCreate] = useState(false)
  const [deployOpen, setDeployOpen] = useState(false)
  const [dataOpen, setDataOpen] = useState(false)
  const [railMenu, setRailMenu] = useState<null | 'settings' | 'accounts'>(null)
  const [overlay, setOverlay] = useState<null | 'command' | 'settings' | 'theme'>(null)
  const { currentNotebookId, notebooks, currentNotebook, createNotebook, addCell, executeCell } = useNotebookStore()
  const openFolder = useWorkspace((s) => s.openFolder)

  useEffect(() => {
    const close = () => setRailMenu(null)
    window.addEventListener('click', close)
    return () => window.removeEventListener('click', close)
  }, [])

  useEffect(() => {
    const saved = (localStorage.getItem('pn-theme') as 'light' | 'dark') || 'dark'
    setTheme(saved)
    document.documentElement.classList.toggle('dark', saved === 'dark')
    const fs = localStorage.getItem('pn-code-size')
    if (fs) document.documentElement.style.setProperty('--pn-code-size', `${fs}px`)
  }, [])

  // Responsive layout: auto-collapse side panels when the window is too narrow
  // to show them comfortably, and restore them when it widens again. Acts only
  // on breakpoint transitions so it doesn't fight manual toggles at a given size.
  useEffect(() => {
    const NARROW = 1000 // below this, hide both side panels
    const TIGHT = 700 //  below this, also hide the bottom panel
    let prev = { narrow: window.innerWidth < NARROW, tight: window.innerWidth < TIGHT }
    if (prev.narrow || prev.tight) {
      setPanels((p) => ({
        files: prev.narrow ? false : p.files,
        ai: prev.narrow ? false : p.ai,
        terminal: prev.tight ? false : p.terminal,
      }))
    }
    const onResize = () => {
      const narrow = window.innerWidth < NARROW
      const tight = window.innerWidth < TIGHT
      if (narrow !== prev.narrow || tight !== prev.tight) {
        prev = { narrow, tight }
        setPanels({ files: !narrow, ai: !narrow, terminal: !tight })
      }
    }
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  // global shortcuts: ⇧⌘P palette · ⌘K search · ⌘⇧⏎ run all · ⌘, settings
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey
      if (mod && e.shiftKey && e.key.toLowerCase() === 'p') {
        e.preventDefault()
        setOverlay('command')
      } else if (mod && !e.shiftKey && e.key.toLowerCase() === 'k') {
        // ⌘K opens global search (Monaco intercepts it for in-cell AI when an
        // editor is focused, so this fires only outside a cell).
        e.preventDefault()
        setSearchOpen(true)
      } else if (mod && e.shiftKey && e.key === 'Enter') {
        e.preventDefault()
        const st: any = useNotebookStore.getState()
        const nb = st.currentNotebook
        if (nb) {
          ;(async () => {
            for (let i = 0; i < nb.cells.length; i++) {
              if (nb.cells[i].cell_type === 'code') await st.executeCell(i)
            }
          })()
        }
      } else if (mod && e.key === ',') {
        e.preventDefault()
        setOverlay('settings')
      } else if (mod && e.key.toLowerCase() === 'n') {
        e.preventDefault()
        newNotebook()
      } else if (mod && e.key.toLowerCase() === 'o') {
        e.preventDefault()
        openFile()
      } else if (mod && e.key.toLowerCase() === 's') {
        e.preventDefault()
        saveCurrent()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])

  const applyTheme = (t: 'light' | 'dark') => {
    setTheme(t)
    document.documentElement.classList.toggle('dark', t === 'dark')
    localStorage.setItem('pn-theme', t)
  }
  const toggleTheme = () => applyTheme(theme === 'light' ? 'dark' : 'light')

  const togglePanel = (p: 'files' | 'terminal' | 'ai') => setPanels((s) => ({ ...s, [p]: !s[p] }))

  const newNotebook = () => {
    const n = window.prompt('Notebook name', 'Untitled')
    if (n) createNotebook(n)
  }

  const openFile = async () => {
    const res = await openNotebookFile()
    if (!res) return
    const cells = (res.data.cells ?? []).map((c: any, i: number) => ({
      id: `${res.name}-${i}`,
      cell_type: c.cell_type ?? 'code',
      source: c.source ?? [],
      outputs: c.outputs ?? [],
      metadata: c.metadata ?? {},
    }))
    const nb = { id: `local-${res.name}`, name: res.name.replace(/\.ipynb$/, ''), cells, metadata: res.data.metadata ?? {} }
    useNotebookStore.setState((s: any) => ({
      notebooks: [...s.notebooks.filter((n: any) => n.id !== nb.id), nb],
      currentNotebookId: nb.id,
      currentNotebook: nb,
    }))
  }

  const saveCurrent = async () => {
    // Read fresh state — this is also called from the ⌘S key handler whose
    // closure would otherwise capture a stale `currentNotebook`.
    const nb = (useNotebookStore.getState() as any).currentNotebook
    if (!nb) return
    const ipynb = {
      cells: nb.cells.map((c: any) => ({
        cell_type: c.cell_type,
        source: Array.isArray(c.source) ? c.source : [c.source],
        outputs: c.outputs ?? [],
        metadata: c.metadata ?? {},
        ...(c.cell_type === 'code' ? { execution_count: null } : {}),
      })),
      metadata: nb.metadata ?? {},
      nbformat: 4,
      nbformat_minor: 5,
    }
    await saveJsonAs(`${nb.name}.ipynb`, ipynb)
  }

  const runAll = async () => {
    if (!currentNotebook) return
    for (let i = 0; i < currentNotebook.cells.length; i++) {
      if (currentNotebook.cells[i].cell_type === 'code') await executeCell(i)
    }
  }

  // ── Command Palette command set ──
  const commands: Command[] = [
    { id: 'new-nb', category: 'File', title: 'New Notebook', shortcut: '⌘N', icon: <Plus size={14} />, run: newNotebook },
    { id: 'open-folder', category: 'File', title: 'Open Folder…', icon: <FolderOpen size={14} />, run: openFolder },
    { id: 'open-file', category: 'File', title: 'Open File…', shortcut: '⌘O', icon: <FileUp size={14} />, keywords: 'notebook ipynb', run: openFile },
    { id: 'save', category: 'File', title: 'Save Notebook', shortcut: '⌘S', icon: <Save size={14} />, run: saveCurrent },
    { id: 'add-cell', category: 'Edit', title: 'Add Code Cell', icon: <Plus size={14} />, run: () => addCell('code') },
    { id: 'run-all', category: 'Run', title: 'Run All Cells', shortcut: '⌘⇧⏎', icon: <Play size={14} />, run: runAll },
    { id: 'jobs', category: 'Run', title: 'Jobs…', icon: <Briefcase size={14} />, keywords: 'schedule cron airflow', run: () => setJobsOpen(true) },
    { id: 'git', category: 'Run', title: 'Source Control…', icon: <GitBranch size={14} />, keywords: 'git github commit push pull clone', run: () => setGitOpen(true) },
    { id: 'deploy', category: 'Run', title: 'Deploy to Cloud…', icon: <Rocket size={14} />, keywords: 'docker kubernetes k8s fly deploy cloud', run: () => setDeployOpen(true) },
    { id: 'data', category: 'Run', title: 'Data & SQL…', icon: <Database size={14} />, keywords: 'database sql query warehouse connection', run: () => setDataOpen(true) },
    { id: 'toggle-files', category: 'View', title: 'Toggle File Explorer', icon: <PanelLeft size={14} />, run: () => togglePanel('files') },
    { id: 'toggle-term', category: 'View', title: 'Toggle Terminal', icon: <PanelBottom size={14} />, run: () => togglePanel('terminal') },
    { id: 'toggle-ai', category: 'View', title: 'Toggle AI Assistant', icon: <PanelRight size={14} />, run: () => togglePanel('ai') },
    { id: 'theme-dark', category: 'Preferences', title: 'Color Theme: Dark', icon: <Palette size={14} />, keywords: 'color palette appearance', run: () => applyTheme('dark') },
    { id: 'theme-light', category: 'Preferences', title: 'Color Theme: Light', icon: <Palette size={14} />, keywords: 'color palette appearance', run: () => applyTheme('light') },
    { id: 'settings', category: 'Preferences', title: 'Open Settings', shortcut: '⌘,', icon: <SettingsIcon size={14} />, run: () => setOverlay('settings') },
  ]

  const themeCommands: Command[] = [
    { id: 't-dark', title: 'Dark (Claude warm)', category: 'Color Theme', run: () => applyTheme('dark') },
    { id: 't-light', title: 'Light', category: 'Color Theme', run: () => applyTheme('light') },
  ]

  const railBtn = (active: boolean, onClick: () => void, title: string, Icon: any, stop = false) => (
    <button
      onClick={(e) => {
        if (stop) e.stopPropagation()
        onClick()
      }}
      title={title}
      className={`relative w-12 h-12 flex items-center justify-center transition-colors ${
        active ? 'pn-text' : 'pn-faint hover:pn-text'
      }`}
    >
      {active && <span className="absolute left-0 top-2 bottom-2 w-[3px] rounded-r prism-bar shadow-[0_0_10px_rgba(167,139,250,0.7)]" />}
      <Icon size={20} />
    </button>
  )

  return (
    <div className="h-screen w-screen flex flex-col pn-app overflow-hidden">
      <MenuBar
        theme={theme}
        onToggleTheme={toggleTheme}
        panels={panels}
        onTogglePanel={togglePanel}
        onOpenSearch={() => setSearchOpen(true)}
        onOpenJobs={(create) => { setJobsCreate(!!create); setJobsOpen(true) }}
        onOpenGit={(focus) => { setGitFocus(focus); setGitOpen(true) }}
        onOpenCommandPalette={() => setOverlay('command')}
      />

      <div className="flex-1 flex overflow-hidden">
        {/* Activity rail */}
        <div className="w-12 shrink-0 pn-bar border-r pn-bd flex flex-col items-center py-1">
          {railBtn(panels.files, () => togglePanel('files'), 'Explorer', Files)}
          {railBtn(searchOpen, () => setSearchOpen(true), 'Search', SearchIcon)}
          {railBtn(panels.terminal, () => togglePanel('terminal'), 'Terminal', TerminalSquare)}
          {railBtn(panels.ai, () => togglePanel('ai'), 'AI Assistant', Sparkles)}
          {railBtn(dataOpen, () => setDataOpen((v) => !v), 'Data & SQL', Database)}
          {railBtn(gitOpen, () => setGitOpen((v) => !v), 'Source Control', GitBranch)}
          {railBtn(jobsOpen, () => setJobsOpen((v) => !v), 'Jobs', Briefcase)}
          {railBtn(deployOpen, () => setDeployOpen((v) => !v), 'Deploy to Cloud', Rocket)}
          <div className="flex-1" />
          {railBtn(railMenu === 'accounts', () => setRailMenu(railMenu === 'accounts' ? null : 'accounts'), 'Accounts', CircleUserRound, true)}
          {railBtn(railMenu === 'settings', () => setRailMenu(railMenu === 'settings' ? null : 'settings'), 'Manage', SettingsIcon, true)}
        </div>

        {/* VS Code-style rail popups (Settings gear / Accounts) */}
        {railMenu && (
          <div
            className="fixed left-12 bottom-7 z-50 min-w-[230px] pn-solid-bg border pn-bd pn-text rounded-lg shadow-2xl shadow-black/50 py-1 text-[13px]"
            onClick={(e) => e.stopPropagation()}
          >
            {railMenu === 'settings'
              ? [
                  { label: 'Command Palette…', shortcut: '⇧⌘P', action: () => setOverlay('command') },
                  { label: 'Settings', shortcut: '⌘,', action: () => setOverlay('settings') },
                  { label: 'Color Theme…', action: () => setOverlay('theme') },
                  { label: 'Extensions', action: () => alert('Extensions — coming soon'), sep: true },
                  { label: 'About PrismNote', action: () => alert('PrismNote — a modern, open-source data-science notebook.\nRust engine · React UI.') },
                ].map((it: any, i) => (
                  <div key={i}>
                    <button
                      onClick={() => { it.action?.(); setRailMenu(null) }}
                      className="w-full flex items-center justify-between px-3 py-1.5 rounded-md hover:bg-violet-600 hover:text-white"
                    >
                      <span>{it.label}</span>
                      {it.shortcut && <span className="text-xs pn-faint ml-6">{it.shortcut}</span>}
                    </button>
                    {it.sep && <div className="my-1 border-t pn-bd" />}
                  </div>
                ))
              : (
                <>
                  <div className="px-3 py-2 flex items-center gap-2 border-b pn-bd">
                    <div className="w-8 h-8 rounded-full prism-bg flex items-center justify-center text-white text-sm font-semibold">G</div>
                    <div className="leading-tight">
                      <div className="pn-text text-[13px] font-medium">Guest</div>
                      <div className="pn-faint text-[11px]">Not signed in</div>
                    </div>
                  </div>
                  <button onClick={() => { alert('Sign in — coming soon'); setRailMenu(null) }} className="w-full text-left px-3 py-1.5 rounded-md hover:bg-violet-600 hover:text-white">
                    Sign in to sync settings…
                  </button>
                  <button onClick={() => { alert('Manage account — coming soon'); setRailMenu(null) }} className="w-full text-left px-3 py-1.5 rounded-md hover:bg-violet-600 hover:text-white">
                    Manage account
                  </button>
                </>
              )}
          </div>
        )}

        {/* Left: file explorer */}
        {panels.files && <FileExplorer />}

        {/* Center: code panel + bottom panel */}
        <div className="flex-1 flex flex-col overflow-hidden min-w-0 relative">
          {jobsOpen && <JobsPanel onClose={() => setJobsOpen(false)} initialCreate={jobsCreate} />}
          {gitOpen && <GitPanel onClose={() => setGitOpen(false)} initialFocus={gitFocus} />}
          {deployOpen && <DeployPanel onClose={() => setDeployOpen(false)} />}
          {dataOpen && <DataPanel onClose={() => setDataOpen(false)} />}
          <div className="flex-1 overflow-hidden">
            {currentNotebookId ? (
              <Notebook />
            ) : (
              <div className="h-full flex items-center justify-center">
                <div className="text-center max-w-[30rem]">
                  <div className="mx-auto mb-5 w-16 h-16 rounded-2xl prism-bg rotate-45 flex items-center justify-center shadow-[0_8px_40px_-6px_rgba(139,92,246,0.7)]">
                    <BookOpen size={28} className="-rotate-45 text-white" />
                  </div>
                  <h1 className="text-4xl font-bold mb-2 tracking-tight">
                    <span className="prism-text">Prism</span><span className="pn-text">Note</span>
                  </h1>
                  <p className="pn-muted mb-7 text-[15px]">A fast, modern, open-source data-science notebook.</p>
                  <button
                    onClick={newNotebook}
                    className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl prism-bg text-white font-medium glow-accent hover:brightness-110 transition"
                  >
                    <Plus size={18} /> New Notebook
                  </button>
                  {notebooks.length > 0 && (
                    <p className="mt-4 text-sm pn-faint">…or pick one from the Explorer on the left.</p>
                  )}
                </div>
              </div>
            )}
          </div>
          {panels.terminal && <BottomPanel onClose={() => togglePanel('terminal')} />}
        </div>

        {/* Right: Cline-style agent (Ollama) */}
        {panels.ai && <AgentPanel onClose={() => togglePanel('ai')} />}
      </div>

      {/* Status bar */}
      <div className="h-6 shrink-0 prism-bg text-white/95 text-[11px] flex items-center px-3 gap-4 select-none">
        <span className="flex items-center gap-1.5"><span className="w-1.5 h-1.5 rounded-full bg-emerald-300 shadow-[0_0_6px_#6ee7b7]" /> Python 3.11</span>
        <span className="opacity-90">{currentNotebook ? `${currentNotebook.name} · ${currentNotebook.cells.length} cells` : 'No notebook'}</span>
        <div className="flex-1" />
        <span className="opacity-90">Kernel: idle</span>
        <span className="opacity-90">UTF-8</span>
        <span className="font-medium">◆ PrismNote</span>
      </div>

      {searchOpen && <UnifiedSearch />}
      {overlay === 'command' && (
        <CommandPalette commands={commands} onClose={() => setOverlay(null)} placeholder="Type a command…" />
      )}
      {overlay === 'theme' && (
        <CommandPalette commands={themeCommands} onClose={() => setOverlay(null)} placeholder="Select Color Theme…" />
      )}
      {overlay === 'settings' && (
        <SettingsModal onClose={() => setOverlay(null)} theme={theme} setTheme={applyTheme} panels={panels} togglePanel={togglePanel} />
      )}
    </div>
  )
}

export default App
