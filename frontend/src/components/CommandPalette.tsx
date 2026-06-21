import { useEffect, useMemo, useRef, useState } from 'react'

export interface Command {
  id: string
  title: string
  category?: string
  shortcut?: string
  keywords?: string
  icon?: React.ReactNode
  run: () => void
}

interface Props {
  commands: Command[]
  onClose: () => void
  placeholder?: string
  initialQuery?: string
}

export default function CommandPalette({ commands, onClose, placeholder = 'Type a command…', initialQuery = '' }: Props) {
  const [q, setQ] = useState(initialQuery)
  const [idx, setIdx] = useState(0)
  const listRef = useRef<HTMLDivElement>(null)

  const filtered = useMemo(() => {
    const t = q.trim().toLowerCase()
    if (!t) return commands
    return commands.filter((c) =>
      `${c.title} ${c.category ?? ''} ${c.keywords ?? ''}`.toLowerCase().includes(t)
    )
  }, [q, commands])

  useEffect(() => setIdx(0), [q])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') return onClose()
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setIdx((i) => Math.min(i + 1, filtered.length - 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setIdx((i) => Math.max(i - 1, 0))
      } else if (e.key === 'Enter') {
        e.preventDefault()
        const c = filtered[idx]
        if (c) {
          onClose()
          c.run()
        }
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [filtered, idx, onClose])

  useEffect(() => {
    const el = listRef.current?.children[idx] as HTMLElement | undefined
    el?.scrollIntoView({ block: 'nearest' })
  }, [idx])

  return (
    <div className="fixed inset-0 z-[100] flex items-start justify-center pt-[12vh]" onMouseDown={onClose}>
      <div className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" />
      <div
        className="relative w-[640px] max-w-[92vw] pn-solid-bg border pn-bd rounded-xl shadow-2xl shadow-black/50 overflow-hidden"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <input
          autoFocus
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder={placeholder}
          className="w-full px-4 py-3 bg-transparent outline-none pn-text text-[14px] border-b pn-bd"
        />
        <div ref={listRef} className="max-h-[50vh] overflow-y-auto py-1">
          {filtered.length === 0 && <div className="px-4 py-3 pn-faint text-[13px]">No matching commands</div>}
          {filtered.map((c, i) => (
            <button
              key={c.id}
              onMouseEnter={() => setIdx(i)}
              onClick={() => {
                onClose()
                c.run()
              }}
              className={`w-full flex items-center justify-between px-4 py-2 text-left text-[13px] ${
                i === idx ? 'bg-violet-600 text-white' : 'pn-text'
              }`}
            >
              <span className="flex items-center gap-2 min-w-0">
                {c.icon && <span className={i === idx ? 'text-white' : 'pn-muted'}>{c.icon}</span>}
                {c.category && <span className={`${i === idx ? 'text-white/70' : 'pn-faint'} shrink-0`}>{c.category}:</span>}
                <span className="truncate">{c.title}</span>
              </span>
              {c.shortcut && <span className={`text-[11px] ml-4 ${i === idx ? 'text-white/70' : 'pn-faint'}`}>{c.shortcut}</span>}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
