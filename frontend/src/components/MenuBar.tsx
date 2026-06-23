import { useEffect, useRef, useState } from 'react'
import { Sun, Moon } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'
import { useWorkspace, openNotebookFile, saveJsonAs, saveTextAs } from '../hooks/useWorkspace'
import { restartKernel, interruptKernel } from '../api/kernel'
import FindReplace from './FindReplace'

interface MenuBarProps {
  theme: 'light' | 'dark'
  onToggleTheme: () => void
  panels: { files: boolean; terminal: boolean; ai: boolean }
  onTogglePanel: (panel: 'files' | 'terminal' | 'ai') => void
  onOpenSearch?: () => void
  onOpenJobs?: (create?: boolean) => void
  onOpenGit?: (focus?: 'commit' | 'clone') => void
  onOpenCommandPalette?: () => void
  onOpenDataExplorer?: () => void
  onOpenData?: () => void
}

interface MenuItem {
  label: string
  shortcut?: string
  action?: () => void
  checked?: boolean
  separatorAfter?: boolean
  disabled?: boolean
}

export default function MenuBar({ theme, onToggleTheme, panels, onTogglePanel, onOpenSearch, onOpenJobs, onOpenGit, onOpenCommandPalette, onOpenDataExplorer, onOpenData }: MenuBarProps) {
  const [open, setOpen] = useState<string | null>(null)
  const [findOpen, setFindOpen] = useState(false)
  const barRef = useRef<HTMLDivElement>(null)
  const {
    currentNotebook,
    selectedCellIndex,
    clipboardCell,
    createNotebook,
    saveNotebook,
    deleteNotebook,
    addCell,
    updateCell,
    deleteCell,
    moveCell,
    copyCell,
    cutCell,
    pasteCell,
    executeCell,
  } = useNotebookStore()

  const sel = selectedCellIndex
  const hasSel = currentNotebook != null && sel != null

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
    const existing = (useNotebookStore.getState() as any).notebooks as { name: string }[]
    let name = 'Untitled'
    for (let i = 1; existing.some((n) => n.name === name); i++) name = `Untitled ${i}`
    createNotebook(name)
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

  const srcOf = (c: any) => (Array.isArray(c.source) ? c.source.join('') : c.source || '')

  const runRange = async (from: number, to: number) => {
    if (!currentNotebook) return
    for (let i = from; i < to; i++) {
      if (currentNotebook.cells[i]?.cell_type === 'code') await executeCell(i)
    }
  }
  const runAll = () => runRange(0, currentNotebook?.cells.length ?? 0)
  const runAbove = () => sel != null && runRange(0, sel)
  const runBelow = () => sel != null && runRange(sel + 1, currentNotebook?.cells.length ?? 0)
  const runSelected = () => sel != null && executeCell(sel)
  const restartAndRunAll = async () => {
    if (!confirm('Restart the kernel and run all cells?')) return
    await restartKernel()
    await runAll()
  }

  // Export the notebook's code as a jupytext-style percent script.
  const exportPy = () => {
    if (!currentNotebook) return
    const py = currentNotebook.cells
      .map((c: any) =>
        c.cell_type === 'markdown'
          ? '# %% [markdown]\n' + srcOf(c).split('\n').map((l: string) => '# ' + l).join('\n')
          : '# %%\n' + srcOf(c),
      )
      .join('\n\n')
    saveTextAs(`${currentNotebook.name}.py`, py)
  }

  const menus: Record<string, MenuItem[]> = {
    File: [
      { label: 'New Notebook', shortcut: '⌘N', action: newNotebook },
      { label: 'Open Data Explorer…', shortcut: '⌘E', action: () => onOpenDataExplorer?.(), separatorAfter: true },
      { label: 'Open File…', shortcut: '⌘O', action: openFile },
      { label: 'Open Folder…', action: openFolder, separatorAfter: true },
      { label: 'Save', shortcut: '⌘S', action: () => saveNotebook?.(), disabled: !currentNotebook, separatorAfter: true },
      { label: 'Export as .ipynb…', action: save, disabled: !currentNotebook },
      { label: 'Export as .py…', action: exportPy, disabled: !currentNotebook, separatorAfter: true },
      { label: 'Close Notebook', action: () => currentNotebook && deleteNotebook(currentNotebook.id), disabled: !currentNotebook },
    ],
    Edit: [
      { label: 'Cut Cell', action: () => hasSel && cutCell(sel!), disabled: !hasSel },
      { label: 'Copy Cell', action: () => hasSel && copyCell(sel!), disabled: !hasSel },
      { label: 'Paste Cell Below', action: () => pasteCell(sel ?? (currentNotebook?.cells.length ?? 1) - 1), disabled: !currentNotebook || !clipboardCell },
      { label: 'Delete Cell', action: () => hasSel && deleteCell(sel!), disabled: !hasSel },
      { label: 'Move Cell Up', action: () => hasSel && moveCell(sel!, -1), disabled: !hasSel },
      { label: 'Move Cell Down', action: () => hasSel && moveCell(sel!, 1), disabled: !hasSel, separatorAfter: true },
      { label: 'Add Code Cell', action: () => addCell('code'), disabled: !currentNotebook },
      { label: 'Add Markdown Cell', action: () => addCell('markdown'), disabled: !currentNotebook, separatorAfter: true },
      { label: 'Find & Replace…', action: () => setFindOpen(true), disabled: !currentNotebook },
      { label: 'Command Palette…', shortcut: '⇧⌘P', action: () => onOpenCommandPalette?.() },
    ],
    View: [
      { label: 'Data Explorer', shortcut: '⌘E', action: () => onOpenDataExplorer?.() },
      { label: 'Data & SQL', action: () => onOpenData?.(), separatorAfter: true },
      { label: 'Files', checked: panels.files, action: () => onTogglePanel('files') },
      { label: 'Terminal & Console', checked: panels.terminal, action: () => onTogglePanel('terminal') },
      { label: 'AI Assistant', checked: panels.ai, action: () => onTogglePanel('ai') },
      { label: 'Search…', shortcut: '⌘K', action: () => onOpenSearch?.(), separatorAfter: true },
      { label: theme === 'dark' ? 'Light Theme' : 'Dark Theme', action: onToggleTheme },
    ],
    Run: [
      { label: 'Run All Cells', shortcut: '⌘⇧⏎', action: runAll, disabled: !currentNotebook },
      { label: 'Run Selected Cell', shortcut: '⇧⏎', action: runSelected, disabled: !hasSel },
      { label: 'Run All Above', action: runAbove, disabled: !hasSel },
      { label: 'Run All Below', action: runBelow, disabled: !hasSel, separatorAfter: true },
      { label: 'Restart & Run All', action: restartAndRunAll, disabled: !currentNotebook },
      { label: 'Clear All Outputs', action: clearOutputs, disabled: !currentNotebook },
    ],
    Kernel: [
      { label: 'Interrupt', action: () => interruptKernel() },
      { label: 'Restart', action: () => { if (confirm('Restart the kernel? All variables will be cleared.')) restartKernel() } },
      { label: 'Restart & Clear Outputs', action: async () => { if (confirm('Restart the kernel and clear all outputs?')) { await restartKernel(); clearOutputs() } } },
      { label: 'Restart & Run All', action: restartAndRunAll, disabled: !currentNotebook },
    ],
    Jobs: [
      { label: 'Open Jobs…', action: () => onOpenJobs?.() },
      { label: 'Run Current Notebook as Job…', action: () => onOpenJobs?.(true), disabled: !currentNotebook },
    ],
    Git: [
      { label: 'Source Control…', action: () => onOpenGit?.() },
      { label: 'Commit & Push…', action: () => onOpenGit?.('commit') },
      { label: 'Clone Repository…', action: () => onOpenGit?.('clone') },
    ],
    Help: [
      { label: 'About PrismNote', action: () => window.alert('PrismNote — a modern, open-source data-science notebook.\nRust engine · React UI.') },
      { label: 'Documentation', action: () => window.open('https://github.com/Mullassery/prismnote#readme', '_blank') },
      { label: 'Keyboard Shortcuts', action: () => window.alert('⌘E Data Explorer · ⌘N New · ⌘O Open · ⌘S Save · ⌘K Search · ⇧⌘P Command Palette · ⌘⇧⏎ Run All · ⇧⏎ Run Cell · ⌘K (in cell) AI edit') },
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
                    className={`w-full flex items-center justify-between px-3 py-1.5 text-left rounded-md hover:bg-blue-600 hover:text-white ${
                      item.disabled ? 'opacity-40 cursor-not-allowed' : ''
                    }`}
                  >
                    <span className="flex items-center gap-2">
                      <span className="w-3 text-blue-400">{item.checked ? '✓' : ''}</span>
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

      {findOpen && <FindReplace onClose={() => setFindOpen(false)} />}
    </div>
  )
}
