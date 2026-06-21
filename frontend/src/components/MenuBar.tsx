import { useEffect, useRef, useState } from 'react'
import { Sun, Moon } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'
import { useWorkspace, openNotebookFile, saveJsonAs } from '../hooks/useWorkspace'
import { restartKernel, interruptKernel } from '../api/kernel'

interface MenuBarProps {
  theme: 'light' | 'dark'
  onToggleTheme: () => void
  panels: { files: boolean; terminal: boolean; ai: boolean }
  onTogglePanel: (panel: 'files' | 'terminal' | 'ai') => void
  onOpenSearch?: () => void
  onOpenJobs?: () => void
  onOpenGit?: () => void
  onOpenCommandPalette?: () => void
}

interface MenuItem {
  label: string
  shortcut?: string
  action?: () => void
  checked?: boolean
  separatorAfter?: boolean
  disabled?: boolean
}

export default function MenuBar({ theme, onToggleTheme, panels, onTogglePanel, onOpenSearch, onOpenJobs, onOpenGit, onOpenCommandPalette }: MenuBarProps) {
  const [open, setOpen] = useState<string | null>(null)
  const barRef = useRef<HTMLDivElement>(null)
  const {
    currentNotebook,
    createNotebook,
    saveNotebook,
    deleteNotebook,
    addCell,
    updateCell,
    executeCell,
  } = useNotebookStore()

  const clearOutputs = () => {
    if (!currentNotebook) return
    currentNotebook.cells.forEach((_: any, i: number) => updateCell(i, { outputs: [], execution_count: null }))
  }

  useEffect(() => {
    const close = (e: MouseEvent) => {
      if (barRef.current && !barRef.current.contains(e.target as Node)) setOpen(null)
    }
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [])

  const openFolder = useWorkspace((s) => s.openFolder)

  const newNotebook = () => {
    const name = window.prompt('Notebook name', 'Untitled')
    if (name) createNotebook(name)
  }

  // Open a .ipynb from disk and load it directly into the store (no backend).
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

  // Save the current notebook to disk as .ipynb via the Save dialog.
  const save = async () => {
    if (saveNotebook) saveNotebook()
    if (!currentNotebook) return
    const ipynb = {
      cells: currentNotebook.cells.map((c: any) => ({
        cell_type: c.cell_type,
        source: Array.isArray(c.source) ? c.source : [c.source],
        outputs: c.outputs ?? [],
        metadata: c.metadata ?? {},
        ...(c.cell_type === 'code' ? { execution_count: null } : {}),
      })),
      metadata: currentNotebook.metadata ?? {},
      nbformat: 4,
      nbformat_minor: 5,
    }
    await saveJsonAs(`${currentNotebook.name}.ipynb`, ipynb)
  }

  const runAll = async () => {
    if (!currentNotebook) return
    for (let i = 0; i < currentNotebook.cells.length; i++) {
      if (currentNotebook.cells[i].cell_type === 'code') await executeCell(i)
    }
  }

  const menus: Record<string, MenuItem[]> = {
    File: [
      { label: 'New Notebook', shortcut: '⌘N', action: newNotebook },
      { label: 'Open File…', shortcut: '⌘O', action: openFile },
      { label: 'Open Folder…', action: openFolder, separatorAfter: true },
      { label: 'Save', shortcut: '⌘S', action: save, disabled: !currentNotebook },
      { label: 'Export as .ipynb…', action: save, disabled: !currentNotebook, separatorAfter: true },
      { label: 'Close Notebook', action: () => currentNotebook && deleteNotebook(currentNotebook.id), disabled: !currentNotebook },
    ],
    Edit: [
      { label: 'Add Code Cell', shortcut: '⌘⏎', action: () => addCell('code'), disabled: !currentNotebook },
      { label: 'Add Markdown Cell', action: () => addCell('markdown'), disabled: !currentNotebook, separatorAfter: true },
      { label: 'Find in Notebook…', shortcut: '⌘F', action: () => onOpenSearch?.() },
      { label: 'Command Palette…', shortcut: '⇧⌘P', action: () => onOpenCommandPalette?.() },
    ],
    View: [
      { label: 'File Explorer', checked: panels.files, action: () => onTogglePanel('files') },
      { label: 'Terminal & Console', checked: panels.terminal, action: () => onTogglePanel('terminal') },
      { label: 'AI Assistant', checked: panels.ai, action: () => onTogglePanel('ai') },
      { label: 'Search…', shortcut: '⌘K', action: () => onOpenSearch?.(), separatorAfter: true },
      { label: theme === 'dark' ? 'Light Theme' : 'Dark Theme', action: onToggleTheme },
    ],
    Run: [
      { label: 'Run All Cells', shortcut: '⌘⇧⏎', action: runAll, disabled: !currentNotebook },
      { label: 'Add & Run Cell', action: () => addCell('code'), disabled: !currentNotebook },
      { label: 'Clear All Outputs', action: clearOutputs, disabled: !currentNotebook, separatorAfter: true },
      { label: 'Interrupt Kernel', action: () => interruptKernel() },
      { label: 'Restart Kernel', action: () => { if (confirm('Restart the kernel? All variables will be cleared.')) restartKernel() } },
    ],
    Jobs: [
      { label: 'Open Jobs…', action: () => onOpenJobs?.() },
      { label: 'Run Current Notebook as Job…', action: () => onOpenJobs?.(), disabled: !currentNotebook },
    ],
    Git: [
      { label: 'Source Control…', action: () => onOpenGit?.() },
      { label: 'Commit & Push…', action: () => onOpenGit?.() },
      { label: 'Clone Repository…', action: () => onOpenGit?.() },
    ],
    Help: [
      { label: 'About PrismNote', action: () => window.alert('PrismNote — a modern, open-source data-science notebook.\nRust engine · React UI.') },
      { label: 'Documentation', action: () => window.open('https://github.com/Mullassery/prismnote#readme', '_blank') },
      { label: 'Keyboard Shortcuts', action: () => window.alert('⌘N New · ⌘O Open · ⌘S Save · ⌘K Search · ⇧⌘P Command Palette · ⌘⇧⏎ Run All · ⌘K (in cell) AI edit') },
    ],
  }

  return (
    <div
      ref={barRef}
      className="flex items-center h-9 px-2 pn-bar backdrop-blur border-b pn-bd text-sm pn-muted select-none relative z-50"
    >
      <span className="flex items-center gap-2 px-2 font-semibold tracking-tight">
        <span className="w-4 h-4 rounded-[5px] prism-bg rotate-45 shadow-[0_0_12px_rgba(139,92,246,0.6)]" />
        <span className="prism-text">PrismNote</span>
      </span>
      {Object.keys(menus).map((name) => (
        <div key={name} className="relative">
          <button
            onClick={() => setOpen(open === name ? null : name)}
            onMouseEnter={() => open && setOpen(name)}
            className={`px-3 h-9 rounded-md pn-hover ${open === name ? 'pn-text' : ''}`}
          >
            {name}
          </button>
          {open === name && (
            <div className="absolute left-0 top-9 min-w-[230px] pn-solid-bg border pn-bd pn-text rounded-xl shadow-2xl shadow-black/30 py-1.5 backdrop-blur-xl">
              {menus[name].map((item, i) => (
                <div key={i}>
                  <button
                    disabled={item.disabled}
                    onClick={() => {
                      item.action?.()
                      setOpen(null)
                    }}
                    className={`w-full flex items-center justify-between px-3 py-1.5 text-left rounded-md hover:bg-violet-600 hover:text-white ${
                      item.disabled ? 'opacity-40 cursor-not-allowed' : ''
                    }`}
                  >
                    <span className="flex items-center gap-2">
                      <span className="w-3 text-violet-400">{item.checked ? '✓' : ''}</span>
                      {item.label}
                    </span>
                    {item.shortcut && <span className="text-xs pn-faint ml-6">{item.shortcut}</span>}
                  </button>
                  {item.separatorAfter && <div className="my-1 border-t pn-bd" />}
                </div>
              ))}
            </div>
          )}
        </div>
      ))}

      {/* right side: quick theme toggle */}
      <div className="flex-1" />
      <button
        onClick={onToggleTheme}
        title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
        className="flex items-center justify-center w-8 h-7 rounded-md pn-hover pn-muted hover:pn-text"
      >
        {theme === 'dark' ? <Sun size={15} /> : <Moon size={15} />}
      </button>
    </div>
  )
}
