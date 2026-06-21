import { useEffect, useRef, useState } from 'react'
import {
  Sparkles,
  Send,
  X,
  ChevronDown,
  Wand2,
  Play,
  Check,
  Ban,
  Plus,
  Pencil,
  CircleDot,
  Loader,
  Plug,
  Download,
  RefreshCw,
} from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'

const OLLAMA = 'http://localhost:11434'

type Mode = 'plan' | 'act'
type Role = 'user' | 'assistant'

interface AgentAction {
  id: string
  kind: 'add_cell' | 'edit_cell' | 'run_cell'
  index?: number
  code?: string
  status: 'pending' | 'done' | 'rejected'
}

interface Message {
  role: Role
  text: string
  actions?: AgentAction[]
}

const planSystem = `You are PrismNote's PLANNING agent. Read the user's request and the current notebook, then reply with a short, numbered plan describing the approach. Do NOT write the final code or take actions yet — planning only. Be concise.`

const actSystem = `You are PrismNote's CODING agent for a Python data-science notebook. Briefly explain what you'll do, then emit actions the notebook can execute. Use EXACTLY these tags:
- Add a code cell:  <action type="add_cell">PYTHON CODE</action>
- Edit cell N:      <action type="edit_cell" index="N">PYTHON CODE</action>
- Run cell N:       <action type="run_cell" index="N"/>
Only emit actions you are confident about. Keep code runnable and self-contained.`

function parseActions(text: string): AgentAction[] {
  const re = /<action\s+type="(add_cell|edit_cell|run_cell)"(?:\s+index="(\d+)")?\s*(?:\/>|>([\s\S]*?)<\/action>)/g
  const out: AgentAction[] = []
  let m: RegExpExecArray | null
  let i = 0
  while ((m = re.exec(text))) {
    out.push({
      id: `${Date.now()}-${i++}`,
      kind: m[1] as AgentAction['kind'],
      index: m[2] !== undefined ? parseInt(m[2], 10) : undefined,
      code: m[3]?.trim(),
      status: 'pending',
    })
  }
  return out
}

function stripActions(text: string) {
  return text.replace(/<action[\s\S]*?(?:\/>|<\/action>)/g, '').trim()
}

export default function AgentPanel({ onClose }: { onClose: () => void }) {
  const { currentNotebook, addCell, updateCell, executeCell } = useNotebookStore()
  const [mode, setMode] = useState<Mode>('plan')
  const [models, setModels] = useState<string[]>([])
  const [model, setModel] = useState('')
  const [modelOpen, setModelOpen] = useState(false)
  const [connected, setConnected] = useState<boolean | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [streaming, setStreaming] = useState(false)
  const endRef = useRef<HTMLDivElement>(null)

  useEffect(() => endRef.current?.scrollIntoView({ behavior: 'smooth' }), [messages, streaming])

  // discover local Ollama models (retryable)
  const checkOllama = () => {
    setConnected(null)
    fetch(`${OLLAMA}/api/tags`)
      .then((r) => r.json())
      .then((d) => {
        const names = (d.models ?? []).map((m: any) => m.name)
        setModels(names)
        // Prefer a coding model by default (this is a coding agent)
        const preferred =
          names.find((n: string) => /coder/i.test(n)) ||
          names.find((n: string) => /code/i.test(n)) ||
          names[0]
        setModel(preferred ?? '')
        setConnected(true)
      })
      .catch(() => setConnected(false))
  }
  useEffect(() => {
    checkOllama()
  }, [])

  const notebookContext = () => {
    const cells = currentNotebook?.cells ?? []
    if (!cells.length) return '(empty notebook)'
    return cells
      .map((c, i) => `# Cell ${i} (${c.cell_type})\n${Array.isArray(c.source) ? c.source.join('') : c.source}`)
      .join('\n\n')
  }

  const runAction = async (msgIdx: number, action: AgentAction) => {
    if (action.kind === 'add_cell') {
      addCell('code')
      // new cell appended at end
      const newIndex = (currentNotebook?.cells.length ?? 0)
      updateCell(newIndex, { source: (action.code ?? '').split('\n') })
    } else if (action.kind === 'edit_cell' && action.index !== undefined) {
      updateCell(action.index, { source: (action.code ?? '').split('\n') })
    } else if (action.kind === 'run_cell' && action.index !== undefined) {
      await executeCell(action.index)
    }
    setMessages((ms) =>
      ms.map((m, i) =>
        i === msgIdx ? { ...m, actions: m.actions?.map((a) => (a.id === action.id ? { ...a, status: 'done' } : a)) } : m
      )
    )
  }

  const rejectAction = (msgIdx: number, id: string) =>
    setMessages((ms) =>
      ms.map((m, i) =>
        i === msgIdx ? { ...m, actions: m.actions?.map((a) => (a.id === id ? { ...a, status: 'rejected' } : a)) } : m
      )
    )

  const send = async () => {
    if (!input.trim() || streaming || !model) return
    const userMsg: Message = { role: 'user', text: input }
    const history = [...messages, userMsg]
    setMessages(history)
    setInput('')
    setStreaming(true)

    const sys = mode === 'plan' ? planSystem : actSystem
    const payload = {
      model,
      stream: true,
      messages: [
        { role: 'system', content: `${sys}\n\nCurrent notebook:\n${notebookContext()}` },
        ...history.map((m) => ({ role: m.role, content: m.text })),
      ],
    }

    setMessages((ms) => [...ms, { role: 'assistant', text: '' }])
    try {
      const res = await fetch(`${OLLAMA}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!res.body) throw new Error('no stream')
      const reader = res.body.getReader()
      const dec = new TextDecoder()
      let buf = ''
      let acc = ''
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        buf += dec.decode(value, { stream: true })
        const lines = buf.split('\n')
        buf = lines.pop() ?? ''
        for (const line of lines) {
          if (!line.trim()) continue
          const json = JSON.parse(line)
          if (json.message?.content) {
            acc += json.message.content
            setMessages((ms) => {
              const copy = [...ms]
              copy[copy.length - 1] = { role: 'assistant', text: acc }
              return copy
            })
          }
        }
      }
      const actions = mode === 'act' ? parseActions(acc) : []
      setMessages((ms) => {
        const copy = [...ms]
        copy[copy.length - 1] = { role: 'assistant', text: acc, actions: actions.length ? actions : undefined }
        return copy
      })
    } catch {
      setMessages((ms) => {
        const copy = [...ms]
        copy[copy.length - 1] = {
          role: 'assistant',
          text: `⚠️ Couldn't reach Ollama at ${OLLAMA}. Make sure it's running (\`ollama serve\`) and that browser requests are allowed:\n\nOLLAMA_ORIGINS=http://localhost:5173 ollama serve`,
        }
        return copy
      })
    } finally {
      setStreaming(false)
    }
  }

  const actionMeta = {
    add_cell: { icon: Plus, label: 'Add code cell', color: 'text-emerald-400' },
    edit_cell: { icon: Pencil, label: 'Edit cell', color: 'text-amber-400' },
    run_cell: { icon: Play, label: 'Run cell', color: 'text-violet-400' },
  } as const

  return (
    <aside className="w-96 shrink-0 pn-surface border-l pn-bd flex flex-col overflow-hidden">
      {/* header */}
      <div className="h-9 flex items-center justify-between px-3 border-b pn-bd">
        <span className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider pn-text">
          <Sparkles size={13} className="text-fuchsia-400" /> PrismNote Agent
        </span>
        <div className="flex items-center gap-2">
          <span
            className={`flex items-center gap-1 text-[10px] ${
              connected === false ? 'text-red-400' : connected ? 'text-emerald-400' : 'pn-faint'
            }`}
            title="Ollama connection"
          >
            <Plug size={11} /> {connected === false ? 'offline' : connected ? 'Ollama' : '…'}
          </span>
          <button onClick={onClose} className="pn-muted hover:pn-text p-1 rounded hover:bg-white/5">
            <X size={14} />
          </button>
        </div>
      </div>

      {/* mode toggle + model picker */}
      <div className="flex items-center gap-2 px-2 py-2 border-b pn-bd">
        <div className="flex rounded-lg bg-white/5 p-0.5 text-[12px]">
          {(['plan', 'act'] as Mode[]).map((m) => (
            <button
              key={m}
              onClick={() => setMode(m)}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-md capitalize transition ${
                mode === m ? 'prism-bg text-white' : 'pn-muted hover:pn-text'
              }`}
            >
              {m === 'plan' ? <Wand2 size={12} /> : <Play size={12} />}
              {m}
            </button>
          ))}
        </div>

        <div className="relative flex-1 min-w-0">
          <button
            onClick={() => setModelOpen((o) => !o)}
            className="w-full flex items-center justify-between gap-1 px-2 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-[12px] pn-text"
          >
            <span className="truncate">{model || (connected === false ? 'no Ollama' : 'select model')}</span>
            <ChevronDown size={13} className="shrink-0" />
          </button>
          {modelOpen && (
            <div className="absolute right-0 top-9 z-20 w-full max-h-60 overflow-auto pn-solid-bg border border-white/10 rounded-lg shadow-2xl py-1">
              {models.length === 0 && <div className="px-3 py-2 text-[12px] pn-faint">No models found</div>}
              {models.map((m) => (
                <button
                  key={m}
                  onClick={() => {
                    setModel(m)
                    setModelOpen(false)
                  }}
                  className={`w-full text-left px-3 py-1.5 text-[12px] hover:bg-violet-500/20 ${
                    m === model ? 'pn-text' : 'pn-muted'
                  }`}
                >
                  {m}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* conversation */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4 min-w-0">
        {/* Ollama not detected → install guidance */}
        {connected === false && (
          <div className="rounded-xl border border-amber-500/30 bg-amber-500/10 p-3">
            <div className="flex items-center gap-2 text-amber-300 text-[13px] font-semibold mb-1.5">
              <Download size={15} /> Ollama not detected
            </div>
            <p className="text-[12.5px] pn-muted leading-relaxed mb-2">
              The agent runs on <span className="pn-text">local models via Ollama</span> — free, private, offline.
              Install it to enable AI:
            </p>
            <ol className="text-[12.5px] pn-muted space-y-1.5 mb-3 list-decimal pl-4">
              <li>
                Get Ollama:{' '}
                <a href="https://ollama.com/download" target="_blank" rel="noreferrer" className="text-violet-300 underline">
                  ollama.com/download
                </a>{' '}
                <span className="pn-faint">(or <code className="pn-code">brew install ollama</code>)</span>
              </li>
              <li>
                Pull a coding model: <code className="pn-code block mt-1 px-2 py-1 rounded pn-solid-bg">ollama pull qwen2.5-coder</code>
              </li>
              <li>
                Allow the browser: <code className="pn-code block mt-1 px-2 py-1 rounded pn-solid-bg break-all">OLLAMA_ORIGINS=http://localhost:5173 ollama serve</code>
              </li>
            </ol>
            <div className="flex gap-2">
              <a
                href="https://ollama.com/download"
                target="_blank"
                rel="noreferrer"
                className="flex-1 text-center px-3 py-1.5 rounded-lg prism-bg text-white text-[12.5px] font-medium hover:brightness-110"
              >
                Install Ollama
              </a>
              <button
                onClick={checkOllama}
                className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 pn-text text-[12.5px]"
              >
                <RefreshCw size={13} /> Retry
              </button>
            </div>
          </div>
        )}

        {messages.length === 0 && connected !== false && (
          <div className="text-[13px] pn-faint leading-relaxed">
            <p className="mb-2">
              <span className="text-violet-300 font-medium">Plan</span> mode discusses an approach;{' '}
              <span className="text-fuchsia-300 font-medium">Act</span> mode proposes cell edits you can run.
            </p>
            <p>The agent sees your whole notebook. Ask it to load data, write a chart, or debug an error.</p>
          </div>
        )}

        {messages.map((m, i) => (
          <div key={i} className="min-w-0">
            <div className="text-[10px] uppercase tracking-wide pn-faint mb-1">
              {m.role === 'user' ? 'You' : 'Agent'}
            </div>
            {m.text && (
              <div
                className={`text-[13px] whitespace-pre-wrap break-words rounded-lg p-2.5 ${
                  m.role === 'user'
                    ? 'bg-violet-500/10 border border-violet-500/20 text-violet-100'
                    : 'bg-white/5 border pn-bd pn-text'
                }`}
              >
                {m.role === 'assistant' ? stripActions(m.text) || (streaming && i === messages.length - 1 ? '…' : '') : m.text}
              </div>
            )}

            {/* action cards */}
            {m.actions?.map((a) => {
              const meta = actionMeta[a.kind]
              const Icon = meta.icon
              return (
                <div key={a.id} className="mt-2 rounded-lg border border-white/10 pn-solid-bg overflow-hidden min-w-0">
                  <div className="flex items-center justify-between px-2.5 py-1.5 bg-white/5">
                    <span className={`flex items-center gap-1.5 text-[12px] ${meta.color}`}>
                      <Icon size={13} /> {meta.label}
                      {a.index !== undefined && <span className="pn-faint">· cell {a.index}</span>}
                    </span>
                    {a.status === 'pending' ? (
                      <span className="flex items-center gap-1">
                        <button
                          onClick={() => runAction(i, a)}
                          className="flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 hover:bg-emerald-500/30 text-[11px]"
                        >
                          <Check size={11} /> Run
                        </button>
                        <button
                          onClick={() => rejectAction(i, a.id)}
                          className="flex items-center gap-1 px-2 py-0.5 rounded bg-white/5 pn-muted hover:pn-text text-[11px]"
                        >
                          <Ban size={11} /> Skip
                        </button>
                      </span>
                    ) : (
                      <span className={`text-[11px] ${a.status === 'done' ? 'text-emerald-400' : 'pn-faint'}`}>
                        {a.status === 'done' ? '✓ applied' : 'skipped'}
                      </span>
                    )}
                  </div>
                  {a.code && (
                    <pre className="px-2.5 py-2 text-[12px] pn-muted font-mono overflow-x-auto whitespace-pre min-w-0 max-w-full">{a.code}</pre>
                  )}
                </div>
              )
            })}
          </div>
        ))}

        {streaming && (
          <div className="flex items-center gap-2 pn-muted text-[13px]">
            <Loader size={14} className="animate-spin" /> {mode === 'plan' ? 'planning' : 'thinking'}…
          </div>
        )}
        <div ref={endRef} />
      </div>

      {/* prompt */}
      <div className="p-2 border-t pn-bd">
        <div className="flex items-end gap-1 pn-solid-bg border border-white/10 rounded-xl px-2 py-1 focus-within:border-violet-500/50">
          <CircleDot size={14} className={`mb-2 shrink-0 ${mode === 'act' ? 'text-fuchsia-400' : 'text-violet-400'}`} />
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                send()
              }
            }}
            rows={1}
            placeholder={mode === 'plan' ? 'Ask the agent to plan…' : 'Tell the agent what to build…'}
            className="flex-1 bg-transparent outline-none text-[13px] pn-text resize-none max-h-32 py-1.5"
          />
          <button
            onClick={send}
            disabled={streaming || !input.trim()}
            className="mb-1 text-violet-300 hover:text-violet-200 p-1 disabled:opacity-40"
          >
            <Send size={15} />
          </button>
        </div>
      </div>
    </aside>
  )
}
