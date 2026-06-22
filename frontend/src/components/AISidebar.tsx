import { useState } from 'react'
import { Sparkles, Send, Copy, Loader, X } from 'lucide-react'
import axios from 'axios'
import { useNotebookStore } from '../hooks/useNotebook'

interface AISidebarProps {
  onClose: () => void
}

type Msg = { role: 'user' | 'ai'; text: string }

export default function AISidebar({ onClose }: AISidebarProps) {
  const { currentNotebook } = useNotebookStore()
  const [prompt, setPrompt] = useState('')
  const [messages, setMessages] = useState<Msg[]>([])
  const [loading, setLoading] = useState(false)

  const notebookContext = () =>
    (currentNotebook?.cells ?? [])
      .map((c) => (Array.isArray(c.source) ? c.source.join('') : c.source))
      .join('\n\n')

  const send = async (action: 'explain' | 'fix' | 'complete' | 'ask', text?: string) => {
    const content = text ?? prompt
    if (!content.trim() && action === 'ask') return
    setMessages((m) => [...m, { role: 'user', text: action === 'ask' ? content : `${action} current code` }])
    setPrompt('')
    setLoading(true)
    try {
      const res = await axios.post(`/api/ai/${action}`, {
        action,
        code: action === 'ask' ? notebookContext() : notebookContext(),
        prompt: content,
        context: 'Python data science notebook',
      })
      setMessages((m) => [...m, { role: 'ai', text: res.data.suggestion ?? res.data.response ?? '(no response)' }])
    } catch {
      setMessages((m) => [
        ...m,
        { role: 'ai', text: 'AI backend not connected. Set an API key in settings to enable assistance.' },
      ])
    } finally {
      setLoading(false)
    }
  }

  return (
    <aside className="w-80 shrink-0 bg-[#0b0d15]/70 border-l border-white/5 flex flex-col overflow-hidden">
      <div className="h-8 flex items-center justify-between px-3 border-b border-white/5">
        <span className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-gray-200">
          <Sparkles size={13} className="text-sky-400" /> AI Assistant
        </span>
        <button onClick={onClose} className="text-gray-400 hover:text-white p-1 rounded hover:bg-slate-800" title="Hide panel">
          <X size={14} />
        </button>
      </div>

      {/* quick actions */}
      <div className="flex gap-1 p-2 border-b border-slate-800">
        {(['explain', 'fix', 'complete'] as const).map((a) => (
          <button
            key={a}
            onClick={() => send(a)}
            className="flex-1 px-2 py-1.5 text-[11px] capitalize rounded-lg bg-white/5 hover:bg-blue-500/20 hover:text-white border border-white/5 text-gray-200 transition"
          >
            {a}
          </button>
        ))}
      </div>

      {/* conversation */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {messages.length === 0 && (
          <div className="text-[13px] text-gray-500 leading-relaxed">
            Ask anything about your notebook, or use <span className="text-gray-300">Explain / Fix / Complete</span> to act
            on the current code.
          </div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={`text-[13px] ${m.role === 'user' ? 'text-blue-300' : 'text-gray-200'}`}>
            <div className="text-[10px] uppercase tracking-wide text-gray-500 mb-0.5">{m.role === 'user' ? 'You' : 'PrismNote AI'}</div>
            <div className="relative group whitespace-pre-wrap bg-slate-900 border border-slate-800 rounded p-2">
              {m.text}
              {m.role === 'ai' && (
                <button
                  onClick={() => navigator.clipboard.writeText(m.text)}
                  className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 text-gray-400 hover:text-white"
                  title="Copy"
                >
                  <Copy size={12} />
                </button>
              )}
            </div>
          </div>
        ))}
        {loading && (
          <div className="flex items-center gap-2 text-gray-400 text-[13px]">
            <Loader size={14} className="animate-spin" /> thinking…
          </div>
        )}
      </div>

      {/* prompt */}
      <div className="p-2 border-t border-slate-800">
        <div className="flex items-end gap-1 bg-slate-900 border border-slate-700 rounded px-2 py-1">
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                send('ask')
              }
            }}
            rows={1}
            placeholder="Ask the AI…  (Enter to send)"
            className="flex-1 bg-transparent outline-none text-[13px] text-gray-100 resize-none max-h-28 py-1"
          />
          <button onClick={() => send('ask')} disabled={loading} className="text-blue-400 hover:text-blue-300 p-1 disabled:opacity-40">
            <Send size={15} />
          </button>
        </div>
      </div>
    </aside>
  )
}
