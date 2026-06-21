import { useState } from 'react'
import { useNotebookStore } from '../hooks/useNotebook'
import Cell from './Cell'
import Toolbar from './Toolbar'
import { Plus, FileCode, Code2, Type, Minus, ChevronDown } from 'lucide-react'
import { useFontSize } from '../hooks/useFontSize'

export default function Notebook() {
  const { currentNotebook, addCell } = useNotebookStore()
  const [selectedCellIndex, setSelectedCellIndex] = useState<number | null>(null)
  const [collapsed, setCollapsed] = useState(false)
  // Code editor font, live across all cells via a window event each Cell listens to.
  const { size: codeFont, inc, dec } = useFontSize('pn-code-size', 16, 9, 40)
  const setFont = (fn: () => void) => {
    fn()
    // read back the just-persisted value and broadcast it
    const v = parseInt(localStorage.getItem('pn-code-size') || '16', 10)
    window.dispatchEvent(new CustomEvent('pn-code-font', { detail: v }))
  }

  if (!currentNotebook) {
    return <div className="p-4">No notebook selected</div>
  }

  // Slim hover strip shown between cells (and above the first) so a Code or
  // Markdown cell can be inserted exactly where you want it.
  const Inserter = ({ at }: { at: number }) => (
    <div className="group relative h-2 hover:h-8 transition-all flex items-center justify-center">
      <div className="absolute inset-x-0 top-1/2 h-px bg-violet-500/0 group-hover:bg-violet-500/40 transition-colors" />
      <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-1 z-10">
        <button
          onClick={() => addCell('code', at)}
          className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] pn-solid-bg border pn-bd hover:border-violet-500 pn-muted hover:pn-text shadow-sm"
          title="Insert code cell here"
        >
          <Code2 size={11} /> Code
        </button>
        <button
          onClick={() => addCell('markdown', at)}
          className="flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] pn-solid-bg border pn-bd hover:border-violet-500 pn-muted hover:pn-text shadow-sm"
          title="Insert markdown / text cell here"
        >
          <Type size={11} /> Markdown
        </button>
      </div>
    </div>
  )

  return (
    <div className="h-full flex flex-col pn-solid-bg overflow-hidden">
      {/* breadcrumb */}
      <div className="h-7 flex items-center gap-1.5 px-3 text-[12px] pn-muted border-b pn-bd pn-surface/40">
        <button onClick={() => setCollapsed((c) => !c)} className="p-0.5 rounded pn-hover" title={collapsed ? 'Expand editor' : 'Collapse editor'}>
          <ChevronDown size={13} className={collapsed ? '-rotate-90 transition-transform' : 'transition-transform'} />
        </button>
        <FileCode size={13} className="text-yellow-400" />
        <span className="pn-muted">{currentNotebook.name}.ipynb</span>
        <span className="pn-faint">— {currentNotebook.cells.length} cells</span>
        <div className="flex-1" />
        <span className="pn-faint mr-1">code font</span>
        <button onClick={() => setFont(dec)} title="Decrease code font" className="p-0.5 rounded pn-hover"><Minus size={12} /></button>
        <span className="tabular-nums w-5 text-center">{codeFont}</span>
        <button onClick={() => setFont(inc)} title="Increase code font" className="p-0.5 rounded pn-hover"><Plus size={12} /></button>
      </div>

      <Toolbar />

      {!collapsed && (
      <div className="flex-1 overflow-y-auto p-4 min-w-0">
        <div className="w-full min-w-0">
          <Inserter at={0} />
          {currentNotebook.cells.map((cell, idx) => (
            <div key={cell.id}>
              <div
                onClick={() => setSelectedCellIndex(idx)}
                className={`cursor-text transition rounded-lg ${
                  selectedCellIndex === idx ? 'ring-2 ring-blue-500/70' : 'ring-1 ring-transparent hover:ring-slate-700'
                }`}
              >
                <Cell cell={cell} cellIndex={idx} />
              </div>
              <Inserter at={idx + 1} />
            </div>
          ))}

          <div className="mt-2 flex items-center justify-center gap-2">
            <button
              onClick={() => addCell('code')}
              className="flex items-center gap-2 px-3 py-2 rounded-lg border border-dashed pn-bd hover:border-violet-500 pn-hover/50 pn-muted hover:pn-text transition"
            >
              <Plus size={16} /> Code
            </button>
            <button
              onClick={() => addCell('markdown')}
              className="flex items-center gap-2 px-3 py-2 rounded-lg border border-dashed pn-bd hover:border-violet-500 pn-hover/50 pn-muted hover:pn-text transition"
            >
              <Type size={16} /> Markdown
            </button>
          </div>
        </div>
      </div>
      )}
    </div>
  )
}
