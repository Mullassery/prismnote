import { useEffect, useState } from 'react'
import { X, Palette, Type, Columns3, Bot, Check, Loader2 } from 'lucide-react'
import { getAiConfig, setAiConfig, CLAUDE_MODELS, OPENAI_MODELS } from '../api/ai'

interface Props {
  onClose: () => void
  theme: 'light' | 'dark'
  setTheme: (t: 'light' | 'dark') => void
  panels: { files: boolean; terminal: boolean; ai: boolean }
  togglePanel: (p: 'files' | 'terminal' | 'ai') => void
}

function Row({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2.5">
      <div className="min-w-0">
        <div className="text-[13px] pn-text">{label}</div>
        {hint && <div className="text-[12px] pn-faint">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

function Section({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <div className="py-3 border-b pn-bd last:border-0">
      <div className="flex items-center gap-2 text-[11px] font-semibold uppercase tracking-wider pn-muted mb-1">
        {icon} {title}
      </div>
      {children}
    </div>
  )
}

function Toggle({ on, onClick }: { on: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`w-9 h-5 rounded-full transition-colors relative ${on ? 'prism-bg' : 'bg-[var(--pn-hover)]'}`}
    >
      <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all ${on ? 'left-[18px]' : 'left-0.5'}`} />
    </button>
  )
}

export default function SettingsModal({ onClose, theme, setTheme, panels, togglePanel }: Props) {
  const [fontSize, setFontSize] = useState<number>(() => {
    // Reflect the current effective size: saved value → live CSS var → 14.
    const saved = parseInt(localStorage.getItem('pn-code-size') || '', 10)
    if (Number.isFinite(saved) && saved > 0) return Math.min(Math.max(saved, 10), 28)
    const css = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--pn-code-size'), 10)
    return Number.isFinite(css) && css > 0 ? Math.min(Math.max(css, 10), 28) : 14
  })
  // ── AI provider config (drives the backend engine: ⌘K edit, Fix, Explain, and
  // — via the shared Ollama endpoint — the chat agent + inline autocomplete) ──
  const [provider, setProvider] = useState<'ollama' | 'claude' | 'openai'>('ollama')
  const [ollamaUrl, setOllamaUrl] = useState(() => localStorage.getItem('pn-ollama') || 'http://localhost:11434')
  const [ollamaModel, setOllamaModel] = useState('')
  const [claudeKey, setClaudeKey] = useState('')
  const [claudeModel, setClaudeModel] = useState('claude-sonnet-4-6')
  const [openaiKey, setOpenaiKey] = useState('')
  const [openaiModel, setOpenaiModel] = useState('gpt-4o')
  const [claudeKeySet, setClaudeKeySet] = useState(false)
  const [openaiKeySet, setOpenaiKeySet] = useState(false)
  const [aiState, setAiState] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle')
  const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'up' | 'down'>('checking')

  // Live-check the selected provider's connection when switching tabs / editing URL.
  useEffect(() => {
    if (provider !== 'ollama') return
    let alive = true
    setOllamaStatus('checking')
    fetch(`${ollamaUrl}/api/tags`)
      .then((r) => r.json())
      .then((d) => { if (alive) setOllamaStatus((d?.models?.length ?? 0) >= 0 ? 'up' : 'down') })
      .catch(() => { if (alive) setOllamaStatus('down') })
    return () => { alive = false }
  }, [provider, ollamaUrl])

  // Status line shown for the active provider (Ollama = live ping; cloud = key presence).
  const providerStatus = (): { ok: boolean | null; text: string } => {
    if (provider === 'ollama')
      return ollamaStatus === 'checking' ? { ok: null, text: 'Checking…' } : ollamaStatus === 'up' ? { ok: true, text: 'Connected' } : { ok: false, text: 'Not reachable — is `ollama serve` running?' }
    if (provider === 'claude') return claudeKeySet ? { ok: true, text: 'API key saved' } : { ok: false, text: 'No API key' }
    return openaiKeySet ? { ok: true, text: 'API key saved' } : { ok: false, text: 'No API key' }
  }

  useEffect(() => {
    getAiConfig()
      .then((c) => {
        if (c.provider) setProvider(c.provider)
        if (c.ollama_url) setOllamaUrl(c.ollama_url)
        if (c.ollama_model) setOllamaModel(c.ollama_model)
        if (c.claude_model) setClaudeModel(c.claude_model)
        if (c.openai_model) setOpenaiModel(c.openai_model)
        setClaudeKeySet(c.claude_key_set)
        setOpenaiKeySet(c.openai_key_set)
      })
      .catch(() => {})
  }, [])

  const saveAi = async () => {
    setAiState('saving')
    try {
      await setAiConfig({
        provider,
        ollama_url: ollamaUrl,
        ollama_model: ollamaModel || undefined,
        claude_api_key: claudeKey || undefined,
        claude_model: claudeModel || undefined,
        openai_api_key: openaiKey || undefined,
        openai_model: openaiModel || undefined,
      })
      // keep the shared Ollama endpoint in sync (chat agent + autocomplete read it)
      localStorage.setItem('pn-ollama', ollamaUrl)
      if (claudeKey) setClaudeKeySet(true)
      if (openaiKey) setOpenaiKeySet(true)
      setClaudeKey(''); setOpenaiKey('')
      setAiState('saved')
      // tell the RHS AI panel to reload the provider/model/connection
      window.dispatchEvent(new Event('pn-ai-config'))
      setTimeout(() => setAiState('idle'), 2000)
    } catch {
      setAiState('error')
    }
  }

  useEffect(() => {
    document.documentElement.style.setProperty('--pn-code-size', `${fontSize}px`)
    localStorage.setItem('pn-code-size', String(fontSize))
  }, [fontSize])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center" onMouseDown={onClose}>
      <div className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" />
      <div
        className="relative w-[560px] max-w-[92vw] max-h-[80vh] overflow-y-auto pn-solid-bg border pn-bd rounded-2xl shadow-2xl shadow-black/50"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 h-12 border-b pn-bd sticky top-0 pn-solid-bg">
          <h2 className="text-[15px] font-semibold pn-text">Settings</h2>
          <button onClick={onClose} className="p-1.5 rounded-lg pn-hover pn-muted hover:pn-text"><X size={16} /></button>
        </div>

        <div className="px-5">
          <Section icon={<Palette size={13} />} title="Appearance">
            <Row label="Theme" hint="Switch between the warm dark and light palette">
              <div className="flex rounded-lg bg-[var(--pn-hover)] p-0.5 text-[12px]">
                {(['dark', 'light'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => setTheme(t)}
                    className={`px-3 py-1 rounded-md capitalize ${theme === t ? 'prism-bg text-white' : 'pn-muted'}`}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </Row>
          </Section>

          <Section icon={<Type size={13} />} title="Editor">
            <Row label="Code font size" hint={`${fontSize}px · terminal, output, code`}>
              <input
                type="range"
                min={10}
                max={28}
                value={fontSize}
                onChange={(e) => setFontSize(parseInt(e.target.value, 10))}
                className="w-40 accent-blue-500"
              />
            </Row>
            <Row label="Preview">
              <code className="pn-code pn-code-size px-2 py-1 rounded pn-solid-bg border pn-bd">df.head(10)</code>
            </Row>
          </Section>

          <Section icon={<Columns3 size={13} />} title="Layout">
            <Row label="Files"><Toggle on={panels.files} onClick={() => togglePanel('files')} /></Row>
            <Row label="Bottom Panel" hint="Output · Variables · Plots · Terminal"><Toggle on={panels.terminal} onClick={() => togglePanel('terminal')} /></Row>
            <Row label="AI Assistant"><Toggle on={panels.ai} onClick={() => togglePanel('ai')} /></Row>
          </Section>

          <Section icon={<Bot size={13} />} title="AI Provider">
            <Row label="Provider" hint="Powers ⌘K edit, Fix, Explain, the chat agent & autocomplete">
              <div className="flex rounded-lg bg-[var(--pn-hover)] p-0.5 text-[12px]">
                {(['ollama', 'claude', 'openai'] as const).map((p) => (
                  <button key={p} onClick={() => setProvider(p)}
                    className={`px-3 py-1 rounded-md capitalize ${provider === p ? 'prism-bg text-white' : 'pn-muted'}`}>
                    {p}
                  </button>
                ))}
              </div>
            </Row>
            {(() => {
              const s = providerStatus()
              return (
                <div className="flex items-center gap-1.5 text-[12px] pb-1">
                  <span className={`w-2 h-2 rounded-full ${s.ok === null ? 'bg-amber-400 animate-pulse' : s.ok ? 'bg-emerald-400' : 'bg-rose-400'}`} />
                  <span className={s.ok === false ? 'text-rose-300' : 'pn-faint'}>{s.text}</span>
                </div>
              )
            })()}

            {provider === 'ollama' && (
              <>
                <Row label="Ollama endpoint" hint="Local model server (chat agent + autocomplete use this too)">
                  <input value={ollamaUrl} onChange={(e) => setOllamaUrl(e.target.value)}
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60" />
                </Row>
                <Row label="Model" hint="blank = auto-discover">
                  <input value={ollamaModel} onChange={(e) => setOllamaModel(e.target.value)} placeholder="qwen2.5-coder, llama3…"
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60" />
                </Row>
              </>
            )}

            {provider === 'claude' && (
              <>
                <Row label="Model" hint="Anthropic Claude">
                  <select value={claudeModel} onChange={(e) => setClaudeModel(e.target.value)}
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60">
                    {CLAUDE_MODELS.map((m) => <option key={m} value={m}>{m}</option>)}
                  </select>
                </Row>
                <Row label="Anthropic API key" hint={claudeKeySet ? 'A key is saved — type to replace' : 'sk-ant-…'}>
                  <input type="password" value={claudeKey} onChange={(e) => setClaudeKey(e.target.value)}
                    placeholder={claudeKeySet ? '•••••••• saved' : 'sk-ant-…'}
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60" />
                </Row>
              </>
            )}

            {provider === 'openai' && (
              <>
                <Row label="Model" hint="OpenAI">
                  <select value={openaiModel} onChange={(e) => setOpenaiModel(e.target.value)}
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60">
                    {OPENAI_MODELS.map((m) => <option key={m} value={m}>{m}</option>)}
                  </select>
                </Row>
                <Row label="OpenAI API key" hint={openaiKeySet ? 'A key is saved — type to replace' : 'sk-…'}>
                  <input type="password" value={openaiKey} onChange={(e) => setOpenaiKey(e.target.value)}
                    placeholder={openaiKeySet ? '•••••••• saved' : 'sk-…'}
                    className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60" />
                </Row>
              </>
            )}

            <div className="flex items-center justify-between pt-2">
              <span className="text-[12px] pn-faint flex items-center gap-1.5">
                {aiState === 'saved' && <><Check size={13} className="text-emerald-400" /> Saved</>}
                {aiState === 'error' && <span className="text-rose-400">Couldn’t save</span>}
                {provider !== 'ollama' && <span>Keys are stored locally in ~/.prismnote/ai_config.json</span>}
              </span>
              <button onClick={saveAi} disabled={aiState === 'saving'}
                className="px-3 py-1.5 rounded-lg prism-bg text-white text-[12px] disabled:opacity-50 flex items-center gap-1.5">
                {aiState === 'saving' && <Loader2 size={13} className="animate-spin" />} Save AI settings
              </button>
            </div>
          </Section>
        </div>
      </div>
    </div>
  )
}
