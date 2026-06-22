import { useMemo, useState } from 'react'
import { X, Replace } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'

// Notebook-wide find & replace (per-cell find is already built into Monaco via ⌘F).
export default function FindReplace({ onClose }: { onClose: () => void }) {
  const { currentNotebook, updateCell } = useNotebookStore()
  const [find, setFind] = useState('')
  const [replace, setReplace] = useState('')
  const [caseSensitive, setCaseSensitive] = useState(false)

  const cells = currentNotebook?.cells ?? []
  const srcOf = (c: any) => (Array.isArray(c.source) ? c.source.join('') : c.source || '')

  const matches = useMemo(() => {
    if (!find) return 0
    const f = caseSensitive ? find : find.toLowerCase()
    return cells.reduce((n: number, c: any) => {
      const s = caseSensitive ? srcOf(c) : srcOf(c).toLowerCase()
      let i = 0, count = 0
      while ((i = s.indexOf(f, i)) !== -1) { count++; i += f.length }
      return n + count
    }, 0)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [find, caseSensitive, currentNotebook])

  const replaceAll = () => {
    if (!find) return
    cells.forEach((c: any, i: number) => {
      const s = srcOf(c)
      let out: string
      if (caseSensitive) {
        out = s.split(find).join(replace)
      } else {
        // case-insensitive replace
        out = s.replace(new RegExp(find.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi'), replace)
      }
      if (out !== s) updateCell(i, { source: out.split(/(?<=\n)/) })
    })
    onClose()
  }

  return (
    <div className="fixed top-12 left-1/2 -translate-x-1/2 z-[60] w-[420px] pn-solid-bg border pn-bd rounded-xl shadow-2xl shadow-black/40 p-3">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium pn-text flex items-center gap-2"><Replace size={15} className="text-blue-400" /> Find &amp; Replace</span>
        <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={15} /></button>
      </div>
      <input autoFocus value={find} onChange={(e) => setFind(e.target.value)} placeholder="Find in all cells"
        className="w-full mb-2 px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-blue-500" />
      <input value={replace} onChange={(e) => setReplace(e.target.value)} placeholder="Replace with"
        onKeyDown={(e) => e.key === 'Enter' && replaceAll()}
        className="w-full mb-2 px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-blue-500" />
      <div className="flex items-center justify-between">
        <label className="flex items-center gap-1.5 text-[12px] pn-muted">
          <input type="checkbox" checked={caseSensitive} onChange={(e) => setCaseSensitive(e.target.checked)} /> Match case
        </label>
        <span className="text-[12px] pn-faint">{find ? `${matches} match${matches === 1 ? '' : 'es'}` : ''}</span>
        <button onClick={replaceAll} disabled={!find || matches === 0}
          className="px-3 py-1 rounded prism-bg text-white text-[13px] disabled:opacity-40">Replace All</button>
      </div>
    </div>
  )
}
